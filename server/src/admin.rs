//! The admin API (`/api/admin/*`). Only the admin's Google account may use
//! it; every change is written to the admin log.

use std::collections::{HashMap, HashSet};

use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::Json;
use codesprout_core::api::*;
use codesprout_core::{catalog, find_course, find_lesson, progress};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::access::{is_admin_email, Access};
use crate::db;
use crate::error::{ApiError, ApiResult};
use crate::payments;
use crate::routes::{access_for, now, AppState};
use crate::store::{self, Payment};

pub(crate) async fn require_admin(state: &AppState, headers: &HeaderMap) -> ApiResult<Access> {
    let access = access_for(state, headers).await?;
    if access.user_id.is_none() {
        return Err(ApiError::Unauthorized);
    }
    if !access.is_admin() {
        return Err(ApiError::Forbidden);
    }
    Ok(access)
}

fn admin_name(a: &Access) -> String {
    a.email.clone().unwrap_or_default()
}

/// Payments that open a course right now, by (user, course) → plan.
fn active_plans(payments: &[Payment], t: i64) -> HashMap<(i64, String), (String, Option<i64>)> {
    let mut out: HashMap<(i64, String), (String, Option<i64>)> = HashMap::new();
    for p in payments.iter().filter(|p| p.active(t)) {
        let Some(uid) = p.user_id else { continue };
        let key = (uid, p.course_id.clone());
        let better = match out.get(&key) {
            None => true,
            Some((kind, _)) => kind != "lifetime" && p.plan == "lifetime",
        };
        if better {
            out.insert(key, (p.plan.clone(), if p.plan == "monthly" { p.until } else { None }));
        }
    }
    out
}

fn is_money(p: &Payment) -> bool {
    !p.provider_ref.starts_with("sub_") && matches!(p.status.as_str(), "paid" | "partial_refund")
}

/// `GET /api/admin/overview`
pub async fn overview(State(state): State<AppState>, headers: HeaderMap) -> ApiResult<Json<AdminOverview>> {
    require_admin(&state, &headers).await?;
    let t = now();
    let today = progress::day_number(t, 0);
    let counts = store::counts(&state.db, t - 7 * 86_400, today - 7).await?;
    let all = store::all_payments(&state.db, 100_000).await?;
    let real: Vec<Payment> = all.iter().filter(|p| !p.test_mode).cloned().collect();
    let plans = active_plans(&real, t);
    let mut revenue: HashMap<String, i64> = HashMap::new();
    for p in real.iter().filter(|p| is_money(p)) {
        *revenue.entry(p.currency.clone()).or_default() += p.amount_cents;
    }
    let overrides = store::all_overrides(&state.db).await?;

    // Learners per course: users who finished at least one lesson in it.
    let mut started: HashMap<&'static str, HashSet<i64>> = HashMap::new();
    for (lesson_id, user_id) in store::learners_per_lesson_prefix(&state.db).await? {
        if let Some((course, _, _)) = find_lesson(&lesson_id) {
            started.entry(course.id).or_default().insert(user_id);
        }
    }
    let courses = catalog()
        .iter()
        .map(|c| {
            let mut full: HashSet<i64> =
                plans.keys().filter(|(_, course)| course == c.id).map(|(u, _)| *u).collect();
            for (u, course, mode) in &overrides {
                if course == c.id && mode == "open" {
                    full.insert(*u);
                }
            }
            (c.id.to_string(), started.get(c.id).map_or(0, |s| s.len() as u64), full.len() as u64)
        })
        .collect();

    let mut refunded: HashMap<String, i64> = HashMap::new();
    for p in real.iter().filter(|p| p.refunded_cents > 0) {
        *refunded.entry(p.currency.clone()).or_default() += p.refunded_cents;
    }
    // Revenue counts what was charged (refunds are shown separately).
    for p in real.iter().filter(|p| p.status == "refunded" && !p.provider_ref.starts_with("sub_")) {
        *revenue.entry(p.currency.clone()).or_default() += p.amount_cents;
    }
    let mut revenue: Vec<(String, i64)> = revenue.into_iter().collect();
    revenue.sort();
    let mut refunded: Vec<(String, i64)> = refunded.into_iter().collect();
    refunded.sort();
    Ok(Json(AdminOverview {
        users: counts["users"] as u64,
        google_users: counts["google_users"] as u64,
        new_users_7d: counts["new_users"] as u64,
        active_users_7d: counts["active_users"] as u64,
        lessons_7d: counts["lessons"] as u64,
        paying_users: plans.keys().map(|(u, _)| *u).collect::<HashSet<_>>().len() as u64,
        lifetime_purchases: real.iter().filter(|p| p.plan == "lifetime" && p.active(t)).count() as u64,
        active_subscriptions: real
            .iter()
            .filter(|p| p.provider_ref.starts_with("sub_") && p.active(t))
            .count() as u64,
        revenue,
        banned_users: counts["banned"] as u64,
        courses,
        payments_enabled: payments::config().is_some(),
        refunded,
        sales_paused: store::sales_paused(&state.db).await?,
    }))
}

#[derive(Deserialize)]
pub struct UsersQuery {
    #[serde(default)]
    q: String,
}

/// `GET /api/admin/users?q=…`
pub async fn users(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<UsersQuery>,
) -> ApiResult<Json<Vec<AdminUser>>> {
    require_admin(&state, &headers).await?;
    let t = now();
    let rows = store::user_rows(&state.db, &q.q, 500).await?;
    let contacts = store::all_contacts(&state.db).await?;
    let all = store::all_payments(&state.db, 100_000).await?;
    let plans = active_plans(&all, t);
    let mut manual: HashMap<(i64, String), String> = HashMap::new();
    for (u, c, m) in store::all_overrides(&state.db).await? {
        manual.insert((u, c), m);
    }
    let out = rows
        .into_iter()
        .map(|u| {
            let mine: Vec<&Payment> = all.iter().filter(|p| p.user_id == Some(u.id)).collect();
            let mut methods: Vec<String> =
                mine.iter().map(|p| p.method.clone()).filter(|m| !m.is_empty()).collect();
            methods.sort();
            methods.dedup();
            let admin = is_admin_email(&u.email);
            let courses = catalog()
                .iter()
                .map(|c| {
                    let m = manual.get(&(u.id, c.id.to_string())).cloned().unwrap_or_default();
                    let paid = plans.get(&(u.id, c.id.to_string()));
                    let (state, until) = if admin {
                        ("admin".to_string(), None)
                    } else if m == "block" {
                        ("blocked".to_string(), None)
                    } else if m == "open" {
                        ("open".to_string(), None)
                    } else if let Some((kind, until)) = paid {
                        (kind.clone(), *until)
                    } else {
                        ("free".to_string(), None)
                    };
                    AdminCourseState { course_id: c.id.into(), state, until, manual: m }
                })
                .collect();
            let (phone, country) = contacts.get(&u.id).cloned().unwrap_or_default();
            AdminUser {
                phone,
                country,
                id: u.id,
                username: u.username,
                email: u.email,
                sign_in: if u.google { "google" } else { "password" }.into(),
                joined_at: u.created_at,
                last_active_day: u.last_active_day,
                xp: u.xp,
                lessons_completed: u.lessons.max(0) as u64,
                banned: u.banned,
                is_admin: admin,
                payment_methods: methods,
                total_paid_cents: mine.iter().filter(|p| is_money(p)).map(|p| p.amount_cents).sum(),
                courses,
            }
        })
        .collect();
    Ok(Json(out))
}

/// `GET /api/admin/payments`
pub async fn payments(State(state): State<AppState>, headers: HeaderMap) -> ApiResult<Json<Vec<AdminPayment>>> {
    require_admin(&state, &headers).await?;
    let names: HashMap<i64, (String, String)> = store::user_rows(&state.db, "", 100_000)
        .await?
        .into_iter()
        .map(|u| (u.id, (u.username, u.email)))
        .collect();
    let contacts = store::all_contacts(&state.db).await?;
    let list = store::all_payments(&state.db, 1000)
        .await?
        .into_iter()
        .map(|p| {
            let (username, email) = p.user_id.and_then(|id| names.get(&id).cloned()).unwrap_or_default();
            let link_ref = if p.provider_ref.starts_with("inv_") { p.manage_url.clone() } else { p.provider_ref.clone() };
            let (phone, country) = p.user_id.and_then(|id| contacts.get(&id).cloned()).unwrap_or_default();
            AdminPayment {
                id: p.id,
                user_id: p.user_id,
                username,
                email: if email.is_empty() { p.email.clone() } else { email },
                course_id: p.course_id.clone(),
                plan: p.plan.clone(),
                status: p.status.clone(),
                amount_cents: p.amount_cents,
                currency: p.currency.clone(),
                method: p.method.clone(),
                provider: p.provider.clone(),
                provider_ref: p.provider_ref.clone(),
                until: p.until,
                test_mode: p.test_mode,
                created_at: p.created_at,
                dashboard_url: payments::dashboard_url(&link_ref),
                refunded_cents: p.refunded_cents,
                refundable_cents: p.refundable_cents(),
                can_cancel: p.can_cancel(),
                customer_name: p.customer_name.clone(),
                phone,
                country,
                raw: p.raw.clone(),
                updated_at: p.updated_at,
            }
        })
        .collect();
    Ok(Json(list))
}

/// `GET /api/admin/log`
pub async fn log(State(state): State<AppState>, headers: HeaderMap) -> ApiResult<Json<Vec<AdminLogEntry>>> {
    require_admin(&state, &headers).await?;
    Ok(Json(
        store::recent_log(&state.db, 300)
            .await?
            .into_iter()
            .map(|(at, admin, action, target, detail)| AdminLogEntry { at, admin, action, target, detail })
            .collect(),
    ))
}

async fn username(state: &AppState, user_id: i64) -> ApiResult<String> {
    Ok(db::get_user(&state.db, user_id).await?.ok_or(ApiError::NotFound)?.username)
}

/// `POST /api/admin/access` – open or block a course for a user, or go back
/// to what their payments give them ("default").
pub async fn set_access(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<AdminAccessRequest>,
) -> ApiResult<Json<Value>> {
    let admin = require_admin(&state, &headers).await?;
    find_course(&req.course_id).ok_or(ApiError::NotFound)?;
    let name = username(&state, req.user_id).await?;
    let mode = match req.mode.as_str() {
        "open" => Some("open"),
        "block" => Some("block"),
        "default" => None,
        _ => return Err(ApiError::BadRequest("mode must be open, block or default".into())),
    };
    let t = now();
    store::set_override(&state.db, req.user_id, &req.course_id, mode, t).await?;
    store::log(&state.db, &admin_name(&admin), "access", &name, &format!("{} → {}", req.course_id, req.mode), t).await?;
    Ok(Json(json!({ "ok": true })))
}

/// `POST /api/admin/ban` – suspend (signs the user out everywhere) or restore.
pub async fn ban(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<AdminBanRequest>,
) -> ApiResult<Json<Value>> {
    let admin = require_admin(&state, &headers).await?;
    if Some(req.user_id) == admin.user_id {
        return Err(ApiError::BadRequest("You can't suspend your own account".into()));
    }
    let name = username(&state, req.user_id).await?;
    let t = now();
    store::set_ban(&state.db, req.user_id, req.banned, t).await?;
    let action = if req.banned { "suspend" } else { "restore" };
    store::log(&state.db, &admin_name(&admin), action, &name, "", t).await?;
    Ok(Json(json!({ "ok": true })))
}

/// `POST /api/admin/delete` – removes an account (payment records are kept).
pub async fn delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<AdminDeleteRequest>,
) -> ApiResult<Json<Value>> {
    let admin = require_admin(&state, &headers).await?;
    if Some(req.user_id) == admin.user_id {
        return Err(ApiError::BadRequest("You can't delete your own account".into()));
    }
    let name = username(&state, req.user_id).await?;
    let t = now();
    store::delete_user(&state.db, req.user_id).await?;
    store::log(&state.db, &admin_name(&admin), "delete", &name, "", t).await?;
    Ok(Json(json!({ "ok": true })))
}

/// `POST /api/admin/settings` – pause or resume new purchases.
pub async fn settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<AdminSettingsRequest>,
) -> ApiResult<Json<Value>> {
    let admin = require_admin(&state, &headers).await?;
    store::set_setting(&state.db, "sales_paused", if req.sales_paused { "1" } else { "0" }).await?;
    let action = if req.sales_paused { "pause purchases" } else { "resume purchases" };
    store::log(&state.db, &admin_name(&admin), action, "-", "", now()).await?;
    Ok(Json(json!({ "ok": true })))
}

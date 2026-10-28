//! Payments through Lemon Squeezy (the "merchant of record": it charges the
//! customer, handles VAT/sales tax worldwide and pays out to the seller).
//!
//! * `POST /api/payments/checkout` creates a checkout for one course and one
//!   plan ($5.99/month subscription or $29 one-time) and returns its URL.
//! * `POST /api/payments/webhook` receives Lemon Squeezy's events (signed
//!   with HMAC-SHA256) and records them; access is derived from the records.
//!
//! Settings (Render → Environment): `LEMONSQUEEZY_API_KEY`,
//! `LEMONSQUEEZY_STORE_ID`, `LEMONSQUEEZY_VARIANT_MONTHLY`,
//! `LEMONSQUEEZY_VARIANT_YEARLY` (older name `LEMONSQUEEZY_VARIANT_LIFETIME`), `LEMONSQUEEZY_WEBHOOK_SECRET`.

use std::time::Duration;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use codesprout_core::access::{ALL_COURSES, PRICE_ALL_MONTHLY, PRICE_ALL_YEARLY, PRICE_MONTHLY, PRICE_YEARLY, YEAR_SECS};
use codesprout_core::api::{CheckoutRequest, CheckoutResponse};
use codesprout_core::find_course;
use hmac::{Hmac, Mac};
use serde_json::{json, Value};
use sha2::Sha256;

use crate::error::{ApiError, ApiResult};
use crate::routes::{now, require_user, AppState};
use crate::store::{self, Payment};

pub const PROVIDER: &str = "lemonsqueezy";

pub struct Config {
    pub api_key: String,
    pub store_id: String,
    pub variant_monthly: String,
    /// The one-time $29 product (a year of access).
    pub variant_yearly: String,
    /// "All courses" products (optional: without them the bundle isn't offered).
    pub variant_all_monthly: Option<String>,
    pub variant_all_yearly: Option<String>,
    pub webhook_secret: String,
    pub public_url: String,
}

fn env(k: &str) -> Option<String> {
    std::env::var(k).ok().map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

impl Config {
    fn monthly_variants(&self) -> Vec<&str> {
        std::iter::once(self.variant_monthly.as_str()).chain(self.variant_all_monthly.as_deref()).collect()
    }
    fn yearly_variants(&self) -> Vec<&str> {
        std::iter::once(self.variant_yearly.as_str()).chain(self.variant_all_yearly.as_deref()).collect()
    }
}

pub fn config() -> Option<Config> {
    Some(Config {
        api_key: env("LEMONSQUEEZY_API_KEY")?,
        store_id: env("LEMONSQUEEZY_STORE_ID")?,
        variant_monthly: env("LEMONSQUEEZY_VARIANT_MONTHLY")?,
        // LEMONSQUEEZY_VARIANT_LIFETIME is the older name of the same setting.
        variant_yearly: env("LEMONSQUEEZY_VARIANT_YEARLY").or_else(|| env("LEMONSQUEEZY_VARIANT_LIFETIME"))?,
        variant_all_monthly: env("LEMONSQUEEZY_VARIANT_ALL_MONTHLY"),
        variant_all_yearly: env("LEMONSQUEEZY_VARIANT_ALL_YEARLY"),
        webhook_secret: env("LEMONSQUEEZY_WEBHOOK_SECRET")?,
        public_url: env("PUBLIC_URL")
            .or_else(|| env("RENDER_EXTERNAL_URL"))
            .unwrap_or_else(|| "https://code-iot.onrender.com".into())
            .trim_end_matches('/')
            .to_string(),
    })
}

/// `GET /api/payments/config`
/// `enabled` is false while purchases are stopped, so the page shows "opening
/// soon" and the waiting list instead of the price buttons.
pub async fn config_info(State(state): State<AppState>) -> ApiResult<Json<Value>> {
    let paused = store::sales_paused(&state.db).await?;
    let all = config().is_some_and(|c| c.variant_all_monthly.is_some() && c.variant_all_yearly.is_some());
    Ok(Json(json!({
        "all": all,
        "courses": codesprout_core::catalog().len(),
        "all_monthly": PRICE_ALL_MONTHLY,
        "all_yearly": PRICE_ALL_YEARLY,
        "enabled": config().is_some() && !paused,
        "paused": paused,
        "monthly": PRICE_MONTHLY,
        "yearly": PRICE_YEARLY,
    })))
}

/// `GET /api/me/waitlist`: the courses the user asked to hear about.
pub async fn my_waitlist(State(state): State<AppState>, headers: HeaderMap) -> ApiResult<Json<Vec<String>>> {
    let user_id = crate::routes::require_user(&state, &headers).await?;
    Ok(Json(store::my_waitlist(&state.db, user_id).await?))
}

/// `POST /api/me/waitlist` `{course_id, join}`.
pub async fn set_waitlist(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<codesprout_core::api::WaitlistRequest>,
) -> ApiResult<Json<Vec<String>>> {
    let user_id = crate::routes::require_user(&state, &headers).await?;
    find_course(&req.course_id).ok_or(ApiError::NotFound)?;
    if req.join {
        store::join_waitlist(&state.db, user_id, &req.course_id, now()).await?;
    } else {
        store::leave_waitlist(&state.db, user_id, &req.course_id).await?;
    }
    Ok(Json(store::my_waitlist(&state.db, user_id).await?))
}

/// Lemon Squeezy's API address (tests point this at a local fake).
fn api_base() -> String {
    env("LEMONSQUEEZY_API_BASE").unwrap_or_else(|| "https://api.lemonsqueezy.com/v1".into()).trim_end_matches('/').to_string()
}

fn http() -> &'static reqwest::Client {
    static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    CLIENT.get_or_init(|| reqwest::Client::builder().timeout(Duration::from_secs(25)).build().expect("http client"))
}

/// `POST /api/payments/checkout` – signed-in users only.
pub async fn checkout(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CheckoutRequest>,
) -> ApiResult<Json<CheckoutResponse>> {
    let cfg = config().ok_or_else(|| ApiError::BadRequest("Payments are not open yet".into()))?;
    let user_id = require_user(&state, &headers).await?;
    if store::sales_paused(&state.db).await? {
        return Err(ApiError::BadRequest("Payments are not open yet".into()));
    }
    let bundle = req.plan.starts_with("all_");
    // The page the buyer came from (any course for the bundle) is where they return.
    let back = find_course(&req.course_id).map(|c| c.id).unwrap_or("python");
    let (course_id, title) = if bundle {
        (ALL_COURSES.to_string(), "All courses".to_string())
    } else {
        let c = find_course(&req.course_id).ok_or(ApiError::NotFound)?;
        (c.id.to_string(), c.title.to_string())
    };
    // A phone number is required so the site owner can reach the buyer.
    let (_, country) = store::contact(&state.db, user_id)
        .await?
        .ok_or_else(|| ApiError::BadRequest("Please add your phone number first".into()))?;
    // "lifetime" is the older name of the yearly plan (apps not yet updated).
    let none = || ApiError::BadRequest("Unknown plan".into());
    let (variant, plan) = match req.plan.as_str() {
        "monthly" => (&cfg.variant_monthly, "monthly"),
        "yearly" | "lifetime" => (&cfg.variant_yearly, "yearly"),
        "all_monthly" => (cfg.variant_all_monthly.as_ref().ok_or_else(none)?, "monthly"),
        "all_yearly" => (cfg.variant_all_yearly.as_ref().ok_or_else(none)?, "yearly"),
        _ => return Err(none()),
    };
    let email = store::user_email(&state.db, user_id).await?.unwrap_or_default();
    let mut checkout_data = json!({
        "custom": { "user_id": user_id.to_string(), "course_id": course_id, "plan": plan }
    });
    if !email.is_empty() {
        checkout_data["email"] = json!(email);
    }
    checkout_data["billing_address"] = json!({ "country": country });
    let body = json!({
        "data": {
            "type": "checkouts",
            "attributes": {
                "checkout_data": checkout_data,
                "product_options": {
                    "redirect_url": format!("{}/unlock/{}?paid=1", cfg.public_url, back),
                    "description": format!("Code-Iot · {}", title)
                }
            },
            "relationships": {
                "store": { "data": { "type": "stores", "id": cfg.store_id } },
                "variant": { "data": { "type": "variants", "id": variant } }
            }
        }
    });
    let resp: Value = http()
        .post(format!("{}/checkouts", api_base()))
        .bearer_auth(&cfg.api_key)
        .header("Accept", "application/vnd.api+json")
        .header("Content-Type", "application/vnd.api+json")
        .body(body.to_string())
        .send()
        .await
        .map_err(|e| ApiError::Internal(format!("checkout: {e}")))?
        .json()
        .await
        .map_err(|e| ApiError::Internal(format!("checkout reply: {e}")))?;
    let url = resp["data"]["attributes"]["url"].as_str().ok_or_else(|| {
        ApiError::Internal(format!("checkout failed: {}", resp["errors"].to_string().chars().take(300).collect::<String>()))
    })?;
    Ok(Json(CheckoutResponse { url: url.to_string() }))
}

/// Checks Lemon Squeezy's `X-Signature` (hex HMAC-SHA256 of the raw body).
pub fn signature_ok(secret: &str, body: &[u8], signature_hex: &str) -> bool {
    let Ok(sig) = hex::decode(signature_hex.trim()) else { return false };
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(secret.as_bytes()) else { return false };
    mac.update(body);
    mac.verify_slice(&sig).is_ok()
}

/// "2024-05-01T10:20:30.000000Z" → unix seconds.
pub fn parse_time(s: &str) -> Option<i64> {
    let s = s.trim();
    if s.len() < 19 {
        return None;
    }
    let n = |a: usize, b: usize| s.get(a..b)?.parse::<i64>().ok();
    let (y, m, d) = (n(0, 4)?, n(5, 7)?, n(8, 10)?);
    let (hh, mm, ss) = (n(11, 13)?, n(14, 16)?, n(17, 19)?);
    // Days from civil (Howard Hinnant's algorithm).
    let y2 = if m <= 2 { y - 1 } else { y };
    let era = (if y2 >= 0 { y2 } else { y2 - 399 }) / 400;
    let yoe = y2 - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    Some(days * 86_400 + hh * 3600 + mm * 60 + ss)
}

/// An amount in cents (Lemon Squeezy sends integers; tolerate decimals).
fn cents(v: &Value) -> i64 {
    v.as_i64().or_else(|| v.as_f64().map(|f| f.round() as i64)).or_else(|| v.as_str().and_then(|s| s.parse().ok())).unwrap_or(0)
}

fn text(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        _ => String::new(),
    }
}

fn card(attrs: &Value) -> String {
    let brand = attrs["card_brand"].as_str().unwrap_or("");
    let last4 = attrs["card_last_four"].as_str().unwrap_or("");
    match (brand.is_empty(), last4.is_empty()) {
        (true, true) => String::new(),
        _ => {
            let mut b = brand.to_string();
            if let Some(f) = b.get_mut(0..1) {
                f.make_ascii_uppercase();
            }
            format!("{b} •••• {last4}").trim().to_string()
        }
    }
}

/// Turns one webhook event into a payment record (None = not relevant).
/// `monthly` and `yearly` list the product variants of each kind (one course,
/// and optionally every course).
pub fn payment_from_event(monthly: &[&str], yearly: &[&str], event: &Value, t: i64) -> Option<Payment> {
    let meta = &event["meta"];
    let name = meta["event_name"].as_str().unwrap_or("");
    let custom = &meta["custom_data"];
    let user_id = text(&custom["user_id"]).parse::<i64>().ok();
    let course_id = text(&custom["course_id"]);
    let data = &event["data"];
    let kind = data["type"].as_str().unwrap_or("");
    let id = text(&data["id"]);
    let a = &data["attributes"];
    // Lemon Squeezy marks test purchases on the object (some payloads also in meta).
    let test_mode = a["test_mode"].as_bool().or_else(|| meta["test_mode"].as_bool()).unwrap_or(false);
    let raw = {
        let s = event.to_string();
        if s.len() > 20_000 { s.chars().take(20_000).collect() } else { s }
    };
    let created = a["created_at"].as_str().and_then(parse_time).unwrap_or(t);
    let base = Payment {
        provider: PROVIDER.into(),
        user_id,
        course_id,
        currency: a["currency"].as_str().unwrap_or("USD").to_string(),
        email: a["user_email"].as_str().unwrap_or("").to_lowercase(),
        test_mode,
        created_at: created,
        updated_at: t,
        customer_name: a["user_name"].as_str().unwrap_or("").to_string(),
        refunded_cents: cents(&a["refunded_amount"]),
        raw,
        ..Default::default()
    };
    match (name, kind) {
        ("order_created" | "order_refunded", "orders") => {
            let variant = text(&a["first_order_item"]["variant_id"]);
            if !yearly.contains(&variant.as_str()) {
                // Monthly plans are covered by the subscription events; other
                // products of the store don't open courses.
                return None;
            }
            Some(Payment {
                provider_ref: format!("ord_{id}"),
                plan: "yearly".into(),
                until: Some(created + YEAR_SECS),
                status: a["status"].as_str().unwrap_or("paid").to_string(),
                amount_cents: cents(&a["total"]),
                manage_url: a["urls"]["receipt"].as_str().unwrap_or("").to_string(),
                method: "Card / PayPal".into(),
                ..base
            })
        }
        (n, "subscriptions") if n.starts_with("subscription_") && monthly.contains(&text(&a["variant_id"]).as_str()) => Some(Payment {
            provider_ref: format!("sub_{id}"),
            plan: "monthly".into(),
            status: a["status"].as_str().unwrap_or("active").to_string(),
            amount_cents: 0, // the money is recorded per invoice
            until: a["ends_at"].as_str().and_then(parse_time).or_else(|| a["renews_at"].as_str().and_then(parse_time)),
            manage_url: a["urls"]["customer_portal"].as_str().unwrap_or("").to_string(),
            method: card(a),
            ..base
        }),
        (n, "subscription-invoices" | "subscription_invoices") if n.starts_with("subscription_payment") => Some(Payment {
            provider_ref: format!("inv_{id}"),
            plan: "monthly".into(),
            status: a["status"].as_str().unwrap_or("paid").to_string(),
            amount_cents: cents(&a["total"]),
            method: card(a),
            // remember which subscription it belongs to
            manage_url: format!("sub_{}", text(&a["subscription_id"])),
            ..base
        }),
        _ => None,
    }
}

/// `POST /api/payments/webhook`
pub async fn webhook(State(state): State<AppState>, headers: HeaderMap, body: Bytes) -> StatusCode {
    let Some(cfg) = config() else { return StatusCode::NOT_FOUND };
    let sig = headers.get("x-signature").and_then(|v| v.to_str().ok()).unwrap_or("");
    if !signature_ok(&cfg.webhook_secret, &body, sig) {
        tracing::warn!("payment webhook with a bad signature");
        return StatusCode::UNAUTHORIZED;
    }
    let Ok(event) = serde_json::from_slice::<Value>(&body) else { return StatusCode::BAD_REQUEST };
    if event["meta"]["event_name"].as_str().is_some_and(|n| n.starts_with("dispute_")) {
        return match dispute(&state, &cfg, &event, &body).await {
            Ok(()) => StatusCode::OK,
            Err(e) => {
                tracing::error!("saving a dispute failed: {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };
    }
    let Some(mut p) = payment_from_event(&cfg.monthly_variants(), &cfg.yearly_variants(), &event, now()) else {
        return StatusCode::OK;
    };
    // Invoices may arrive without our custom data: copy it from the subscription.
    if p.provider_ref.starts_with("inv_") && (p.user_id.is_none() || p.course_id.is_empty()) {
        if let Ok(rows) = store::all_payments(&state.db, 1000).await {
            if let Some(sub) = rows.iter().find(|r| r.provider_ref == p.manage_url) {
                p.user_id = p.user_id.or(sub.user_id);
                if p.course_id.is_empty() {
                    p.course_id = sub.course_id.clone();
                }
            }
        }
    }
    // Buying another year while one is still running adds a year to its end.
    if p.plan == "yearly" {
        if let (Some(uid), Ok(rows)) = (p.user_id, store::payments_for_user(&state.db, p.user_id.unwrap_or(0)).await) {
            let known = rows.iter().any(|r| r.provider_ref == p.provider_ref);
            let running = rows
                .iter()
                .filter(|r| r.user_id == Some(uid) && r.course_id == p.course_id && r.plan == "yearly" && r.provider_ref != p.provider_ref)
                .filter(|r| r.active(p.created_at))
                .filter_map(|r| r.until)
                .max();
            if let (false, Some(end)) = (known, running) {
                p.until = Some(end.max(p.created_at) + YEAR_SECS);
            }
        }
    }
    match store::upsert_payment(&state.db, &p).await {
        Ok(()) => {
            tracing::info!("payment {} {} {} for user {:?}", p.provider_ref, p.plan, p.status, p.user_id);
            StatusCode::OK
        }
        Err(e) => {
            tracing::error!("saving payment failed: {e}");
            StatusCode::INTERNAL_SERVER_ERROR // Lemon Squeezy retries
        }
    }
}

/// Link to a payment in the Lemon Squeezy dashboard (refunds are made there).
pub fn dashboard_url(provider_ref: &str) -> String {
    match provider_ref.split_once('_') {
        Some(("ord", id)) => format!("https://app.lemonsqueezy.com/orders/{id}"),
        Some(("sub", id)) => format!("https://app.lemonsqueezy.com/subscriptions/{id}"),
        _ => "https://app.lemonsqueezy.com/orders".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signatures() {
        let mut mac = Hmac::<Sha256>::new_from_slice(b"s3cret").unwrap();
        mac.update(b"{\"a\":1}");
        let good = hex::encode(mac.finalize().into_bytes());
        assert!(signature_ok("s3cret", b"{\"a\":1}", &good));
        assert!(!signature_ok("s3cret", b"{\"a\":2}", &good));
        assert!(!signature_ok("other", b"{\"a\":1}", &good));
        assert!(!signature_ok("s3cret", b"{\"a\":1}", "zz"));
    }

    #[test]
    fn times() {
        assert_eq!(parse_time("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(parse_time("2024-05-01T10:20:30.000000Z"), Some(1_714_558_830));
        assert_eq!(parse_time("bad"), None);
    }

    #[test]
    fn events() {
        let order = json!({
            "meta": { "event_name": "order_created", "test_mode": true,
                      "custom_data": { "user_id": "7", "course_id": "python" } },
            "data": { "type": "orders", "id": 55, "attributes": {
                "status": "paid", "total": 2500, "currency": "USD", "user_email": "A@B.com",
                "first_order_item": { "variant_id": 222 }, "urls": { "receipt": "https://r" },
                "created_at": "2024-05-01T10:20:30.000000Z" } }
        });
        let p = payment_from_event(&["111"], &["222"], &order, 9).unwrap();
        assert_eq!((p.provider_ref.as_str(), p.plan.as_str(), p.user_id, p.amount_cents), ("ord_55", "yearly", Some(7), 2500));
        assert_eq!(p.email, "a@b.com");
        let bought = p.created_at;
        assert_eq!(p.until, Some(bought + YEAR_SECS));
        assert!(p.active(bought + 10) && !p.active(bought + YEAR_SECS + 1));

        let sub = json!({
            "meta": { "event_name": "subscription_cancelled", "custom_data": { "user_id": 7, "course_id": "rust" } },
            "data": { "type": "subscriptions", "id": "9", "attributes": {
                "variant_id": 111, "status": "cancelled", "card_brand": "visa", "card_last_four": "4242",
                "renews_at": "2024-06-01T00:00:00Z", "ends_at": "2024-06-01T00:00:00Z",
                "urls": { "customer_portal": "https://portal" } } }
        });
        let s = payment_from_event(&["111"], &["222"], &sub, 9).unwrap();
        assert_eq!((s.provider_ref.as_str(), s.plan.as_str(), s.method.as_str()), ("sub_9", "monthly", "Visa •••• 4242"));
        assert!(s.active(parse_time("2024-05-20T00:00:00Z").unwrap()));
        assert!(!s.active(parse_time("2024-06-02T00:00:00Z").unwrap()));

        let monthly_order = json!({ "meta": { "event_name": "order_created" },
            "data": { "type": "orders", "id": 1, "attributes": { "first_order_item": { "variant_id": "111" } } } });
        assert!(payment_from_event(&["111"], &["222"], &monthly_order, 0).is_none());
        let other_product = json!({ "meta": { "event_name": "order_created", "custom_data": { "user_id": "7", "course_id": "python" } },
            "data": { "type": "orders", "id": 2, "attributes": { "status": "paid", "total": 900, "first_order_item": { "variant_id": "999" } } } });
        assert!(payment_from_event(&["111"], &["222"], &other_product, 0).is_none(), "other products never open courses");
        let refunded = json!({ "meta": { "event_name": "order_refunded", "custom_data": { "user_id": "7", "course_id": "python" } },
            "data": { "type": "orders", "id": 55, "attributes": { "status": "partial_refund", "total": 2500, "refunded_amount": 1000,
                "test_mode": true, "user_name": "Noa", "first_order_item": { "variant_id": 222 } } } });
        let r = payment_from_event(&["111"], &["222"], &refunded, 0).unwrap();
        assert_eq!((r.refunded_cents, r.refundable_cents(), r.test_mode, r.customer_name.as_str()), (1000, 1500, true, "Noa"));
    }
}

/// `GET /api/admin/payments-setup` – helps the admin fill in the settings:
/// lists the stores and product variants that the API key can see, and
/// which settings are present.
pub async fn setup_info(
    axum::extract::State(state): axum::extract::State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Value>> {
    let access = crate::routes::access_for(&state, &headers).await?;
    if !access.is_admin() {
        return Err(ApiError::Forbidden);
    }
    let present = json!({
        "LEMONSQUEEZY_API_KEY": env("LEMONSQUEEZY_API_KEY").is_some(),
        "LEMONSQUEEZY_STORE_ID": env("LEMONSQUEEZY_STORE_ID").is_some(),
        "LEMONSQUEEZY_VARIANT_MONTHLY": env("LEMONSQUEEZY_VARIANT_MONTHLY").is_some(),
        "LEMONSQUEEZY_VARIANT_YEARLY (or _LIFETIME)": env("LEMONSQUEEZY_VARIANT_YEARLY").or_else(|| env("LEMONSQUEEZY_VARIANT_LIFETIME")).is_some(),
        "LEMONSQUEEZY_VARIANT_ALL_MONTHLY": env("LEMONSQUEEZY_VARIANT_ALL_MONTHLY").is_some(),
        "LEMONSQUEEZY_VARIANT_ALL_YEARLY": env("LEMONSQUEEZY_VARIANT_ALL_YEARLY").is_some(),
        "LEMONSQUEEZY_WEBHOOK_SECRET": env("LEMONSQUEEZY_WEBHOOK_SECRET").is_some(),
    });
    let Some(key) = env("LEMONSQUEEZY_API_KEY") else {
        return Ok(Json(json!({ "present": present, "stores": [], "variants": [] })));
    };
    let get = |path: &'static str| {
        let key = key.clone();
        async move {
            http()
                .get(format!("https://api.lemonsqueezy.com/v1/{path}"))
                .bearer_auth(key)
                .header("Accept", "application/vnd.api+json")
                .send()
                .await
                .ok()?
                .json::<Value>()
                .await
                .ok()
        }
    };
    let stores = get("stores").await.unwrap_or_default();
    let products = get("products?page[size]=100").await.unwrap_or_default();
    let variants = get("variants?page[size]=100").await.unwrap_or_default();
    let product_name = |id: &str| {
        products["data"]
            .as_array()
            .and_then(|a| a.iter().find(|p| text(&p["id"]) == id))
            .and_then(|p| p["attributes"]["name"].as_str())
            .unwrap_or("")
            .to_string()
    };
    let stores: Vec<Value> = stores["data"]
        .as_array()
        .map(|a| a.iter().map(|s| json!({ "id": text(&s["id"]), "name": s["attributes"]["name"] })).collect())
        .unwrap_or_default();
    let variants: Vec<Value> = variants["data"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|v| {
                    let at = &v["attributes"];
                    json!({
                        "id": text(&v["id"]),
                        "product": product_name(&text(&at["product_id"])),
                        "name": at["name"],
                        "price_cents": at["price"],
                        "subscription": at["is_subscription"],
                        "interval": at["interval"],
                        "status": at["status"],
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(Json(json!({ "present": present, "stores": stores, "variants": variants })))
}

// ------------------------------------------------------- admin: refunds and cancellations

/// Payments being refunded or cancelled right now (blocks a double click).
static IN_FLIGHT: std::sync::Mutex<Vec<i64>> = std::sync::Mutex::new(Vec::new());

struct InFlight(i64);

impl InFlight {
    fn start(id: i64) -> Option<InFlight> {
        let mut busy = IN_FLIGHT.lock().ok()?;
        if busy.contains(&id) {
            return None;
        }
        busy.push(id);
        Some(InFlight(id))
    }
}

impl Drop for InFlight {
    fn drop(&mut self) {
        if let Ok(mut busy) = IN_FLIGHT.lock() {
            busy.retain(|i| *i != self.0);
        }
    }
}

fn dollars(cents: i64, currency: &str) -> String {
    format!("{:.2} {currency}", cents as f64 / 100.0)
}

/// One call to the Lemon Squeezy API. Never retried: a money action must
/// not run twice. When there is no clear answer the admin is told to check.
async fn ls_call(cfg: &Config, method: reqwest::Method, path: &str, body: Option<Value>) -> Result<Value, String> {
    let mut req = http()
        .request(method, format!("{}/{path}", api_base()))
        .bearer_auth(&cfg.api_key)
        .header("Accept", "application/vnd.api+json")
        .header("Content-Type", "application/vnd.api+json");
    if let Some(b) = body {
        req = req.body(b.to_string());
    }
    let resp = req.send().await.map_err(|e| {
        format!("No clear answer from Lemon Squeezy ({e}). Check this payment in Lemon Squeezy before trying again.")
    })?;
    let status = resp.status();
    let value: Value = resp.json().await.unwrap_or(Value::Null);
    if status.is_success() {
        Ok(value)
    } else {
        let detail = value["errors"][0]["detail"].as_str().or(value["errors"][0]["title"].as_str()).unwrap_or("");
        Err(format!("Lemon Squeezy refused ({status}): {detail}"))
    }
}

/// `POST /api/admin/refund` – refunds part or all of a charge.
pub async fn admin_refund(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<codesprout_core::api::AdminRefundRequest>,
) -> ApiResult<Json<Value>> {
    let admin = crate::admin::require_admin(&state, &headers).await?;
    let cfg = config().ok_or_else(|| ApiError::BadRequest("Payments are not connected".into()))?;
    let p = store::payment_by_id(&state.db, req.payment_id).await?.ok_or(ApiError::NotFound)?;
    let (kind, id) = match p.provider_ref.split_once('_') {
        Some(("ord", id)) => ("orders", id.to_string()),
        Some(("inv", id)) => ("subscription-invoices", id.to_string()),
        _ => return Err(ApiError::BadRequest("Only a charge can be refunded (cancel a subscription instead)".into())),
    };
    let refundable = p.refundable_cents();
    if refundable <= 0 {
        return Err(ApiError::BadRequest("Nothing is left to refund on this payment".into()));
    }
    if req.amount_cents <= 0 || req.amount_cents > refundable {
        return Err(ApiError::BadRequest(format!(
            "The refund must be between 0.01 and {}",
            dollars(refundable, &p.currency)
        )));
    }
    let _guard = InFlight::start(p.id)
        .ok_or_else(|| ApiError::Conflict("This payment is already being handled".into()))?;
    let who = admin.email.clone().unwrap_or_default();
    let target = format!("{} ({})", p.email, p.provider_ref);
    let body = json!({ "data": { "type": kind, "id": id, "attributes": { "amount": req.amount_cents } } });
    let t = now();
    match ls_call(&cfg, reqwest::Method::POST, &format!("{kind}/{id}/refund"), Some(body)).await {
        Ok(resp) => {
            let a = &resp["data"]["attributes"];
            let reported = cents(&a["refunded_amount"]);
            let refunded = if reported > 0 { reported } else { p.refunded_cents + req.amount_cents };
            let status = a["status"]
                .as_str()
                .map(str::to_string)
                .unwrap_or_else(|| if refunded >= p.amount_cents { "refunded".into() } else { "partial_refund".into() });
            store::update_payment_state(&state.db, p.id, &status, refunded, None, t).await?;
            store::log(&state.db, &who, "refund", &target, &format!("{} → {status}", dollars(req.amount_cents, &p.currency)), t)
                .await?;
            if req.lock {
                close_access(&state, &cfg, &p, &who, t).await?;
            }
            Ok(Json(json!({ "ok": true, "status": status, "refunded_cents": refunded })))
        }
        Err(msg) => {
            store::log(&state.db, &who, "refund failed", &target, &format!("{}: {msg}", dollars(req.amount_cents, &p.currency)), t)
                .await?;
            Err(ApiError::BadRequest(msg))
        }
    }
}

/// The order or invoice a dispute (chargeback) is about, as our reference.
pub fn dispute_target(event: &Value) -> Option<String> {
    let d = &event["data"];
    let a = &d["attributes"];
    let rel = |k: &str| text(&d["relationships"][k]["data"]["id"]);
    let order = [text(&a["order_id"]), rel("order")].into_iter().find(|s| !s.is_empty() && s != "null");
    let invoice = [text(&a["subscription_invoice_id"]), rel("subscription-invoice"), rel("subscription_invoice")]
        .into_iter()
        .find(|s| !s.is_empty() && s != "null");
    invoice.map(|i| format!("inv_{i}")).or_else(|| order.map(|o| format!("ord_{o}")))
}

/// `dispute_created`: the customer asked their bank to reverse a charge. The
/// payment stops opening anything (a monthly one also cancels its
/// subscription) and the admin sees it. `dispute_resolved` is only logged:
/// reopening is the admin's call.
async fn dispute(state: &AppState, cfg: &Config, event: &Value, body: &[u8]) -> ApiResult<()> {
    let name = event["meta"]["event_name"].as_str().unwrap_or("");
    let t = now();
    let detail = format!("{} {}", name, event["data"]["attributes"]["status"].as_str().unwrap_or(""));
    let Some(target) = dispute_target(event) else {
        store::log(&state.db, "Lemon Squeezy", "dispute (no matching payment)", &text(&event["data"]["id"]), &detail, t).await?;
        return Ok(());
    };
    if name != "dispute_created" {
        store::log(&state.db, "Lemon Squeezy", "dispute update", &target, &detail, t).await?;
        return Ok(());
    }
    let raw = String::from_utf8_lossy(body).to_string();
    match store::mark_disputed(&state.db, &target, &raw, t).await? {
        Some(p) => {
            store::log(&state.db, "Lemon Squeezy", "dispute", &format!("{} ({target})", p.email), &detail, t).await?;
            if p.provider_ref.starts_with("inv_") {
                close_access(state, cfg, &p, "Lemon Squeezy", t).await?;
            }
        }
        None => store::log(&state.db, "Lemon Squeezy", "dispute (no matching payment)", &target, &detail, t).await?,
    }
    Ok(())
}

/// Ends what a payment opened, right now: a yearly purchase closes; for a
/// monthly charge its subscription is cancelled at Lemon Squeezy (no more
/// charges) and closed. Kept closed whatever events arrive later.
async fn close_access(state: &AppState, cfg: &Config, p: &store::Payment, who: &str, t: i64) -> ApiResult<()> {
    let target = format!("{} ({})", p.email, p.provider_ref);
    let closing = if p.provider_ref.starts_with("inv_") {
        store::payment_by_ref(&state.db, &p.manage_url).await?
    } else {
        Some(p.clone())
    };
    let Some(c) = closing else { return Ok(()) };
    if c.provider_ref.starts_with("sub_") && c.can_cancel() {
        let id = c.provider_ref.trim_start_matches("sub_");
        match ls_call(cfg, reqwest::Method::DELETE, &format!("subscriptions/{id}"), None).await {
            Ok(resp) => {
                let status = resp["data"]["attributes"]["status"].as_str().unwrap_or("cancelled").to_string();
                store::update_payment_state(&state.db, c.id, &status, c.refunded_cents, None, t).await?;
                store::log(&state.db, who, "cancel subscription", &target, &status, t).await?;
            }
            Err(msg) => store::log(&state.db, who, "cancel failed", &target, &msg, t).await?,
        }
    }
    store::close_payment(&state.db, c.id, t).await?;
    store::log(&state.db, who, "closed after refund", &target, &c.course_id, t).await?;
    Ok(())
}

/// `POST /api/admin/cancel` – cancels a subscription. Lemon Squeezy stops
/// charging; the course stays open until the end of the period already paid.
pub async fn admin_cancel(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<codesprout_core::api::AdminCancelRequest>,
) -> ApiResult<Json<Value>> {
    let admin = crate::admin::require_admin(&state, &headers).await?;
    let p = store::payment_by_id(&state.db, req.payment_id).await?.ok_or(ApiError::NotFound)?;
    let who = admin.email.clone().unwrap_or_default();
    cancel_subscription(&state, p, &who).await
}

/// `POST /api/me/cancel`: a user cancels their own subscription. Charges stop
/// right away; the course stays open until the end of the paid month.
pub async fn user_cancel(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<codesprout_core::api::AdminCancelRequest>,
) -> ApiResult<Json<Value>> {
    let user_id = crate::routes::require_user(&state, &headers).await?;
    let p = store::payment_by_id(&state.db, req.payment_id).await?.ok_or(ApiError::NotFound)?;
    // Only the owner may cancel; anyone else gets "not found".
    if p.user_id != Some(user_id) {
        return Err(ApiError::NotFound);
    }
    cancel_subscription(&state, p, &format!("user {user_id}")).await
}

async fn cancel_subscription(state: &AppState, p: store::Payment, who: &str) -> ApiResult<Json<Value>> {
    let cfg = config().ok_or_else(|| ApiError::BadRequest("Payments are not connected".into()))?;
    if !p.can_cancel() {
        return Err(ApiError::BadRequest("This is not an active subscription".into()));
    }
    let id = p.provider_ref.trim_start_matches("sub_").to_string();
    let _guard = InFlight::start(p.id)
        .ok_or_else(|| ApiError::Conflict("This payment is already being handled".into()))?;
    let target = format!("{} ({})", p.email, p.provider_ref);
    let t = now();
    match ls_call(&cfg, reqwest::Method::DELETE, &format!("subscriptions/{id}"), None).await {
        Ok(resp) => {
            let a = &resp["data"]["attributes"];
            let status = a["status"].as_str().unwrap_or("cancelled").to_string();
            let until = a["ends_at"].as_str().and_then(parse_time);
            store::update_payment_state(&state.db, p.id, &status, p.refunded_cents, until, t).await?;
            store::log(&state.db, who, "cancel subscription", &target, &status, t).await?;
            Ok(Json(json!({ "ok": true, "status": status, "until": until })))
        }
        Err(msg) => {
            store::log(&state.db, who, "cancel failed", &target, &msg, t).await?;
            Err(ApiError::BadRequest(msg))
        }
    }
}

/// `GET /api/me/purchases`
pub async fn my_purchases(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<codesprout_core::api::MyPurchase>>> {
    let user_id = crate::routes::require_user(&state, &headers).await?;
    let rows = store::payments_for_user(&state.db, user_id).await?;
    let mut out: Vec<codesprout_core::api::MyPurchase> = rows
        .iter()
        .filter(|p| !p.provider_ref.starts_with("inv_"))
        .map(|p| {
            // A subscription's amount and refunds live on its invoices.
            let invoices = rows.iter().filter(|i| i.provider_ref.starts_with("inv_") && i.manage_url == p.provider_ref);
            let (paid, back) = if p.provider_ref.starts_with("sub_") {
                invoices.fold((0, 0), |(a, r), i| (a + i.amount_cents, r + i.refunded_cents))
            } else {
                (p.amount_cents, p.refunded_cents)
            };
            codesprout_core::api::MyPurchase {
                id: p.id,
                course_id: p.course_id.clone(),
                plan: p.plan.clone(),
                status: p.status.clone(),
                amount_cents: paid,
                refunded_cents: back,
                currency: p.currency.clone(),
                created_at: p.created_at,
                until: p.until,
                can_cancel: p.can_cancel(),
                test_mode: p.test_mode,
            }
        })
        .collect();
    out.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(Json(out))
}

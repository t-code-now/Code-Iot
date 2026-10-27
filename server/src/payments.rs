//! Payments through Lemon Squeezy (the "merchant of record": it charges the
//! customer, handles VAT/sales tax worldwide and pays out to the seller).
//!
//! * `POST /api/payments/checkout` creates a checkout for one course and one
//!   plan ($5/month subscription or $25 one-time) and returns its URL.
//! * `POST /api/payments/webhook` receives Lemon Squeezy's events (signed
//!   with HMAC-SHA256) and records them; access is derived from the records.
//!
//! Settings (Render → Environment): `LEMONSQUEEZY_API_KEY`,
//! `LEMONSQUEEZY_STORE_ID`, `LEMONSQUEEZY_VARIANT_MONTHLY`,
//! `LEMONSQUEEZY_VARIANT_LIFETIME`, `LEMONSQUEEZY_WEBHOOK_SECRET`.

use std::time::Duration;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use codesprout_core::access::{PRICE_LIFETIME, PRICE_MONTHLY};
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
    pub variant_lifetime: String,
    pub webhook_secret: String,
    pub public_url: String,
}

fn env(k: &str) -> Option<String> {
    std::env::var(k).ok().map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

pub fn config() -> Option<Config> {
    Some(Config {
        api_key: env("LEMONSQUEEZY_API_KEY")?,
        store_id: env("LEMONSQUEEZY_STORE_ID")?,
        variant_monthly: env("LEMONSQUEEZY_VARIANT_MONTHLY")?,
        variant_lifetime: env("LEMONSQUEEZY_VARIANT_LIFETIME")?,
        webhook_secret: env("LEMONSQUEEZY_WEBHOOK_SECRET")?,
        public_url: env("PUBLIC_URL")
            .or_else(|| env("RENDER_EXTERNAL_URL"))
            .unwrap_or_else(|| "https://code-iot.onrender.com".into())
            .trim_end_matches('/')
            .to_string(),
    })
}

/// `GET /api/payments/config`
pub async fn config_info() -> Json<Value> {
    Json(json!({ "enabled": config().is_some(), "monthly": PRICE_MONTHLY, "lifetime": PRICE_LIFETIME }))
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
    let course = find_course(&req.course_id).ok_or(ApiError::NotFound)?;
    let variant = match req.plan.as_str() {
        "monthly" => &cfg.variant_monthly,
        "lifetime" => &cfg.variant_lifetime,
        _ => return Err(ApiError::BadRequest("Unknown plan".into())),
    };
    let email = store::user_email(&state.db, user_id).await?.unwrap_or_default();
    let mut checkout_data = json!({
        "custom": { "user_id": user_id.to_string(), "course_id": course.id, "plan": req.plan }
    });
    if !email.is_empty() {
        checkout_data["email"] = json!(email);
    }
    let body = json!({
        "data": {
            "type": "checkouts",
            "attributes": {
                "checkout_data": checkout_data,
                "product_options": {
                    "redirect_url": format!("{}/unlock/{}?paid=1", cfg.public_url, course.id),
                    "description": format!("Code-Iot · {}", course.title)
                }
            },
            "relationships": {
                "store": { "data": { "type": "stores", "id": cfg.store_id } },
                "variant": { "data": { "type": "variants", "id": variant } }
            }
        }
    });
    let resp: Value = http()
        .post("https://api.lemonsqueezy.com/v1/checkouts")
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
pub fn payment_from_event(cfg_monthly: &str, cfg_lifetime: &str, event: &Value, t: i64) -> Option<Payment> {
    let meta = &event["meta"];
    let name = meta["event_name"].as_str().unwrap_or("");
    let custom = &meta["custom_data"];
    let user_id = text(&custom["user_id"]).parse::<i64>().ok();
    let course_id = text(&custom["course_id"]);
    let test_mode = meta["test_mode"].as_bool().unwrap_or(false);
    let data = &event["data"];
    let kind = data["type"].as_str().unwrap_or("");
    let id = text(&data["id"]);
    let a = &data["attributes"];
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
        ..Default::default()
    };
    match (name, kind) {
        ("order_created" | "order_refunded", "orders") => {
            let variant = text(&a["first_order_item"]["variant_id"]);
            if variant == cfg_monthly {
                return None; // the subscription events cover monthly plans
            }
            let _ = cfg_lifetime;
            Some(Payment {
                provider_ref: format!("ord_{id}"),
                plan: "lifetime".into(),
                status: a["status"].as_str().unwrap_or("paid").to_string(),
                amount_cents: a["total"].as_i64().unwrap_or(0),
                manage_url: a["urls"]["receipt"].as_str().unwrap_or("").to_string(),
                method: "Card / PayPal".into(),
                ..base
            })
        }
        (n, "subscriptions") if n.starts_with("subscription_") => Some(Payment {
            provider_ref: format!("sub_{id}"),
            plan: "monthly".into(),
            status: a["status"].as_str().unwrap_or("active").to_string(),
            amount_cents: 0, // the money is recorded per invoice
            until: a["ends_at"].as_str().and_then(parse_time).or_else(|| a["renews_at"].as_str().and_then(parse_time)),
            manage_url: a["urls"]["customer_portal"].as_str().unwrap_or("").to_string(),
            method: card(a),
            ..base
        }),
        (n, "subscription-invoices") if n.starts_with("subscription_payment") => Some(Payment {
            provider_ref: format!("inv_{id}"),
            plan: "monthly".into(),
            status: a["status"].as_str().unwrap_or("paid").to_string(),
            amount_cents: a["total"].as_i64().unwrap_or(0),
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
    let Some(mut p) = payment_from_event(&cfg.variant_monthly, &cfg.variant_lifetime, &event, now()) else {
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
        let p = payment_from_event("111", "222", &order, 9).unwrap();
        assert_eq!((p.provider_ref.as_str(), p.plan.as_str(), p.user_id, p.amount_cents), ("ord_55", "lifetime", Some(7), 2500));
        assert_eq!(p.email, "a@b.com");
        assert!(p.active(0));

        let sub = json!({
            "meta": { "event_name": "subscription_cancelled", "custom_data": { "user_id": 7, "course_id": "rust" } },
            "data": { "type": "subscriptions", "id": "9", "attributes": {
                "status": "cancelled", "card_brand": "visa", "card_last_four": "4242",
                "renews_at": "2024-06-01T00:00:00Z", "ends_at": "2024-06-01T00:00:00Z",
                "urls": { "customer_portal": "https://portal" } } }
        });
        let s = payment_from_event("111", "222", &sub, 9).unwrap();
        assert_eq!((s.provider_ref.as_str(), s.plan.as_str(), s.method.as_str()), ("sub_9", "monthly", "Visa •••• 4242"));
        assert!(s.active(parse_time("2024-05-20T00:00:00Z").unwrap()));
        assert!(!s.active(parse_time("2024-06-02T00:00:00Z").unwrap()));

        let monthly_order = json!({ "meta": { "event_name": "order_created" },
            "data": { "type": "orders", "id": 1, "attributes": { "first_order_item": { "variant_id": "111" } } } });
        assert!(payment_from_event("111", "222", &monthly_order, 0).is_none());
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
        "LEMONSQUEEZY_VARIANT_LIFETIME": env("LEMONSQUEEZY_VARIANT_LIFETIME").is_some(),
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

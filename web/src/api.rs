//! Thin HTTP client for the Code-Iot API (under /api).
//!
//! The website talks to its own origin. The Android app is built with
//! `CODESPROUT_API=https://your-server` so it talks to the hosted server.

use codesprout_core::api::*;
use gloo_net::http::{Request, RequestBuilder, Response};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::i18n::Lang;

#[derive(Debug, Clone, PartialEq)]
pub struct ApiError {
    pub status: u16,
    pub message: String,
}

impl ApiError {
    pub fn is_unauthorized(&self) -> bool {
        self.status == 401
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

fn network(e: gloo_net::Error) -> ApiError {
    ApiError { status: 0, message: format!("Can't reach the server ({e})") }
}

fn authorize(req: RequestBuilder, token: &Option<String>) -> RequestBuilder {
    match token {
        Some(t) => req.header("Authorization", &format!("Bearer {t}")),
        None => req,
    }
}

async fn handle<T: DeserializeOwned>(resp: Response) -> ApiResult<T> {
    let status = resp.status();
    if resp.ok() {
        resp.json::<T>().await.map_err(|e| ApiError { status, message: e.to_string() })
    } else {
        let message = match resp.json::<ApiErrorBody>().await {
            Ok(body) => body.error,
            Err(_) => format!("Request failed ({status})"),
        };
        Err(ApiError { status, message })
    }
}

pub fn url(path: &str) -> String {
    let base = option_env!("CODESPROUT_API").unwrap_or("");
    format!("{}{path}", base.trim_end_matches('/'))
}

async fn get<T: DeserializeOwned>(path: &str, token: &Option<String>) -> ApiResult<T> {
    let resp = authorize(Request::get(&url(path)), token).send().await.map_err(network)?;
    handle(resp).await
}

async fn post<B: Serialize, T: DeserializeOwned>(path: &str, token: &Option<String>, body: &B) -> ApiResult<T> {
    let req = authorize(Request::post(&url(path)), token).json(body).map_err(network)?;
    let resp = req.send().await.map_err(network)?;
    handle(resp).await
}

async fn put<B: Serialize, T: DeserializeOwned>(path: &str, token: &Option<String>, body: &B) -> ApiResult<T> {
    let req = authorize(Request::put(&url(path)), token).json(body).map_err(network)?;
    let resp = req.send().await.map_err(network)?;
    handle(resp).await
}

/// Set (or with None, remove) the signed-in user's profile picture.
pub async fn set_avatar(token: Option<String>, avatar: Option<String>) -> ApiResult<Profile> {
    put("/api/me/avatar", &token, &AvatarRequest { avatar }).await
}

/// Which extra sign-in methods the server offers, e.g. `{"google": true}`.
pub async fn providers() -> ApiResult<serde_json::Value> {
    get("/api/auth/providers", &None).await
}

/// Signs in with the ID token from the Android app's in-app Google sheet.
pub async fn google_id_token(id_token: String) -> ApiResult<serde_json::Value> {
    post("/api/auth/google/id-token", &None, &serde_json::json!({ "id_token": id_token })).await
}

/// Has the browser sign-in with id `poll` finished?
pub async fn google_poll(poll: &str) -> ApiResult<serde_json::Value> {
    get(&format!("/api/auth/google/poll?id={poll}"), &None).await
}

pub async fn signup(username: String, password: String) -> ApiResult<AuthResponse> {
    post("/api/auth/signup", &None, &AuthRequest { username, password }).await
}

pub async fn login(username: String, password: String) -> ApiResult<AuthResponse> {
    post("/api/auth/login", &None, &AuthRequest { username, password }).await
}

pub async fn logout(token: Option<String>) -> ApiResult<serde_json::Value> {
    post("/api/auth/logout", &token, &serde_json::json!({})).await
}

pub async fn me(token: Option<String>) -> ApiResult<Profile> {
    get("/api/me", &token).await
}

pub async fn courses(token: Option<String>, lang: Lang) -> ApiResult<Vec<CourseSummary>> {
    get(&format!("/api/courses?lang={}", lang.code()), &token).await
}

pub async fn course(id: String, token: Option<String>, lang: Lang) -> ApiResult<CourseDetail> {
    get(&format!("/api/courses/{id}?lang={}", lang.code()), &token).await
}

pub async fn lesson(id: String, token: Option<String>, lang: Lang) -> ApiResult<LessonDto> {
    get(&format!("/api/lessons/{id}?lang={}", lang.code()), &token).await
}

pub async fn check(
    lesson_id: String,
    token: Option<String>,
    step: usize,
    answer: Answer,
    lang: Lang,
) -> ApiResult<CheckResponse> {
    post(&format!("/api/lessons/{lesson_id}/check?lang={}", lang.code()), &token, &CheckRequest { step, answer }).await
}

pub async fn complete(lesson_id: String, token: Option<String>, correct: u32) -> ApiResult<CompleteResponse> {
    // getTimezoneOffset() is minutes WEST of UTC; the API wants minutes east.
    let tz_offset_minutes = -(js_sys::Date::new_0().get_timezone_offset() as i32);
    post(
        &format!("/api/lessons/{lesson_id}/complete"),
        &token,
        &CompleteRequest { correct, tz_offset_minutes },
    )
    .await
}

pub async fn leaderboard(scope: &str, token: Option<String>) -> ApiResult<Leaderboard> {
    get(&format!("/api/leaderboard?scope={scope}"), &token).await
}

/// Result of running playground code on the server (C++, Rust, Arduino).
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
pub struct RunResult {
    pub status: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i64>,
}

#[derive(serde::Serialize)]
struct RunBody<'a> {
    language: &'a str,
    code: &'a str,
    stdin: &'a str,
}

pub async fn run(language: String, code: String, stdin: String) -> ApiResult<RunResult> {
    post("/api/run", &None, &RunBody { language: &language, code: &code, stdin: &stdin }).await
}

// ---------------------------------------------------------------- payments

/// `{"enabled": bool, "paused": bool, "monthly": "$5.99", "yearly": "$29"}`
pub async fn payments_config() -> ApiResult<serde_json::Value> {
    get("/api/payments/config", &None).await
}

pub async fn checkout(token: Option<String>, course_id: String, plan: &str) -> ApiResult<CheckoutResponse> {
    post("/api/payments/checkout", &token, &CheckoutRequest { course_id, plan: plan.to_string() }).await
}

// ---------------------------------------------------------------- admin

pub async fn admin_overview(token: Option<String>) -> ApiResult<AdminOverview> {
    get("/api/admin/overview", &token).await
}

pub async fn admin_users(token: Option<String>, q: String) -> ApiResult<Vec<AdminUser>> {
    let q = String::from(js_sys::encode_uri_component(&q));
    get(&format!("/api/admin/users?q={q}"), &token).await
}

pub async fn admin_payments(token: Option<String>) -> ApiResult<Vec<AdminPayment>> {
    get("/api/admin/payments", &token).await
}

pub async fn admin_log(token: Option<String>) -> ApiResult<Vec<AdminLogEntry>> {
    get("/api/admin/log", &token).await
}

pub async fn admin_access(token: Option<String>, user_id: i64, course_id: String, mode: &str) -> ApiResult<serde_json::Value> {
    post("/api/admin/access", &token, &AdminAccessRequest { user_id, course_id, mode: mode.to_string() }).await
}

pub async fn admin_ban(token: Option<String>, user_id: i64, banned: bool) -> ApiResult<serde_json::Value> {
    post("/api/admin/ban", &token, &AdminBanRequest { user_id, banned }).await
}

pub async fn admin_delete(token: Option<String>, user_id: i64) -> ApiResult<serde_json::Value> {
    post("/api/admin/delete", &token, &AdminDeleteRequest { user_id }).await
}

pub async fn admin_payments_setup(token: Option<String>) -> ApiResult<serde_json::Value> {
    get("/api/admin/payments-setup", &token).await
}

pub async fn set_contact(token: Option<String>, country: String, phone: String) -> ApiResult<Profile> {
    put("/api/me/contact", &token, &ContactRequest { country, phone }).await
}

pub async fn admin_refund(token: Option<String>, payment_id: i64, amount_cents: i64, lock: bool) -> ApiResult<serde_json::Value> {
    post("/api/admin/refund", &token, &AdminRefundRequest { payment_id, amount_cents, lock }).await
}

pub async fn admin_cancel(token: Option<String>, payment_id: i64) -> ApiResult<serde_json::Value> {
    post("/api/admin/cancel", &token, &AdminCancelRequest { payment_id }).await
}

pub async fn admin_settings(token: Option<String>, sales_paused: bool) -> ApiResult<serde_json::Value> {
    post("/api/admin/settings", &token, &AdminSettingsRequest { sales_paused }).await
}

/// The signed-in user's own purchases.
pub async fn my_purchases(token: Option<String>) -> ApiResult<Vec<MyPurchase>> {
    get("/api/me/purchases", &token).await
}

/// Cancel one of the user's own subscriptions.
pub async fn cancel_my_subscription(token: Option<String>, payment_id: i64) -> ApiResult<serde_json::Value> {
    post("/api/me/cancel", &token, &AdminCancelRequest { payment_id }).await
}

/// The courses the user asked to hear about.
pub async fn my_waitlist(token: Option<String>) -> ApiResult<Vec<String>> {
    get("/api/me/waitlist", &token).await
}

/// Join (or leave) a course's waiting list; returns the user's list.
pub async fn set_waitlist(token: Option<String>, course_id: String, join: bool) -> ApiResult<Vec<String>> {
    post("/api/me/waitlist", &token, &WaitlistRequest { course_id, join }).await
}

pub async fn admin_waitlist(token: Option<String>) -> ApiResult<Vec<AdminWaitEntry>> {
    get("/api/admin/waitlist", &token).await
}

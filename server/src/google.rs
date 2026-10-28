//! "Continue with Google" (OAuth 2.0 authorization-code flow, run by the
//! server so the client secret never leaves it).
//!
//! Website: `/api/auth/google/start` → Google → `/api/auth/google/callback`
//! → back to `/auth/done#token=…`.
//!
//! Android app: Google doesn't allow sign-in inside an embedded web view, so
//! the app opens the start URL in the phone's browser with a random `poll`
//! id and asks `/api/auth/google/poll?id=…` until the sign-in has finished.
//!
//! Enabled when `GOOGLE_CLIENT_ID` and `GOOGLE_CLIENT_SECRET` are set. The
//! public address comes from `PUBLIC_URL` (or Render's `RENDER_EXTERNAL_URL`).

use std::time::Duration;

use axum::extract::{Query, State};
use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::db;
use crate::error::{ApiError, ApiResult};
use crate::routes::{now, random_hex, AppState};

const STATE_TTL: i64 = 15 * 60;

struct Config {
    client_id: String,
    client_secret: String,
    public_url: String,
}

fn config() -> Option<Config> {
    let get = |k: &str| std::env::var(k).ok().map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
    Some(Config {
        client_id: get("GOOGLE_CLIENT_ID")?,
        client_secret: get("GOOGLE_CLIENT_SECRET")?,
        public_url: get("PUBLIC_URL")
            .or_else(|| get("RENDER_EXTERNAL_URL"))?
            .trim_end_matches('/')
            .to_string(),
    })
}

impl Config {
    fn redirect_uri(&self) -> String {
        format!("{}/api/auth/google/callback", self.public_url)
    }
}

fn enc(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

fn http() -> &'static reqwest::Client {
    static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    CLIENT.get_or_init(|| reqwest::Client::builder().timeout(Duration::from_secs(20)).build().expect("http client"))
}

/// `GET /api/auth/providers` – which extra sign-in methods are available.
pub async fn providers() -> Json<Value> {
    Json(json!({ "google": config().is_some() }))
}

#[derive(Deserialize)]
pub struct StartQuery {
    poll: Option<String>,
}

fn valid_poll(id: &str) -> bool {
    (16..=64).contains(&id.len()) && id.bytes().all(|b| b.is_ascii_alphanumeric())
}

/// `GET /api/auth/google/start[?poll=<id>]` – sends the browser to Google.
pub async fn start(State(state): State<AppState>, Query(q): Query<StartQuery>) -> ApiResult<Response> {
    let cfg = config().ok_or(ApiError::NotFound)?;
    let poll = q.poll.filter(|p| valid_poll(p));
    let st = random_hex(16);
    let t = now();
    db::save_oauth_state(&state.db, &st, poll.as_deref(), t).await?;
    db::purge_oauth(&state.db, t - STATE_TTL).await?;
    let url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}&prompt=select_account",
        enc(&cfg.client_id),
        enc(&cfg.redirect_uri()),
        enc("openid email profile"),
        st,
    );
    Ok(Redirect::to(&url).into_response())
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

struct GoogleUser {
    sub: String,
    email: String,
    name: String,
}

async fn exchange(cfg: &Config, code: &str) -> Result<GoogleUser, String> {
    let body = format!(
        "code={}&client_id={}&client_secret={}&redirect_uri={}&grant_type=authorization_code",
        enc(code),
        enc(&cfg.client_id),
        enc(&cfg.client_secret),
        enc(&cfg.redirect_uri()),
    );
    let tokens: Value = http()
        .post("https://oauth2.googleapis.com/token")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    let access = tokens["access_token"]
        .as_str()
        .ok_or_else(|| format!("Google refused the sign-in ({})", tokens["error"].as_str().unwrap_or("no token")))?;
    let info: Value = http()
        .get("https://openidconnect.googleapis.com/v1/userinfo")
        .bearer_auth(access)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    let sub = info["sub"].as_str().ok_or("Google didn't return an account id")?.to_string();
    // Only a verified address is kept: it identifies the admin account.
    let verified = info["email_verified"].as_bool().unwrap_or(false)
        || info["email_verified"].as_str() == Some("true");
    Ok(GoogleUser {
        sub,
        email: if verified { info["email"].as_str().unwrap_or("").to_lowercase() } else { String::new() },
        name: info["given_name"].as_str().or(info["name"].as_str()).unwrap_or("").to_string(),
    })
}

/// A free username built from the person's name or e-mail (3–20 letters,
/// digits or underscores, like regular sign-ups).
fn base_username(user: &GoogleUser) -> String {
    let source = if user.name.chars().any(|c| c.is_ascii_alphanumeric()) {
        user.name.clone()
    } else {
        user.email.split('@').next().unwrap_or("").to_string()
    };
    let mut name: String = source
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string();
    while name.contains("__") {
        name = name.replace("__", "_");
    }
    name.truncate(14);
    if name.len() < 3 {
        name = "coder".into();
    }
    name
}

/// Finds or creates the Code-Iot account for a Google account. Returns
/// (user id, created now).
async fn account_for(state: &AppState, user: &GoogleUser) -> ApiResult<(i64, bool)> {
    if let Some(id) = db::google_user(&state.db, &user.sub).await? {
        // Keep the stored e-mail current (it identifies the admin).
        db::link_google(&state.db, &user.sub, id, &user.email).await?;
        return Ok((id, false));
    }
    let base = base_username(user);
    // Google accounts have no password: this hash never matches one.
    let no_password = "!google";
    for attempt in 0..30 {
        let candidate = if attempt == 0 {
            base.clone()
        } else {
            let n = u32::from_str_radix(&random_hex(2), 16).unwrap_or(attempt) % 10_000;
            format!("{base}{n}")
        };
        if let Some(id) = db::create_user(&state.db, &candidate, no_password, now()).await? {
            db::link_google(&state.db, &user.sub, id, &user.email).await?;
            return Ok((id, true));
        }
    }
    Err(ApiError::Internal("could not pick a username".into()))
}

/// Brings the Android app back to the front after signing in in the phone's
/// browser (an Android intent link to the app's package).
const APP_INTENT: &str = "intent://open#Intent;package=com.tcodenow.codeiot;end";

fn app_page(ok: bool, detail: &str) -> Response {
    let (title, he, en) = if ok {
        ("✅", "התחברת ל-Code-Iot! אפשר לחזור לאפליקציה.", "You're signed in to Code-Iot. You can go back to the app now.")
    } else {
        ("⚠️", "ההתחברות עם Google לא הצליחה. חזרו לאפליקציה ונסו שוב.", "Signing in with Google didn't work. Go back to the app and try again.")
    };
    let detail = detail.replace('<', "&lt;");
    Html(format!(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
         <title>Code-Iot</title><style>body{{margin:0;min-height:100vh;display:grid;place-items:center;font-family:system-ui,sans-serif;\
         background:#8fb4e6;color:#1a1f3d;text-align:center}}.card{{background:#fff;border-radius:20px;padding:28px 22px;max-width:340px;\
         box-shadow:0 8px 30px rgba(0,0,0,.15)}}.big{{font-size:52px}}p{{font-size:18px;line-height:1.5;margin:10px 0}}small{{color:#777}}\
         .back{{display:block;margin:18px auto 6px;padding:14px 20px;border-radius:14px;background:#ffc72c;color:#1a1f3d;font-weight:800;\
         font-size:18px;text-decoration:none;box-shadow:0 4px 0 #c48a00}}</style></head>\
         <body><div class=\"card\"><div class=\"big\">{title}</div><p dir=\"rtl\"><b>{he}</b></p><p>{en}</p>\
         <a class=\"back\" href=\"{back}\">חזרה לאפליקציה · Back to the app</a><small>{detail}</small></div>\
         <script>setTimeout(function(){{location.href={back_js};}},300);</script></body></html>",
        back = APP_INTENT,
        back_js = format!("\"{APP_INTENT}\""),
    ))
    .into_response()
}

/// `GET /api/auth/google/callback` – Google sends the browser back here.
pub async fn callback(State(state): State<AppState>, Query(q): Query<CallbackQuery>) -> Response {
    let Some(cfg) = config() else { return StatusCode::NOT_FOUND.into_response() };
    let web_fail = |msg: &str| Redirect::to(&format!("/auth/done#error={}", enc(msg))).into_response();

    let Some(st) = q.state.as_deref() else { return web_fail("missing state") };
    let pending = match db::take_oauth_state(&state.db, st).await {
        Ok(Some(p)) => p,
        Ok(None) => return web_fail("This sign-in link has expired. Please try again."),
        Err(e) => return web_fail(&e.to_string()),
    };
    let (poll, created_at) = pending;
    let expired = now() - created_at > STATE_TTL;

    let result: Result<(String, bool), String> = async {
        if expired {
            return Err("This sign-in link has expired. Please try again.".to_string());
        }
        if let Some(err) = q.error.as_deref() {
            return Err(if err == "access_denied" { "Sign-in was cancelled.".to_string() } else { err.to_string() });
        }
        let code = q.code.as_deref().ok_or("Google didn't send a sign-in code")?;
        let user = exchange(&cfg, code).await?;
        let (user_id, is_new) = account_for(&state, &user).await.map_err(|e| format!("{e:?}"))?;
        if crate::store::is_banned(&state.db, user_id).await.map_err(|e| e.to_string())? {
            return Err("This account is suspended.".to_string());
        }
        let token = random_hex(32);
        db::create_session(&state.db, &token, user_id, now()).await.map_err(|e| e.to_string())?;
        Ok((token, is_new))
    }
    .await;

    match (poll, result) {
        (Some(poll), Ok((token, is_new))) => match db::save_oauth_result(&state.db, &poll, Some(&token), is_new, None, now()).await {
            Ok(()) => app_page(true, ""),
            Err(e) => app_page(false, &e.to_string()),
        },
        (Some(poll), Err(msg)) => {
            let _ = db::save_oauth_result(&state.db, &poll, None, false, Some(&msg), now()).await;
            app_page(false, &msg)
        }
        (None, Ok((token, is_new))) => {
            Redirect::to(&format!("/auth/done#token={token}&new={}", if is_new { 1 } else { 0 })).into_response()
        }
        (None, Err(msg)) => web_fail(&msg),
    }
}

#[derive(Deserialize)]
pub struct PollQuery {
    id: String,
}

/// `GET /api/auth/google/poll?id=…` – the app asks whether the browser
/// sign-in has finished. A finished result is handed out only once.
pub async fn poll(State(state): State<AppState>, Query(q): Query<PollQuery>) -> ApiResult<Json<Value>> {
    if !valid_poll(&q.id) {
        return Err(ApiError::BadRequest("bad id".into()));
    }
    Ok(Json(match db::take_oauth_result(&state.db, &q.id).await? {
        None => json!({ "status": "pending" }),
        Some((Some(token), is_new, _)) => json!({ "status": "done", "token": token, "is_new": is_new }),
        Some((None, _, err)) => json!({ "status": "error", "message": err.unwrap_or_default() }),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usernames_from_google_profiles() {
        let u = |name: &str, email: &str| GoogleUser { sub: "1".into(), email: email.into(), name: name.into() };
        assert_eq!(base_username(&u("Noa", "x@gmail.com")), "noa");
        assert_eq!(base_username(&u("שגיא", "sagi.m@gmail.com")), "sagi_m");
        assert_eq!(base_username(&u("", "a@b.c")), "coder");
        assert_eq!(base_username(&u("Jean-Luc Picard Enterprise", "")), "jean_luc_picar");
        assert_eq!(enc("a b&c"), "a%20b%26c");
    }
}

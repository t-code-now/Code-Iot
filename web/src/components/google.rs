//! "Continue with Google". On the website the page goes to Google and comes
//! back to `/auth/done`. In the Android app, Google's own account sheet opens
//! over the app (Android's Credential Manager) and the server checks the ID
//! token it returns: no browser. If that isn't available (an older app, or
//! no Google services), the phone's browser is used and the app waits until
//! the server reports that the sign-in finished.

use leptos::*;
use leptos_router::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::api;
use crate::i18n;
use crate::session::{use_session, Session};

#[wasm_bindgen(inline_js = r#"
export function in_app() {
  return !!(window.__TAURI__ || window.__TAURI_INTERNALS__);
}
export function open_external(url) {
  try {
    if (window.__TAURI__ && window.__TAURI__.opener) { window.__TAURI__.opener.openUrl(url); return true; }
    if (window.__TAURI_INTERNALS__) { window.__TAURI_INTERNALS__.invoke('plugin:opener|open_url', { url: url }); return true; }
  } catch (e) {}
  window.open(url, '_blank');
  return false;
}
export function native_google(clientId) {
  var inv = (window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke)
    || (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke);
  if (!inv) { return Promise.reject('unavailable'); }
  return inv('plugin:google-signin|sign_in', { clientId: clientId }).then(function (r) { return r.idToken; });
}
export function go_to(url) { window.location.href = url; }
export function sleep(ms) { return new Promise(function (r) { setTimeout(r, ms); }); }
export function random_id() {
  var a = new Uint8Array(16); crypto.getRandomValues(a);
  return Array.from(a, function (b) { return ('0' + b.toString(16)).slice(-2); }).join('');
}
"#)]
extern "C" {
    pub fn in_app() -> bool;
    pub fn open_external(url: &str) -> bool;
    pub fn go_to(url: &str);
    fn native_google(client_id: &str) -> js_sys::Promise;
    pub fn sleep(ms: u32) -> js_sys::Promise;
    fn random_id() -> String;
}

/// Signs in with a session token from the server and moves on: new
/// accounts first pick a profile picture.
pub async fn finish_sign_in(session: Session, token: String, is_new: bool) -> Result<String, String> {
    match api::me(Some(token.clone())).await {
        Ok(profile) => {
            session.sign_in(token, profile);
            Ok(if is_new { "/avatar?first=1".to_string() } else { "/".to_string() })
        }
        Err(e) => Err(e.message),
    }
}

#[component]
pub fn GoogleButton() -> impl IntoView {
    let session = use_session();
    let navigate = use_navigate();
    // (enabled, the server's Google client id for the in-app sign-in)
    let providers = create_local_resource(|| (), |_| async {
        api::providers()
            .await
            .map(|v| (v["google"].as_bool().unwrap_or(false), v["google_client_id"].as_str().unwrap_or("").to_string()))
            .unwrap_or((false, String::new()))
    });
    let enabled = Signal::derive(move || providers.get().map(|(on, _)| on));
    let waiting = create_rw_signal(false);
    let error = create_rw_signal(None::<String>);

    let start = move |_| {
        error.set(None);
        let base = api::url("/api/auth/google/start");
        if !in_app() {
            go_to(&base);
            return;
        }
        let client_id = providers.get_untracked().map(|(_, id)| id).unwrap_or_default();
        waiting.set(true);
        let navigate = navigate.clone();
        let lang = session.lang.get_untracked();
        spawn_local(async move {
            // 1. Google's sheet inside the app.
            if !client_id.is_empty() {
                match JsFuture::from(native_google(&client_id)).await {
                    Ok(v) => {
                        let id_token = v.as_string().unwrap_or_default();
                        let result = match api::google_id_token(id_token).await {
                            Ok(r) => finish_sign_in(
                                session,
                                r["token"].as_str().unwrap_or("").to_string(),
                                r["is_new"].as_bool().unwrap_or(false),
                            )
                            .await,
                            Err(e) => Err(e.message),
                        };
                        waiting.set(false);
                        match result {
                            Ok(next) => navigate(&next, Default::default()),
                            Err(e) => error.set(Some(i18n::error(lang, &e))),
                        }
                        return;
                    }
                    Err(e) => {
                        let msg = e.as_string().unwrap_or_default();
                        if msg.to_lowercase().contains("cancel") {
                            waiting.set(false);
                            return;
                        }
                        // Not available here: fall back to the browser below.
                        leptos::logging::warn!("in-app Google sign-in unavailable: {msg}");
                    }
                }
            }
            // 2. The phone's browser, while the app waits for the result.
            let poll = random_id();
            open_external(&format!("{base}?poll={poll}"));
            // Wait up to ~10 minutes for the browser sign-in.
            for _ in 0..300 {
                let _ = JsFuture::from(sleep(2000)).await;
                if !waiting.get_untracked() {
                    return; // cancelled
                }
                let Ok(reply) = api::google_poll(&poll).await else { continue };
                match reply["status"].as_str() {
                    Some("done") => {
                        waiting.set(false);
                        let token = reply["token"].as_str().unwrap_or("").to_string();
                        let is_new = reply["is_new"].as_bool().unwrap_or(false);
                        match finish_sign_in(session, token, is_new).await {
                            Ok(next) => navigate(&next, Default::default()),
                            Err(e) => error.set(Some(i18n::error(lang, &e))),
                        }
                        return;
                    }
                    Some("error") => {
                        waiting.set(false);
                        error.set(Some(i18n::t(lang, "auth.google_failed").to_string()));
                        return;
                    }
                    _ => {}
                }
            }
            waiting.set(false);
        });
    };

    // Stored so the views below can copy it freely.
    let start = store_value(start);

    view! {
        <Suspense fallback=|| ()>
            {move || enabled.get().filter(|on| *on).map(|_| {
                view! {
                <div class="google-area">
                    {move || if waiting.get() {
                        view! {
                            <div class="google-wait">
                                <div class="spinner small"></div>
                                <p>{session.t("auth.google_wait")}</p>
                                <button class="link-btn" on:click=move |_| waiting.set(false)>{session.t("avatar.cancel")}</button>
                            </div>
                        }.into_view()
                    } else {
                        view! {
                            <button class="btn btn-block btn-google" on:click=move |ev| start.with_value(|f| f(ev))>
                                <svg class="google-g" viewBox="0 0 48 48" aria-hidden="true">
                                    <path fill="#EA4335" d="M24 9.5c3.54 0 6.71 1.22 9.21 3.6l6.85-6.85C35.9 2.38 30.47 0 24 0 14.62 0 6.51 5.38 2.56 13.22l7.98 6.19C12.43 13.72 17.74 9.5 24 9.5z"/>
                                    <path fill="#4285F4" d="M46.98 24.55c0-1.57-.15-3.09-.38-4.55H24v9.02h12.94c-.58 2.96-2.26 5.48-4.78 7.18l7.73 6c4.51-4.18 7.09-10.36 7.09-17.65z"/>
                                    <path fill="#FBBC05" d="M10.53 28.59c-.48-1.45-.76-2.99-.76-4.59s.27-3.14.76-4.59l-7.98-6.19C.92 16.46 0 20.12 0 24c0 3.88.92 7.54 2.56 10.78l7.97-6.19z"/>
                                    <path fill="#34A853" d="M24 48c6.48 0 11.93-2.13 15.89-5.81l-7.73-6c-2.15 1.45-4.92 2.3-8.16 2.3-6.26 0-11.57-4.22-13.47-9.91l-7.98 6.19C6.51 42.62 14.62 48 24 48z"/>
                                </svg>
                                <span>{session.t("auth.google")}</span>
                            </button>
                        }.into_view()
                    }}
                    {move || error.get().map(|e| view! { <div class="form-error">{e}</div> })}
                </div>
                }
            })}
        </Suspense>
    }
}

/// Where the website lands after Google: `/auth/done#token=…&new=1`.
#[component]
pub fn AuthDone() -> impl IntoView {
    let session = use_session();
    let navigate = use_navigate();
    let error = create_rw_signal(None::<String>);
    let hash = window().location().hash().unwrap_or_default();
    let params: Vec<(String, String)> = hash
        .trim_start_matches('#')
        .split('&')
        .filter_map(|kv| kv.split_once('='))
        .map(|(k, v)| (k.to_string(), js_sys::decode_uri_component(v).ok().and_then(|s| s.as_string()).unwrap_or_default()))
        .collect();
    let get = |k: &str| params.iter().find(|(key, _)| key == k).map(|(_, v)| v.clone());
    // Don't leave the token in the address bar or the history.
    let _ = window().history().map(|h| h.replace_state_with_url(&JsValue::NULL, "", Some("/auth/done")));

    match (get("token"), get("error")) {
        (Some(token), _) => {
            let is_new = get("new").as_deref() == Some("1");
            let lang = session.lang.get_untracked();
            spawn_local(async move {
                match finish_sign_in(session, token, is_new).await {
                    Ok(next) => navigate(&next, Default::default()),
                    Err(e) => error.set(Some(i18n::error(lang, &e))),
                }
            });
        }
        (None, Some(e)) => error.set(Some(e)),
        (None, None) => error.set(Some(i18n::t(session.lang.get_untracked(), "auth.google_failed").to_string())),
    }

    view! {
        <div class="auth">
            {move || match error.get() {
                None => view! { <div class="center-load"><div class="spinner"></div></div> }.into_view(),
                Some(e) => view! {
                    <h1 class="auth-title">{session.t("auth.google_failed")}</h1>
                    <div class="form-error">{e}</div>
                    <a href="/welcome" class="btn btn-ghost btn-block">{session.t("back")}</a>
                }.into_view(),
            }}
        </div>
    }
}

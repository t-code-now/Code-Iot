//! "Open this course": shown when a locked lesson is tapped.
//! Guests are asked to sign in with Google (which opens one more topic for
//! free); signed-in users choose $5/month or $25 once for this course only.
//! After paying, the page waits until the payment is confirmed.

use leptos::*;
use leptos_router::*;
use wasm_bindgen_futures::JsFuture;

use crate::api;
use crate::components::google::{go_to, in_app, open_external, sleep, GoogleButton};
use crate::components::icons::{IconCheck, IconLock};
use crate::i18n::{self, tf};
use crate::pages::Loading;
use crate::session::use_session;

#[component]
pub fn UnlockPage() -> impl IntoView {
    let session = use_session();
    let params = use_params_map();
    let query = use_query_map();
    let course_id = move || params.with(|p| p.get("course").cloned().unwrap_or_default());
    let just_paid = move || query.with(|q| q.get("paid").is_some());

    // Re-fetched while we wait for a payment to be confirmed.
    let refresh = create_rw_signal(0u32);
    let course = create_local_resource(
        move || (course_id(), session.token.get(), session.lang.get(), refresh.get()),
        |(id, token, lang, _)| api::course(id, token, lang),
    );
    let config = create_local_resource(|| (), |_| api::payments_config());
    let waiting = create_rw_signal(false);
    let busy = create_rw_signal(false);
    let error = create_rw_signal(None::<String>);

    // Poll until the course opens (after paying here or in the browser).
    let wait_for_unlock = move || {
        if waiting.get_untracked() {
            return;
        }
        waiting.set(true);
        spawn_local(async move {
            for _ in 0..200 {
                let _ = JsFuture::from(sleep(3000)).await;
                if !waiting.get_untracked() {
                    return;
                }
                let token = session.token.get_untracked();
                if let Ok(c) = api::course(course_id(), token, session.lang.get_untracked()).await {
                    if c.unlocked {
                        waiting.set(false);
                        refresh.update(|n| *n += 1);
                        session.refresh_profile();
                        return;
                    }
                }
            }
            waiting.set(false);
        });
    };
    if just_paid() {
        wait_for_unlock();
    }

    let buy = move |plan: &'static str| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        error.set(None);
        let lang = session.lang.get_untracked();
        spawn_local(async move {
            let result = api::checkout(session.token.get_untracked(), course_id(), plan).await;
            busy.set(false);
            match result {
                Ok(r) if in_app() => {
                    // Pay in the phone's browser; this screen opens the course when it's confirmed.
                    open_external(&r.url);
                    wait_for_unlock();
                }
                Ok(r) => go_to(&r.url),
                Err(e) => error.set(Some(i18n::error(lang, &e.message))),
            }
        });
    };

    view! {
        <div class="auth unlock-page">
            <a href="/" class="back-link">{session.t("back")}</a>
            <Suspense fallback=|| view! { <Loading/> }>
                {move || course.get().map(|r| match r {
                    Err(e) => view! { <div class="form-error">{i18n::error(session.lang.get(), &e.message)}</div> }.into_view(),
                    Ok(c) if c.unlocked => view! {
                        <div class="unlock-card unlock-done">
                            <div class="unlock-icon done"><IconCheck/></div>
                            <h1 class="auth-title">{tf(session.lang.get(), "unlock.opened", &[&c.title])}</h1>
                            <p>{session.t("unlock.opened_sub")}</p>
                            <a href="/" class="btn btn-primary btn-block" on:click=move |_| session.set_course(&course_id())>
                                {session.t("home.continue")}
                            </a>
                        </div>
                    }.into_view(),
                    Ok(c) => {
                        let lang = session.lang.get();
                        let total = c.sections.len();
                        let open = c.sections.iter().filter(|s| s.lessons.first().is_some_and(|l| !l.locked)).count();
                        view! {
                            <div class="unlock-card">
                                <div class="unlock-head">
                                    <span class="course-banner-icon">{c.icon.clone()}</span>
                                    <div>
                                        <h1 class="auth-title">{c.title.clone()}</h1>
                                        <p class="muted">{tf(lang, "unlock.open_of", &[&open, &total])}</p>
                                    </div>
                                    <span class="unlock-icon"><IconLock/></span>
                                </div>

                                {move || waiting.get().then(|| view! {
                                    <div class="google-wait">
                                        <div class="spinner small"></div>
                                        <p>{session.t("unlock.waiting")}</p>
                                        <button class="link-btn" on:click=move |_| waiting.set(false)>{session.t("avatar.cancel")}</button>
                                    </div>
                                })}

                                {if !session.logged_in() {
                                    view! {
                                        <p class="unlock-text">{session.t("unlock.guest")}</p>
                                        <GoogleButton/>
                                        <p class="muted unlock-small">{session.t("unlock.guest_then")}</p>
                                    }.into_view()
                                } else {
                                    view! {
                                        <p class="unlock-text">{session.t("unlock.choose")}</p>
                                        {move || config.get().map(|cfg| {
                                            let cfg = cfg.unwrap_or_default();
                                            if !cfg["enabled"].as_bool().unwrap_or(false) {
                                                return view! { <div class="guest-note">{session.t("unlock.soon")}</div> }.into_view();
                                            }
                                            let monthly = cfg["monthly"].as_str().unwrap_or("$5").to_string();
                                            let lifetime = cfg["lifetime"].as_str().unwrap_or("$25").to_string();
                                            view! {
                                                <div class="plans">
                                                    <button class="plan-card" disabled=move || busy.get() on:click=move |_| buy("monthly")>
                                                        <span class="plan-name">{session.t("unlock.monthly")}</span>
                                                        <span class="plan-price">{monthly}<small>{session.t("unlock.per_month")}</small></span>
                                                        <span class="plan-note">{session.t("unlock.monthly_note")}</span>
                                                    </button>
                                                    <button class="plan-card best" disabled=move || busy.get() on:click=move |_| buy("lifetime")>
                                                        <span class="plan-badge">{session.t("unlock.best")}</span>
                                                        <span class="plan-name">{session.t("unlock.lifetime")}</span>
                                                        <span class="plan-price">{lifetime}<small>{session.t("unlock.once")}</small></span>
                                                        <span class="plan-note">{session.t("unlock.lifetime_note")}</span>
                                                    </button>
                                                </div>
                                                <p class="muted unlock-small">{session.t("unlock.only_this")}</p>
                                            }.into_view()
                                        })}
                                    }.into_view()
                                }}
                                {move || error.get().map(|e| view! { <div class="form-error">{e}</div> })}
                            </div>
                        }.into_view()
                    }
                })}
            </Suspense>
        </div>
    }
}

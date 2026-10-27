//! "Open this course": shown when a locked lesson is tapped.
//! Guests are asked to sign in with Google (which opens one more topic for
//! free); signed-in users choose $5/month or $25 once for this course only.
//! After paying, the page waits until the payment is confirmed.

use codesprout_core::phone::{flag, COUNTRIES};
use crate::components::lang_icon::LangIcon;
use leptos::*;
use leptos_router::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::api;
use crate::components::google::{go_to, in_app, open_external, sleep, GoogleButton};
use crate::components::icons::{IconCheck, IconLock};
use crate::i18n::{self, tf};
use crate::pages::Loading;
use crate::session::use_session;

#[wasm_bindgen(inline_js = "export function locale_region() { var p = (navigator.language || '').split('-'); return p.length > 1 ? p[1].toUpperCase() : ''; }")]
extern "C" {
    fn locale_region() -> String;
}

/// The buyer's phone, needed before paying so the site owner can reach them.
#[component]
fn PhoneForm(#[prop(into)] on_saved: Callback<()>) -> impl IntoView {
    let session = use_session();
    let saved = session.profile.with_untracked(|p| p.as_ref().and_then(|p| Some((p.country.clone()?, p.phone.clone()?))));
    let default_country = saved.as_ref().map(|(c, _)| c.clone()).unwrap_or_else(|| {
        let region = locale_region();
        if COUNTRIES.iter().any(|(c, _, _)| *c == region) {
            region
        } else if session.lang.get_untracked().is_rtl() {
            "IL".to_string()
        } else {
            "US".to_string()
        }
    });
    let country = create_rw_signal(default_country);
    let number = create_rw_signal(saved.map(|(_, p)| p).unwrap_or_default());
    let busy = create_rw_signal(false);
    let error = create_rw_signal(None::<String>);
    let dial = move || COUNTRIES.iter().find(|(c, _, _)| *c == country.get()).map_or("", |(_, d, _)| *d);

    let save = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        error.set(None);
        let lang = session.lang.get_untracked();
        spawn_local(async move {
            let r = api::set_contact(session.token.get_untracked(), country.get_untracked(), number.get_untracked()).await;
            busy.set(false);
            match r {
                Ok(p) => {
                    session.profile.set(Some(p));
                    on_saved.call(());
                }
                Err(e) => error.set(Some(i18n::error(lang, &e.message))),
            }
        });
    };

    view! {
        <form class="phone-form" on:submit=save>
            <p class="unlock-text">{session.t("unlock.phone_why")}</p>
            <div class="phone-row" dir="ltr">
                <select on:change=move |ev| country.set(event_target_value(&ev))>
                    {COUNTRIES.iter().map(|(code, d, name)| {
                        let code = code.to_string();
                        let selected = code == country.get_untracked();
                        view! { <option value=code.clone() selected=selected>{format!("{} {} +{}", flag(&code), name, d)}</option> }
                    }).collect_view()}
                </select>
                <span class="phone-dial">{move || format!("+{}", dial())}</span>
                <input
                    type="tel"
                    inputmode="tel"
                    autocomplete="tel-national"
                    placeholder="50 123 4567"
                    prop:value=move || number.get()
                    on:input=move |ev| number.set(event_target_value(&ev))
                />
            </div>
            {move || error.get().map(|e| view! { <div class="form-error">{e}</div> })}
            <button type="submit" class="btn btn-primary btn-block" disabled=move || busy.get()>
                {move || if busy.get() { session.t("auth.wait")() } else { session.t("unlock.phone_save")() }}
            </button>
        </form>
    }
}

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
    let editing_phone = create_rw_signal(false);
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
                let id = course_id();
                if id.is_empty() {
                    // Left this screen.
                    waiting.set(false);
                    return;
                }
                let token = session.token.get_untracked();
                if let Ok(c) = api::course(id, token, session.lang.get_untracked()).await {
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
                    Ok(c) if c.unlocked => {
                        // Read the id now: by the time the click runs, the page may have moved on.
                        let id = c.id.clone();
                        view! {
                        <div class="unlock-card unlock-done">
                            <div class="unlock-icon done"><IconCheck/></div>
                            <h1 class="auth-title">{tf(session.lang.get(), "unlock.opened", &[&c.title])}</h1>
                            <p>{session.t("unlock.opened_sub")}</p>
                            <a href="/" class="btn btn-primary btn-block" on:click=move |_| session.set_course(&id)>
                                {session.t("home.continue")}
                            </a>
                        </div>
                    }.into_view()
                    }
                    Ok(c) => {
                        let lang = session.lang.get();
                        let total = c.sections.len();
                        let open = c.sections.iter().filter(|s| s.lessons.first().is_some_and(|l| !l.locked)).count();
                        view! {
                            <div class="unlock-card">
                                <div class="unlock-head">
                                    <span class="course-banner-icon"><LangIcon id=c.id.clone()/></span>
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
                                    let has_phone = move || session.profile.with(|p| p.as_ref().is_some_and(|p| p.phone.is_some()));
                                    view! {
                                        {move || (!has_phone() || editing_phone.get()).then(|| view! {
                                            <PhoneForm on_saved=move |_| editing_phone.set(false)/>
                                        })}
                                        {move || (has_phone() && !editing_phone.get()).then(|| view! {
                                        <p class="phone-line">
                                            "📞 " <bdi dir="ltr">{move || session.profile.with(|p| p.as_ref().and_then(|p| p.phone.clone()).unwrap_or_default())}</bdi>
                                            " " <button class="link-btn small" on:click=move |_| editing_phone.set(true)>{session.t("unlock.phone_change")}</button>
                                        </p>
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

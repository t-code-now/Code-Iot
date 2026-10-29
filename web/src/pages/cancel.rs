//! "Cancel a purchase": the dedicated cancellation page required for online
//! sales. Subscriptions are cancelled right here (charges stop at once, the
//! course stays open until the end of the paid month); one-time purchases
//! are refunded on request by e-mail.

use codesprout_core::api::MyPurchase;
use leptos::*;
use crate::components::icons::IconBack;
use wasm_bindgen::prelude::*;

use crate::api;
use crate::components::lang_icon::LangIcon;
use crate::i18n::{self, tf};
use crate::pages::legal::CONTACT_EMAIL;
use crate::pages::profile::{course_name, format_date};
use crate::pages::Loading;
use crate::session::use_session;

#[wasm_bindgen(inline_js = "export function ask_cancel(message) { return window.confirm(message); }")]
extern "C" {
    fn ask_cancel(message: &str) -> bool;
}

/// Days during which a purchase can be refunded in full on request.
const REFUND_DAYS: i64 = 14;

fn money(cents: i64, currency: &str) -> String {
    let sign = if currency.eq_ignore_ascii_case("USD") { "$" } else { "" };
    format!("{sign}{:.2}", cents as f64 / 100.0)
}

fn mail_link(subject: &str, body: &str) -> String {
    format!(
        "mailto:{CONTACT_EMAIL}?subject={}&body={}",
        String::from(js_sys::encode_uri_component(subject)),
        String::from(js_sys::encode_uri_component(body))
    )
}

#[component]
pub fn CancelPage() -> impl IntoView {
    let session = use_session();
    let reload = create_rw_signal(0u32);
    let purchases = create_local_resource(
        move || (session.token.get(), reload.get()),
        |(token, _)| async move {
            if token.is_none() {
                return Ok(Vec::new());
            }
            api::my_purchases(token).await
        },
    );
    let note = create_rw_signal(None::<(bool, String)>);
    let busy = create_rw_signal(false);

    let cancel = move |p: MyPurchase| {
        let lang = session.lang.get_untracked();
        if busy.get_untracked() || !ask_cancel(&tf(lang, "cancel.confirm", &[&course_name(&p.course_id)])) {
            return;
        }
        busy.set(true);
        note.set(None);
        spawn_local(async move {
            let r = api::cancel_my_subscription(session.token.get_untracked(), p.id).await;
            busy.set(false);
            match r {
                Ok(v) => {
                    let until = v["until"].as_i64().or(p.until).map(|u| format_date(u, lang)).unwrap_or_default();
                    note.set(Some((true, tf(lang, "cancel.done", &[&until]))));
                    reload.update(|n| *n += 1);
                    session.refresh_profile();
                }
                Err(e) => note.set(Some((false, i18n::error(lang, &e.message)))),
            }
        });
    };

    view! {
        <div class="auth cancel-page">
            <a href="/profile" class="back-link"><IconBack/>{session.t("back.word")}</a>
            <h1 class="auth-title">{session.t("cancel.title")}</h1>
            <p class="admin-hint">{session.t("cancel.intro")}</p>
            {move || note.get().map(|(ok, msg)| view! { <div class={if ok { "ok-note" } else { "form-error" }}>{msg}</div> })}
            {move || (!session.logged_in()).then(|| view! {
                <div class="guest-note"><p>{session.t("cancel.sign_in")}</p>
                    <a href="/login" class="btn btn-primary btn-block">{session.t("auth.login")}</a></div>
            })}
            <Suspense fallback=|| view! { <Loading/> }>
                {move || purchases.get().map(|r| {
                    let lang = session.lang.get();
                    let t = move |k| i18n::t(lang, k);
                    match r {
                        Err(e) => view! { <div class="form-error">{i18n::error(lang, &e.message)}</div> }.into_view(),
                        Ok(list) if list.is_empty() => session.logged_in().then(|| view! {
                            <div class="empty">{t("cancel.none")}</div>
                        }).into_view(),
                        Ok(list) => {
                            let now = (js_sys::Date::now() / 1000.0) as i64;
                            list.into_iter().map(|p| {
                                let name = course_name(&p.course_id);
                                let paid = money(p.amount_cents, &p.currency);
                                let bought = format_date(p.created_at, lang);
                                let within = now - p.created_at <= REFUND_DAYS * 86_400;
                                let fully_refunded = p.amount_cents > 0 && p.refunded_cents >= p.amount_cents;
                                let monthly = p.plan == "monthly";
                                let state = if fully_refunded {
                                    t("cancel.s_refunded").to_string()
                                } else if monthly && p.can_cancel {
                                    tf(lang, "cancel.s_active", &[&p.until.map(|u| format_date(u, lang)).unwrap_or_default()])
                                } else if monthly {
                                    tf(lang, "cancel.s_cancelled", &[&p.until.map(|u| format_date(u, lang)).unwrap_or_default()])
                                } else if p.plan == "yearly" {
                                    tf(lang, "cancel.s_yearly", &[&p.until.map(|u| format_date(u, lang)).unwrap_or_default()])
                                } else {
                                    t("cancel.s_lifetime").to_string()
                                };
                                let subject = tf(lang, "cancel.mail_subject", &[&name]);
                                let body = tf(lang, "cancel.mail_body", &[&name, &bought, &paid, &p.id]);
                                let refund_link = mail_link(&subject, &body);
                                let p2 = p.clone();
                                view! {
                                    <div class="plan-row cancel-row">
                                        <div class="cancel-main">
                                            <strong class="course-label"><LangIcon id=p.course_id.clone()/> {name.clone()}</strong>
                                            <span class="muted">{state}</span>
                                            <span class="muted">{tf(lang, "cancel.bought", &[&bought, &paid])}</span>
                                        </div>
                                        <div class="cancel-actions">
                                            {(monthly && p.can_cancel).then(|| view! {
                                                <button class="btn btn-small btn-danger" disabled=move || busy.get()
                                                    on:click=move |_| cancel(p2.clone())>{t("cancel.sub_button")}</button>
                                            })}
                                            {(!fully_refunded).then(|| view! {
                                                <a class="btn btn-small btn-ghost" href=refund_link.clone()>
                                                    {if within { t("cancel.refund_button") } else { t("cancel.contact_button") }}
                                                </a>
                                            })}
                                        </div>
                                        {(!fully_refunded && within).then(|| view! { <p class="cancel-note">{t("cancel.refund_note")}</p> })}
                                        {(monthly && !fully_refunded).then(|| view! { <p class="cancel-note">{t("cancel.prorata_note")}</p> })}
                                    </div>
                                }
                            }).collect_view()
                        }
                    }
                })}
            </Suspense>
            <p class="cancel-contact">{session.t("cancel.other_ways")} " " <a href=format!("mailto:{CONTACT_EMAIL}") dir="ltr">{CONTACT_EMAIL}</a></p>
            <a href="/terms" class="link-btn">{session.t("cancel.terms_link")}</a>
        </div>
    }
}

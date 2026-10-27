//! The admin page. Only the admin's Google account can see data here: the
//! server checks every request, this page only hides itself for others.

use codesprout_core::api::{AdminOverview, AdminPayment, AdminUser, CourseSummary};
use leptos::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

use crate::api;
use crate::i18n::{self, tf, Lang};
use crate::pages::Loading;
use crate::session::use_session;

#[wasm_bindgen(inline_js = r#"
export function ask(message) { return window.confirm(message); }
export function download_text(name, text) {
  var blob = new Blob(['﻿' + text], { type: 'text/csv;charset=utf-8' });
  var a = document.createElement('a');
  a.href = URL.createObjectURL(blob); a.download = name;
  document.body.appendChild(a); a.click(); a.remove();
  setTimeout(function () { URL.revokeObjectURL(a.href); }, 2000);
}
"#)]
extern "C" {
    fn ask(message: &str) -> bool;
    fn download_text(name: &str, text: &str);
}

fn date(unix: i64, lang: Lang) -> String {
    if unix <= 0 {
        return "—".into();
    }
    let d = js_sys::Date::new(&JsValue::from_f64(unix as f64 * 1000.0));
    String::from(d.to_locale_date_string(i18n::locale(lang), &JsValue::UNDEFINED))
}

fn day(day_number: Option<i64>, lang: Lang) -> String {
    day_number.map_or("—".into(), |d| date(d * 86_400, lang))
}

fn money(cents: i64, currency: &str) -> String {
    let sym = match currency {
        "USD" => "$",
        "EUR" => "€",
        "ILS" => "₪",
        "GBP" => "£",
        _ => "",
    };
    if sym.is_empty() {
        format!("{:.2} {currency}", cents as f64 / 100.0)
    } else {
        format!("{sym}{:.2}", cents as f64 / 100.0)
    }
}

fn csv_cell(s: &str) -> String {
    if s.contains([',', '"', '\n']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Overview,
    Users,
    Payments,
    Log,
}

#[component]
pub fn AdminPage() -> impl IntoView {
    let session = use_session();
    session.refresh_profile();
    let is_admin = move || session.profile.with(|p| p.as_ref().is_some_and(|p| p.is_admin));
    let tab = create_rw_signal(Tab::Overview);
    let courses = create_local_resource(
        move || (session.token.get(), session.lang.get()),
        |(token, lang)| async move { api::courses(token, lang).await.unwrap_or_default() },
    );

    let tab_button = move |t: Tab, key: &'static str| {
        view! {
            <button class="pg-tab" class:active={move || tab.get() == t} on:click=move |_| tab.set(t)>
                {session.t(key)}
            </button>
        }
    };

    view! {
        <div class="admin">
            <header class="admin-head">
                <a href="/profile" class="back-link">{session.t("back")}</a>
                <h1 class="topbar-title">"🛠 " {session.t("admin.title")}</h1>
            </header>
            {move || if !is_admin() {
                view! {
                    <div class="guest-note">
                        <p>{session.t("admin.denied")}</p>
                        <a href="/" class="btn btn-primary btn-block">{session.t("back.learn")}</a>
                    </div>
                }.into_view()
            } else {
                view! {
                    <div class="pg-tabs admin-tabs">
                        {tab_button(Tab::Overview, "admin.tab_overview")}
                        {tab_button(Tab::Users, "admin.tab_users")}
                        {tab_button(Tab::Payments, "admin.tab_payments")}
                        {tab_button(Tab::Log, "admin.tab_log")}
                    </div>
                    {move || {
                        let list = courses.get().unwrap_or_default();
                        match tab.get() {
                            Tab::Overview => view! { <OverviewTab courses=list/> }.into_view(),
                            Tab::Users => view! { <UsersTab courses=list/> }.into_view(),
                            Tab::Payments => view! { <PaymentsTab courses=list/> }.into_view(),
                            Tab::Log => view! { <LogTab/> }.into_view(),
                        }
                    }}
                }.into_view()
            }}
        </div>
    }
}

fn course_label(courses: &[CourseSummary], id: &str) -> String {
    courses
        .iter()
        .find(|c| c.id == id)
        .map_or_else(|| id.to_string(), |c| format!("{} {}", c.icon, c.title))
}

#[component]
fn OverviewTab(courses: Vec<CourseSummary>) -> impl IntoView {
    let session = use_session();
    let courses = store_value(courses);
    let data = create_local_resource(move || session.token.get(), api::admin_overview);
    view! {
        <Suspense fallback=|| view! { <Loading/> }>
            {move || data.get().map(|r| match r {
                Err(e) => view! { <div class="form-error">{e.message}</div> }.into_view(),
                Ok(o) => {
                    let o: AdminOverview = o;
                    let lang = session.lang.get();
                    let t = move |k| i18n::t(lang, k);
                    let revenue = if o.revenue.is_empty() {
                        money(0, "USD")
                    } else {
                        o.revenue.iter().map(|(c, v)| money(*v, c)).collect::<Vec<_>>().join(" + ")
                    };
                    let cards = vec![
                        ("👥", o.users.to_string(), t("admin.users")),
                        ("🟢", o.active_users_7d.to_string(), t("admin.active_7d")),
                        ("✨", o.new_users_7d.to_string(), t("admin.new_7d")),
                        ("📘", o.lessons_7d.to_string(), t("admin.lessons_7d")),
                        ("💳", o.paying_users.to_string(), t("admin.paying")),
                        ("♾️", o.lifetime_purchases.to_string(), t("admin.lifetime")),
                        ("🔁", o.active_subscriptions.to_string(), t("admin.subs")),
                        ("💰", revenue, t("admin.revenue")),
                        ("G", o.google_users.to_string(), t("admin.google_users")),
                        ("⛔", o.banned_users.to_string(), t("admin.banned")),
                    ];
                    view! {
                        {(!o.payments_enabled).then(|| view! { <div class="guest-note">{t("admin.payments_off")}</div> })}
                        <div class="admin-cards">
                            {cards.into_iter().map(|(icon, value, label)| view! {
                                <div class="admin-card">
                                    <span class="admin-card-icon">{icon}</span>
                                    <strong>{value}</strong>
                                    <span>{label}</span>
                                </div>
                            }).collect_view()}
                        </div>
                        <h3 class="section-title">{t("admin.by_course")}</h3>
                        <div class="admin-table-wrap">
                            <table class="admin-table">
                                <thead><tr>
                                    <th>{t("admin.course")}</th><th>{t("admin.learners")}</th><th>{t("admin.full_access")}</th>
                                </tr></thead>
                                <tbody>
                                    {o.courses.iter().map(|(id, learners, full)| view! {
                                        <tr><td>{courses.with_value(|cs| course_label(cs, id))}</td><td>{*learners}</td><td>{*full}</td></tr>
                                    }).collect_view()}
                                </tbody>
                            </table>
                        </div>
                    }.into_view()
                }
            })}
        </Suspense>
    }
}

fn state_label(lang: Lang, state: &str, until: Option<i64>) -> String {
    let key = match state {
        "lifetime" => "admin.state_lifetime",
        "monthly" => "admin.state_monthly",
        "open" => "admin.state_open",
        "blocked" => "admin.state_blocked",
        "admin" => "admin.state_admin",
        _ => "admin.state_free",
    };
    match until {
        Some(u) => format!("{} ({})", i18n::t(lang, key), date(u, lang)),
        None => i18n::t(lang, key).to_string(),
    }
}

#[component]
fn UsersTab(courses: Vec<CourseSummary>) -> impl IntoView {
    let session = use_session();
    let search = create_rw_signal(String::new());
    let query = create_rw_signal(String::new());
    let reload = create_rw_signal(0u32);
    let note = create_rw_signal(None::<String>);
    let users = create_local_resource(
        move || (session.token.get(), query.get(), reload.get()),
        |(token, q, _)| api::admin_users(token, q),
    );
    let courses = store_value(courses);

    let after = move |r: Result<serde_json::Value, api::ApiError>| match r {
        Ok(_) => {
            note.set(None);
            reload.update(|n| *n += 1);
        }
        Err(e) => note.set(Some(e.message)),
    };

    let export = move |_| {
        let lang = session.lang.get_untracked();
        let Some(Ok(list)) = users.get() else { return };
        let ids: Vec<String> = courses.with_value(|c| c.iter().map(|c| c.id.clone()).collect());
        let mut out = String::from("id,username,email,sign_in,joined,last_active,xp,lessons,paid,payment_methods,banned");
        for id in &ids {
            out.push(',');
            out.push_str(id);
        }
        out.push('\n');
        for u in &list {
            let mut row = vec![
                u.id.to_string(),
                u.username.clone(),
                u.email.clone(),
                u.sign_in.clone(),
                date(u.joined_at, lang),
                day(u.last_active_day, lang),
                u.xp.to_string(),
                u.lessons_completed.to_string(),
                format!("{:.2}", u.total_paid_cents as f64 / 100.0),
                u.payment_methods.join(" / "),
                u.banned.to_string(),
            ];
            for id in &ids {
                row.push(u.courses.iter().find(|c| &c.course_id == id).map_or(String::new(), |c| c.state.clone()));
            }
            out.push_str(&row.iter().map(|c| csv_cell(c)).collect::<Vec<_>>().join(","));
            out.push('\n');
        }
        download_text("code-iot-users.csv", &out);
    };

    view! {
        <form class="admin-search" on:submit=move |ev: ev::SubmitEvent| { ev.prevent_default(); query.set(search.get_untracked()); }>
            <input
                type="search"
                dir="auto"
                placeholder=session.t("admin.search_ph")
                prop:value=move || search.get()
                on:input=move |ev| search.set(event_target_value(&ev))
            />
            <button type="submit" class="btn btn-small btn-check">{session.t("admin.search")}</button>
            <button type="button" class="btn btn-small btn-ghost" on:click=export>"⬇ CSV"</button>
        </form>
        {move || note.get().map(|n| view! { <div class="form-error">{n}</div> })}
        <Suspense fallback=|| view! { <Loading/> }>
            {move || users.get().map(|r| match r {
                Err(e) => view! { <div class="form-error">{e.message}</div> }.into_view(),
                Ok(list) if list.is_empty() => view! { <div class="empty">{session.t("admin.no_users")}</div> }.into_view(),
                Ok(list) => {
                    let lang = session.lang.get();
                    view! {
                        <p class="muted admin-count">{tf(lang, "admin.count", &[&list.len()])}</p>
                        <div class="admin-users">
                            {list.into_iter().map(|u| user_card(u, lang, courses.get_value(), after)).collect_view()}
                        </div>
                    }.into_view()
                }
            })}
        </Suspense>
    }
}

fn user_card(
    u: AdminUser,
    lang: Lang,
    courses: Vec<CourseSummary>,
    after: impl Fn(Result<serde_json::Value, api::ApiError>) + Copy + 'static,
) -> impl IntoView {
    let session = use_session();
    let t = move |k| i18n::t(lang, k);
    let uid = u.id;
    let name = u.username.clone();
    let initial = u.username.chars().next().unwrap_or('?').to_uppercase().to_string();
    let banned = u.banned;
    let methods = if u.payment_methods.is_empty() { "—".to_string() } else { u.payment_methods.join(", ") };

    let set_mode = move |course: String, mode: String| {
        let token = session.token.get_untracked();
        spawn_local(async move { after(api::admin_access(token, uid, course, &mode).await) });
    };
    let toggle_ban = move |_| {
        let token = session.token.get_untracked();
        spawn_local(async move { after(api::admin_ban(token, uid, !banned).await) });
    };
    let name_for_delete = name.clone();
    let remove = move |_| {
        if !ask(&tf(lang, "admin.delete_confirm", &[&name_for_delete])) {
            return;
        }
        let token = session.token.get_untracked();
        spawn_local(async move { after(api::admin_delete(token, uid).await) });
    };

    view! {
        <div class="admin-user" class:banned=banned>
            <div class="admin-user-head">
                <span class="avatar">{initial}</span>
                <div class="admin-user-id">
                    <strong><bdi>{u.username.clone()}</bdi></strong>
                    <span class="muted" dir="ltr">{if u.email.is_empty() { "—".to_string() } else { u.email.clone() }}</span>
                </div>
                <div class="admin-tags">
                    <span class="tag">{if u.sign_in == "google" { "Google" } else { t("admin.password") }}</span>
                    {u.is_admin.then(|| view! { <span class="tag gold">{t("admin.state_admin")}</span> })}
                    {banned.then(|| view! { <span class="tag red">{t("admin.suspended")}</span> })}
                </div>
            </div>
            <div class="admin-facts">
                <span>{t("admin.joined")} ": " {date(u.joined_at, lang)}</span>
                <span>{t("admin.last_active")} ": " {day(u.last_active_day, lang)}</span>
                <span>"XP: " {u.xp}</span>
                <span>{t("admin.lessons")} ": " {u.lessons_completed}</span>
                <span>{t("admin.paid")} ": " {money(u.total_paid_cents, "USD")}</span>
                <span>{t("admin.method")} ": " {methods}</span>
            </div>
            <div class="admin-courses">
                {u.courses.iter().map(|c| {
                    let course = c.course_id.clone();
                    let state = c.state.clone();
                    let current = if c.manual.is_empty() { "default".to_string() } else { c.manual.clone() };
                    let label = course_label(&courses, &c.course_id);
                    view! {
                        <div class=format!("admin-course state-{state}")>
                            <span class="admin-course-name">{label}</span>
                            <span class="admin-course-state">{state_label(lang, &c.state, c.until)}</span>
                            {(!u.is_admin).then(|| {
                                let course = course.clone();
                                view! {
                                    <select on:change=move |ev| set_mode(course.clone(), event_target_value(&ev))>
                                        <option value="default" selected={current == "default"}>{t("admin.mode_default")}</option>
                                        <option value="open" selected={current == "open"}>{t("admin.mode_open")}</option>
                                        <option value="block" selected={current == "block"}>{t("admin.mode_block")}</option>
                                    </select>
                                }
                            })}
                        </div>
                    }
                }).collect_view()}
            </div>
            {(!u.is_admin).then(|| view! {
                <div class="admin-actions">
                    <button class="btn btn-small btn-ghost" on:click=toggle_ban>
                        {if banned { t("admin.restore") } else { t("admin.suspend") }}
                    </button>
                    <button class="btn btn-small btn-danger" on:click=remove.clone()>{t("admin.delete")}</button>
                </div>
            })}
        </div>
    }
}

#[component]
fn PaymentsTab(courses: Vec<CourseSummary>) -> impl IntoView {
    let session = use_session();
    let courses = store_value(courses);
    let data = create_local_resource(move || session.token.get(), api::admin_payments);
    view! {
        <Suspense fallback=|| view! { <Loading/> }>
            {move || data.get().map(|r| match r {
                Err(e) => view! { <div class="form-error">{e.message}</div> }.into_view(),
                Ok(list) if list.is_empty() => view! { <div class="empty">{session.t("admin.no_payments")}</div> }.into_view(),
                Ok(list) => {
                    let lang = session.lang.get();
                    let t = move |k| i18n::t(lang, k);
                    view! {
                        <p class="muted admin-count">{t("admin.refund_hint")}</p>
                        <div class="admin-table-wrap">
                            <table class="admin-table">
                                <thead><tr>
                                    <th>{t("admin.date")}</th><th>{t("admin.user")}</th><th>{t("admin.course")}</th>
                                    <th>{t("admin.plan")}</th><th>{t("admin.status")}</th><th>{t("admin.amount")}</th>
                                    <th>{t("admin.method")}</th><th></th>
                                </tr></thead>
                                <tbody>
                                    {list.into_iter().map(|p: AdminPayment| {
                                        let plan = if p.provider_ref.starts_with("inv_") { t("admin.invoice") }
                                            else if p.plan == "lifetime" { t("admin.state_lifetime") } else { t("admin.state_monthly") };
                                        view! {
                                            <tr>
                                                <td>{date(p.created_at, lang)}</td>
                                                <td><bdi>{p.username.clone()}</bdi><br/><small dir="ltr">{p.email.clone()}</small></td>
                                                <td>{courses.with_value(|cs| course_label(cs, &p.course_id))}</td>
                                                <td>{plan}{p.until.map(|u| format!(" → {}", date(u, lang)))}</td>
                                                <td>{p.status.clone()}{p.test_mode.then(|| view! { <span class="tag">"TEST"</span> })}</td>
                                                <td dir="ltr">{if p.amount_cents > 0 { money(p.amount_cents, &p.currency) } else { "—".into() }}</td>
                                                <td>{if p.method.is_empty() { "—".to_string() } else { p.method.clone() }}</td>
                                                <td><a href=p.dashboard_url.clone() target="_blank" rel="noopener">{t("admin.open_in_ls")}</a></td>
                                            </tr>
                                        }
                                    }).collect_view()}
                                </tbody>
                            </table>
                        </div>
                    }.into_view()
                }
            })}
        </Suspense>
    }
}

#[component]
fn LogTab() -> impl IntoView {
    let session = use_session();
    let data = create_local_resource(move || session.token.get(), api::admin_log);
    view! {
        <Suspense fallback=|| view! { <Loading/> }>
            {move || data.get().map(|r| match r {
                Err(e) => view! { <div class="form-error">{e.message}</div> }.into_view(),
                Ok(list) if list.is_empty() => view! { <div class="empty">{session.t("admin.no_log")}</div> }.into_view(),
                Ok(list) => {
                    let lang = session.lang.get();
                    view! {
                        <ul class="admin-log">
                            {list.into_iter().map(|e| view! {
                                <li>
                                    <span class="muted">{date(e.at, lang)}</span>
                                    <strong>{e.action}</strong>
                                    <bdi>{e.target}</bdi>
                                    <span dir="ltr">{e.detail}</span>
                                </li>
                            }).collect_view()}
                        </ul>
                    }.into_view()
                }
            })}
        </Suspense>
    }
}

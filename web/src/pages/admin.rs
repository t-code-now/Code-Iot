//! The admin page. Only the admin's Google account can see data here: the
//! server checks every request, this page only hides itself for others.

use codesprout_core::api::{AdminOverview, AdminPayment, AdminUser, CourseSummary};
use crate::components::lang_icon::LangIcon;
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

/// A payment status in the viewer's language (the raw value stays in the tooltip).
fn status_label(lang: crate::i18n::Lang, status: &str) -> String {
    let key = match status {
        "paid" => "status.paid",
        "partial_refund" => "status.partial_refund",
        "refunded" => "status.refunded",
        "active" => "status.active",
        "cancelled" => "status.cancelled",
        "expired" => "status.expired",
        "past_due" => "status.past_due",
        "unpaid" => "status.unpaid",
        "paused" => "status.paused",
        "on_trial" => "status.on_trial",
        "pending" => "status.pending",
        "failed" => "status.failed",
        "void" => "status.void",
        other => return other.to_string(),
    };
    i18n::t(lang, key).to_string()
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

/// Shared between the tabs: "show this user's payments" jumps tabs.
#[derive(Clone, Copy)]
struct AdminNav {
    tab: RwSignal<Tab>,
    /// (user id, username) to filter the payments by.
    user: RwSignal<Option<(i64, String)>>,
}

#[component]
pub fn AdminPage() -> impl IntoView {
    let session = use_session();
    session.refresh_profile();
    let is_admin = move || session.profile.with(|p| p.as_ref().is_some_and(|p| p.is_admin));
    let tab = create_rw_signal(Tab::Overview);
    provide_context(AdminNav { tab, user: create_rw_signal(None) });
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
        .map_or_else(|| id.to_string(), |c| c.title.clone())
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
                    let refunded = if o.refunded.is_empty() {
                        money(0, "USD")
                    } else {
                        o.refunded.iter().map(|(c, v)| money(*v, c)).collect::<Vec<_>>().join(" + ")
                    };
                    let paused = o.sales_paused;
                    let toggle_sales = move |_| {
                        let msg = if paused { t("admin.resume_confirm") } else { t("admin.pause_confirm") };
                        if !ask(msg) {
                            return;
                        }
                        let token = session.token.get_untracked();
                        spawn_local(async move {
                            let _ = api::admin_settings(token, !paused).await;
                            data.refetch();
                        });
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
                        ("↩️", refunded, t("admin.refunded")),
                        ("G", o.google_users.to_string(), t("admin.google_users")),
                        ("⛔", o.banned_users.to_string(), t("admin.banned")),
                    ];
                    view! {
                        {(!o.payments_enabled).then(|| view! {
                            <div class="guest-note">{t("admin.payments_off")}</div>
                            <PaymentsSetup/>
                        })}
                        <div class="sales-switch" class:paused=paused>
                            <div>
                                <strong>{if paused { t("admin.sales_paused") } else { t("admin.sales_open") }}</strong>
                                <span class="muted">{t("admin.sales_hint")}</span>
                            </div>
                            <button class={if paused { "btn btn-small btn-check" } else { "btn btn-small btn-danger" }} on:click=toggle_sales>
                                {if paused { t("admin.resume") } else { t("admin.pause") }}
                            </button>
                        </div>
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
                                        <tr><td><span class="course-label"><LangIcon id=id.clone()/> {courses.with_value(|cs| course_label(cs, id))}</span></td><td>{*learners}</td><td>{*full}</td></tr>
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
    let nav = use_context::<AdminNav>();
    let name_for_payments = name.clone();
    let show_payments = move |_| {
        if let Some(nav) = nav {
            nav.user.set(Some((uid, name_for_payments.clone())));
            nav.tab.set(Tab::Payments);
        }
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
                    {(!u.phone.is_empty()).then(|| phone_links(&u.phone, &u.country))}
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
                            <span class="admin-course-name course-label"><LangIcon id=c.course_id.clone()/> {label}</span>
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
                    <button class="btn btn-small btn-ghost" on:click=show_payments.clone()>"💳 " {t("admin.user_payments")}</button>
                    <button class="btn btn-small btn-ghost" on:click=toggle_ban>
                        {if banned { t("admin.restore") } else { t("admin.suspend") }}
                    </button>
                    <button class="btn btn-small btn-danger" on:click=remove.clone()>{t("admin.delete")}</button>
                </div>
            })}
        </div>
    }
}

/// Phone with country flag, plus call and WhatsApp links.
fn phone_links(phone: &str, country: &str) -> impl IntoView {
    let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
    view! {
        <span class="admin-phone" dir="ltr">
            {codesprout_core::phone::flag(country)} " "
            <a href=format!("tel:{phone}")>{phone.to_string()}</a>
            " · "
            <a href=format!("https://wa.me/{digits}") target="_blank" rel="noopener">"WhatsApp"</a>
        </span>
    }
}

/// "12.50" → 1250 cents.
fn parse_cents(s: &str) -> Option<i64> {
    let s = s.trim().replace(',', ".");
    let v: f64 = s.parse().ok()?;
    if !v.is_finite() || v <= 0.0 {
        return None;
    }
    let cents = (v * 100.0).round() as i64;
    // Reject more than two decimals ("1.005").
    ((cents as f64 / 100.0 - v).abs() < 1e-9).then_some(cents)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PayFilter {
    All,
    Charges,
    Refunds,
    Subscriptions,
    Test,
}

#[component]
fn PaymentsTab(courses: Vec<CourseSummary>) -> impl IntoView {
    let session = use_session();
    let nav = use_context::<AdminNav>().expect("admin nav");
    let courses = store_value(courses);
    let reload = create_rw_signal(0u32);
    let data = create_local_resource(move || (session.token.get(), reload.get()), |(t, _)| api::admin_payments(t));
    let filter = create_rw_signal(PayFilter::All);
    let search = create_rw_signal(String::new());
    let note = create_rw_signal(None::<(bool, String)>);

    let matches = move |p: &AdminPayment| -> bool {
        if let Some((uid, _)) = nav.user.get() {
            if p.user_id != Some(uid) {
                return false;
            }
        }
        let q = search.get().trim().to_lowercase();
        if !q.is_empty()
            && ![&p.email, &p.username, &p.customer_name, &p.phone, &p.provider_ref]
                .iter()
                .any(|f| f.to_lowercase().contains(&q))
        {
            return false;
        }
        match filter.get() {
            PayFilter::All => true,
            PayFilter::Charges => !p.provider_ref.starts_with("sub_"),
            PayFilter::Refunds => p.refunded_cents > 0,
            PayFilter::Subscriptions => p.provider_ref.starts_with("sub_"),
            PayFilter::Test => p.test_mode,
        }
    };

    let filter_button = move |f: PayFilter, key: &'static str| {
        view! {
            <button class="pg-tab" class:active={move || filter.get() == f} on:click=move |_| filter.set(f)>{session.t(key)}</button>
        }
    };

    view! {
        <p class="admin-hint">{session.t("admin.refund_hint")}</p>
        <div class="pay-filters">
            {filter_button(PayFilter::All, "admin.f_all")}
            {filter_button(PayFilter::Charges, "admin.f_charges")}
            {filter_button(PayFilter::Refunds, "admin.f_refunds")}
            {filter_button(PayFilter::Subscriptions, "admin.f_subs")}
            {filter_button(PayFilter::Test, "admin.f_test")}
        </div>
        <div class="admin-search">
            <input
                type="search"
                dir="auto"
                placeholder=session.t("admin.pay_search_ph")
                prop:value=move || search.get()
                on:input=move |ev| search.set(event_target_value(&ev))
            />
        </div>
        {move || nav.user.get().map(|(_, name)| view! {
            <p class="pay-user-filter">
                {session.t("admin.payments_of")} " " <bdi><strong>{name}</strong></bdi>
                " " <button class="link-btn small" on:click=move |_| nav.user.set(None)>"✕ " {session.t("admin.show_all")}</button>
            </p>
        })}
        {move || note.get().map(|(ok, msg)| view! { <div class={if ok { "ok-note" } else { "form-error" }}>{msg}</div> })}
        <Suspense fallback=|| view! { <Loading/> }>
            {move || data.get().map(|r| match r {
                Err(e) => view! { <div class="form-error">{e.message}</div> }.into_view(),
                Ok(list) => {
                    let lang = session.lang.get();
                    let shown: Vec<AdminPayment> = list.into_iter().filter(|p| matches(p)).collect();
                    // Totals of what is shown (real payments only).
                    let mut gross: Vec<(String, i64)> = Vec::new();
                    let mut back: Vec<(String, i64)> = Vec::new();
                    let add = |v: &mut Vec<(String, i64)>, cur: &str, n: i64| {
                        match v.iter_mut().find(|(c, _)| c == cur) {
                            Some(e) => e.1 += n,
                            None => v.push((cur.to_string(), n)),
                        }
                    };
                    for p in shown.iter().filter(|p| !p.test_mode && !p.provider_ref.starts_with("sub_")) {
                        if matches!(p.status.as_str(), "paid" | "partial_refund" | "refunded") {
                            add(&mut gross, &p.currency, p.amount_cents);
                        }
                        if p.refunded_cents > 0 {
                            add(&mut back, &p.currency, p.refunded_cents);
                        }
                    }
                    let fmt = |v: &Vec<(String, i64)>| if v.is_empty() { money(0, "USD") } else {
                        v.iter().map(|(c, n)| money(*n, c)).collect::<Vec<_>>().join(" + ")
                    };
                    let net: Vec<(String, i64)> = gross.iter().map(|(c, g)| {
                        let r = back.iter().find(|(c2, _)| c2 == c).map_or(0, |x| x.1);
                        (c.clone(), g - r)
                    }).collect();
                    let t = move |k| i18n::t(lang, k);
                    view! {
                        <div class="pay-totals">
                            <span>{t("admin.count_payments")} ": " <b>{shown.len()}</b></span>
                            <span>{t("admin.gross")} ": " <b dir="ltr">{fmt(&gross)}</b></span>
                            <span>{t("admin.refunded")} ": " <b dir="ltr">{fmt(&back)}</b></span>
                            <span>{t("admin.net")} ": " <b dir="ltr">{fmt(&net)}</b></span>
                        </div>
                        {shown.is_empty().then(|| view! { <div class="empty">{t("admin.no_payments")}</div> })}
                        <div class="pay-list">
                            {shown.into_iter().map(|p| {
                                let label = courses.with_value(|cs| course_label(cs, &p.course_id));
                                view! { <PaymentCard p=p course=label note=note reload=reload/> }
                            }).collect_view()}
                        </div>
                    }.into_view()
                }
            })}
        </Suspense>
    }
}

#[component]
fn PaymentCard(
    p: AdminPayment,
    course: String,
    note: RwSignal<Option<(bool, String)>>,
    reload: RwSignal<u32>,
) -> impl IntoView {
    let session = use_session();
    let lang = session.lang.get_untracked();
    let t = move |k| i18n::t(lang, k);
    let open = create_rw_signal(false);
    let busy = create_rw_signal(false);
    let amount = create_rw_signal(format!("{:.2}", p.refundable_cents as f64 / 100.0));
    let id = p.id;
    let refundable = p.refundable_cents;
    let currency = p.currency.clone();
    let who = if p.username.is_empty() { p.email.clone() } else { format!("{} ({})", p.username, p.email) };
    let kind = if p.provider_ref.starts_with("inv_") {
        t("admin.invoice")
    } else if p.provider_ref.starts_with("sub_") {
        t("admin.subscription")
    } else {
        t("admin.state_lifetime")
    };

    let refund = {
        let currency = currency.clone();
        let who = who.clone();
        move |_| {
            if busy.get_untracked() {
                return;
            }
            let Some(cents) = parse_cents(&amount.get_untracked()) else {
                note.set(Some((false, t("admin.amount_invalid").to_string())));
                return;
            };
            if cents > refundable {
                note.set(Some((false, tf(lang, "admin.amount_too_high", &[&money(refundable, &currency)]))));
                return;
            }
            let question = tf(lang, "admin.refund_confirm", &[&money(cents, &currency), &who]);
            if !ask(&question) {
                return;
            }
            busy.set(true);
            note.set(None);
            let token = session.token.get_untracked();
            let currency = currency.clone();
            spawn_local(async move {
                let r = api::admin_refund(token, id, cents).await;
                busy.set(false);
                match r {
                    Ok(_) => {
                        note.set(Some((true, tf(lang, "admin.refund_done", &[&money(cents, &currency)]))));
                        reload.update(|n| *n += 1);
                    }
                    Err(e) => note.set(Some((false, e.message))),
                }
            });
        }
    };
    let cancel = {
        let who = who.clone();
        move |_| {
            if busy.get_untracked() || !ask(&tf(lang, "admin.cancel_confirm", &[&who])) {
                return;
            }
            busy.set(true);
            note.set(None);
            let token = session.token.get_untracked();
            spawn_local(async move {
                let r = api::admin_cancel(token, id).await;
                busy.set(false);
                match r {
                    Ok(_) => {
                        note.set(Some((true, t("admin.cancel_done").to_string())));
                        reload.update(|n| *n += 1);
                    }
                    Err(e) => note.set(Some((false, e.message))),
                }
            });
        }
    };

    let raw_pretty = serde_json::from_str::<serde_json::Value>(&p.raw)
        .ok()
        .and_then(|v| serde_json::to_string_pretty(&v).ok())
        .unwrap_or_default();
    let details: Vec<(&'static str, String)> = vec![
        ("admin.d_ref", p.provider_ref.clone()),
        ("admin.d_customer", p.customer_name.clone()),
        ("admin.d_email", p.email.clone()),
        ("admin.d_phone", p.phone.clone()),
        ("admin.d_country", p.country.clone()),
        ("admin.d_user", p.user_id.map(|u| u.to_string()).unwrap_or_default()),
        ("admin.d_amount", money(p.amount_cents, &p.currency)),
        ("admin.d_refunded", money(p.refunded_cents, &p.currency)),
        ("admin.d_refundable", money(p.refundable_cents, &p.currency)),
        ("admin.d_until", p.until.map(|u| date(u, lang)).unwrap_or_default()),
        ("admin.d_created", date(p.created_at, lang)),
        ("admin.d_updated", date(p.updated_at, lang)),
        ("admin.d_test", if p.test_mode { "TEST".into() } else { "LIVE".into() }),
    ];

    view! {
        <div class="pay-card" class:test=p.test_mode>
            <div class="pay-head">
                <div class="pay-who">
                    <strong><bdi>{if p.username.is_empty() { "—".to_string() } else { p.username.clone() }}</bdi></strong>
                    <span class="muted" dir="ltr">{p.email.clone()}</span>
                    {(!p.phone.is_empty()).then(|| phone_links(&p.phone, &p.country))}
                </div>
                <div class="pay-amount" dir="ltr">
                    {if p.amount_cents > 0 {
                        money(p.amount_cents, &p.currency)
                    } else if p.provider_ref.starts_with("sub_") {
                        // A subscription itself carries no charge; show its price.
                        format!("{}{}", codesprout_core::access::PRICE_MONTHLY, t("admin.per_month"))
                    } else {
                        "—".into()
                    }}
                    {(p.refunded_cents > 0).then(|| view! { <small>"−" {money(p.refunded_cents, &p.currency)}</small> })}
                </div>
            </div>
            <div class="pay-meta">
                <span>{date(p.created_at, lang)}</span>
                <span class="course-label"><LangIcon id=p.course_id.clone()/> {course}</span>
                <span>{kind}</span>
                <span class=format!("tag status-{}", p.status) title=p.status.clone()>{status_label(lang, &p.status)}</span>
                {p.test_mode.then(|| view! { <span class="tag">"TEST"</span> })}
                {(!p.method.is_empty()).then(|| view! { <span>{p.method.clone()}</span> })}
                {p.until.map(|u| view! { <span>{t("admin.d_until")} ": " {date(u, lang)}</span> })}
            </div>
            <div class="pay-actions">
                <button class="btn btn-small btn-ghost" on:click=move |_| open.update(|o| *o = !*o)>
                    {move || if open.get() { t("admin.hide_details") } else { t("admin.details") }}
                </button>
                {p.can_cancel.then(|| view! {
                    <button class="btn btn-small btn-danger" disabled=move || busy.get() on:click=cancel.clone()>{t("admin.cancel_sub")}</button>
                })}
                <a class="btn btn-small btn-ghost" href=p.dashboard_url.clone() target="_blank" rel="noopener">{t("admin.open_in_ls")}</a>
            </div>
            {(refundable > 0).then(|| view! {
                <div class="refund-box">
                    <label>
                        <span>{t("admin.refund_amount")} " (" {currency.clone()} ")"</span>
                        <input type="text" inputmode="decimal" dir="ltr"
                            prop:value=move || amount.get()
                            on:input=move |ev| amount.set(event_target_value(&ev))/>
                    </label>
                    <span class="muted">{tf(lang, "admin.max_refund", &[&money(refundable, &currency)])}</span>
                    <button class="btn btn-small btn-ghost" on:click=move |_| amount.set(format!("{:.2}", refundable as f64 / 100.0))>
                        {t("admin.full_refund")}
                    </button>
                    <button class="btn btn-small btn-check" disabled=move || busy.get() on:click=refund.clone()>
                        {move || if busy.get() { t("auth.wait") } else { t("admin.refund") }}
                    </button>
                </div>
            })}
            {move || open.get().then(|| view! {
                <div class="pay-details">
                    <dl>
                        {details.iter().map(|(k, v)| view! { <dt>{t(*k)}</dt><dd dir="auto">{if v.is_empty() { "—".to_string() } else { v.clone() }}</dd> }).collect_view()}
                    </dl>
                    {(!raw_pretty.is_empty()).then(|| view! {
                        <p class="muted">{t("admin.raw")}</p>
                        <pre class="pay-raw" dir="ltr">{raw_pretty.clone()}</pre>
                    })}
                </div>
            })}
        </div>
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

/// Shows which payment settings are present and the store/variant ids the
/// Lemon Squeezy API key can see, to copy into Render.
#[component]
fn PaymentsSetup() -> impl IntoView {
    let session = use_session();
    let data = create_local_resource(move || session.token.get(), api::admin_payments_setup);
    view! {
        <Suspense fallback=|| ()>
            {move || data.get().and_then(|r| r.ok()).map(|v| {
                let present: Vec<(String, bool)> = v["present"]
                    .as_object()
                    .map(|o| o.iter().map(|(k, b)| (k.clone(), b.as_bool().unwrap_or(false))).collect())
                    .unwrap_or_default();
                let stores = v["stores"].as_array().cloned().unwrap_or_default();
                let variants = v["variants"].as_array().cloned().unwrap_or_default();
                view! {
                    <div class="admin-setup" dir="ltr">
                        <h3>"Lemon Squeezy → Render (Environment)"</h3>
                        <ul>
                            {present.into_iter().map(|(k, ok)| view! {
                                <li><span>{if ok { "✅" } else { "⬜" }}</span><code>{k}</code></li>
                            }).collect_view()}
                        </ul>
                        {(!stores.is_empty()).then(|| view! {
                            <p><b>"LEMONSQUEEZY_STORE_ID"</b></p>
                            <ul>{stores.iter().map(|s| view! {
                                <li><code>{s["id"].as_str().unwrap_or("").to_string()}</code>" — "{s["name"].as_str().unwrap_or("").to_string()}</li>
                            }).collect_view()}</ul>
                        })}
                        {(!variants.is_empty()).then(|| view! {
                            <p><b>"LEMONSQUEEZY_VARIANT_MONTHLY / LEMONSQUEEZY_VARIANT_LIFETIME"</b></p>
                            <ul>{variants.iter().map(|x| {
                                let price = x["price_cents"].as_f64().unwrap_or(0.0) / 100.0;
                                let kind = if x["subscription"].as_bool().unwrap_or(false) {
                                    format!("subscription / {}", x["interval"].as_str().unwrap_or(""))
                                } else {
                                    "one-time".to_string()
                                };
                                view! {
                                    <li>
                                        <code>{x["id"].as_str().unwrap_or("").to_string()}</code>
                                        " — " {x["product"].as_str().unwrap_or("").to_string()} " · "
                                        {x["name"].as_str().unwrap_or("").to_string()} " · "
                                        {format!("${price:.2}")} " · " {kind}
                                    </li>
                                }
                            }).collect_view()}</ul>
                        })}
                    </div>
                }
            })}
        </Suspense>
    }
}

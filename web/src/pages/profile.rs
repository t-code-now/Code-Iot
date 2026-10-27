//! Profile: level, streak, stats, badge collection and language setting.

use codesprout_core::progress::all_badges;
use leptos::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

use crate::components::avatar::UserAvatar;
use crate::components::icons::{IconBolt, IconFlame, IconShare, IconShield};

use crate::components::lang_switch::LangSwitch;
use crate::i18n::{self, tf, Lang};
use crate::session::use_session;

#[wasm_bindgen(inline_js = "export function share_text(text, url) {
  if (navigator.share) { navigator.share({ title: 'Code-Iot', text: text, url: url }).catch(() => {}); return 1; }
  if (navigator.clipboard) { navigator.clipboard.writeText(text + ' ' + url); return 2; }
  return 0;
}")]
extern "C" {
    fn share_text(text: &str, url: &str) -> u32;
}

/// League tier from total XP (a simple ladder; everyone starts in Bronze).
fn tier(xp: u64) -> (&'static str, &'static str) {
    match xp {
        0..=499 => ("tier.bronze", "bronze"),
        500..=1999 => ("tier.silver", "silver"),
        2000..=4999 => ("tier.gold", "gold"),
        _ => ("tier.diamond", "diamond"),
    }
}

fn course_name(id: &str) -> &str {
    match id {
        "python" => "🐍 Python",
        "javascript" => "⚡ JavaScript",
        "web" => "🌐 HTML & CSS",
        "sql" => "🗄️ SQL",
        "rust" => "🦀 Rust",
        "cpp" => "⚙️ C++",
        "arduino" => "🔌 Arduino",
        other => other,
    }
}

fn format_date(unix_secs: i64, lang: Lang) -> String {
    let date = js_sys::Date::new(&JsValue::from_f64(unix_secs as f64 * 1000.0));
    String::from(date.to_locale_date_string(i18n::locale(lang), &JsValue::UNDEFINED))
}

#[component]
pub fn ProfilePage() -> impl IntoView {
    let session = use_session();
    session.refresh_profile();

    let language_card = move || view! {
        <div class="setting-card">
            <span class="setting-label">{session.t("profile.language")}</span>
            <LangSwitch/>
        </div>
    };

    view! {
        <div class="page">
            <header class="topbar">
                <h1 class="topbar-title">{session.t("profile.title")}</h1>
            </header>
            {move || {
                let lang = session.lang.get();
                let t = move |key| i18n::t(lang, key);
                match session.profile.get() {
                    Some(p) => {
                        let level_pct = if p.level_xp_needed == 0 { 0 } else { p.level_xp * 100 / p.level_xp_needed };
                        let earned: Vec<String> = p.badges.iter().map(|b| b.id.clone()).collect();
                        view! {
                            <div class="profile-columns">
                            <div class="profile-col">
                            <div class="profile-card">
                                <a href="/avatar" class="avatar-edit" title=t("avatar.title")>
                                    <UserAvatar class="avatar big"/>
                                    <span class="avatar-edit-badge">"✎"</span>
                                </a>
                                <h2><bdi>{p.username.clone()}</bdi></h2>
                                <p class="muted">{tf(lang, "profile.joined", &[&format_date(p.joined_at, lang)])}</p>
                            </div>

                            <div class="big-stats">
                                <div class="big-stat streak">
                                    <span class="big-stat-icon"><IconFlame/></span>
                                    <strong>{p.streak}</strong>
                                    <span>{t("profile.day_streak")}</span>
                                </div>
                                <div class="big-stat xp">
                                    <span class="big-stat-icon"><IconBolt/></span>
                                    <strong>{p.xp}</strong>
                                    <span>"XP"</span>
                                </div>
                                <a href="/leaderboard" class=format!("big-stat league {}", tier(p.xp).1)>
                                    <span class="big-stat-icon"><IconShield/></span>
                                    <strong>{t(tier(p.xp).0)}</strong>
                                    <span>{t("profile.league")}</span>
                                </a>
                            </div>

                            <div class="best-streak">
                                <span class="best-streak-icon"><IconFlame/></span>
                                <strong>{tf(lang, "profile.days", &[&p.longest_streak])}</strong>
                                <span class="best-streak-label">{t("profile.best_streak")}</span>
                            </div>

                            {
                                let copied = create_rw_signal(false);
                                let text = tf(lang, "profile.share_text", &[&p.xp, &p.streak]);
                                view! {
                                    <button class="btn btn-outline btn-block share-btn" on:click=move |_| {
                                        let url = web_sys_origin();
                                        if share_text(&text, &url) == 2 { copied.set(true); }
                                    }>
                                        <IconShare/>{t("profile.share")}
                                    </button>
                                    {move || copied.get().then(|| view! { <p class="share-note">{t("profile.copied")}</p> })}
                                }
                            }

                            <div class="level-card">
                                <div class="level-row">
                                    <span class="level-chip">{tf(lang, "profile.level", &[&p.level])}</span>
                                    <span class="muted">{tf(lang, "profile.level_xp", &[&p.level_xp, &p.level_xp_needed])}</span>
                                </div>
                                <div class="bar"><div class="bar-fill" style=format!("width:{level_pct}%")></div></div>
                            </div>

                            {p.is_admin.then(|| view! {
                                <a href="/admin" class="btn btn-primary btn-block admin-link">"🛠 " {t("admin.title")}</a>
                            })}
                            {(!p.plans.is_empty()).then(|| view! {
                                <h3 class="section-title">{t("profile.plans")}</h3>
                                <div class="plans-list">
                                    {p.plans.iter().map(|pl| {
                                        let kind = match pl.kind.as_str() {
                                            "lifetime" => t("admin.state_lifetime").to_string(),
                                            "monthly" => match pl.until {
                                                Some(u) => tf(lang, "profile.until", &[&format_date(u, lang)]),
                                                None => t("admin.state_monthly").to_string(),
                                            },
                                            _ => t("admin.state_open").to_string(),
                                        };
                                        view! {
                                            <div class="plan-row">
                                                <strong>{course_name(&pl.course_id)}</strong>
                                                <span class="muted">{kind}</span>
                                                {pl.manage_url.clone().filter(|_| pl.kind == "monthly").map(|u| view! {
                                                    <a href=u target="_blank" rel="noopener">{t("profile.manage")}</a>
                                                })}
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            })}
                            </div>
                            <div class="profile-col">
                            <h3 class="section-title">{t("profile.more")}</h3>
                            <div class="stats-grid">
                                <div class="stat-tile"><span class="stat-emoji">"📅"</span><strong>{p.weekly_xp}</strong><span>{t("profile.week_xp")}</span></div>
                                <div class="stat-tile"><span class="stat-emoji">"📘"</span><strong>{p.lessons_completed}</strong><span>{t("profile.lessons")}</span></div>
                                <div class="stat-tile"><span class="stat-emoji">"🎓"</span><strong>{p.courses_completed}</strong><span>{t("profile.courses")}</span></div>
                                <div class="stat-tile"><span class="stat-emoji">"🧩"</span><strong>{p.badges.len()}</strong><span>{t("profile.badges")}</span></div>
                            </div>

                            {language_card}

                            <h3 class="section-title">{t("profile.badges")}</h3>
                            <div class="badge-grid">
                                {all_badges().into_iter().map(|b| {
                                    let has = earned.contains(&b.id);
                                    let (title, desc) = i18n::badge(lang, &b);
                                    view! {
                                        <div class="badge" class:earned=has title=desc.clone()>
                                            <span class="badge-icon">{b.icon}</span>
                                            <strong>{title}</strong>
                                            <span class="badge-desc">{desc}</span>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>

                            <button class="btn btn-ghost btn-block logout" on:click=move |_| session.sign_out()>{t("profile.logout")}</button>
                            </div>
                            </div>
                        }.into_view()
                    }
                    None if session.logged_in() => view! { <div class="center-load"><div class="spinner"></div></div> }.into_view(),
                    None => view! {
                        <>
                        <div class="profile-card">
                            <img class="avatar big mascot" src="/assets/mascot.png" alt=""/>
                            <h2>{t("profile.guest")}</h2>
                            <p class="muted">{t("profile.guest_desc")}</p>
                        </div>
                        {language_card}
                        <div class="guest-note">
                            <a href="/signup" class="btn btn-primary btn-block">{t("profile.create_free")}</a>
                            <a href="/login" class="btn btn-ghost btn-block">{t("auth.login")}</a>
                        </div>
                        </>
                    }.into_view(),
                }
            }}
        </div>
    }
}

fn web_sys_origin() -> String {
    js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("location"))
        .and_then(|loc| js_sys::Reflect::get(&loc, &JsValue::from_str("origin")))
        .ok()
        .and_then(|v| v.as_string())
        .filter(|o| o.starts_with("http") && !o.contains("tauri.localhost"))
        .unwrap_or_else(|| "https://code-iot.onrender.com".to_string())
}

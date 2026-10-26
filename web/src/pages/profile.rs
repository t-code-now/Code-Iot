//! Profile: level, streak, stats, badge collection and language setting.

use codesprout_core::progress::all_badges;
use leptos::*;
use wasm_bindgen::JsValue;

use crate::components::lang_switch::LangSwitch;
use crate::i18n::{self, tf, Lang};
use crate::session::use_session;

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
                        let initial = p.username.chars().next().unwrap_or('?').to_ascii_uppercase().to_string();
                        let level_pct = if p.level_xp_needed == 0 { 0 } else { p.level_xp * 100 / p.level_xp_needed };
                        let earned: Vec<String> = p.badges.iter().map(|b| b.id.clone()).collect();
                        view! {
                            <>
                            <div class="profile-card">
                                <div class="avatar big">{initial}</div>
                                <h2><bdi>{p.username.clone()}</bdi></h2>
                                <p class="muted">{tf(lang, "profile.joined", &[&format_date(p.joined_at, lang)])}</p>
                            </div>

                            <div class="level-card">
                                <div class="level-row">
                                    <span class="level-chip">{tf(lang, "profile.level", &[&p.level])}</span>
                                    <span class="muted">{tf(lang, "profile.level_xp", &[&p.level_xp, &p.level_xp_needed])}</span>
                                </div>
                                <div class="bar"><div class="bar-fill" style=format!("width:{level_pct}%")></div></div>
                            </div>

                            <div class="stats-grid">
                                <div class="stat-tile"><span class="stat-emoji">"🔥"</span><strong>{p.streak}</strong><span>{t("profile.day_streak")}</span></div>
                                <div class="stat-tile"><span class="stat-emoji">"⚡"</span><strong>{p.xp}</strong><span>{t("profile.total_xp")}</span></div>
                                <div class="stat-tile"><span class="stat-emoji">"📅"</span><strong>{p.weekly_xp}</strong><span>{t("profile.week_xp")}</span></div>
                                <div class="stat-tile"><span class="stat-emoji">"🏅"</span><strong>{p.longest_streak}</strong><span>{t("profile.best_streak")}</span></div>
                                <div class="stat-tile"><span class="stat-emoji">"📘"</span><strong>{p.lessons_completed}</strong><span>{t("profile.lessons")}</span></div>
                                <div class="stat-tile"><span class="stat-emoji">"🎓"</span><strong>{p.courses_completed}</strong><span>{t("profile.courses")}</span></div>
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
                            </>
                        }.into_view()
                    }
                    None if session.logged_in() => view! { <div class="center-load"><div class="spinner"></div></div> }.into_view(),
                    None => view! {
                        <>
                        <div class="profile-card">
                            <div class="avatar big">"🌱"</div>
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

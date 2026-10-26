//! Weekly league and all-time leaderboard.

use leptos::*;

use crate::api;
use crate::components::shell::StatPills;
use crate::i18n::{self, tf};
use crate::pages::{ErrorState, Loading};
use crate::session::use_session;

#[component]
pub fn LeaderboardPage() -> impl IntoView {
    let session = use_session();
    let scope = create_rw_signal("weekly");
    let board = create_local_resource(
        move || (scope.get(), session.token.get()),
        |(scope, token)| async move { api::leaderboard(scope, token).await },
    );

    view! {
        <div class="page">
            <header class="topbar">
                <h1 class="topbar-title">{session.t("league.title")}</h1>
                <StatPills/>
            </header>

            <div class="league-banner">
                <div class="league-emblem">"🏆"</div>
                <div>
                    <h2>{session.t("league.name")}</h2>
                    <p>{session.t("league.desc")}</p>
                </div>
            </div>

            <div class="segmented">
                <button class:active=move || scope.get() == "weekly" on:click=move |_| scope.set("weekly")>
                    {session.t("league.week")}
                </button>
                <button class:active=move || scope.get() == "all_time" on:click=move |_| scope.set("all_time")>
                    {session.t("league.all")}
                </button>
            </div>

            <Transition fallback=|| view! { <Loading/> }>
                {move || board.get().map(|r| match r {
                    Ok(b) if b.entries.is_empty() => view! {
                        <div class="empty">{session.t("league.empty")}</div>
                    }.into_view(),
                    Ok(b) => view! {
                        <ol class="ranking">
                            {b.entries.into_iter().map(|e| {
                                let lang = session.lang.get_untracked();
                                let initial = e.username.chars().next().unwrap_or('?').to_ascii_uppercase().to_string();
                                let medal = match e.rank { 1 => "🥇", 2 => "🥈", 3 => "🥉", _ => "" };
                                view! {
                                    <li class="rank-row" class:me=e.is_me class:top={e.rank <= 3}>
                                        <span class="rank-num">
                                            {if medal.is_empty() { e.rank.to_string() } else { medal.to_string() }}
                                        </span>
                                        <span class="avatar">{initial}</span>
                                        <span class="rank-name">
                                            <bdi>{e.username}</bdi>
                                            {e.is_me.then(|| view! { <span class="you-tag">{i18n::t(lang, "league.you")}</span> })}
                                        </span>
                                        <span class="rank-xp">{tf(lang, "league.xp", &[&e.xp])}</span>
                                    </li>
                                }
                            }).collect_view()}
                        </ol>
                    }.into_view(),
                    Err(e) => view! { <ErrorState message=e.message/> }.into_view(),
                })}
            </Transition>

            {move || (!session.logged_in()).then(|| view! {
                <div class="guest-note">
                    <p>{session.t("league.guest")}</p>
                    <a href="/signup" class="btn btn-primary btn-block">{session.t("league.create")}</a>
                </div>
            })}
        </div>
    }
}

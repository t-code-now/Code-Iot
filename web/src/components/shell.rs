//! The main app frame: page content plus the bottom tab bar.

use leptos::*;
use leptos_router::*;

use crate::components::icons::*;
use crate::session::use_session;

#[component]
pub fn Shell() -> impl IntoView {
    let session = use_session();
    view! {
        <>
        {move || (!session.onboarded()).then(|| view! { <Redirect path="/welcome"/> })}
        <div class="app">
            <main class="screen">
                <Outlet/>
            </main>
            <TabBar/>
        </div>
        </>
    }
}

#[component]
fn TabBar() -> impl IntoView {
    let session = use_session();
    let pathname = use_location().pathname; // Memo<String> is Copy
    let is = move |prefix: &'static str| {
        move || {
            let path = pathname.get();
            if prefix == "/" {
                path == "/" || path.starts_with("/courses")
            } else {
                path.starts_with(prefix)
            }
        }
    };
    view! {
        <nav class="tabbar">
            <a href="/" class="tab" class:active=is("/")>
                <IconLearn/><span>{session.t("tab.learn")}</span>
            </a>
            <a href="/playground" class="tab" class:active=is("/playground")>
                <IconCode/><span>{session.t("tab.playground")}</span>
            </a>
            <a href="/leaderboard" class="tab" class:active=is("/leaderboard")>
                <IconTrophy/><span>{session.t("tab.league")}</span>
            </a>
            <a href="/profile" class="tab" class:active=is("/profile")>
                <IconUser/><span>{session.t("tab.profile")}</span>
            </a>
        </nav>
    }
}

/// Streak and XP pills shown at the end of page headers.
#[component]
pub fn StatPills() -> impl IntoView {
    let session = use_session();
    let streak = move || session.profile.with(|p| p.as_ref().map_or(0, |p| p.streak));
    let xp = move || session.profile.with(|p| p.as_ref().map_or(0, |p| p.xp));
    view! {
        <div class="pills">
            <span class="pill pill-streak" class:lit=move || streak() > 0>
                <span class="pill-icon">"🔥"</span>{streak}
            </span>
            <span class="pill pill-xp">
                <span class="pill-icon">"⚡"</span>{xp}
            </span>
        </div>
    }
}

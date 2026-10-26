pub mod auth;
pub mod avatar;
pub mod courses;
pub mod home;
pub mod leaderboard;
pub mod lesson;
pub mod playground;
pub mod profile;

use leptos::*;

/// Centered spinner used while a page loads.
#[component]
pub fn Loading() -> impl IntoView {
    view! { <div class="center-load"><div class="spinner"></div></div> }
}

/// Friendly error with a way back.
#[component]
pub fn ErrorState(#[prop(into)] message: String) -> impl IntoView {
    let message = crate::i18n::error(crate::session::use_session().lang.get_untracked(), &message);
    view! {
        <div class="error-state">
            <div class="error-emoji">"🤔"</div>
            <p>{message}</p>
            <a href="/" class="btn btn-primary">{crate::session::use_session().t("back.learn")}</a>
        </div>
    }
}

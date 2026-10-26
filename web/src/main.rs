//! Code-Iot web app (Leptos, client-side rendered to WebAssembly).

mod api;
mod components;
mod highlight;
mod i18n;
mod pages;
mod session;

use leptos::*;
use leptos_router::*;

use crate::components::code_bg::CodeBackground;
use crate::components::shell::Shell;
use crate::pages::auth::{AuthMode, AuthPage, Welcome};
use crate::pages::avatar::AvatarPage;
use crate::pages::courses::Courses;
use crate::pages::home::Home;
use crate::pages::leaderboard::LeaderboardPage;
use crate::pages::lesson::LessonPage;
use crate::pages::playground::Playground;
use crate::pages::profile::ProfilePage;
use crate::session::Session;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> })
}

#[component]
fn App() -> impl IntoView {
    let session = Session::load();
    provide_context(session);
    session.refresh_profile();

    // Keep <html lang> and <html dir> in sync with the chosen language, so
    // the whole layout flips to right-to-left for Hebrew.
    create_effect(move |_| {
        let lang = session.lang.get();
        if let Some(root) = document().document_element() {
            let _ = root.set_attribute("lang", lang.code());
            let _ = root.set_attribute("dir", if lang.is_rtl() { "rtl" } else { "ltr" });
        }
    });

    view! {
        <Router>
            <CodeBackground/>
            <Routes>
                <Route path="/welcome" view=Welcome/>
                <Route path="/login" view=|| view! { <AuthPage mode=AuthMode::Login/> }/>
                <Route path="/signup" view=|| view! { <AuthPage mode=AuthMode::Signup/> }/>
                <Route path="/avatar" view=AvatarPage/>
                <Route path="/lesson/:id" view=LessonPage/>
                <Route path="/" view=Shell>
                    <Route path="" view=Home/>
                    <Route path="courses" view=Courses/>
                    <Route path="playground" view=Playground/>
                    <Route path="leaderboard" view=LeaderboardPage/>
                    <Route path="profile" view=ProfilePage/>
                </Route>
            </Routes>
        </Router>
    }
}

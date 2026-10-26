//! Welcome, sign-up and log-in screens.

use leptos::*;
use leptos_router::*;

use crate::api;
use crate::components::lang_switch::LangSwitch;
use crate::i18n;
use crate::session::use_session;

#[component]
pub fn Welcome() -> impl IntoView {
    let session = use_session();
    let navigate = use_navigate();
    let guest = move |_| {
        session.continue_as_guest();
        navigate("/", Default::default());
    };
    view! {
        <div class="welcome">
            <div class="welcome-top"><LangSwitch/></div>
            <div class="welcome-hero">
                <div class="logo-mark">"🌱"</div>
                <h1 class="welcome-title">"Code-Iot"</h1>
                <p class="welcome-sub">{session.t("welcome.sub")}</p>
                <div class="welcome-langs">
                    <span>"🐍"</span><span>"⚡"</span><span>"🌐"</span><span>"🗄️"</span><span>"🦀"</span><span>"🔌"</span>
                </div>
            </div>
            <div class="welcome-actions">
                <a href="/signup" class="btn btn-primary btn-block">{session.t("welcome.start")}</a>
                <a href="/login" class="btn btn-ghost btn-block">{session.t("welcome.have_account")}</a>
                <button class="link-btn" on:click=guest>{session.t("welcome.guest")}</button>
            </div>
        </div>
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AuthMode {
    Login,
    Signup,
}

#[component]
pub fn AuthPage(mode: AuthMode) -> impl IntoView {
    let session = use_session();
    let navigate = use_navigate();
    let username = create_rw_signal(String::new());
    let password = create_rw_signal(String::new());
    let error = create_rw_signal(None::<String>);
    let busy = create_rw_signal(false);

    let (title, button, switch_text, switch_href, switch_label) = match mode {
        AuthMode::Signup => ("auth.signup_title", "auth.signup", "auth.have_account", "/login", "auth.login"),
        AuthMode::Login => ("auth.login_title", "auth.login", "auth.new_here", "/signup", "auth.create_account"),
    };

    let submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        error.set(None);
        let (u, p) = (username.get_untracked(), password.get_untracked());
        let lang = session.lang.get_untracked();
        let navigate = navigate.clone();
        spawn_local(async move {
            let result = match mode {
                AuthMode::Signup => api::signup(u, p).await,
                AuthMode::Login => api::login(u, p).await,
            };
            busy.set(false);
            match result {
                Ok(r) => {
                    session.sign_in(r.token, r.profile);
                    navigate("/", Default::default());
                }
                Err(e) => error.set(Some(i18n::error(lang, &e.message))),
            }
        });
    };

    view! {
        <div class="auth">
            <a href="/welcome" class="back-link">{session.t("back")}</a>
            <h1 class="auth-title">{session.t(title)}</h1>
            <form class="auth-form" on:submit=submit>
                <label class="field">
                    <span>{session.t("auth.username")}</span>
                    <input
                        type="text"
                        dir="ltr"
                        autocomplete="username"
                        autocapitalize="off"
                        placeholder=session.t("auth.username_ph")
                        prop:value=move || username.get()
                        on:input=move |ev| username.set(event_target_value(&ev))
                    />
                </label>
                <label class="field">
                    <span>{session.t("auth.password")}</span>
                    <input
                        type="password"
                        dir="ltr"
                        autocomplete={if mode == AuthMode::Signup { "new-password" } else { "current-password" }}
                        placeholder=session.t("auth.password_ph")
                        prop:value=move || password.get()
                        on:input=move |ev| password.set(event_target_value(&ev))
                    />
                </label>
                {move || error.get().map(|e| view! { <div class="form-error">{e}</div> })}
                <button type="submit" class="btn btn-primary btn-block" disabled=move || busy.get()>
                    {move || if busy.get() { session.t("auth.wait")() } else { session.t(button)() }}
                </button>
            </form>
            <p class="auth-switch">
                {session.t(switch_text)} " " <a href=switch_href>{session.t(switch_label)}</a>
            </p>
        </div>
    }
}

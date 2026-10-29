//! Welcome, sign-up and log-in screens.

use leptos::*;
use crate::components::icons::IconBack;
use leptos_router::*;

use crate::api;
use crate::components::google::GoogleButton;
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
                <div class="logo-mark">
                    <img src="/assets/mascot.png" alt="Code-Iot"/>
                </div>
                <h1 class="welcome-title">"Code-Iot"</h1>
            </div>
            <div class="welcome-actions">
                <p class="welcome-sub">{session.t("welcome.sub")}</p>
                <a href="/signup" class="btn btn-primary btn-block">{session.t("welcome.start")}</a>
                <a href="/login" class="btn btn-ghost btn-block">{session.t("welcome.have_account")}</a>
                <GoogleButton/>
                <button class="link-btn" on:click=guest>{session.t("welcome.guest")}</button>
                <div class="legal-links">
                    <a href="/privacy">{session.t("legal.privacy")}</a>
                    <span>"·"</span>
                    <a href="/terms">{session.t("legal.terms")}</a>
                </div>
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
    // New accounts are created with Google only.
    if mode == AuthMode::Signup {
        return view! {
            <div class="auth">
                <a href="/welcome" class="back-link"><IconBack/>{session.t("back.word")}</a>
                <img class="auth-logo" src="/assets/mascot.png" alt=""/>
                <h1 class="auth-title">{session.t("auth.signup_title")}</h1>
                <div class="auth-form signup-google">
                    <p class="signup-why">{session.t("auth.signup_google")}</p>
                    <GoogleButton/>
                </div>
                <p class="auth-switch">
                    {session.t("auth.have_account")} " " <a href="/login">{session.t("auth.login")}</a>
                </p>
            </div>
        }
        .into_view();
    }
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
                    // New accounts pick a profile picture first (they can skip).
                    let next = if mode == AuthMode::Signup { "/avatar?first=1" } else { "/" };
                    navigate(next, Default::default());
                }
                Err(e) => error.set(Some(i18n::error(lang, &e.message))),
            }
        });
    };

    view! {
        <div class="auth">
            <a href="/welcome" class="back-link"><IconBack/>{session.t("back.word")}</a>
            <img class="auth-logo" src="/assets/mascot.png" alt=""/>
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
            <div class="auth-or"><span>{session.t("auth.or")}</span></div>
            <GoogleButton/>
            <p class="auth-switch">
                {session.t(switch_text)} " " <a href=switch_href>{session.t(switch_label)}</a>
            </p>
        </div>
    }
    .into_view()
}

//! Global app state shared through Leptos context, persisted in localStorage.

use codesprout_core::api::Profile;
use gloo_storage::{LocalStorage, Storage};
use leptos::*;

use crate::i18n::{self, Lang};

const TOKEN_KEY: &str = "cs.token";
const GUEST_KEY: &str = "cs.guest";
const COURSE_KEY: &str = "cs.course";
const LANG_KEY: &str = "cs.lang";

#[derive(Clone, Copy)]
pub struct Session {
    pub token: RwSignal<Option<String>>,
    pub profile: RwSignal<Option<Profile>>,
    /// Browsing without an account (progress isn't saved).
    pub guest: RwSignal<bool>,
    /// The course shown on the Learn tab.
    pub course: RwSignal<String>,
    /// Interface and content language.
    pub lang: RwSignal<Lang>,
}

pub fn use_session() -> Session {
    expect_context::<Session>()
}

/// A one-time yearly purchase shows a renewal reminder this many days before it ends.
pub const RENEW_NOTICE_DAYS: i64 = 14;

impl Session {
    /// When the user's yearly access to `course` ends, if that is within the
    /// next `RENEW_NOTICE_DAYS` days (tracked).
    pub fn plan_ending(&self, course: &str) -> Option<i64> {
        let now = (js_sys::Date::now() / 1000.0) as i64;
        self.profile.with(|p| {
            p.as_ref()?
                .plans
                .iter()
                .find(|pl| pl.course_id == course && pl.kind == "yearly")
                .and_then(|pl| pl.until)
                .filter(|u| *u > now && *u - now <= RENEW_NOTICE_DAYS * 86_400)
        })
    }

    pub fn load() -> Self {
        let lang = LocalStorage::get::<String>(LANG_KEY).map(|c| Lang::parse(&c)).unwrap_or_default();
        Session {
            token: create_rw_signal(LocalStorage::get::<String>(TOKEN_KEY).ok()),
            profile: create_rw_signal(None),
            guest: create_rw_signal(LocalStorage::get::<bool>(GUEST_KEY).unwrap_or(false)),
            course: create_rw_signal(
                LocalStorage::get::<String>(COURSE_KEY)
                    .ok()
                    .filter(|c| !c.is_empty())
                    .unwrap_or_else(|| "python".to_string()),
            ),
            lang: create_rw_signal(lang),
        }
    }

    /// A reactive closure returning the UI string for `key` in the current language.
    pub fn t(&self, key: &'static str) -> impl Fn() -> &'static str + Copy + 'static {
        let lang = self.lang;
        move || i18n::t(lang.get(), key)
    }

    /// The current language (tracked).
    pub fn lang(&self) -> Lang {
        self.lang.get()
    }

    pub fn set_lang(&self, lang: Lang) {
        let _ = LocalStorage::set(LANG_KEY, lang.code());
        self.lang.set(lang);
    }

    pub fn logged_in(&self) -> bool {
        self.token.with(Option::is_some)
    }

    /// Has the user picked "log in", "sign up" or "continue as guest"?
    pub fn onboarded(&self) -> bool {
        self.logged_in() || self.guest.get()
    }

    pub fn sign_in(&self, token: String, profile: Profile) {
        let _ = LocalStorage::set(TOKEN_KEY, &token);
        LocalStorage::delete(GUEST_KEY);
        self.token.set(Some(token));
        self.profile.set(Some(profile));
        self.guest.set(false);
    }

    pub fn continue_as_guest(&self) {
        let _ = LocalStorage::set(GUEST_KEY, true);
        self.guest.set(true);
    }

    pub fn sign_out(&self) {
        let token = self.token.get_untracked();
        spawn_local(async move {
            let _ = crate::api::logout(token).await;
        });
        LocalStorage::delete(TOKEN_KEY);
        LocalStorage::delete(GUEST_KEY);
        self.token.set(None);
        self.profile.set(None);
        self.guest.set(false);
    }

    pub fn set_course(&self, id: &str) {
        if id.is_empty() {
            return;
        }
        let _ = LocalStorage::set(COURSE_KEY, id);
        self.course.set(id.to_string());
    }

    /// Re-fetch the profile (XP, streak…). Clears a stale token on 401.
    pub fn refresh_profile(&self) {
        let session = *self;
        let token = session.token.get_untracked();
        if token.is_none() {
            return;
        }
        spawn_local(async move {
            match crate::api::me(token).await {
                Ok(p) => session.profile.set(Some(p)),
                Err(e) if e.is_unauthorized() => {
                    LocalStorage::delete(TOKEN_KEY);
                    session.token.set(None);
                    session.profile.set(None);
                }
                Err(_) => {}
            }
        });
    }
}

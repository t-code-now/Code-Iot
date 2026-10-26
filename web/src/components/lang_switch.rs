//! English / עברית toggle. Switching flips the whole layout direction and
//! reloads course content in the chosen language.

use leptos::*;

use crate::i18n::Lang;
use crate::session::use_session;

#[component]
pub fn LangSwitch() -> impl IntoView {
    let session = use_session();
    let option = move |lang: Lang, label: &'static str| {
        view! {
            <button
                class="lang-option"
                class:active=move || session.lang.get() == lang
                lang=lang.code()
                on:click=move |_| session.set_lang(lang)
            >
                {label}
            </button>
        }
    };
    view! {
        <div class="lang-switch" role="group" aria-label="Language / שפה">
            {option(Lang::En, "English")}
            {option(Lang::He, "עברית")}
        </div>
    }
}

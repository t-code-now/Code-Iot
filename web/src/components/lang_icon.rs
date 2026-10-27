//! The icon of a course's language, used everywhere a course or language is
//! shown. Official logos are used where their owners allow it (JavaScript,
//! HTML, CSS, C++). Python and Rust keep their emoji until the PSF and the
//! Rust Foundation answer our permission requests. Arduino's guidelines do
//! not allow its logo to identify a paid course, and SQL has no official
//! logo, so both get original drawings (a board and a database).

use leptos::*;

enum Icon {
    Files(&'static [&'static str]),
    Emoji(&'static str),
}

fn icon(id: &str) -> Icon {
    match id {
        "javascript" => Icon::Files(&["javascript"]),
        "web" => Icon::Files(&["html5", "css"]),
        "cpp" => Icon::Files(&["cpp"]),
        "arduino" => Icon::Files(&["board"]),
        "sql" => Icon::Files(&["database"]),
        "python" => Icon::Emoji("🐍"),
        "rust" => Icon::Emoji("🦀"),
        _ => Icon::Emoji("📘"),
    }
}

#[component]
pub fn LangIcon(#[prop(into)] id: String) -> impl IntoView {
    match icon(&id) {
        Icon::Files(names) => view! {
            <span class=format!("lang-icon lang-{id}") class:lang-pair={names.len() > 1} aria-hidden="true">
                {names.iter().map(|n| view! { <img src=format!("/assets/lang/{n}.svg") alt="" draggable="false"/> }).collect_view()}
            </span>
        }
        .into_view(),
        Icon::Emoji(e) => view! { <span class=format!("lang-icon lang-emoji lang-{id}") aria-hidden="true">{e}</span> }.into_view(),
    }
}

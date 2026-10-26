//! Profile pictures: built-in avatars or a photo the user uploaded.

use leptos::*;

use crate::session::use_session;

/// Image URL for a stored avatar value ("preset:<name>" or a data URL).
pub fn avatar_src(avatar: &str) -> Option<String> {
    match avatar.strip_prefix("preset:") {
        Some("mascot") => Some("/assets/mascot.png".into()),
        Some(name) if name.chars().all(|c| c.is_ascii_alphanumeric()) => Some(format!("/assets/avatars/{name}.jpg")),
        Some(_) => None,
        None if avatar.starts_with("data:image/") => Some(avatar.to_string()),
        None => None,
    }
}

/// The signed-in user's picture. Without one: their initial when signed in,
/// the Code-Iot mascot for guests.
#[component]
pub fn UserAvatar(#[prop(into)] class: String) -> impl IntoView {
    let session = use_session();
    move || {
        let class = class.clone();
        session.profile.with(|p| match p {
            Some(p) => match p.avatar.as_deref().and_then(avatar_src) {
                Some(src) => view! { <img class=format!("{class} has-photo") src=src alt=""/> }.into_view(),
                None => {
                    let initial = p.username.chars().next().unwrap_or('?').to_uppercase().to_string();
                    view! { <div class=class>{initial}</div> }.into_view()
                }
            },
            None => view! { <img class=format!("{class} has-photo") src="/assets/mascot.png" alt=""/> }.into_view(),
        })
    }
}

//! Simple line icons (original drawings) used by the tab bar and headers.

use leptos::*;

#[component]
pub fn IconLearn() -> impl IntoView {
    view! {
        <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M4 5.5A2.5 2.5 0 0 1 6.5 3H20v15H6.5A2.5 2.5 0 0 0 4 20.5z"/>
            <path d="M4 20.5A2.5 2.5 0 0 0 6.5 23H20v-5"/>
            <path d="M9 8h7M9 12h5"/>
        </svg>
    }
}

#[component]
pub fn IconCode() -> impl IntoView {
    view! {
        <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M8 7l-5 5 5 5M16 7l5 5-5 5M13.5 4l-3 16"/>
        </svg>
    }
}

#[component]
pub fn IconTrophy() -> impl IntoView {
    view! {
        <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M7 4h10v5a5 5 0 0 1-10 0z"/>
            <path d="M7 6H4v1a3 3 0 0 0 3 3M17 6h3v1a3 3 0 0 1-3 3"/>
            <path d="M12 14v4M8 21h8M9 18h6"/>
        </svg>
    }
}

#[component]
pub fn IconUser() -> impl IntoView {
    view! {
        <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="8" r="4"/>
            <path d="M4 21a8 8 0 0 1 16 0"/>
        </svg>
    }
}

#[component]
pub fn IconClose() -> impl IntoView {
    view! {
        <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
            <path d="M6 6l12 12M18 6L6 18"/>
        </svg>
    }
}

#[component]
pub fn IconPlay() -> impl IntoView {
    view! {
        <svg class="icon" viewBox="0 0 24 24" fill="currentColor">
            <path d="M8 5.5v13a1 1 0 0 0 1.5.86l11-6.5a1 1 0 0 0 0-1.72l-11-6.5A1 1 0 0 0 8 5.5z"/>
        </svg>
    }
}

#[component]
pub fn IconBolt() -> impl IntoView {
    view! {
        <svg class="icon" viewBox="0 0 24 24" fill="currentColor">
            <path d="M13.2 2.5 4.8 13.4a.8.8 0 0 0 .6 1.3H11l-1.2 6.8a.5.5 0 0 0 .9.4l8.4-10.9a.8.8 0 0 0-.6-1.3H13l1.1-6.8a.5.5 0 0 0-.9-.4z"/>
        </svg>
    }
}

#[component]
pub fn IconCheck() -> impl IntoView {
    view! {
        <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3.2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M5 12.5l4.5 4.5L19 7.5"/>
        </svg>
    }
}

/// A jigsaw piece: the mark for project lessons and section banners.
#[component]
pub fn IconPuzzle() -> impl IntoView {
    view! {
        <svg class="icon" viewBox="0 0 24 24" fill="currentColor">
            <path d="M9.5 3a2.5 2.5 0 0 1 2.5 2.5V6h4a1 1 0 0 1 1 1v4h.5a2.5 2.5 0 0 1 0 5H17v4a1 1 0 0 1-1 1h-3.5v-.8a2 2 0 1 0-4 0v.8H5a1 1 0 0 1-1-1v-3.5h.8a2 2 0 1 0 0-4H4V7a1 1 0 0 1 1-1h2v-.5A2.5 2.5 0 0 1 9.5 3z"/>
        </svg>
    }
}

#[component]
pub fn IconFlame() -> impl IntoView {
    view! {
        <svg class="icon" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12.6 2.3c.2 3-1.7 4.6-3.2 6.2C7.9 10.1 6.5 11.8 6.5 14.6A5.5 5.5 0 0 0 12 20.5a5.5 5.5 0 0 0 5.5-5.6c0-2.2-.9-3.9-2-5.2-.2 1.3-.8 2.3-1.8 2.8.4-3.6-.3-7.6-1.1-10.2zM12 18.5a2.3 2.3 0 0 1-2.3-2.3c0-1.5 1.1-2.3 2-3.4.3 1 .9 1.6 1.6 2.1.6.4 1 .9 1 1.6a2.3 2.3 0 0 1-2.3 2z"/>
        </svg>
    }
}

#[component]
pub fn IconShare() -> impl IntoView {
    view! {
        <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 3v12M7.5 7.5 12 3l4.5 4.5"/>
            <path d="M5 12v6.5A2.5 2.5 0 0 0 7.5 21h9a2.5 2.5 0 0 0 2.5-2.5V12"/>
        </svg>
    }
}

#[component]
pub fn IconShield() -> impl IntoView {
    view! {
        <svg class="icon" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 2.5 4.5 5.3v6.1c0 4.6 3.1 8.6 7.5 10.1 4.4-1.5 7.5-5.5 7.5-10.1V5.3z"/>
        </svg>
    }
}

#[component]
pub fn IconLock() -> impl IntoView {
    view! {
        <svg class="icon" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 2.5a4.5 4.5 0 0 0-4.5 4.5v3H6.5A1.5 1.5 0 0 0 5 11.5v8A1.5 1.5 0 0 0 6.5 21h11a1.5 1.5 0 0 0 1.5-1.5v-8a1.5 1.5 0 0 0-1.5-1.5h-1V7A4.5 4.5 0 0 0 12 2.5zm-2.5 4.5a2.5 2.5 0 0 1 5 0v3h-5zm2.5 6.5a1.6 1.6 0 0 1 .8 3v1.6h-1.6v-1.6a1.6 1.6 0 0 1 .8-3z"/>
        </svg>
    }
}

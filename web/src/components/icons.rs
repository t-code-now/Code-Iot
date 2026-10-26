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

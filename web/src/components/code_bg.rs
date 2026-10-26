//! The living background ("code fog"): faint, blurred words of code drift
//! slowly upward behind every screen. Every 1.5 s two or three of them light
//! up in a neon colour for 4 s and fade back into the fog. On the learning
//! path the background is a still picture: no drifting and no glowing.

use std::time::Duration;

use leptos::*;
use leptos_router::use_location;

const WORDS: &[&str] = &[
    "const", "let", "function()", "=>", "<div>", "import", "from", "return;", "{}", "[]",
    "if()", "else", "true", "false", "class", "await", "async", "console.log", "===", "null",
    "export", "map()", "filter()", "<App />",
    "def", "print()", "self", "SELECT", "WHERE", "fn main()", "let mut", "impl",
    "#include", "std::cout", "int main()", "void loop()", "digitalWrite()",
];

const NEON: &[&str] = &["#ff007f", "#00f3ff", "#f1fa8c", "#50fa7b"];

const COUNT: usize = 45;

fn rnd() -> f64 {
    js_sys::Math::random()
}

fn pick<T: Copy>(items: &[T]) -> T {
    items[((rnd() * items.len() as f64) as usize).min(items.len() - 1)]
}

#[component]
pub fn CodeBackground() -> impl IntoView {
    let pathname = use_location().pathname;
    // The learning path ("/") gets the still version.
    let still = move || pathname.get() == "/";

    // Neon colour of each word while it glows; None = resting in the fog.
    let glow: Vec<RwSignal<Option<&'static str>>> = (0..COUNT).map(|_| create_rw_signal(None)).collect();

    {
        let glow = glow.clone();
        let tick = move || {
            if pathname.get_untracked() == "/" {
                return;
            }
            let mut free: Vec<usize> = (0..COUNT).filter(|&i| glow[i].get_untracked().is_none()).collect();
            let n = 2 + (rnd() * 2.0) as usize; // 2 or 3
            for _ in 0..n {
                if free.is_empty() {
                    break;
                }
                let i = free.swap_remove(((rnd() * free.len() as f64) as usize).min(free.len() - 1));
                let word = glow[i];
                word.set(Some(pick(NEON)));
                set_timeout(move || word.set(None), Duration::from_millis(4000));
            }
        };
        if let Ok(handle) = set_interval_with_handle(tick, Duration::from_millis(1500)) {
            on_cleanup(move || handle.clear());
        }
    }

    let words = glow
        .into_iter()
        .map(|g| {
            let base = format!(
                "left:{:.1}vw;font-size:{:.2}rem;--float-duration:{:.1}s;animation-delay:{:.1}s;--drift:{:.0}px;--y:{:.0}",
                rnd() * 90.0,
                rnd() * 0.8 + 0.9,
                rnd() * 40.0 + 20.0,
                -rnd() * 60.0,
                rnd() * 200.0 - 100.0,
                rnd() * 100.0,
            );
            let style = move || match g.get() {
                Some(color) => format!("{base};--neon-color:{color}"),
                None => base.clone(),
            };
            view! {
                <span class="code-word" class:active-glow=move || g.get().is_some() style=style>{pick(WORDS)}</span>
            }
        })
        .collect_view();

    view! {
        <div class="code-bg" class:still=still aria-hidden="true">{words}</div>
    }
}

//! The app background: painted clouds with faint words of code drifting
//! slowly upward. The words appear only on the welcome and sign-in screens;
//! everywhere else the clouds are shown on their own.

use leptos::*;
use leptos_router::use_location;

const WORDS: &[&str] = &[
    "const", "let", "function()", "=>", "<div>", "import", "from", "return;", "{}", "[]",
    "if()", "else", "true", "false", "class", "await", "async", "console.log", "===", "null",
    "export", "map()", "filter()", "<App />",
    "def", "print()", "self", "SELECT", "WHERE", "fn main()", "let mut", "impl",
    "#include", "std::cout", "int main()", "void loop()", "digitalWrite()",
];

const COUNT: usize = 45;

fn rnd() -> f64 {
    js_sys::Math::random()
}

#[component]
pub fn CodeBackground() -> impl IntoView {
    let pathname = use_location().pathname;
    // Drifting words only on the welcome / sign-in screens.
    let still = move || !matches!(pathname.get().as_str(), "/welcome" | "/login" | "/signup");

    let words = (0..COUNT)
        .map(|_| {
            let style = format!(
                "left:{:.1}vw;font-size:{:.2}rem;--float-duration:{:.1}s;animation-delay:{:.1}s;--drift:{:.0}px;--y:{:.0}",
                rnd() * 90.0,
                rnd() * 0.8 + 0.9,
                rnd() * 40.0 + 20.0,
                -rnd() * 60.0,
                rnd() * 200.0 - 100.0,
                rnd() * 100.0,
            );
            let word = WORDS[((rnd() * WORDS.len() as f64) as usize).min(WORDS.len() - 1)];
            view! { <span class="code-word" style=style>{word}</span> }
        })
        .collect_view();

    view! {
        <div class="code-bg" class:still=still aria-hidden="true">{words}</div>
    }
}

//! The living background: snippets of code in editor colours float slowly
//! upward behind every screen. On the learning path they stay still, so the
//! path itself is calm to read.

use leptos::*;
use leptos_router::use_location;

const SNIPPETS: &[&str] = &[
    "def main():", "print()", "import", "for i in range(10):", "lambda x:", "class", "self",
    "const", "let", "=>", "async", "await", "console.log()", "{}", "[]", "===",
    "<div />", "</>", "<script>", "margin: 0;", "color: #fff;", "display: flex;",
    "SELECT *", "FROM", "WHERE", "JOIN", "GROUP BY",
    "fn main()", "let mut", "impl", "match", "Vec<T>", "&str", "Option<T>",
    "#include", "std::cout", "int main()", "nullptr", "template<T>", "::",
    "void setup()", "void loop()", "digitalWrite()", "delay(1000);", "Serial.println()",
    "if ()", "else", "return;", "true", "false", "null", "while", "0x2A",
];

const COLORS: &[&str] = &["#569cd6", "#c586c0", "#dcdcaa", "#b5cea8", "#9cdcfe", "#4ec9b0", "#ce9178"];

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

    let snippets = (0..40)
        .map(|_| {
            let size = rnd() * 1.2 + 0.6; // rem
            let blur = if size > 1.4 { "blur(1px)" } else if size < 0.8 { "blur(2px)" } else { "none" };
            let style = format!(
                "left:{:.1}vw;font-size:{:.2}rem;color:{};animation-duration:{:.1}s;animation-delay:{:.1}s;\
                 --target-opacity:{:.2};--drift:{:.0}px;--y:{:.0};filter:{}",
                rnd() * 100.0,
                size,
                pick(COLORS),
                rnd() * 20.0 + 15.0,
                -rnd() * 30.0,
                rnd() * 0.3 + 0.1,
                rnd() * 200.0 - 100.0,
                rnd() * 100.0,
                blur,
            );
            view! { <span class="code-snippet" style=style>{pick(SNIPPETS)}</span> }
        })
        .collect_view();

    view! {
        <div class="code-bg" class:still=still aria-hidden="true">{snippets}</div>
    }
}

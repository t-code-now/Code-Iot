use leptos::*;

use crate::highlight::tokens;

/// Highlighted code as a list of spans (no wrapper), for embedding.
pub fn highlighted(code: &str, lang: &str) -> View {
    tokens(code, lang)
        .into_iter()
        .map(|(kind, text)| view! { <span class=kind.class()>{text}</span> })
        .collect_view()
}

/// A dark code card, like the snippets shown in lessons.
#[component]
pub fn CodeBlock(#[prop(into)] code: String, lang: &'static str) -> impl IntoView {
    view! {
        <pre class="code"><code>{highlighted(&code, lang)}</code></pre>
    }
}

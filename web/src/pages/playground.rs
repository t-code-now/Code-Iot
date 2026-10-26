//! Code playground: write HTML, CSS and JavaScript and run it in a
//! sandboxed iframe. Runs fully in the browser; code is kept in localStorage.

use gloo_storage::{LocalStorage, Storage};
use leptos::*;

use crate::components::icons::IconPlay;
use crate::session::use_session;

const KEY: &str = "cs.playground";

const STARTER_HTML: &str = "<h1>Hello, Code-Iot!</h1>\n<button id=\"btn\">Click me</button>\n<p id=\"out\"></p>";
const STARTER_CSS: &str = "body {\n  font-family: sans-serif;\n  padding: 16px;\n}\nh1 { color: #10b981; }\nbutton {\n  padding: 8px 16px;\n  border-radius: 8px;\n}";
const STARTER_JS: &str = "let clicks = 0;\ndocument.getElementById(\"btn\").onclick = () => {\n  clicks++;\n  document.getElementById(\"out\").textContent = `Clicked ${clicks} times`;\n};";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Html,
    Css,
    Js,
    Result,
}

fn build_doc(html: &str, css: &str, js: &str) -> String {
    // Escape closing script tags so user code can't break out of the block.
    let js = js.replace("</script", "<\\/script");
    format!(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><style>{css}</style></head>\
         <body>{html}<script>\
         window.onerror=function(m,s,l){{var d=document.createElement('pre');d.style.color='#e11d48';d.textContent='Error: '+m+' (line '+l+')';document.body.appendChild(d);}};\
         </script><script>{js}</script></body></html>"
    )
}

#[component]
pub fn Playground() -> impl IntoView {
    let session = use_session();
    let saved: (String, String, String) = LocalStorage::get(KEY)
        .unwrap_or_else(|_| (STARTER_HTML.into(), STARTER_CSS.into(), STARTER_JS.into()));
    let html = create_rw_signal(saved.0);
    let css = create_rw_signal(saved.1);
    let js = create_rw_signal(saved.2);
    let tab = create_rw_signal(Tab::Html);
    let doc = create_rw_signal(build_doc(&html.get_untracked(), &css.get_untracked(), &js.get_untracked()));

    let run = move |_| {
        let (h, c, j) = (html.get_untracked(), css.get_untracked(), js.get_untracked());
        let _ = LocalStorage::set(KEY, (&h, &c, &j));
        doc.set(build_doc(&h, &c, &j));
        tab.set(Tab::Result);
    };
    let reset = move |_| {
        html.set(STARTER_HTML.into());
        css.set(STARTER_CSS.into());
        js.set(STARTER_JS.into());
        LocalStorage::delete(KEY);
        tab.set(Tab::Html);
    };

    let editor = move |signal: RwSignal<String>| {
        view! {
            <textarea
                class="editor mono"
                dir="ltr"
                spellcheck="false"
                autocapitalize="off"
                autocomplete="off"
                autocorrect="off"
                prop:value=move || signal.get()
                on:input=move |ev| signal.set(event_target_value(&ev))
            ></textarea>
        }
    };

    let tab_button = move |t: Tab, label: &'static str| {
        view! {
            <button class="pg-tab" class:active={move || tab.get() == t} on:click=move |_| tab.set(t)>
                {move || if label.starts_with("pg.") { session.t(label)() } else { label }}
            </button>
        }
    };

    view! {
        <div class="page playground">
            <header class="topbar">
                <h1 class="topbar-title">{session.t("pg.title")}</h1>
                <div class="pg-actions">
                    <button class="btn btn-small btn-ghost" on:click=reset>{session.t("pg.reset")}</button>
                    <button class="btn btn-small btn-check" on:click=run><IconPlay/>{session.t("pg.run")}</button>
                </div>
            </header>
            <div class="pg-tabs">
                {tab_button(Tab::Html, "HTML")}
                {tab_button(Tab::Css, "CSS")}
                {tab_button(Tab::Js, "JS")}
                {tab_button(Tab::Result, "pg.result")}
            </div>
            <div class="pg-pane">
                {move || match tab.get() {
                    Tab::Html => editor(html).into_view(),
                    Tab::Css => editor(css).into_view(),
                    Tab::Js => editor(js).into_view(),
                    Tab::Result => view! {
                        <iframe
                            class="pg-result"
                            title="Result"
                            sandbox="allow-scripts allow-modals"
                            srcdoc=move || doc.get()
                        ></iframe>
                    }.into_view(),
                }}
            </div>
            <p class="muted pg-hint">{session.t("pg.tip")}</p>
        </div>
    }
}

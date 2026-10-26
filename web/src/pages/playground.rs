//! Code playground for every language in the app.
//!
//! * HTML/CSS/JS, JavaScript and SQL (sql.js) run in a
//!   sandboxed iframe right in the browser.
//! * Python, C++, Rust and Arduino sketches are compiled/run by the server's
//!   `/api/run` (Compiler Explorer; Arduino through a board simulator).
//!
//! Each language keeps its own code in localStorage.

use gloo_storage::{LocalStorage, Storage};
use leptos::*;

use crate::api::{self, RunResult};
use crate::components::icons::IconPlay;
use crate::i18n::tf;
use crate::session::use_session;

const WEB_KEY: &str = "cs.playground";
const LANG_KEY: &str = "cs.pg.lang";

const STARTER_HTML: &str = "<h1>Hello, Code-Iot!</h1>\n<button id=\"btn\">Click me</button>\n<p id=\"out\"></p>";
const STARTER_CSS: &str = "body {\n  font-family: sans-serif;\n  padding: 16px;\n}\nh1 { color: #10b981; }\nbutton {\n  padding: 8px 16px;\n  border-radius: 8px;\n}";
const STARTER_JS: &str = "let clicks = 0;\ndocument.getElementById(\"btn\").onclick = () => {\n  clicks++;\n  document.getElementById(\"out\").textContent = `Clicked ${clicks} times`;\n};";

#[derive(Clone, Copy, PartialEq, Eq)]
struct Lang {
    id: &'static str,
    name: &'static str,
    icon: &'static str,
    starter: &'static str,
    stdin: &'static str,
    /// Runs on the server rather than in the browser.
    server: bool,
    tip: &'static str,
}

const LANGS: &[Lang] = &[
    Lang {
        id: "python", name: "Python", icon: "🐍", server: true, tip: "pg.tip_python", stdin: "Code-Iot",
        starter: "name = input(\"What's your name? \")\nprint(f\"Hello, {name}!\")\n\nfor i in range(1, 4):\n    print(i, \"squared is\", i * i)\n",
    },
    Lang {
        id: "javascript", name: "JavaScript", icon: "⚡", server: false, tip: "pg.tip_js", stdin: "",
        starter: "const langs = [\"Python\", \"JavaScript\", \"Rust\"];\nlangs.forEach((lang, i) => console.log(`${i + 1}. ${lang}`));\n\nconst app = { name: \"Code-Iot\", courses: 7 };\nconsole.log(app);\n",
    },
    Lang { id: "web", name: "HTML/CSS", icon: "🌐", server: false, tip: "pg.tip", stdin: "", starter: "" },
    Lang {
        id: "sql", name: "SQL", icon: "🗄️", server: false, tip: "pg.tip_sql", stdin: "",
        starter: "CREATE TABLE students (id INTEGER PRIMARY KEY, name TEXT, xp INTEGER);\n\nINSERT INTO students (name, xp) VALUES\n  ('Noa', 320), ('Ari', 150), ('Dana', 480);\n\nSELECT name, xp FROM students ORDER BY xp DESC;\n",
    },
    Lang {
        id: "rust", name: "Rust", icon: "🦀", server: true, tip: "pg.tip_server", stdin: "Code-Iot",
        starter: "use std::io::{self, Read};\n\nfn main() {\n    let mut input = String::new();\n    io::stdin().read_to_string(&mut input).unwrap();\n    let name = input.trim();\n    println!(\"Hello, {}!\", if name.is_empty() { \"world\" } else { name });\n\n    let squares: Vec<u32> = (1..=5).map(|n| n * n).collect();\n    println!(\"Squares: {:?}\", squares);\n}\n",
    },
    Lang {
        id: "cpp", name: "C++", icon: "⚙️", server: true, tip: "pg.tip_server", stdin: "Code-Iot",
        starter: "#include <iostream>\n#include <string>\n#include <vector>\n\nint main() {\n    std::string name;\n    std::getline(std::cin, name);\n    std::cout << \"Hello, \" << (name.empty() ? \"world\" : name) << \"!\\n\";\n\n    std::vector<int> nums = {3, 1, 2};\n    for (int n : nums) std::cout << n * n << ' ';\n    std::cout << '\\n';\n}\n",
    },
    Lang {
        id: "arduino", name: "Arduino", icon: "🔌", server: true, tip: "pg.tip_arduino", stdin: "",
        starter: "void setup() {\n  pinMode(LED_BUILTIN, OUTPUT);\n  Serial.begin(9600);\n  Serial.println(\"Hello from the simulator!\");\n}\n\nvoid loop() {\n  digitalWrite(LED_BUILTIN, HIGH);\n  delay(500);\n  digitalWrite(LED_BUILTIN, LOW);\n  delay(500);\n  Serial.print(\"Seconds: \");\n  Serial.println(millis() / 1000);\n}\n",
    },
];

fn lang_by_id(id: &str) -> Lang {
    *LANGS.iter().find(|l| l.id == id).unwrap_or(&LANGS[0])
}

fn code_key(id: &str) -> String {
    format!("cs.pg.code.{id}")
}

/// A JavaScript string literal that is safe inside a <script> block.
fn js_string(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".into()).replace("</", "<\\/").replace("<!--", "<\\!--")
}

const OUT_STYLE: &str = "<style>body{margin:0;background:#0b1028;color:#e6e8f5;font:14px/1.6 'JetBrains Mono',ui-monospace,Menlo,monospace}\
#out{margin:0;padding:14px;white-space:pre-wrap;word-break:break-word}.err{color:#fb7185}.info{color:#8b97c9}\
table{border-collapse:collapse;margin:8px 14px 14px}td,th{border:1px solid #34448a;padding:4px 10px;text-align:left}th{background:#1c2753;color:#ffc72c}</style>";

const OUT_HELPERS: &str = "<script>var out=document.getElementById('out');\
function w(t,c){var s=document.createElement('span');if(c)s.className=c;s.textContent=t+'\\n';out.appendChild(s);}\
function fmt(v){if(typeof v==='string')return v;try{return JSON.stringify(v,null,2)}catch(e){return String(v)}}\
window.onerror=function(m,s,l){w('Error: '+m+(l?' (line '+l+')':''),'err')};\
window.onunhandledrejection=function(e){w('Error: '+(e.reason&&e.reason.message||e.reason),'err')};</script>";

fn web_doc(html: &str, css: &str, js: &str) -> String {
    // Escape closing script tags so user code can't break out of the block.
    let js = js.replace("</script", "<\\/script");
    format!(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><style>{css}</style></head>\
         <body>{html}<script>\
         window.onerror=function(m,s,l){{var d=document.createElement('pre');d.style.color='#e11d48';d.textContent='Error: '+m+' (line '+l+')';document.body.appendChild(d);}};\
         </script><script>{js}</script></body></html>"
    )
}

fn js_doc(code: &str) -> String {
    format!(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\">{OUT_STYLE}</head><body><pre id=\"out\"></pre>{OUT_HELPERS}\
         <script>['log','info','warn','error','debug'].forEach(function(k){{console[k]=function(){{\
         w(Array.prototype.map.call(arguments,fmt).join(' '),k==='error'||k==='warn'?'err':'');}};}});</script>\
         <script type=\"module\">{}</script></body></html>",
        code.replace("</script", "<\\/script")
    )
}

fn sql_doc(code: &str) -> String {
    format!(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\">{OUT_STYLE}</head><body><pre id=\"out\"></pre>{OUT_HELPERS}\
         <script src=\"https://cdn.jsdelivr.net/npm/sql.js@1.10.3/dist/sql-wasm.js\"></script>\
         <script>(async function(){{\
           try{{\
             var SQL=await initSqlJs({{locateFile:function(f){{return 'https://cdn.jsdelivr.net/npm/sql.js@1.10.3/dist/'+f}}}});\
             var db=new SQL.Database();\
             var results=db.exec({code});\
             var changes=db.getRowsModified();\
             if(!results.length)w('OK'+(changes?' ('+changes+' row(s) changed)':''),'info');\
             results.forEach(function(r){{\
               var t=document.createElement('table');var h=t.insertRow();\
               r.columns.forEach(function(c){{var th=document.createElement('th');th.textContent=c;h.appendChild(th)}});\
               r.values.forEach(function(row){{var tr=t.insertRow();row.forEach(function(v){{tr.insertCell().textContent=v===null?'NULL':v}})}});\
               document.body.appendChild(t);\
             }});\
           }}catch(e){{w(String(e.message||e),'err');}}\
         }})();</script></body></html>",
        code = js_string(code),
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    // HTML/CSS/JS project
    Html,
    Css,
    Js,
    // every other language
    Code,
    Input,
    Output,
}

#[derive(Clone, PartialEq)]
enum Output {
    Empty,
    Frame(String),
    Running,
    Done(RunResult),
    Failed(String),
}

#[component]
pub fn Playground() -> impl IntoView {
    let session = use_session();
    let lang = create_rw_signal(lang_by_id(&LocalStorage::get::<String>(LANG_KEY).unwrap_or_default()));

    let load_code = |l: Lang| LocalStorage::get::<String>(code_key(l.id)).unwrap_or_else(|_| l.starter.to_string());
    let code = create_rw_signal(load_code(lang.get_untracked()));
    let stdin = create_rw_signal(lang.get_untracked().stdin.to_string());

    let saved: (String, String, String) = LocalStorage::get(WEB_KEY)
        .unwrap_or_else(|_| (STARTER_HTML.into(), STARTER_CSS.into(), STARTER_JS.into()));
    let html = create_rw_signal(saved.0);
    let css = create_rw_signal(saved.1);
    let js = create_rw_signal(saved.2);

    let is_web = move || lang.get().id == "web";
    let tab = create_rw_signal(if lang.get_untracked().id == "web" { Tab::Html } else { Tab::Code });
    let output = create_rw_signal(Output::Empty);

    let switch_to = move |l: Lang| {
        if l == lang.get_untracked() {
            return;
        }
        let _ = LocalStorage::set(LANG_KEY, l.id);
        code.set(load_code(l));
        stdin.set(l.stdin.to_string());
        output.set(Output::Empty);
        tab.set(if l.id == "web" { Tab::Html } else { Tab::Code });
        lang.set(l);
    };

    let run = move |_| {
        let l = lang.get_untracked();
        if l.id == "web" {
            let (h, c, j) = (html.get_untracked(), css.get_untracked(), js.get_untracked());
            let _ = LocalStorage::set(WEB_KEY, (&h, &c, &j));
            output.set(Output::Frame(web_doc(&h, &c, &j)));
            tab.set(Tab::Output);
            return;
        }
        let src = code.get_untracked();
        let _ = LocalStorage::set(code_key(l.id), &src);
        tab.set(Tab::Output);
        match l.id {
            "javascript" => output.set(Output::Frame(js_doc(&src))),
            "sql" => output.set(Output::Frame(sql_doc(&src))),
            _ => {
                output.set(Output::Running);
                let input = stdin.get_untracked();
                spawn_local(async move {
                    let result = api::run(l.id.to_string(), src, input).await;
                    // Ignore the reply if the user switched language meanwhile.
                    if lang.get_untracked() == l {
                        output.set(match result {
                            Ok(r) => Output::Done(r),
                            Err(e) => Output::Failed(e.message),
                        });
                    }
                });
            }
        }
    };

    let reset = move |_| {
        let l = lang.get_untracked();
        if l.id == "web" {
            html.set(STARTER_HTML.into());
            css.set(STARTER_CSS.into());
            js.set(STARTER_JS.into());
            LocalStorage::delete(WEB_KEY);
            tab.set(Tab::Html);
        } else {
            code.set(l.starter.to_string());
            stdin.set(l.stdin.to_string());
            LocalStorage::delete(code_key(l.id));
            tab.set(Tab::Code);
        }
        output.set(Output::Empty);
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

    let output_view = move || {
        let lang_now = session.lang.get();
        match output.get() {
            Output::Empty => view! { <div class="pg-empty">{session.t("pg.press_run")}</div> }.into_view(),
            Output::Frame(doc) => view! {
                <iframe class="pg-result" class:dark={!is_web()} title="Result" sandbox="allow-scripts allow-modals" srcdoc=doc></iframe>
            }
            .into_view(),
            Output::Running => view! {
                <div class="pg-console"><span class="pg-info">{session.t("pg.running")}</span><div class="spinner small"></div></div>
            }
            .into_view(),
            Output::Failed(msg) => view! { <pre class="pg-console"><span class="pg-err">{msg}</span></pre> }.into_view(),
            Output::Done(r) => {
                let heading = match r.status.as_str() {
                    "compile_error" => Some(crate::i18n::t(lang_now, "pg.compile_error").to_string()),
                    "runtime_error" => Some(tf(lang_now, "pg.runtime_error", &[&r.exit_code.unwrap_or(-1)])),
                    _ => None,
                };
                let empty = r.stdout.trim().is_empty() && r.stderr.trim().is_empty();
                view! {
                    <pre class="pg-console">
                        {r.stdout}
                        {(!r.stderr.trim().is_empty()).then(|| view! { <span class="pg-err">"\n"{r.stderr.clone()}</span> })}
                        {heading.map(|h| view! { <span class="pg-err pg-status">"\n"{h}</span> })}
                        {empty.then(|| view! { <span class="pg-info">{session.t("pg.no_output")}</span> })}
                    </pre>
                }
                .into_view()
            }
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
            <div class="pg-langs" dir="ltr">
                {LANGS.iter().map(|l| {
                    let l = *l;
                    view! {
                        <button class="pg-lang" class:active={move || lang.get() == l} on:click=move |_| switch_to(l)>
                            <span>{l.icon}</span>{l.name}
                        </button>
                    }
                }).collect_view()}
            </div>
            <div class="pg-tabs">
                {move || if is_web() {
                    view! { <>{tab_button(Tab::Html, "HTML")}{tab_button(Tab::Css, "CSS")}{tab_button(Tab::Js, "JS")}{tab_button(Tab::Output, "pg.result")}</> }.into_view()
                } else {
                    let l = lang.get();
                    let has_input = matches!(l.id, "python" | "rust" | "cpp" | "arduino");
                    view! { <>
                        {tab_button(Tab::Code, "pg.code")}
                        {has_input.then(|| tab_button(Tab::Input, "pg.input"))}
                        {tab_button(Tab::Output, "pg.output")}
                    </> }.into_view()
                }}
            </div>
            <div class="pg-pane">
                {move || match tab.get() {
                    Tab::Html => editor(html).into_view(),
                    Tab::Css => editor(css).into_view(),
                    Tab::Js => editor(js).into_view(),
                    Tab::Code => editor(code).into_view(),
                    Tab::Input => view! {
                        <div class="pg-input">
                            <p class="muted">{session.t("pg.input_hint")}</p>
                            {editor(stdin)}
                        </div>
                    }.into_view(),
                    Tab::Output => output_view().into_view(),
                }}
            </div>
            <p class="muted pg-hint">{move || session.t(lang.get().tip)()}</p>
        </div>
    }
}

//! The lesson step views. Each exercise writes the learner's current answer
//! into `answer` (None while incomplete) and stops reacting to taps once
//! `locked` is true (after Check).

use codesprout_core::api::Answer;
use codesprout_core::model::BLANK;
use leptos::*;

use crate::components::code::{highlighted, CodeBlock};
use crate::i18n;
use crate::session::use_session;

/// Show text in monospace when it looks like code.
fn looks_like_code(s: &str) -> bool {
    s.chars().any(|c| "()<>{}[];=_\"#.:`$&*/".contains(c))
}

#[component]
pub fn LearnCard(
    title: String,
    body: String,
    code: Option<String>,
    lang: &'static str,
) -> impl IntoView {
    view! {
        <div class="step learn">
            <div class="learn-badge">{use_session().t("lesson.learn_badge")}</div>
            <h2 class="prompt">{title}</h2>
            <p class="learn-body">{body}</p>
            {code.map(|c| view! { <CodeBlock code=c lang=lang/> })}
        </div>
    }
}

#[component]
pub fn MultipleChoice(
    prompt: String,
    code: Option<String>,
    options: Vec<String>,
    lang: &'static str,
    answer: RwSignal<Option<Answer>>,
    #[prop(into)] locked: Signal<bool>,
) -> impl IntoView {
    let selected = create_rw_signal(None::<usize>);
    let session = use_session();
    let lang_now = move || session.lang.get();
    let buttons = options
        .into_iter()
        .enumerate()
        .map(|(i, text)| {
            let mono = looks_like_code(&text);
            view! {
                <button
                    class="option"
                    class:selected=move || selected.get() == Some(i)
                    disabled=move || locked.get()
                    on:click=move |_| {
                        selected.set(Some(i));
                        answer.set(Some(Answer::Choice(i)));
                    }
                >
                    <span class="option-key">{move || i18n::option_letter(lang_now(), i)}</span>
                    <span class="option-text" class:mono=mono>{text}</span>
                </button>
            }
        })
        .collect_view();

    view! {
        <div class="step">
            <h2 class="prompt">{prompt}</h2>
            {code.map(|c| view! { <CodeBlock code=c lang=lang/> })}
            <div class="options">{buttons}</div>
        </div>
    }
}

#[component]
pub fn FillBlanks(
    prompt: String,
    code: String,
    blanks: u32,
    options: Vec<String>,
    lang: &'static str,
    answer: RwSignal<Option<Answer>>,
    #[prop(into)] locked: Signal<bool>,
) -> impl IntoView {
    let n = blanks as usize;
    // For each blank, the index of the tile placed in it.
    let filled = create_rw_signal(vec![None::<usize>; n]);
    let opts = store_value(options);

    create_effect(move |_| {
        let f = filled.get();
        if n > 0 && f.iter().all(Option::is_some) {
            let tokens = opts.with_value(|o| f.iter().flatten().map(|&i| o[i].clone()).collect());
            answer.set(Some(Answer::Tokens(tokens)));
        } else {
            answer.set(None);
        }
    });

    let code_view = code
        .split(BLANK)
        .enumerate()
        .map(|(i, segment)| {
            let blank = (i < n).then(|| {
                view! {
                    <button
                        class="blank"
                        class:filled=move || filled.with(|f| f[i].is_some())
                        disabled=move || locked.get()
                        on:click=move |_| filled.update(|f| f[i] = None)
                    >
                        {move || {
                            filled
                                .with(|f| f[i])
                                .map(|o| opts.with_value(|v| v[o].clone()))
                                .unwrap_or_else(|| "\u{a0}\u{a0}\u{a0}\u{a0}".to_string())
                        }}
                    </button>
                }
            });
            view! { <>{highlighted(segment, lang)}{blank}</> }
        })
        .collect_view();

    let tiles = opts.with_value(|o| {
        o.iter()
            .enumerate()
            .map(|(oi, text)| {
                let text = text.clone();
                let used = move || filled.with(|f| f.contains(&Some(oi)));
                view! {
                    <button
                        class="tile mono"
                        class:used=used
                        disabled=move || locked.get() || used()
                        on:click=move |_| {
                            filled.update(|f| {
                                if let Some(slot) = f.iter_mut().find(|s| s.is_none()) {
                                    *slot = Some(oi);
                                }
                            })
                        }
                    >
                        {text}
                    </button>
                }
            })
            .collect_view()
    });

    view! {
        <div class="step">
            <h2 class="prompt">{prompt}</h2>
            <pre class="code"><code>{code_view}</code></pre>
            <div class="tiles">{tiles}</div>
        </div>
    }
}

#[component]
pub fn Reorder(
    prompt: String,
    lines: Vec<String>,
    answer: RwSignal<Option<Answer>>,
    #[prop(into)] locked: Signal<bool>,
) -> impl IntoView {
    let total = lines.len();
    let lines = store_value(lines);
    // Indices into `lines`, in the order the learner tapped them.
    let picked = create_rw_signal(Vec::<usize>::new());

    create_effect(move |_| {
        let p = picked.get();
        if total > 0 && p.len() == total {
            let tokens = lines.with_value(|l| p.iter().map(|&i| l[i].clone()).collect());
            answer.set(Some(Answer::Tokens(tokens)));
        } else {
            answer.set(None);
        }
    });

    let placed = move || {
        picked
            .get()
            .into_iter()
            .enumerate()
            .map(|(pos, li)| {
                view! {
                    <button
                        class="line-tile placed"
                        disabled=move || locked.get()
                        on:click=move |_| picked.update(|p| { p.remove(pos); })
                    >
                        {lines.with_value(|l| l[li].clone())}
                    </button>
                }
            })
            .collect_view()
    };

    let pool = (0..total)
        .map(|li| {
            let used = move || picked.with(|p| p.contains(&li));
            view! {
                <button
                    class="line-tile"
                    class:used=used
                    disabled=move || locked.get() || used()
                    on:click=move |_| picked.update(|p| p.push(li))
                >
                    {lines.with_value(|l| l[li].clone())}
                </button>
            }
        })
        .collect_view();

    view! {
        <div class="step">
            <h2 class="prompt">{prompt}</h2>
            <div class="answer-area">
                {placed}
                <Show when=move || picked.with(Vec::is_empty)>
                    <div class="answer-hint">{use_session().t("lesson.reorder_hint")}</div>
                </Show>
            </div>
            <div class="line-pool">{pool}</div>
        </div>
    }
}

#[component]
pub fn TypeIn(
    prompt: String,
    code: Option<String>,
    lang: &'static str,
    answer: RwSignal<Option<Answer>>,
    #[prop(into)] locked: Signal<bool>,
) -> impl IntoView {
    let text = create_rw_signal(String::new());
    let placeholder = use_session().t("lesson.type_placeholder");
    view! {
        <div class="step">
            <h2 class="prompt">{prompt}</h2>
            {code.map(|c| view! { <CodeBlock code=c lang=lang/> })}
            <textarea
                class="type-input mono"
                rows="3"
                placeholder=move || placeholder()
                autocapitalize="off"
                autocomplete="off"
                autocorrect="off"
                spellcheck="false"
                prop:value=move || text.get()
                disabled=move || locked.get()
                on:input=move |ev| {
                    let v = event_target_value(&ev);
                    answer.set((!v.trim().is_empty()).then(|| Answer::Text(v.clone())));
                    text.set(v);
                }
            ></textarea>
        </div>
    }
}

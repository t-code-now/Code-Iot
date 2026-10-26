//! Full-screen lesson player: progress bar, one step at a time, a Check
//! button, a green/red feedback sheet, and a lesson-complete screen.

use codesprout_core::api::{Answer, CheckResponse, CompleteResponse, LessonDto, StepDto};
use codesprout_core::progress::xp_for_lesson;
use leptos::*;
use leptos_router::*;

use crate::api;
use crate::components::exercises::*;
use crate::components::icons::IconClose;
use crate::highlight::lang_for_course;
use crate::pages::{ErrorState, Loading};
use crate::i18n::{self, tf};
use crate::session::use_session;

const PRAISE: [&str; 5] = ["praise.0", "praise.1", "praise.2", "praise.3", "praise.4"];

#[component]
pub fn LessonPage() -> impl IntoView {
    let session = use_session();
    let params = use_params_map();
    let key = move || (params.with(|p| p.get("id").cloned().unwrap_or_default()), session.lang.get());
    let lesson = create_local_resource(key, |(id, lang)| api::lesson(id, lang));

    view! {
        <div class="lesson-screen">
            <Suspense fallback=|| view! { <Loading/> }>
                {move || lesson.get().map(|r| match r {
                    Ok(l) => view! { <Player lesson=l/> }.into_view(),
                    Err(e) => view! { <ErrorState message=e.message/> }.into_view(),
                })}
            </Suspense>
        </div>
    }
}

#[component]
fn Player(lesson: LessonDto) -> impl IntoView {
    let session = use_session();
    session.set_course(&lesson.course_id);

    let total_steps = lesson.steps.len();
    let exercise_total =
        lesson.steps.iter().filter(|s| !matches!(s, StepDto::Learn { .. })).count() as u32;
    let lang = lang_for_course(&lesson.course_id);
    let color = lesson.color.clone();
    let lesson = store_value(lesson);

    let index = create_rw_signal(0usize);
    let answer = create_rw_signal(None::<Answer>);
    let feedback = create_rw_signal(None::<CheckResponse>);
    let checking = create_rw_signal(false);
    let net_error = create_rw_signal(None::<String>);
    let correct_first = create_rw_signal(0u32);
    let finished = create_rw_signal(false);
    // Saved result from the server (logged-in users only).
    let saved = create_rw_signal(None::<Result<CompleteResponse, String>>);

    let is_learn = move || {
        let i = index.get();
        lesson.with_value(|l| matches!(l.steps.get(i), Some(StepDto::Learn { .. })))
    };

    let progress = move || {
        let done = index.get() + usize::from(feedback.with(Option::is_some) || is_learn());
        (done as f64 / total_steps.max(1) as f64 * 100.0).min(100.0)
    };

    let finish = move || {
        finished.set(true);
        if session.logged_in() {
            let id = lesson.with_value(|l| l.id.clone());
            let token = session.token.get_untracked();
            let correct = correct_first.get_untracked();
            spawn_local(async move {
                let result = api::complete(id, token, correct).await;
                if let Ok(r) = &result {
                    session.profile.set(Some(r.profile.clone()));
                }
                saved.set(Some(result.map_err(|e| e.message)));
            });
        }
    };

    let advance = move || {
        feedback.set(None);
        answer.set(None);
        net_error.set(None);
        if index.get_untracked() + 1 >= total_steps {
            finish();
        } else {
            index.update(|i| *i += 1);
        }
    };

    let check = move || {
        let Some(ans) = answer.get_untracked() else { return };
        if checking.get_untracked() {
            return;
        }
        checking.set(true);
        net_error.set(None);
        let id = lesson.with_value(|l| l.id.clone());
        let step = index.get_untracked();
        let ui = session.lang.get_untracked();
        spawn_local(async move {
            match api::check(id, step, ans, ui).await {
                Ok(resp) => {
                    if resp.correct {
                        correct_first.update(|c| *c += 1);
                    }
                    feedback.set(Some(resp));
                }
                Err(e) => net_error.set(Some(i18n::error(ui, &e.message))),
            }
            checking.set(false);
        });
    };

    let on_main = move |_| {
        if is_learn() || feedback.with_untracked(Option::is_some) {
            advance();
        } else {
            check();
        }
    };

    let main_label = move || {
        let key = if is_learn() || feedback.with(Option::is_some) {
            "lesson.continue"
        } else if checking.get() {
            "lesson.checking"
        } else {
            "lesson.check"
        };
        i18n::t(session.lang.get(), key)
    };
    let main_disabled =
        move || !is_learn() && feedback.with(Option::is_none) && (answer.with(Option::is_none) || checking.get());

    let locked = Signal::derive(move || feedback.with(Option::is_some));

    let step_view = move || {
        let i = index.get();
        let step = lesson.with_value(|l| l.steps[i].clone());
        let inner = match step {
            StepDto::Learn { title, body, code } => view! {
                <LearnCard title=title body=body code=code lang=lang/>
            }
            .into_view(),
            StepDto::MultipleChoice { prompt, code, options } => view! {
                <MultipleChoice prompt=prompt code=code options=options lang=lang answer=answer locked=locked/>
            }
            .into_view(),
            StepDto::FillBlanks { prompt, code, blanks, options } => view! {
                <FillBlanks prompt=prompt code=code blanks=blanks options=options lang=lang answer=answer locked=locked/>
            }
            .into_view(),
            StepDto::Reorder { prompt, lines } => view! {
                <Reorder prompt=prompt lines=lines answer=answer locked=locked/>
            }
            .into_view(),
            StepDto::TypeIn { prompt, code } => view! {
                <TypeIn prompt=prompt code=code lang=lang answer=answer locked=locked/>
            }
            .into_view(),
        };
        view! { <div class="step-wrap">{inner}</div> }
    };

    let feedback_view = move || {
        feedback.get().map(|f| {
            let ok = f.correct;
            let ui = session.lang.get_untracked();
            let praise = i18n::t(ui, PRAISE[index.get_untracked() % PRAISE.len()]);
            view! {
                <div class="feedback-body">
                    <div class="feedback-head">
                        <span class="feedback-icon">{if ok { "✓" } else { "✕" }}</span>
                        <strong>{if ok { praise } else { i18n::t(ui, "lesson.not_quite") }}</strong>
                    </div>
                    {(!ok && !f.expected.is_empty()).then(|| view! {
                        <>
                            <div class="feedback-label">{i18n::t(ui, "lesson.correct_answer")}</div>
                            <pre class="feedback-answer">{f.expected.clone()}</pre>
                        </>
                    })}
                    {(!f.explanation.is_empty()).then(|| view! {
                        <p class="feedback-text">{f.explanation.clone()}</p>
                    })}
                </div>
            }
        })
    };

    let footer_state = move || match feedback.with(|f| f.as_ref().map(|f| f.correct)) {
        Some(true) => "footer ok",
        Some(false) => "footer bad",
        None => "footer",
    };

    view! {
        <Show
            when=move || !finished.get()
            fallback=move || view! {
                <LessonComplete
                    lesson=lesson.get_value()
                    correct=correct_first.get_untracked()
                    total=exercise_total
                    saved=saved
                />
            }
        >
            <div class="player" style=format!("--course:{color}")>
                <header class="player-top">
                    <a href="/" class="icon-btn" aria-label=session.t("lesson.close")><IconClose/></a>
                    <div class="bar lesson-bar">
                        <div class="bar-fill" style=move || format!("width:{}%", progress())></div>
                    </div>
                </header>
                <div class="player-body">{step_view}</div>
                <footer class=footer_state>
                    {feedback_view}
                    {move || net_error.get().map(|e| view! { <div class="form-error">{e}</div> })}
                    <button class="btn btn-check btn-block" disabled=main_disabled on:click=on_main>
                        {main_label}
                    </button>
                </footer>
            </div>
        </Show>
    }
}

#[component]
fn LessonComplete(
    lesson: LessonDto,
    correct: u32,
    total: u32,
    saved: RwSignal<Option<Result<CompleteResponse, String>>>,
) -> impl IntoView {
    let session = use_session();
    let accuracy = if total == 0 { 100 } else { correct * 100 / total };
    let guest_xp = xp_for_lesson(correct, total, true);
    let next_href = lesson.next_lesson_id.as_ref().map(|id| format!("/lesson/{id}"));
    let logged_in = session.logged_in();
    let ui = session.lang.get_untracked();

    let xp_text = move || {
        if !logged_in {
            return format!("+{guest_xp}");
        }
        match saved.get() {
            Some(Ok(r)) => format!("+{}", r.xp_gained),
            Some(Err(_)) => "—".to_string(),
            None => "…".to_string(),
        }
    };
    let streak_text = move || {
        saved
            .get()
            .and_then(|r| r.ok())
            .map(|r| r.profile.streak.to_string())
            .unwrap_or_else(|| if logged_in { "…".into() } else { "—".into() })
    };

    view! {
        <div class="complete" style=format!("--course:{}", lesson.color)>
            <div class="complete-hero">
                <div class="complete-emoji">{if accuracy == 100 { "🏆" } else { "🎉" }}</div>
                <h1>{i18n::t(ui, "done.title")}</h1>
                <p class="complete-sub">{lesson.title.clone()}</p>
            </div>
            <div class="stat-cards">
                <div class="stat-card xp">
                    <div class="stat-label">{i18n::t(ui, "done.xp")}</div>
                    <div class="stat-value">{xp_text}</div>
                </div>
                <div class="stat-card acc">
                    <div class="stat-label">{i18n::t(ui, "done.accuracy")}</div>
                    <div class="stat-value">{format!("{accuracy}%")}</div>
                </div>
                <div class="stat-card streak">
                    <div class="stat-label">{i18n::t(ui, "done.streak")}</div>
                    <div class="stat-value">"🔥 " {streak_text}</div>
                </div>
            </div>

            {move || saved.get().and_then(|r| r.ok()).filter(|r| !r.new_badges.is_empty()).map(|r| view! {
                <div class="new-badges">
                    <h3>{i18n::t(ui, "done.new_badge")}</h3>
                    {r.new_badges.into_iter().map(|b| {
                        let (title, desc) = i18n::badge(ui, &b);
                        view! {
                            <div class="badge-row">
                                <span class="badge-icon">{b.icon}</span>
                                <div><strong>{title}</strong><p>{desc}</p></div>
                            </div>
                        }
                    }).collect_view()}
                </div>
            })}

            {move || saved.get().and_then(|r| r.err()).map(|e| view! {
                <div class="form-error">{tf(ui, "done.not_saved", &[&i18n::error(ui, &e)])}</div>
            })}

            {(!logged_in).then(|| view! {
                <div class="guest-note">
                    <p>{i18n::t(ui, "done.guest_note")}</p>
                    <a href="/signup" class="btn btn-ghost btn-block">{i18n::t(ui, "done.guest_button")}</a>
                </div>
            })}

            <div class="complete-actions">
                {next_href.map(|href| view! { <a href=href class="btn btn-check btn-block">{i18n::t(ui, "done.next")}</a> })}
                <a href="/" class="btn btn-ghost btn-block">{i18n::t(ui, "done.back")}</a>
            </div>
        </div>
    }
}

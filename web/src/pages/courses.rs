//! Course picker. All courses are open.

use crate::components::lang_icon::LangIcon;
use leptos::*;
use crate::components::icons::IconBack;
use leptos_router::*;

use crate::api;
use crate::i18n::tf;
use crate::pages::{ErrorState, Loading};
use crate::session::use_session;

#[component]
pub fn Courses() -> impl IntoView {
    let session = use_session();
    let navigate = use_navigate();
    let courses = create_local_resource(
        move || (session.token.get(), session.lang.get()),
        |(token, lang)| api::courses(token, lang),
    );

    view! {
        <div class="page">
            <header class="page-header">
                <a href="/" class="back-link icon-only" aria-label=session.t("back.word")><IconBack/></a>
                <h1>{session.t("courses.title")}</h1>
            </header>
            <Suspense fallback=|| view! { <Loading/> }>
                {
                    let navigate = navigate.clone();
                    move || courses.get().map(|r| match r {
                        Ok(list) => view! { <div class="course-grid">{list
                            .into_iter()
                            .map(|c| {
                                let navigate = navigate.clone();
                                let id = c.id.clone();
                                let selected = move || session.course.with(|cur| *cur == id);
                                let pick_id = c.id.clone();
                                let percent = if c.lesson_count == 0 { 0 } else { c.completed_count * 100 / c.lesson_count };
                                view! {
                                    <button
                                        class="course-card"
                                        class:selected=selected
                                        style=format!("--course:{}", c.color)
                                        on:click=move |_| {
                                            session.set_course(&pick_id);
                                            navigate("/", Default::default());
                                        }
                                    >
                                        <div class="course-icon"><LangIcon id=c.id.clone()/></div>
                                        <div class="course-info">
                                            <h3>{c.title} {c.unlocked.then(|| view! { <span class="tag gold">"🔓"</span> })}</h3>
                                            <p>{c.description}</p>
                                            <div class="bar small">
                                                <div class="bar-fill" style=format!("width:{percent}%")></div>
                                            </div>
                                            <span class="course-meta">
                                                {tf(session.lang.get_untracked(), "courses.count", &[&c.completed_count, &c.lesson_count])}
                                            </span>
                                        </div>
                                    </button>
                                }
                            })
                            .collect_view()}</div> }.into_view(),
                        Err(e) => view! { <ErrorState message=e.message/> }.into_view(),
                    })
                }
            </Suspense>
        </div>
    }
}

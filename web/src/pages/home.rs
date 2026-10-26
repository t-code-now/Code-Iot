//! The Learn tab: course header and the winding lesson path.
//! Every node is tappable — nothing is locked.

use codesprout_core::api::CourseDetail;
use leptos::*;

use crate::api;
use crate::i18n::tf;
use crate::components::shell::StatPills;
use crate::pages::{ErrorState, Loading};
use crate::session::use_session;

/// Horizontal offsets (px) that make the path zig-zag.
const OFFSETS: [i32; 8] = [0, 44, 70, 44, 0, -44, -70, -44];

#[component]
pub fn Home() -> impl IntoView {
    let session = use_session();
    let course = create_local_resource(
        move || (session.course.get(), session.token.get(), session.lang.get()),
        |(id, token, lang)| api::course(id, token, lang),
    );

    view! {
        <Suspense fallback=|| view! { <Loading/> }>
            {move || course.get().map(|r| match r {
                Ok(c) => view! { <LearningPath course=c/> }.into_view(),
                Err(e) => view! { <ErrorState message=e.message/> }.into_view(),
            })}
        </Suspense>
    }
}

#[component]
fn LearningPath(course: CourseDetail) -> impl IntoView {
    let session = use_session();
    let lang = session.lang.get_untracked();
    let color = course.color.clone();
    let total: usize = course.sections.iter().map(|s| s.lessons.len()).sum();
    let done: usize = course.sections.iter().flat_map(|s| &s.lessons).filter(|l| l.completed).count();
    // The first lesson not yet finished gets the "START" bubble.
    let current = course
        .sections
        .iter()
        .flat_map(|s| &s.lessons)
        .find(|l| !l.completed)
        .map(|l| l.id.clone());

    let mut sections = Vec::new();
    let mut position = 0usize;
    for (si, section) in course.sections.into_iter().enumerate() {
        let section_done = section.lessons.iter().filter(|l| l.completed).count();
        let section_total = section.lessons.len();

        let mut nodes = Vec::new();
        for lesson in section.lessons {
            let offset = OFFSETS[position % OFFSETS.len()];
            position += 1;
            let is_current = current.as_deref() == Some(lesson.id.as_str());
            let label = if lesson.completed { "✓".to_string() } else { position.to_string() };
            nodes.push(view! {
                <a
                    href=format!("/lesson/{}", lesson.id)
                    class="node-wrap"
                    style=format!("transform: translateX({offset}px)")
                >
                    {is_current.then(|| view! { <div class="start-bubble">{session.t("home.start")}</div> })}
                    <div class="node" class:done=lesson.completed class:current=is_current>
                        <span class="node-face">{label}</span>
                    </div>
                    <div class="node-label">{lesson.title}</div>
                </a>
            });
        }

        sections.push(view! {
            <section class="unit">
                <div class="unit-banner" style=format!("--unit:{color}")>
                    <div>
                        <div class="unit-kicker">{tf(lang, "home.section", &[&(si + 1)])}</div>
                        <h2 class="unit-title">{section.title}</h2>
                    </div>
                    <div class="unit-count">{format!("{section_done}/{section_total}")}</div>
                </div>
                <div class="path">{nodes}</div>
            </section>
        });
    }

    let percent = if total == 0 { 0 } else { done * 100 / total };

    view! {
        <div class="learn" style=format!("--course:{color}")>
            <header class="topbar">
                <a href="/courses" class="course-chip">
                    <span class="course-chip-icon">{course.icon}</span>
                    <span>{course.title}</span>
                    <span class="chev">"▾"</span>
                </a>
                <StatPills/>
            </header>
            <div class="course-progress">
                <div class="bar"><div class="bar-fill" style=format!("width:{percent}%")></div></div>
                <span>{tf(lang, "home.lessons_progress", &[&done, &total])}</span>
            </div>
            {sections}
            <div class="path-end">{session.t("home.finish")}</div>
        </div>
    }
}

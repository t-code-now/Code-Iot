//! The Learn tab: course banner and the lesson path on a blueprint board.
//! Lessons are square 3D platforms joined by cables. Every node is tappable;
//! nothing is locked.

use codesprout_core::api::CourseDetail;
use leptos::*;

use crate::api;
use crate::components::icons::{IconBolt, IconCheck, IconPlay, IconPuzzle};
use crate::components::shell::StatPills;
use crate::i18n::tf;
use crate::pages::{ErrorState, Loading};
use crate::session::use_session;

/// Column (0..=2) of each node; the pattern makes the path snake across the board.
const COLS: [usize; 6] = [1, 2, 2, 1, 0, 0];
/// Board geometry in px.
const COL_W: i32 = 96;
const ROW_H: i32 = 160;
const BOARD_W: i32 = COL_W * 3;
const TOP: i32 = 46; // room for the START bubble above the first node

fn center(col: usize, row: usize) -> (i32, i32) {
    (col as i32 * COL_W + COL_W / 2, TOP + row as i32 * ROW_H + 36)
}

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
    let total: usize = course.sections.iter().map(|s| s.lessons.len()).sum();
    let done: usize = course.sections.iter().flat_map(|s| &s.lessons).filter(|l| l.completed).count();
    // The first lesson not yet finished gets the "START" bubble.
    let current = course
        .sections
        .iter()
        .flat_map(|s| &s.lessons)
        .find(|l| !l.completed)
        .map(|l| l.id.clone());
    let last_id = course.sections.last().and_then(|s| s.lessons.last()).map(|l| l.id.clone());

    let mut sections = Vec::new();
    let mut number = 0usize;
    for (si, section) in course.sections.into_iter().enumerate() {
        let section_done = section.lessons.iter().filter(|l| l.completed).count();
        let section_total = section.lessons.len();
        let rows = section.lessons.len();
        let height = TOP + rows as i32 * ROW_H;

        // Cables between consecutive nodes: down, across, down.
        let mut cables = Vec::new();
        for i in 1..rows {
            let (x1, y1) = center(COLS[(i - 1) % COLS.len()], i - 1);
            let (x2, y2) = center(COLS[i % COLS.len()], i);
            let mid = y1 + 104; // below the label, above the next tile
            let d = format!("M{x1} {y1} V{mid} H{x2} V{y2}");
            let lit = section.lessons[i - 1].completed;
            cables.push(view! {
                <path class="cable-shell" d=d.clone()/>
                <path class="cable-core" class:lit=lit d=d/>
            });
        }

        let mut nodes = Vec::new();
        for (i, lesson) in section.lessons.into_iter().enumerate() {
            number += 1;
            let (x, y) = center(COLS[i % COLS.len()], i);
            let is_current = current.as_deref() == Some(lesson.id.as_str());
            let is_project = lesson.id.ends_with("-p") || last_id.as_deref() == Some(lesson.id.as_str());
            let face = if lesson.completed && !is_project {
                view! { <IconCheck/> }.into_view()
            } else if is_project {
                view! { <IconPuzzle/> }.into_view()
            } else if is_current {
                view! { <IconPlay/> }.into_view()
            } else {
                view! { <IconBolt/> }.into_view()
            };
            nodes.push(view! {
                <a
                    href=format!("/lesson/{}", lesson.id)
                    class="node-wrap"
                    style=format!("left:{}px;top:{}px", x - 60, y - 36)
                    title=lesson.title.clone()
                >
                    {is_current.then(|| view! { <div class="start-bubble">{session.t("home.start")}</div> })}
                    <div class="pad"></div>
                    <div
                        class="node"
                        class:done=lesson.completed
                        class:current=is_current
                        class:project=is_project
                    >
                        <span class="node-face">{face}</span>
                        <span class="node-num">{number}</span>
                    </div>
                    <div class="node-label">{lesson.title}</div>
                </a>
            });
        }

        sections.push(view! {
            <section class="unit">
                <div class="unit-banner">
                    <span class="unit-puzzle"><IconPuzzle/></span>
                    <div class="unit-text">
                        <div class="unit-kicker">{tf(lang, "home.section", &[&(si + 1)])}</div>
                        <h2 class="unit-title">{section.title}</h2>
                    </div>
                    <div class="unit-count">{format!("{section_done}/{section_total}")}</div>
                </div>
                <div class="board" style=format!("width:{BOARD_W}px;height:{height}px")>
                    <svg class="cables" width=BOARD_W height=height viewBox=format!("0 0 {BOARD_W} {height}")>
                        {cables}
                    </svg>
                    {nodes}
                </div>
            </section>
        });
    }

    let percent = if total == 0 { 0 } else { done * 100 / total };

    view! {
        <div class="learn">
            <header class="topbar">
                <img class="top-logo" src="/assets/mascot.png" alt="Code-Iot"/>
                <StatPills/>
            </header>
            <a href="/courses" class="course-banner">
                <span class="course-banner-icon">{course.icon}</span>
                <span class="course-banner-text">
                    <strong>{course.title}</strong>
                    <span class="course-banner-sub">{tf(lang, "home.lessons_progress", &[&done, &total])}</span>
                </span>
                <span class="chev">"▾"</span>
                <span class="course-banner-bar"><span style=format!("width:{percent}%")></span></span>
            </a>
            {sections}
            <div class="path-end">{session.t("home.finish")}</div>
        </div>
    }
}

//! The Learn tab: course banner and the lesson path on a blueprint board.
//! Lessons are square 3D platforms joined by cables. Every node is tappable;
//! nothing is locked.

use codesprout_core::api::CourseDetail;
use crate::components::lang_icon::LangIcon;
use leptos::*;

use crate::api;
use crate::components::icons::{IconBolt, IconCheck, IconLock, IconPlay, IconPuzzle};
use crate::components::avatar::UserAvatar;
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
    // For the desktop side panel: the lesson to continue with.
    let next_lesson = course
        .sections
        .iter()
        .flat_map(|s| &s.lessons)
        .find(|l| !l.completed)
        .or_else(|| course.sections.first().and_then(|s| s.lessons.first()))
        .map(|l| (format!("/lesson/{}", l.id), l.title.clone()));
    let course_icon = course.id.clone();
    let course_id = course.id.clone();
    let unlocked = course.unlocked;
    let course_title = course.title.clone();

    let mut sections = Vec::new();
    let mut number = 0usize;
    for (si, section) in course.sections.into_iter().enumerate() {
        let section_done = section.lessons.iter().filter(|l| l.completed).count();
        let section_locked = section.lessons.first().is_some_and(|l| l.locked);
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
            let locked = lesson.locked;
            let face = if locked {
                view! { <IconLock/> }.into_view()
            } else if lesson.completed && !is_project {
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
                    href=if locked { format!("/unlock/{course_id}") } else { format!("/lesson/{}", lesson.id) }
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
                        class:locked=locked
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
                    <span class="unit-puzzle" class:locked=section_locked>
                        {if section_locked { view! { <IconLock/> }.into_view() } else { view! { <IconPuzzle/> }.into_view() }}
                    </span>
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

    let level = move || session.profile.with(|p| p.as_ref().map(|p| p.level));
    let name = move || session.profile.with(|p| p.as_ref().map(|p| p.username.clone()));

    view! {
        <div class="learn-layout">
        <div class="learn">
            <header class="topbar">
                <a href="/profile" class="top-avatar-link"><UserAvatar class="top-logo"/></a>
                <StatPills/>
            </header>
            <a href="/courses" class="course-banner">
                <span class="course-banner-icon"><LangIcon id=course.id.clone()/></span>
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

        // Wide screens only: stats and shortcuts beside the path.
        <aside class="side-panel">
            <div class="panel-card panel-me">
                <a href="/profile" class="panel-avatar"><UserAvatar class="avatar"/></a>
                <div class="panel-me-text">
                    <strong>{move || name().unwrap_or_else(|| session.t("profile.guest")().to_string())}</strong>
                    <span class="muted">{move || level().map(|l| tf(session.lang.get(), "profile.level", &[&l])).unwrap_or_default()}</span>
                </div>
                <StatPills/>
            </div>

            <div class="panel-card panel-course">
                <div class="panel-course-head">
                    <span class="course-banner-icon"><LangIcon id=course_icon/></span>
                    <div>
                        <strong>{course_title}</strong>
                        <span class="muted">{tf(lang, "home.lessons_progress", &[&done, &total])}</span>
                    </div>
                </div>
                <div class="bar"><div class="bar-fill" style=format!("width:{percent}%")></div></div>
                <span class="panel-percent">{format!("{percent}%")}</span>
                {next_lesson.map(|(href, title)| view! {
                    <a href=href class="btn btn-primary btn-block panel-continue">
                        <IconPlay/><span>{session.t("home.continue")}</span>
                    </a>
                    <p class="panel-next muted">{title}</p>
                })}
                {(!unlocked).then(|| view! {
                    <a href=format!("/unlock/{}", course_id.clone()) class="btn btn-outline btn-block">
                        <IconLock/><span>{session.t("unlock.open_course")}</span>
                    </a>
                })}
                <a href="/courses" class="btn btn-ghost btn-block">{session.t("home.all_courses")}</a>
            </div>

            <a href="/leaderboard" class="panel-card panel-league">
                <span class="panel-league-emblem">"🏆"</span>
                <div>
                    <strong>{session.t("league.name")}</strong>
                    <span class="muted">{session.t("home.league_cta")}</span>
                </div>
            </a>

            {move || (!session.logged_in()).then(|| view! {
                <div class="panel-card panel-guest">
                    <p>{session.t("profile.guest_desc")}</p>
                    <a href="/signup" class="btn btn-primary btn-block">{session.t("profile.create_free")}</a>
                </div>
            })}
        </aside>
        </div>
    }
}

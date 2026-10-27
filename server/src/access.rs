//! What the person making a request may open (see `codesprout_core::access`
//! for the rules). The admin is recognised by the verified e-mail of their
//! Google sign-in (`ADMIN_EMAILS`, comma-separated).

use std::collections::{HashMap, HashSet};

use codesprout_core::access::{open_sections, section_locked, CourseAccess, Viewer};
use codesprout_core::api::PlanDto;
use codesprout_core::model::Course;

use crate::sql::{Db, DbResult};
use crate::store;

const DEFAULT_ADMINS: &str = "tamirmaidani@gmail.com";

pub fn admin_emails() -> Vec<String> {
    std::env::var("ADMIN_EMAILS")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_ADMINS.to_string())
        .split(',')
        .map(|e| e.trim().to_lowercase())
        .filter(|e| !e.is_empty())
        .collect()
}

pub fn is_admin_email(email: &str) -> bool {
    let email = email.trim().to_lowercase();
    !email.is_empty() && admin_emails().contains(&email)
}

#[derive(Debug, Clone)]
pub struct Access {
    pub viewer: Viewer,
    pub user_id: Option<i64>,
    pub email: Option<String>,
    full: HashSet<String>,
    blocked: HashSet<String>,
    pub plans: Vec<PlanDto>,
}

fn plan_rank(kind: &str) -> u8 {
    match kind {
        "lifetime" => 4,
        "yearly" => 3,
        "monthly" => 2,
        _ => 1,
    }
}

impl Access {
    pub fn guest() -> Access {
        Access {
            viewer: Viewer::Guest,
            user_id: None,
            email: None,
            full: HashSet::new(),
            blocked: HashSet::new(),
            plans: Vec::new(),
        }
    }

    pub async fn for_user(db: &Db, user_id: i64, now: i64) -> DbResult<Access> {
        let email = store::user_email(db, user_id).await?;
        let viewer = if email.as_deref().is_some_and(is_admin_email) { Viewer::Admin } else { Viewer::User };
        let overrides = store::overrides(db, user_id).await?;
        let payments = store::payments_for_user(db, user_id).await?;

        let mut best: HashMap<String, PlanDto> = HashMap::new();
        let mut offer = |plan: PlanDto| {
            let keep = best.get(&plan.course_id).map_or(true, |old| plan_rank(&plan.kind) > plan_rank(&old.kind));
            if keep {
                best.insert(plan.course_id.clone(), plan);
            }
        };
        for p in payments.iter().filter(|p| p.active(now)) {
            offer(PlanDto {
                course_id: p.course_id.clone(),
                kind: p.plan.clone(),
                until: if matches!(p.plan.as_str(), "monthly" | "yearly") { p.until } else { None },
                manage_url: (!p.manage_url.is_empty()).then(|| p.manage_url.clone()),
            });
        }
        let mut blocked = HashSet::new();
        for (course, mode) in &overrides {
            match mode.as_str() {
                "open" => offer(PlanDto { course_id: course.clone(), kind: "manual".into(), until: None, manage_url: None }),
                "block" => {
                    blocked.insert(course.clone());
                }
                _ => {}
            }
        }
        let mut plans: Vec<PlanDto> = best.into_values().filter(|p| !blocked.contains(&p.course_id)).collect();
        plans.sort_by(|a, b| a.course_id.cmp(&b.course_id));
        let full = plans.iter().map(|p| p.course_id.clone()).collect();
        Ok(Access { viewer, user_id: Some(user_id), email, full, blocked, plans })
    }

    pub fn is_admin(&self) -> bool {
        self.viewer == Viewer::Admin
    }

    pub fn viewer_name(&self) -> &'static str {
        match self.viewer {
            Viewer::Guest => "guest",
            Viewer::User => "user",
            Viewer::Admin => "admin",
        }
    }

    pub fn course_access(&self, course_id: &str) -> CourseAccess {
        if self.blocked.contains(course_id) {
            CourseAccess::Blocked
        } else if self.full.contains(course_id) {
            CourseAccess::Full
        } else {
            CourseAccess::Free
        }
    }

    /// Every topic of the course is open.
    pub fn unlocked(&self, course: &Course) -> bool {
        let n = course.sections.len();
        open_sections(self.viewer, self.course_access(course.id), n) == n
    }

    pub fn section_locked(&self, course: &Course, section_index: usize) -> bool {
        section_locked(self.viewer, self.course_access(course.id), section_index, course.sections.len())
    }

    pub fn lesson_locked(&self, course: &Course, lesson_id: &str) -> bool {
        match course.sections.iter().position(|s| s.lessons.iter().any(|l| l.id == lesson_id)) {
            Some(i) => self.section_locked(course, i),
            None => true,
        }
    }
}

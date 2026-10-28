//! Who may open which lessons.
//!
//! * Guests: the first topic (section) of every course.
//! * Signed-in users: the first two topics of every course.
//! * A course the user paid for ($5.99/month while subscribed, or $29 once for a year),
//!   or that the admin opened for them: every topic of that course.
//! * The admin account: everything.
//! * A course the admin blocked for a user: nothing in that course.

/// Topics open to visitors without an account.
pub const FREE_SECTIONS_GUEST: usize = 1;
/// Topics open to every signed-in user.
pub const FREE_SECTIONS_ACCOUNT: usize = 2;

/// Prices shown in the app (the real amounts are set in the payment service).
pub const PRICE_MONTHLY: &str = "$5.99";
pub const PRICE_YEARLY: &str = "$29";
/// "All courses": every course at once (course id `ALL_COURSES` on the payment).
pub const PRICE_ALL_MONTHLY: &str = "$9.99";
pub const PRICE_ALL_YEARLY: &str = "$59";
/// The course id a purchase of every course carries.
pub const ALL_COURSES: &str = "*";
/// How long a one-time ($29) purchase keeps a course open.
pub const YEAR_SECS: i64 = 365 * 86_400;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Viewer {
    Guest,
    User,
    Admin,
}

/// Access to one course for one viewer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CourseAccess {
    /// Only the free topics.
    Free,
    /// Every topic.
    Full,
    /// Nothing (blocked by the admin).
    Blocked,
}

/// How many leading topics are open.
pub fn open_sections(viewer: Viewer, access: CourseAccess, total_sections: usize) -> usize {
    match (viewer, access) {
        (Viewer::Admin, _) => total_sections,
        (_, CourseAccess::Blocked) => 0,
        (_, CourseAccess::Full) => total_sections,
        (Viewer::Guest, CourseAccess::Free) => FREE_SECTIONS_GUEST.min(total_sections),
        (Viewer::User, CourseAccess::Free) => FREE_SECTIONS_ACCOUNT.min(total_sections),
    }
}

/// Is the topic at `section_index` (0-based) locked?
pub fn section_locked(viewer: Viewer, access: CourseAccess, section_index: usize, total_sections: usize) -> bool {
    section_index >= open_sections(viewer, access, total_sections)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules() {
        use CourseAccess::*;
        use Viewer::*;
        assert!(!section_locked(Guest, Free, 0, 18));
        assert!(section_locked(Guest, Free, 1, 18));
        assert!(!section_locked(User, Free, 1, 18));
        assert!(section_locked(User, Free, 2, 18));
        assert!(!section_locked(User, Full, 17, 18));
        assert!(section_locked(User, Blocked, 0, 18));
        assert!(!section_locked(Admin, Blocked, 17, 18));
        assert_eq!(open_sections(Guest, Full, 18), 18);
    }
}

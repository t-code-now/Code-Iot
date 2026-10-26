//! XP, levels, streaks, calendar weeks and badges.

use crate::api::Badge;

const SECS_PER_DAY: i64 = 86_400;

/// Local calendar day number (days since 1970-01-01) for a Unix timestamp
/// and a UTC offset in minutes (positive = east of UTC).
pub fn day_number(unix_secs: i64, tz_offset_minutes: i32) -> i64 {
    (unix_secs + tz_offset_minutes as i64 * 60).div_euclid(SECS_PER_DAY)
}

/// Week number with weeks starting on Monday. 1970-01-01 was a Thursday.
pub fn week_number(day: i64) -> i64 {
    (day + 3).div_euclid(7)
}

/// Unix timestamp at which the given week starts (Monday 00:00 local).
pub fn week_start_unix(week: i64, tz_offset_minutes: i32) -> i64 {
    let day = week * 7 - 3;
    day * SECS_PER_DAY - tz_offset_minutes as i64 * 60
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Streak {
    pub current: u32,
    pub longest: u32,
}

/// Update a streak after activity on `today`.
/// * same day as last activity → unchanged
/// * the day after → +1
/// * otherwise → restart at 1
pub fn record_activity(streak: Streak, last_day: Option<i64>, today: i64) -> Streak {
    let current = match last_day {
        Some(d) if d == today => streak.current.max(1),
        Some(d) if d == today - 1 => streak.current + 1,
        _ => 1,
    };
    Streak { current, longest: streak.longest.max(current) }
}

/// The streak to display: it is broken if the learner missed a full day.
pub fn effective_streak(current: u32, last_day: Option<i64>, today: i64) -> u32 {
    match last_day {
        Some(d) if today - d <= 1 => current,
        _ => 0,
    }
}

/// XP for finishing a lesson.
/// First completion: 10 + 2 per first-try correct answer, +5 for a perfect run.
/// Replays: 2 + 1 per correct answer (practice is still rewarded).
pub fn xp_for_lesson(correct: u32, total: u32, first_time: bool) -> u32 {
    let correct = correct.min(total);
    if first_time {
        let perfect = if total > 0 && correct == total { 5 } else { 0 };
        10 + 2 * correct + perfect
    } else {
        2 + correct
    }
}

/// XP needed to go from `level` to `level + 1`.
pub fn xp_to_next(level: u32) -> u64 {
    50 + 25 * (level as u64 - 1)
}

/// (level, xp inside level, xp needed for this level). Levels start at 1.
pub fn level_for_xp(xp: u64) -> (u32, u64, u64) {
    let mut level = 1u32;
    let mut remaining = xp;
    loop {
        let need = xp_to_next(level);
        if remaining < need {
            return (level, remaining, need);
        }
        remaining -= need;
        level += 1;
    }
}

/// Learner statistics used to award badges.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Stats {
    pub xp: u64,
    pub lessons_completed: u32,
    pub courses_completed: u32,
    pub longest_streak: u32,
    pub perfect_lessons: u32,
}

struct BadgeDef {
    id: &'static str,
    title: &'static str,
    description: &'static str,
    icon: &'static str,
    earned: fn(&Stats) -> bool,
}

const BADGES: &[BadgeDef] = &[
    BadgeDef { id: "first_lesson", title: "First Steps", description: "Finish your first lesson", icon: "🌱", earned: |s| s.lessons_completed >= 1 },
    BadgeDef { id: "ten_lessons", title: "On a Roll", description: "Finish 10 lessons", icon: "📚", earned: |s| s.lessons_completed >= 10 },
    BadgeDef { id: "perfect", title: "Flawless", description: "Finish a lesson with every answer right", icon: "💎", earned: |s| s.perfect_lessons >= 1 },
    BadgeDef { id: "xp_100", title: "Century", description: "Earn 100 XP", icon: "⚡", earned: |s| s.xp >= 100 },
    BadgeDef { id: "xp_1000", title: "Powerhouse", description: "Earn 1,000 XP", icon: "🚀", earned: |s| s.xp >= 1000 },
    BadgeDef { id: "streak_3", title: "Warming Up", description: "Reach a 3-day streak", icon: "🔥", earned: |s| s.longest_streak >= 3 },
    BadgeDef { id: "streak_7", title: "Week Warrior", description: "Reach a 7-day streak", icon: "🏅", earned: |s| s.longest_streak >= 7 },
    BadgeDef { id: "course_done", title: "Graduate", description: "Complete every lesson in a course", icon: "🎓", earned: |s| s.courses_completed >= 1 },
];

fn to_badge(b: &BadgeDef) -> Badge {
    Badge {
        id: b.id.into(),
        title: b.title.into(),
        description: b.description.into(),
        icon: b.icon.into(),
    }
}

/// Every badge that exists, earned or not (for the profile's badge grid).
pub fn all_badges() -> Vec<Badge> {
    BADGES.iter().map(to_badge).collect()
}

/// All badges earned for the given stats.
pub fn badges(stats: &Stats) -> Vec<Badge> {
    BADGES.iter().filter(|b| (b.earned)(stats)).map(to_badge).collect()
}

/// Badges earned in `after` that were not earned in `before`.
pub fn new_badges(before: &Stats, after: &Stats) -> Vec<Badge> {
    let had: Vec<String> = badges(before).into_iter().map(|b| b.id).collect();
    badges(after).into_iter().filter(|b| !had.contains(&b.id)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn days_and_weeks() {
        assert_eq!(day_number(0, 0), 0);
        assert_eq!(day_number(-1, 0), -1);
        // 23:30 UTC on day 0 is already day 1 at UTC+2.
        assert_eq!(day_number(23 * 3600 + 1800, 120), 1);
        // 1970-01-05 (day 4) was a Monday → starts week 1.
        assert_eq!(week_number(3), 0);
        assert_eq!(week_number(4), 1);
        assert_eq!(week_start_unix(1, 0), 4 * SECS_PER_DAY);
    }

    #[test]
    fn streaks() {
        let s = Streak { current: 4, longest: 6 };
        assert_eq!(record_activity(s, Some(10), 10), s);
        assert_eq!(record_activity(s, Some(9), 10), Streak { current: 5, longest: 6 });
        assert_eq!(record_activity(s, Some(7), 10), Streak { current: 1, longest: 6 });
        assert_eq!(record_activity(Streak { current: 0, longest: 0 }, None, 10), Streak { current: 1, longest: 1 });
        assert_eq!(record_activity(Streak { current: 6, longest: 6 }, Some(9), 10).longest, 7);
        assert_eq!(effective_streak(5, Some(9), 10), 5);
        assert_eq!(effective_streak(5, Some(8), 10), 0);
    }

    #[test]
    fn xp_and_levels() {
        assert_eq!(xp_for_lesson(4, 4, true), 23);
        assert_eq!(xp_for_lesson(2, 4, true), 14);
        assert_eq!(xp_for_lesson(9, 4, true), 23); // clamped
        assert_eq!(xp_for_lesson(3, 4, false), 5);
        assert_eq!(level_for_xp(0), (1, 0, 50));
        assert_eq!(level_for_xp(49), (1, 49, 50));
        assert_eq!(level_for_xp(50), (2, 0, 75));
        assert_eq!(level_for_xp(130), (3, 5, 100));
    }

    #[test]
    fn badge_awards() {
        let before = Stats::default();
        let after = Stats { xp: 120, lessons_completed: 1, perfect_lessons: 1, ..Default::default() };
        let ids: Vec<String> = new_badges(&before, &after).into_iter().map(|b| b.id).collect();
        assert_eq!(ids, vec!["first_lesson", "perfect", "xp_100"]);
        assert!(new_badges(&after, &after).is_empty());
    }
}

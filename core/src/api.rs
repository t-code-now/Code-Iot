//! JSON types exchanged between the server and the web app.
//! Serialization is enabled with the `serde` feature.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ---------- auth & profile ----------

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AuthResponse {
    pub token: String,
    pub profile: Profile,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Badge {
    pub id: String,
    pub title: String,
    pub description: String,
    pub icon: String,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Profile {
    pub username: String,
    pub xp: u64,
    pub weekly_xp: u64,
    pub level: u32,
    /// XP earned inside the current level.
    pub level_xp: u64,
    /// XP needed to finish the current level.
    pub level_xp_needed: u64,
    pub streak: u32,
    pub longest_streak: u32,
    pub lessons_completed: u32,
    pub courses_completed: u32,
    pub badges: Vec<Badge>,
    /// Unix seconds.
    pub joined_at: i64,
    /// Profile picture: a built-in avatar ("preset:<name>") or an uploaded
    /// image as a data URL. None until the user picks one.
    #[cfg_attr(feature = "serde", serde(default))]
    pub avatar: Option<String>,
}

/// Body of `PUT /api/me/avatar`. None removes the picture.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AvatarRequest {
    pub avatar: Option<String>,
}

/// The built-in avatars a user can pick.
pub const AVATAR_PRESETS: &[&str] = &["mascot", "a1", "a2", "a3", "a4"];

/// Largest accepted uploaded picture (the app sends a 256 px JPEG, ~20-40 KB).
pub const AVATAR_MAX_BYTES: usize = 300_000;

/// Checks an avatar value; returns an error message when it isn't acceptable.
pub fn validate_avatar(avatar: &str) -> Result<(), &'static str> {
    if let Some(name) = avatar.strip_prefix("preset:") {
        return if AVATAR_PRESETS.contains(&name) { Ok(()) } else { Err("Unknown avatar") };
    }
    let ok_type = ["data:image/jpeg;base64,", "data:image/png;base64,", "data:image/webp;base64,"]
        .iter()
        .any(|p| avatar.starts_with(p));
    if !ok_type {
        return Err("The picture must be a JPEG, PNG or WebP image");
    }
    if avatar.len() > AVATAR_MAX_BYTES {
        return Err("The picture is too large");
    }
    let data = &avatar[avatar.find(',').map_or(0, |i| i + 1)..];
    if !data.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'+' || b == b'/' || b == b'=') {
        return Err("The picture data is not valid");
    }
    Ok(())
}

// ---------- catalog ----------

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CourseSummary {
    pub id: String,
    pub title: String,
    pub language: String,
    pub description: String,
    pub color: String,
    pub icon: String,
    pub lesson_count: u32,
    pub completed_count: u32,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CourseDetail {
    pub id: String,
    pub title: String,
    pub language: String,
    pub description: String,
    pub color: String,
    pub icon: String,
    pub sections: Vec<SectionDto>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SectionDto {
    pub id: String,
    pub title: String,
    pub lessons: Vec<LessonSummary>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LessonSummary {
    pub id: String,
    pub title: String,
    pub exercise_count: u32,
    pub completed: bool,
    pub best_correct: u32,
}

/// A lesson ready to be played. Answers are never included.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LessonDto {
    pub id: String,
    pub title: String,
    pub course_id: String,
    pub course_title: String,
    pub color: String,
    pub next_lesson_id: Option<String>,
    pub steps: Vec<StepDto>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", rename_all = "snake_case"))]
pub enum StepDto {
    Learn {
        title: String,
        body: String,
        code: Option<String>,
    },
    MultipleChoice {
        prompt: String,
        code: Option<String>,
        options: Vec<String>,
    },
    FillBlanks {
        prompt: String,
        code: String,
        blanks: u32,
        /// Already shuffled.
        options: Vec<String>,
    },
    Reorder {
        prompt: String,
        /// Already shuffled.
        lines: Vec<String>,
    },
    TypeIn {
        prompt: String,
        code: Option<String>,
    },
}

// ---------- answering ----------

/// A learner's answer to one step.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "value", rename_all = "snake_case"))]
pub enum Answer {
    /// Index of the chosen option (multiple choice).
    Choice(usize),
    /// Tiles in order: blank fillers, or the reordered lines.
    Tokens(Vec<String>),
    /// Free text (type-in).
    Text(String),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CheckRequest {
    pub step: usize,
    pub answer: Answer,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CheckResponse {
    pub correct: bool,
    /// The correct answer, human readable.
    pub expected: String,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CompleteRequest {
    /// Exercises answered correctly on the first try.
    pub correct: u32,
    /// Minutes east of UTC (e.g. Israel winter time = 120).
    pub tz_offset_minutes: i32,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CompleteResponse {
    pub xp_gained: u32,
    pub first_time: bool,
    pub profile: Profile,
    pub new_badges: Vec<Badge>,
}

// ---------- leaderboard ----------

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LeaderboardEntry {
    pub rank: u32,
    pub username: String,
    pub xp: u64,
    pub is_me: bool,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Leaderboard {
    /// "weekly" or "all_time".
    pub scope: String,
    pub entries: Vec<LeaderboardEntry>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ApiErrorBody {
    pub error: String,
}

#[cfg(test)]
mod avatar_tests {
    use super::*;

    #[test]
    fn avatar_values() {
        assert!(validate_avatar("preset:a1").is_ok());
        assert!(validate_avatar("preset:mascot").is_ok());
        assert!(validate_avatar("preset:../etc").is_err());
        assert!(validate_avatar("data:image/jpeg;base64,/9j/4AAQSkZJRg==").is_ok());
        assert!(validate_avatar("data:text/html;base64,PGgxPg==").is_err());
        assert!(validate_avatar("data:image/png;base64,abc\"<script>").is_err());
        let big = format!("data:image/jpeg;base64,{}", "A".repeat(AVATAR_MAX_BYTES));
        assert!(validate_avatar(&big).is_err());
    }
}

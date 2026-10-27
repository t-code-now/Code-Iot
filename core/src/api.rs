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
    #[cfg_attr(feature = "serde", serde(default))]
    pub id: i64,
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
    /// The verified Google e-mail, when the account signs in with Google.
    #[cfg_attr(feature = "serde", serde(default))]
    pub email: Option<String>,
    /// Signed in with the admin Google account.
    #[cfg_attr(feature = "serde", serde(default))]
    pub is_admin: bool,
    /// Courses this account has fully opened (bought, subscribed or opened by the admin).
    #[cfg_attr(feature = "serde", serde(default))]
    pub plans: Vec<PlanDto>,
    /// Phone left for purchases, international format ("+972501234567").
    #[cfg_attr(feature = "serde", serde(default))]
    pub phone: Option<String>,
    /// ISO country of that phone ("IL").
    #[cfg_attr(feature = "serde", serde(default))]
    pub country: Option<String>,
}

/// `PUT /api/me/contact`
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ContactRequest {
    pub country: String,
    pub phone: String,
}

/// A course the user can fully open.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PlanDto {
    pub course_id: String,
    /// "yearly", "monthly", "manual" or (older purchases) "lifetime".
    pub kind: String,
    /// Subscription end/renewal (unix seconds), if any.
    pub until: Option<i64>,
    /// Where the user manages a subscription or sees a receipt.
    pub manage_url: Option<String>,
}

/// `POST /api/me/waitlist`
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct WaitlistRequest {
    pub course_id: String,
    pub join: bool,
}

/// `GET /api/admin/waitlist`: one person waiting for one course.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AdminWaitEntry {
    pub user_id: i64,
    pub username: String,
    pub email: String,
    pub phone: String,
    pub country: String,
    pub course_id: String,
    pub created_at: i64,
}

/// `GET /api/me/purchases`: the signed-in user's own purchases (one-time
/// orders and subscriptions; monthly invoices are folded into their subscription).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MyPurchase {
    pub id: i64,
    pub course_id: String,
    /// "lifetime" or "monthly".
    pub plan: String,
    pub status: String,
    pub amount_cents: i64,
    pub refunded_cents: i64,
    pub currency: String,
    pub created_at: i64,
    /// Subscriptions: paid until / ends at.
    pub until: Option<i64>,
    pub can_cancel: bool,
    pub test_mode: bool,
}

/// `POST /api/payments/checkout`
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CheckoutRequest {
    pub course_id: String,
    /// "monthly" or "lifetime".
    pub plan: String,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CheckoutResponse {
    pub url: String,
}

// ---------- admin ----------

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AdminOverview {
    pub users: u64,
    pub google_users: u64,
    pub new_users_7d: u64,
    pub active_users_7d: u64,
    pub lessons_7d: u64,
    pub paying_users: u64,
    pub lifetime_purchases: u64,
    pub active_subscriptions: u64,
    /// Total paid, per currency, e.g. [("USD", 12500)] in cents.
    pub revenue: Vec<(String, i64)>,
    pub banned_users: u64,
    /// Refunded so far, per currency (cents).
    #[cfg_attr(feature = "serde", serde(default))]
    pub refunded: Vec<(String, i64)>,
    /// New purchases are paused by the admin.
    #[cfg_attr(feature = "serde", serde(default))]
    pub sales_paused: bool,
    /// (course id, users who started it, users with full access)
    pub courses: Vec<(String, u64, u64)>,
    pub payments_enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AdminCourseState {
    pub course_id: String,
    /// "free", "lifetime", "monthly", "open" (opened by admin) or "blocked".
    pub state: String,
    pub until: Option<i64>,
    /// The admin's manual setting: "open", "block" or "" (none).
    pub manual: String,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AdminUser {
    pub id: i64,
    pub username: String,
    pub email: String,
    /// "google" or "password".
    pub sign_in: String,
    pub joined_at: i64,
    pub last_active_day: Option<i64>,
    pub xp: i64,
    pub lessons_completed: u64,
    pub banned: bool,
    pub is_admin: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    pub phone: String,
    #[cfg_attr(feature = "serde", serde(default))]
    pub country: String,
    /// Payment methods seen, e.g. "Visa •••• 4242".
    pub payment_methods: Vec<String>,
    pub total_paid_cents: i64,
    pub courses: Vec<AdminCourseState>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AdminPayment {
    pub id: i64,
    pub user_id: Option<i64>,
    pub username: String,
    pub email: String,
    pub course_id: String,
    /// "lifetime" or "monthly".
    pub plan: String,
    pub status: String,
    pub amount_cents: i64,
    pub currency: String,
    pub method: String,
    pub provider: String,
    pub provider_ref: String,
    pub until: Option<i64>,
    pub test_mode: bool,
    pub created_at: i64,
    /// Opens this payment in the payment service.
    pub dashboard_url: String,
    #[cfg_attr(feature = "serde", serde(default))]
    pub refunded_cents: i64,
    /// What can still be refunded (cents); 0 when nothing can.
    #[cfg_attr(feature = "serde", serde(default))]
    pub refundable_cents: i64,
    /// An active subscription that can be cancelled.
    #[cfg_attr(feature = "serde", serde(default))]
    pub can_cancel: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    pub customer_name: String,
    #[cfg_attr(feature = "serde", serde(default))]
    pub phone: String,
    #[cfg_attr(feature = "serde", serde(default))]
    pub country: String,
    /// The last event received from the payment service (JSON), for the details view.
    #[cfg_attr(feature = "serde", serde(default))]
    pub raw: String,
    #[cfg_attr(feature = "serde", serde(default))]
    pub updated_at: i64,
    /// Subscriptions (and their invoices): when the subscription was cancelled.
    #[cfg_attr(feature = "serde", serde(default))]
    pub cancelled_at: Option<i64>,
    /// Suggested refund for the unused part of the month (cents), on the last
    /// invoice of a cancelled subscription; 0 when there is none.
    #[cfg_attr(feature = "serde", serde(default))]
    pub prorata_cents: i64,
    #[cfg_attr(feature = "serde", serde(default))]
    pub prorata_unused_days: i64,
    #[cfg_attr(feature = "serde", serde(default))]
    pub prorata_total_days: i64,
}

/// `POST /api/admin/refund` – amount in cents (partial or full).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AdminRefundRequest {
    pub payment_id: i64,
    pub amount_cents: i64,
}

/// `POST /api/admin/cancel` – cancels a subscription.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AdminCancelRequest {
    pub payment_id: i64,
}

/// `POST /api/admin/settings`
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AdminSettingsRequest {
    pub sales_paused: bool,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AdminLogEntry {
    pub at: i64,
    pub admin: String,
    pub action: String,
    pub target: String,
    pub detail: String,
}

/// `POST /api/admin/access`
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AdminAccessRequest {
    pub user_id: i64,
    pub course_id: String,
    /// "open", "block" or "default".
    pub mode: String,
}

/// `POST /api/admin/ban`
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AdminBanRequest {
    pub user_id: i64,
    pub banned: bool,
}

/// `POST /api/admin/delete`
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AdminDeleteRequest {
    pub user_id: i64,
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
    /// Every topic is open for this viewer.
    #[cfg_attr(feature = "serde", serde(default))]
    pub unlocked: bool,
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
    /// Every topic is open for this viewer.
    #[cfg_attr(feature = "serde", serde(default))]
    pub unlocked: bool,
    /// "guest", "user" or "admin".
    #[cfg_attr(feature = "serde", serde(default))]
    pub viewer: String,
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
    #[cfg_attr(feature = "serde", serde(default))]
    pub locked: bool,
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

//! HTTP handlers. Every course and lesson is available to everyone;
//! logging in is only needed to save progress.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::routing::{any, get, post, put};
use axum::{Json, Router};
use rand::RngCore;
use serde::Deserialize;

use codesprout_core::api::*;
use codesprout_core::model::{Step, BLANK};
use codesprout_core::progress::{self, Stats};
use codesprout_core::i18n::{self, Lang};
use codesprout_core::{catalog, find_course, find_lesson, grading, shuffle};

use crate::access::Access;
use crate::db;
use crate::sql::Db;
use crate::error::{ApiError, ApiResult};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Db>,
}

pub fn api_router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(|| async { "ok" }))
        .route("/api/auth/signup", post(signup))
        .route("/api/auth/login", post(login))
        .route("/api/auth/logout", post(logout))
        .route("/api/auth/providers", get(crate::google::providers))
        .route("/api/auth/google/start", get(crate::google::start))
        .route("/api/auth/google/callback", get(crate::google::callback))
        .route("/api/auth/google/poll", get(crate::google::poll))
        .route("/api/auth/google/id-token", post(crate::google::id_token))
        .route("/api/me", get(me))
        .route("/api/me/avatar", put(set_avatar))
        .route("/api/courses", get(list_courses))
        .route("/api/courses/:id", get(get_course))
        .route("/api/lessons/:id", get(get_lesson))
        .route("/api/lessons/:id/check", post(check_answer))
        .route("/api/lessons/:id/complete", post(complete_lesson))
        .route("/api/leaderboard", get(leaderboard))
        .route("/api/run", post(crate::run::run))
        .route("/api/payments/config", get(crate::payments::config_info))
        .route("/api/payments/checkout", post(crate::payments::checkout))
        .route("/api/payments/webhook", post(crate::payments::webhook))
        .route("/api/admin/overview", get(crate::admin::overview))
        .route("/api/admin/payments-setup", get(crate::payments::setup_info))
        .route("/api/admin/users", get(crate::admin::users))
        .route("/api/admin/payments", get(crate::admin::payments))
        .route("/api/admin/log", get(crate::admin::log))
        .route("/api/admin/access", post(crate::admin::set_access))
        .route("/api/admin/ban", post(crate::admin::ban))
        .route("/api/admin/delete", post(crate::admin::delete))
        .route("/api/admin/refund", post(crate::payments::admin_refund))
        .route("/api/admin/cancel", post(crate::payments::admin_cancel))
        .route("/api/admin/settings", post(crate::admin::settings))
        .route("/api/me/contact", put(set_contact))
        .route("/api/me/purchases", get(crate::payments::my_purchases))
        .route("/api/me/waitlist", get(crate::payments::my_waitlist).post(crate::payments::set_waitlist))
        .route("/api/admin/waitlist", get(crate::admin::waitlist))
        .route("/api/me/cancel", post(crate::payments::user_cancel))
        // Unknown API paths answer with a JSON 404 instead of the web page.
        .route("/api/*rest", any(|| async { ApiError::NotFound }))
        .with_state(state)
}

// ---------------------------------------------------------------- helpers

pub(crate) fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

pub(crate) fn random_hex(bytes: usize) -> String {
    let mut buf = vec![0u8; bytes];
    rand::thread_rng().fill_bytes(&mut buf);
    buf.iter().map(|b| format!("{b:02x}")).collect()
}

fn bearer(headers: &HeaderMap) -> Option<String> {
    headers
        .get("authorization")?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(|t| t.trim().to_string())
}

/// The logged-in user id, if any.
async fn optional_user(state: &AppState, headers: &HeaderMap) -> ApiResult<Option<i64>> {
    match bearer(headers) {
        Some(token) => Ok(db::user_for_token(&state.db, &token).await?),
        None => Ok(None),
    }
}

pub(crate) async fn require_user(state: &AppState, headers: &HeaderMap) -> ApiResult<i64> {
    optional_user(state, headers).await?.ok_or(ApiError::Unauthorized)
}

/// What the person making this request may open.
pub(crate) async fn access_for(state: &AppState, headers: &HeaderMap) -> ApiResult<Access> {
    match optional_user(state, headers).await? {
        Some(id) => Ok(Access::for_user(&state.db, id, now()).await?),
        None => Ok(Access::guest()),
    }
}

fn validate(req: &AuthRequest) -> ApiResult<()> {
    let name = req.username.trim();
    if !(3..=20).contains(&name.chars().count())
        || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(ApiError::BadRequest(
            "Username must be 3–20 letters, numbers or underscores".into(),
        ));
    }
    if req.password.chars().count() < 6 {
        return Err(ApiError::BadRequest("Password must be at least 6 characters".into()));
    }
    Ok(())
}

/// Lesson ids from the catalog that the user has finished.
fn finished_ids(done: &HashMap<String, u32>) -> impl Fn(&str) -> bool + '_ {
    move |id| done.contains_key(id)
}

fn stats_for(user: &db::User, done: &HashMap<String, u32>) -> Stats {
    let is_done = finished_ids(done);
    let lessons_completed = catalog()
        .iter()
        .flat_map(|c| c.lessons())
        .filter(|l| is_done(l.id))
        .count() as u32;
    let courses_completed = catalog()
        .iter()
        .filter(|c| c.lessons().all(|l| is_done(l.id)))
        .count() as u32;
    Stats {
        xp: user.xp.max(0) as u64,
        lessons_completed,
        courses_completed,
        longest_streak: user.longest_streak as u32,
        perfect_lessons: user.perfect_lessons as u32,
    }
}

async fn build_profile(conn: &Db, user_id: i64) -> ApiResult<(Profile, Stats)> {
    let user = db::get_user(conn, user_id).await?.ok_or(ApiError::Unauthorized)?;
    let access = Access::for_user(conn, user_id, now()).await?;
    let contact = crate::store::contact(conn, user_id).await?;
    let done = db::completions(conn, user_id).await?;
    let stats = stats_for(&user, &done);

    let t = now();
    let tz = user.tz_offset as i32;
    let today = progress::day_number(t, tz);
    let week_start = progress::week_start_unix(progress::week_number(today), tz);
    let weekly_xp = db::xp_since(conn, user_id, week_start).await?;
    let (level, level_xp, level_xp_needed) = progress::level_for_xp(stats.xp);

    let profile = Profile {
        id: user.id,
        username: user.username.clone(),
        xp: stats.xp,
        weekly_xp: weekly_xp.max(0) as u64,
        level,
        level_xp,
        level_xp_needed,
        streak: progress::effective_streak(user.streak as u32, user.last_active_day, today),
        longest_streak: user.longest_streak as u32,
        lessons_completed: stats.lessons_completed,
        courses_completed: stats.courses_completed,
        badges: progress::badges(&stats),
        joined_at: user.created_at,
        avatar: db::get_avatar(conn, user_id).await?,
        phone: contact.as_ref().map(|c| c.0.clone()),
        country: contact.map(|c| c.1),
        email: access.email.clone(),
        is_admin: access.is_admin(),
        plans: access.plans.clone(),
    };
    Ok((profile, stats))
}

// ---------------------------------------------------------------- auth

async fn signup(State(state): State<AppState>, Json(req): Json<AuthRequest>) -> ApiResult<Json<AuthResponse>> {
    // New accounts sign up with Google. (Tests and self-hosting can turn
    // password sign-up back on with ALLOW_PASSWORD_SIGNUP=1.)
    if std::env::var("ALLOW_PASSWORD_SIGNUP").as_deref() != Ok("1") {
        return Err(ApiError::BadRequest("Please sign up with Google".into()));
    }
    validate(&req)?;
    let username = req.username.trim().to_string();
    let password = req.password.clone();

    let hash = tokio::task::spawn_blocking(move || {
        let mut salt_bytes = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut salt_bytes);
        let salt = SaltString::encode_b64(&salt_bytes).map_err(|e| ApiError::Internal(e.to_string()))?;
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| ApiError::Internal(e.to_string()))
    })
    .await??;

    let conn = &state.db;
    let t = now();
    let user_id = db::create_user(conn, &username, &hash, t)
        .await?
        .ok_or_else(|| ApiError::Conflict("That username is taken".into()))?;
    let token = random_hex(32);
    db::create_session(conn, &token, user_id, t).await?;
    let (profile, _) = build_profile(conn, user_id).await?;
    Ok(Json(AuthResponse { token, profile }))
}

async fn login(State(state): State<AppState>, Json(req): Json<AuthRequest>) -> ApiResult<Json<AuthResponse>> {
    let creds = db::credentials(&state.db, req.username.trim()).await?;
    let wrong = || ApiError::BadRequest("Wrong username or password".into());
    let (user_id, hash) = creds.ok_or_else(wrong)?;

    let password = req.password.clone();
    let ok = tokio::task::spawn_blocking(move || {
        PasswordHash::new(&hash)
            .map(|parsed| Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok())
            .unwrap_or(false)
    })
    .await?;
    if !ok {
        return Err(wrong());
    }
    if crate::store::is_banned(&state.db, user_id).await? {
        return Err(ApiError::BadRequest("This account is suspended".into()));
    }

    let conn = &state.db;
    let token = random_hex(32);
    db::create_session(conn, &token, user_id, now()).await?;
    let (profile, _) = build_profile(conn, user_id).await?;
    Ok(Json(AuthResponse { token, profile }))
}

async fn logout(State(state): State<AppState>, headers: HeaderMap) -> ApiResult<Json<serde_json::Value>> {
    if let Some(token) = bearer(&headers) {
        db::delete_session(&state.db, &token).await?;
    }
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn me(State(state): State<AppState>, headers: HeaderMap) -> ApiResult<Json<Profile>> {
    let user_id = require_user(&state, &headers).await?;
    let (profile, _) = build_profile(&state.db, user_id).await?;
    Ok(Json(profile))
}

async fn set_avatar(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<AvatarRequest>,
) -> ApiResult<Json<Profile>> {
    let user_id = require_user(&state, &headers).await?;
    if let Some(a) = &req.avatar {
        validate_avatar(a).map_err(|m| ApiError::BadRequest(m.into()))?;
    }
    db::set_avatar(&state.db, user_id, req.avatar.as_deref()).await?;
    let (profile, _) = build_profile(&state.db, user_id).await?;
    Ok(Json(profile))
}

async fn set_contact(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ContactRequest>,
) -> ApiResult<Json<Profile>> {
    let user_id = require_user(&state, &headers).await?;
    let phone = codesprout_core::phone::normalize(&req.country, &req.phone)
        .map_err(|m| ApiError::BadRequest(m.into()))?;
    let country = req.country.trim().to_uppercase();
    crate::store::set_contact(&state.db, user_id, &phone, &country, now()).await?;
    let (profile, _) = build_profile(&state.db, user_id).await?;
    Ok(Json(profile))
}

// ---------------------------------------------------------------- catalog

async fn user_completions(state: &AppState, headers: &HeaderMap) -> ApiResult<HashMap<String, u32>> {
    match optional_user(state, headers).await? {
        Some(id) => Ok(db::completions(&state.db, id).await?),
        None => Ok(HashMap::new()),
    }
}

/// `?lang=he` selects the content language (English by default).
#[derive(Deserialize, Default)]
struct LangQuery {
    lang: Option<String>,
}

impl LangQuery {
    fn lang(&self) -> Lang {
        self.lang.as_deref().map(Lang::parse).unwrap_or_default()
    }
}

async fn list_courses(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<LangQuery>,
) -> ApiResult<Json<Vec<CourseSummary>>> {
    let lang = q.lang();
    let done = user_completions(&state, &headers).await?;
    let access = access_for(&state, &headers).await?;
    let courses = catalog()
        .iter()
        .map(|c| CourseSummary {
            id: c.id.into(),
            title: i18n::course_title(lang, c.id, c.title).into(),
            language: c.language.into(),
            description: i18n::course_description(lang, c.id, c.description).into(),
            color: c.color.into(),
            icon: c.icon.into(),
            lesson_count: c.lesson_count() as u32,
            completed_count: c.lessons().filter(|l| done.contains_key(l.id)).count() as u32,
            unlocked: access.unlocked(c),
        })
        .collect();
    Ok(Json(courses))
}

async fn get_course(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(q): Query<LangQuery>,
) -> ApiResult<Json<CourseDetail>> {
    let lang = q.lang();
    let course = find_course(&id).ok_or(ApiError::NotFound)?;
    let done = user_completions(&state, &headers).await?;
    let access = access_for(&state, &headers).await?;
    Ok(Json(CourseDetail {
        id: course.id.into(),
        title: i18n::course_title(lang, course.id, course.title).into(),
        language: course.language.into(),
        description: i18n::course_description(lang, course.id, course.description).into(),
        color: course.color.into(),
        icon: course.icon.into(),
        sections: course
            .sections
            .iter()
            .enumerate()
            .map(|(si, s)| SectionDto {
                id: s.id.into(),
                title: i18n::section_title(lang, s.id, s.title).into(),
                lessons: s
                    .lessons
                    .iter()
                    .map(|l| LessonSummary {
                        id: l.id.into(),
                        title: i18n::lesson_title(lang, l.id, l.title).into(),
                        exercise_count: l.exercise_count() as u32,
                        completed: done.contains_key(l.id),
                        best_correct: done.get(l.id).copied().unwrap_or(0),
                        locked: access.section_locked(course, si),
                    })
                    .collect(),
            })
            .collect(),
        unlocked: access.unlocked(course),
        viewer: access.viewer_name().into(),
    }))
}

fn seed() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(42)
}

/// Convert a content step to what the client sees: prose translated,
/// answers removed, tiles and lines shuffled. Code is never translated.
fn step_dto(step: &Step, lesson_id: &str, idx: usize, lang: Lang, seed: u64) -> StepDto {
    let owned = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let tr = |field: &str, fallback: &'static str| i18n::step_text(lang, lesson_id, idx, field, fallback).to_string();
    match step {
        Step::Learn { title, body, code } => StepDto::Learn {
            title: tr("title", *title),
            body: tr("body", *body),
            code: code.map(str::to_string),
        },
        Step::MultipleChoice { prompt, code, options, .. } => StepDto::MultipleChoice {
            prompt: tr("prompt", *prompt),
            code: code.map(str::to_string),
            options: options.iter().enumerate().map(|(n, o)| tr(&format!("opt{n}"), *o)).collect(),
        },
        Step::FillBlanks { prompt, code, options, .. } => StepDto::FillBlanks {
            prompt: tr("prompt", *prompt),
            code: code.to_string(),
            blanks: code.matches(BLANK).count() as u32,
            options: shuffle::shuffled(&owned(options), seed),
        },
        Step::Reorder { prompt, lines, .. } => StepDto::Reorder {
            prompt: tr("prompt", *prompt),
            lines: shuffle::shuffled(&owned(lines), seed),
        },
        Step::TypeIn { prompt, code, .. } => StepDto::TypeIn {
            prompt: tr("prompt", *prompt),
            code: code.map(str::to_string),
        },
    }
}

async fn get_lesson(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(q): Query<LangQuery>,
) -> ApiResult<Json<LessonDto>> {
    let lang = q.lang();
    let (course, lesson, next) = find_lesson(&id).ok_or(ApiError::NotFound)?;
    if access_for(&state, &headers).await?.lesson_locked(course, lesson.id) {
        return Err(ApiError::Locked(course.id.into()));
    }
    let base = seed();
    Ok(Json(LessonDto {
        id: lesson.id.into(),
        title: i18n::lesson_title(lang, lesson.id, lesson.title).into(),
        course_id: course.id.into(),
        course_title: i18n::course_title(lang, course.id, course.title).into(),
        color: course.color.into(),
        next_lesson_id: next.map(str::to_string),
        steps: lesson
            .steps
            .iter()
            .enumerate()
            .map(|(i, s)| step_dto(s, lesson.id, i, lang, base.wrapping_add(i as u64 * 7919)))
            .collect(),
    }))
}

async fn check_answer(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(q): Query<LangQuery>,
    Json(req): Json<CheckRequest>,
) -> ApiResult<Json<CheckResponse>> {
    let lang = q.lang();
    let (course, lesson, _) = find_lesson(&id).ok_or(ApiError::NotFound)?;
    if access_for(&state, &headers).await?.lesson_locked(course, lesson.id) {
        return Err(ApiError::Locked(course.id.into()));
    }
    let step = lesson.steps.get(req.step).ok_or(ApiError::NotFound)?;
    let verdict = grading::check(step, &req.answer)
        .ok_or_else(|| ApiError::BadRequest("This step is not an exercise".into()))?;
    // A multiple-choice answer is shown as the (translated) option text.
    let expected = match step {
        Step::MultipleChoice { options, correct, .. } => options
            .get(*correct)
            .map(|o| i18n::step_text(lang, lesson.id, req.step, &format!("opt{correct}"), *o).to_string())
            .unwrap_or(verdict.expected),
        _ => verdict.expected,
    };
    Ok(Json(CheckResponse {
        correct: verdict.correct,
        expected,
        explanation: i18n::step_text(lang, lesson.id, req.step, "explanation", verdict.explanation).to_string(),
    }))
}

async fn complete_lesson(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<CompleteRequest>,
) -> ApiResult<Json<CompleteResponse>> {
    let user_id = require_user(&state, &headers).await?;
    let (course, lesson, _) = find_lesson(&id).ok_or(ApiError::NotFound)?;
    if Access::for_user(&state.db, user_id, now()).await?.lesson_locked(course, lesson.id) {
        return Err(ApiError::Locked(course.id.into()));
    }
    let total = lesson.exercise_count() as u32;
    let tz = req.tz_offset_minutes.clamp(-14 * 60, 14 * 60);
    let t = now();

    let conn = &state.db;
    let (_, before) = build_profile(conn, user_id).await?;
    let result = db::record_completion(
        conn,
        db::CompletionUpdate {
            user_id,
            lesson_id: lesson.id,
            correct: req.correct.min(total),
            total,
            now: t,
            tz_offset: tz,
            today: progress::day_number(t, tz),
        },
    )
    .await?;
    let (profile, after) = build_profile(conn, user_id).await?;

    Ok(Json(CompleteResponse {
        xp_gained: result.xp_gained,
        first_time: result.first_time,
        profile,
        new_badges: progress::new_badges(&before, &after),
    }))
}

// ---------------------------------------------------------------- leaderboard

#[derive(Deserialize)]
struct LeaderboardQuery {
    scope: Option<String>,
}

async fn leaderboard(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<LeaderboardQuery>,
) -> ApiResult<Json<Leaderboard>> {
    let me = match optional_user(&state, &headers).await? {
        Some(id) => db::get_user(&state.db, id).await?.map(|u| u.username),
        None => None,
    };
    let all_time = q.scope.as_deref() == Some("all_time");
    let conn = &state.db;
    let rows = if all_time {
        db::leaderboard_all_time(conn, 50).await?
    } else {
        // Weekly leagues reset Monday 00:00 UTC.
        let week = progress::week_number(progress::day_number(now(), 0));
        db::leaderboard_since(conn, progress::week_start_unix(week, 0), 50).await?
    };
    let entries = rows
        .into_iter()
        .enumerate()
        .map(|(i, (username, xp))| LeaderboardEntry {
            rank: i as u32 + 1,
            is_me: me.as_deref().is_some_and(|m| m.eq_ignore_ascii_case(&username)),
            username,
            xp: xp.max(0) as u64,
        })
        .collect();
    Ok(Json(Leaderboard { scope: if all_time { "all_time" } else { "weekly" }.into(), entries }))
}

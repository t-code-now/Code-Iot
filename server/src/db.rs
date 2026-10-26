//! Storage: users, sessions, completed lessons, XP history and profile
//! pictures. The same SQL runs on Turso (production) and on a local SQLite
//! file (development and tests); see `sql.rs`.

use std::collections::HashMap;

use crate::args;
use crate::sql::{Db, DbResult, Stmt, Value};

const SCHEMA: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS users (
        id               INTEGER PRIMARY KEY,
        username         TEXT    NOT NULL UNIQUE COLLATE NOCASE,
        password_hash    TEXT    NOT NULL,
        created_at       INTEGER NOT NULL,
        xp               INTEGER NOT NULL DEFAULT 0,
        streak           INTEGER NOT NULL DEFAULT 0,
        longest_streak   INTEGER NOT NULL DEFAULT 0,
        last_active_day  INTEGER,
        tz_offset        INTEGER NOT NULL DEFAULT 0,
        perfect_lessons  INTEGER NOT NULL DEFAULT 0
    )",
    "CREATE TABLE IF NOT EXISTS sessions (
        token       TEXT    PRIMARY KEY,
        user_id     INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
        created_at  INTEGER NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS completions (
        user_id       INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
        lesson_id     TEXT    NOT NULL,
        best_correct  INTEGER NOT NULL,
        completed_at  INTEGER NOT NULL,
        PRIMARY KEY (user_id, lesson_id)
    )",
    "CREATE TABLE IF NOT EXISTS xp_events (
        id          INTEGER PRIMARY KEY,
        user_id     INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
        amount      INTEGER NOT NULL,
        created_at  INTEGER NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS xp_events_by_time ON xp_events(created_at)",
    "CREATE INDEX IF NOT EXISTS sessions_by_user ON sessions(user_id)",
    "CREATE TABLE IF NOT EXISTS avatars (
        user_id  INTEGER PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
        data     TEXT    NOT NULL
    )",
];

/// Creates the tables if they don't exist yet.
pub async fn migrate(db: &Db) -> DbResult<()> {
    if let Db::Local(_) = db {
        db.execute("PRAGMA foreign_keys = ON", args![]).await?;
    }
    db.run(SCHEMA.iter().map(|s| Stmt::new(*s, args![])).collect(), false).await?;
    Ok(())
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub created_at: i64,
    pub xp: i64,
    pub streak: i64,
    pub longest_streak: i64,
    pub last_active_day: Option<i64>,
    pub tz_offset: i64,
    pub perfect_lessons: i64,
}

const USER_COLUMNS: &str =
    "id, username, created_at, xp, streak, longest_streak, last_active_day, tz_offset, perfect_lessons";

fn user_from_row(r: &[Value]) -> User {
    User {
        id: r[0].int(),
        username: r[1].text(),
        created_at: r[2].int(),
        xp: r[3].int(),
        streak: r[4].int(),
        longest_streak: r[5].int(),
        last_active_day: r[6].opt_int(),
        tz_offset: r[7].int(),
        perfect_lessons: r[8].int(),
    }
}

/// Returns `Ok(None)` when the username is taken.
pub async fn create_user(db: &Db, username: &str, hash: &str, now: i64) -> DbResult<Option<i64>> {
    match db
        .execute(
            "INSERT INTO users (username, password_hash, created_at) VALUES (?1, ?2, ?3)",
            args![username, hash, now],
        )
        .await
    {
        Ok(r) => match r.last_insert_rowid {
            Some(id) if id > 0 => Ok(Some(id)),
            // Fall back to looking the new row up by name.
            _ => Ok(credentials(db, username).await?.map(|(id, _)| id)),
        },
        Err(e) if e.is_unique_violation() => Ok(None),
        Err(e) => Err(e),
    }
}

/// (user id, password hash) for a username.
pub async fn credentials(db: &Db, username: &str) -> DbResult<Option<(i64, String)>> {
    Ok(db
        .query_one("SELECT id, password_hash FROM users WHERE username = ?1", args![username])
        .await?
        .map(|r| (r[0].int(), r[1].text())))
}

pub async fn get_user(db: &Db, id: i64) -> DbResult<Option<User>> {
    Ok(db
        .query_one(&format!("SELECT {USER_COLUMNS} FROM users WHERE id = ?1"), args![id])
        .await?
        .map(|r| user_from_row(&r)))
}

pub async fn create_session(db: &Db, token: &str, user_id: i64, now: i64) -> DbResult<()> {
    db.execute(
        "INSERT INTO sessions (token, user_id, created_at) VALUES (?1, ?2, ?3)",
        args![token, user_id, now],
    )
    .await?;
    Ok(())
}

pub async fn user_for_token(db: &Db, token: &str) -> DbResult<Option<i64>> {
    Ok(db
        .query_one("SELECT user_id FROM sessions WHERE token = ?1", args![token])
        .await?
        .map(|r| r[0].int()))
}

pub async fn delete_session(db: &Db, token: &str) -> DbResult<()> {
    db.execute("DELETE FROM sessions WHERE token = ?1", args![token]).await?;
    Ok(())
}

/// lesson id → best first-try correct count.
pub async fn completions(db: &Db, user_id: i64) -> DbResult<HashMap<String, u32>> {
    Ok(db
        .query("SELECT lesson_id, best_correct FROM completions WHERE user_id = ?1", args![user_id])
        .await?
        .into_iter()
        .map(|r| (r[0].text(), r[1].int() as u32))
        .collect())
}

pub async fn xp_since(db: &Db, user_id: i64, since: i64) -> DbResult<i64> {
    Ok(db
        .query_one(
            "SELECT COALESCE(SUM(amount), 0) FROM xp_events WHERE user_id = ?1 AND created_at >= ?2",
            args![user_id, since],
        )
        .await?
        .map_or(0, |r| r[0].int()))
}

/// Everything that changes when a lesson is finished.
pub struct CompletionUpdate<'a> {
    pub user_id: i64,
    pub lesson_id: &'a str,
    pub correct: u32,
    pub total: u32,
    pub now: i64,
    pub tz_offset: i32,
    pub today: i64,
}

pub struct CompletionResult {
    pub first_time: bool,
    pub xp_gained: u32,
}

/// Record a finished lesson: completion row, XP event, user XP, streak and
/// perfect-lesson count. The writes are applied together or not at all.
pub async fn record_completion(db: &Db, u: CompletionUpdate<'_>) -> DbResult<CompletionResult> {
    use codesprout_core::progress::{record_activity, xp_for_lesson, Streak};

    let read = db
        .run(
            vec![
                Stmt::new(
                    "SELECT best_correct FROM completions WHERE user_id = ?1 AND lesson_id = ?2",
                    args![u.user_id, u.lesson_id],
                ),
                Stmt::new(
                    "SELECT streak, longest_streak, last_active_day FROM users WHERE id = ?1",
                    args![u.user_id],
                ),
            ],
            false,
        )
        .await?;
    let previous: Option<i64> = read[0].rows.first().map(|r| r[0].int());
    let (streak, longest, last_day) = read[1]
        .rows
        .first()
        .map(|r| (r[0].int(), r[1].int(), r[2].opt_int()))
        .unwrap_or((0, 0, None));

    let first_time = previous.is_none();
    let xp_gained = xp_for_lesson(u.correct, u.total, first_time);
    let perfect = u.total > 0 && u.correct >= u.total;
    let newly_perfect = perfect && previous.map_or(true, |p| (p as u32) < u.total);
    let s = record_activity(Streak { current: streak as u32, longest: longest as u32 }, last_day, u.today);

    db.run(
        vec![
            Stmt::new(
                "INSERT INTO completions (user_id, lesson_id, best_correct, completed_at) VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(user_id, lesson_id) DO UPDATE SET
                    best_correct = MAX(best_correct, excluded.best_correct),
                    completed_at = excluded.completed_at",
                args![u.user_id, u.lesson_id, u.correct, u.now],
            ),
            Stmt::new(
                "INSERT INTO xp_events (user_id, amount, created_at) VALUES (?1, ?2, ?3)",
                args![u.user_id, xp_gained, u.now],
            ),
            Stmt::new(
                "UPDATE users SET xp = xp + ?2, streak = ?3, longest_streak = ?4, last_active_day = ?5,
                        tz_offset = ?6, perfect_lessons = perfect_lessons + ?7
                 WHERE id = ?1",
                args![u.user_id, xp_gained, s.current, s.longest, u.today, u.tz_offset, newly_perfect],
            ),
        ],
        true,
    )
    .await?;
    Ok(CompletionResult { first_time, xp_gained })
}

/// (username, xp) ranked by XP earned since `since`.
pub async fn leaderboard_since(db: &Db, since: i64, limit: u32) -> DbResult<Vec<(String, i64)>> {
    Ok(db
        .query(
            "SELECT u.username, SUM(e.amount) AS total
             FROM xp_events e JOIN users u ON u.id = e.user_id
             WHERE e.created_at >= ?1
             GROUP BY u.id
             ORDER BY total DESC, u.username ASC
             LIMIT ?2",
            args![since, limit],
        )
        .await?
        .into_iter()
        .map(|r| (r[0].text(), r[1].int()))
        .collect())
}

/// (username, xp) ranked by total XP.
pub async fn leaderboard_all_time(db: &Db, limit: u32) -> DbResult<Vec<(String, i64)>> {
    Ok(db
        .query(
            "SELECT username, xp FROM users WHERE xp > 0 ORDER BY xp DESC, username ASC LIMIT ?1",
            args![limit],
        )
        .await?
        .into_iter()
        .map(|r| (r[0].text(), r[1].int()))
        .collect())
}

pub async fn get_avatar(db: &Db, user_id: i64) -> DbResult<Option<String>> {
    Ok(db
        .query_one("SELECT data FROM avatars WHERE user_id = ?1", args![user_id])
        .await?
        .and_then(|r| r[0].opt_text()))
}

pub async fn set_avatar(db: &Db, user_id: i64, avatar: Option<&str>) -> DbResult<()> {
    match avatar {
        Some(data) => {
            db.execute(
                "INSERT INTO avatars (user_id, data) VALUES (?1, ?2)
                 ON CONFLICT(user_id) DO UPDATE SET data = excluded.data",
                args![user_id, data],
            )
            .await?
        }
        None => db.execute("DELETE FROM avatars WHERE user_id = ?1", args![user_id]).await?,
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn mem() -> Db {
        let db = Db::memory().unwrap();
        migrate(&db).await.unwrap();
        db
    }

    #[tokio::test]
    async fn users_sessions_and_completions() {
        let c = mem().await;
        let id = create_user(&c, "ada", "hash", 100).await.unwrap().unwrap();
        assert!(create_user(&c, "ADA", "hash", 100).await.unwrap().is_none(), "usernames are case-insensitive");
        create_session(&c, "tok", id, 100).await.unwrap();
        assert_eq!(user_for_token(&c, "tok").await.unwrap(), Some(id));

        let upd = |correct, today| CompletionUpdate {
            user_id: id, lesson_id: "py-1-1", correct, total: 3, now: 1000, tz_offset: 120, today,
        };
        let r1 = record_completion(&c, upd(2, 10)).await.unwrap();
        assert!(r1.first_time);
        assert_eq!(r1.xp_gained, 14);
        let r2 = record_completion(&c, upd(3, 11)).await.unwrap();
        assert!(!r2.first_time);

        let u = get_user(&c, id).await.unwrap().unwrap();
        assert_eq!(u.xp, 14 + 5);
        assert_eq!(u.streak, 2);
        assert_eq!(u.perfect_lessons, 1);
        assert_eq!(completions(&c, id).await.unwrap()["py-1-1"], 3);
        assert_eq!(leaderboard_since(&c, 0, 10).await.unwrap(), vec![("ada".to_string(), 19)]);

        assert_eq!(get_avatar(&c, id).await.unwrap(), None);
        set_avatar(&c, id, Some("preset:a1")).await.unwrap();
        set_avatar(&c, id, Some("preset:a2")).await.unwrap();
        assert_eq!(get_avatar(&c, id).await.unwrap().as_deref(), Some("preset:a2"));
        set_avatar(&c, id, None).await.unwrap();
        assert_eq!(get_avatar(&c, id).await.unwrap(), None);

        delete_session(&c, "tok").await.unwrap();
        assert_eq!(user_for_token(&c, "tok").await.unwrap(), None);
    }

    #[tokio::test]
    async fn atomic_batches_roll_back() {
        let c = mem().await;
        let id = create_user(&c, "bob", "hash", 1).await.unwrap().unwrap();
        let bad = c
            .run(
                vec![
                    Stmt::new("UPDATE users SET xp = 50 WHERE id = ?1", args![id]),
                    Stmt::new("INSERT INTO no_such_table VALUES (1)", args![]),
                ],
                true,
            )
            .await;
        assert!(bad.is_err());
        assert_eq!(get_user(&c, id).await.unwrap().unwrap().xp, 0, "the first update was rolled back");
    }
}

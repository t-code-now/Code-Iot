//! SQLite storage. All functions take a `&Connection` (or `&mut` for
//! transactions); the caller holds the mutex.

use std::collections::HashMap;

use rusqlite::{params, Connection, ErrorCode, OptionalExtension};

const SCHEMA: &str = "
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS users (
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
);

CREATE TABLE IF NOT EXISTS sessions (
    token       TEXT    PRIMARY KEY,
    user_id     INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at  INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS completions (
    user_id       INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    lesson_id     TEXT    NOT NULL,
    best_correct  INTEGER NOT NULL,
    completed_at  INTEGER NOT NULL,
    PRIMARY KEY (user_id, lesson_id)
);

CREATE TABLE IF NOT EXISTS xp_events (
    id          INTEGER PRIMARY KEY,
    user_id     INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    amount      INTEGER NOT NULL,
    created_at  INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS xp_events_by_time ON xp_events(created_at);
";

pub fn open(path: &str) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(SCHEMA)?;
    Ok(conn)
}

#[derive(Debug, Clone)]
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

fn user_from_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<User> {
    Ok(User {
        id: r.get(0)?,
        username: r.get(1)?,
        created_at: r.get(2)?,
        xp: r.get(3)?,
        streak: r.get(4)?,
        longest_streak: r.get(5)?,
        last_active_day: r.get(6)?,
        tz_offset: r.get(7)?,
        perfect_lessons: r.get(8)?,
    })
}

/// Returns `Ok(None)` when the username is taken.
pub fn create_user(conn: &Connection, username: &str, hash: &str, now: i64) -> rusqlite::Result<Option<i64>> {
    match conn.execute(
        "INSERT INTO users (username, password_hash, created_at) VALUES (?1, ?2, ?3)",
        params![username, hash, now],
    ) {
        Ok(_) => Ok(Some(conn.last_insert_rowid())),
        Err(rusqlite::Error::SqliteFailure(e, _)) if e.code == ErrorCode::ConstraintViolation => Ok(None),
        Err(e) => Err(e),
    }
}

/// (user id, password hash) for a username.
pub fn credentials(conn: &Connection, username: &str) -> rusqlite::Result<Option<(i64, String)>> {
    conn.query_row(
        "SELECT id, password_hash FROM users WHERE username = ?1",
        params![username],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .optional()
}

pub fn get_user(conn: &Connection, id: i64) -> rusqlite::Result<Option<User>> {
    conn.query_row(&format!("SELECT {USER_COLUMNS} FROM users WHERE id = ?1"), params![id], user_from_row)
        .optional()
}

pub fn create_session(conn: &Connection, token: &str, user_id: i64, now: i64) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO sessions (token, user_id, created_at) VALUES (?1, ?2, ?3)",
        params![token, user_id, now],
    )?;
    Ok(())
}

pub fn user_for_token(conn: &Connection, token: &str) -> rusqlite::Result<Option<i64>> {
    conn.query_row("SELECT user_id FROM sessions WHERE token = ?1", params![token], |r| r.get(0))
        .optional()
}

pub fn delete_session(conn: &Connection, token: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM sessions WHERE token = ?1", params![token])?;
    Ok(())
}

/// lesson id → best first-try correct count.
pub fn completions(conn: &Connection, user_id: i64) -> rusqlite::Result<HashMap<String, u32>> {
    let mut stmt = conn.prepare("SELECT lesson_id, best_correct FROM completions WHERE user_id = ?1")?;
    let rows = stmt.query_map(params![user_id], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? as u32)))?;
    rows.collect()
}

pub fn xp_since(conn: &Connection, user_id: i64, since: i64) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COALESCE(SUM(amount), 0) FROM xp_events WHERE user_id = ?1 AND created_at >= ?2",
        params![user_id, since],
        |r| r.get(0),
    )
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
/// perfect-lesson count, all in one transaction.
pub fn record_completion(conn: &mut Connection, u: CompletionUpdate<'_>) -> rusqlite::Result<CompletionResult> {
    use codesprout_core::progress::{record_activity, xp_for_lesson, Streak};

    let tx = conn.transaction()?;

    let previous: Option<i64> = tx
        .query_row(
            "SELECT best_correct FROM completions WHERE user_id = ?1 AND lesson_id = ?2",
            params![u.user_id, u.lesson_id],
            |r| r.get(0),
        )
        .optional()?;
    let first_time = previous.is_none();
    let xp_gained = xp_for_lesson(u.correct, u.total, first_time);
    let perfect = u.total > 0 && u.correct >= u.total;
    let newly_perfect = perfect && previous.map_or(true, |p| (p as u32) < u.total);

    tx.execute(
        "INSERT INTO completions (user_id, lesson_id, best_correct, completed_at) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(user_id, lesson_id) DO UPDATE SET
            best_correct = MAX(best_correct, excluded.best_correct),
            completed_at = excluded.completed_at",
        params![u.user_id, u.lesson_id, u.correct, u.now],
    )?;
    tx.execute(
        "INSERT INTO xp_events (user_id, amount, created_at) VALUES (?1, ?2, ?3)",
        params![u.user_id, xp_gained, u.now],
    )?;

    let (streak, longest, last_day): (i64, i64, Option<i64>) = tx.query_row(
        "SELECT streak, longest_streak, last_active_day FROM users WHERE id = ?1",
        params![u.user_id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    let s = record_activity(
        Streak { current: streak as u32, longest: longest as u32 },
        last_day,
        u.today,
    );

    tx.execute(
        "UPDATE users SET xp = xp + ?2, streak = ?3, longest_streak = ?4, last_active_day = ?5,
                tz_offset = ?6, perfect_lessons = perfect_lessons + ?7
         WHERE id = ?1",
        params![u.user_id, xp_gained, s.current, s.longest, u.today, u.tz_offset, newly_perfect as i64],
    )?;

    tx.commit()?;
    Ok(CompletionResult { first_time, xp_gained })
}

/// (username, xp) ranked by XP earned since `since`.
pub fn leaderboard_since(conn: &Connection, since: i64, limit: u32) -> rusqlite::Result<Vec<(String, i64)>> {
    let mut stmt = conn.prepare(
        "SELECT u.username, SUM(e.amount) AS total
         FROM xp_events e JOIN users u ON u.id = e.user_id
         WHERE e.created_at >= ?1
         GROUP BY u.id
         ORDER BY total DESC, u.username ASC
         LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![since, limit], |r| Ok((r.get(0)?, r.get(1)?)))?;
    rows.collect()
}

/// (username, xp) ranked by total XP.
pub fn leaderboard_all_time(conn: &Connection, limit: u32) -> rusqlite::Result<Vec<(String, i64)>> {
    let mut stmt = conn.prepare(
        "SELECT username, xp FROM users WHERE xp > 0 ORDER BY xp DESC, username ASC LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit], |r| Ok((r.get(0)?, r.get(1)?)))?;
    rows.collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mem() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        c.execute_batch(SCHEMA).unwrap();
        c
    }

    #[test]
    fn users_sessions_and_completions() {
        let mut c = mem();
        let id = create_user(&c, "ada", "hash", 100).unwrap().unwrap();
        assert!(create_user(&c, "ADA", "hash", 100).unwrap().is_none(), "usernames are case-insensitive");
        create_session(&c, "tok", id, 100).unwrap();
        assert_eq!(user_for_token(&c, "tok").unwrap(), Some(id));

        let upd = |correct, today| CompletionUpdate {
            user_id: id, lesson_id: "py-1-1", correct, total: 3, now: 1000, tz_offset: 120, today,
        };
        let r1 = record_completion(&mut c, upd(2, 10)).unwrap();
        assert!(r1.first_time);
        assert_eq!(r1.xp_gained, 14);
        let r2 = record_completion(&mut c, upd(3, 11)).unwrap();
        assert!(!r2.first_time);

        let u = get_user(&c, id).unwrap().unwrap();
        assert_eq!(u.xp, 14 + 5);
        assert_eq!(u.streak, 2);
        assert_eq!(u.perfect_lessons, 1);
        assert_eq!(completions(&c, id).unwrap()["py-1-1"], 3);
        assert_eq!(leaderboard_since(&c, 0, 10).unwrap(), vec![("ada".to_string(), 19)]);

        delete_session(&c, "tok").unwrap();
        assert_eq!(user_for_token(&c, "tok").unwrap(), None);
    }
}

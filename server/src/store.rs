//! Storage for paid access, the admin's manual settings, suspended accounts
//! and the admin log.

use std::collections::HashMap;

use crate::args;
use crate::sql::{Db, DbResult, Stmt, Value};

pub const SCHEMA: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS payments (
        id            INTEGER PRIMARY KEY,
        provider      TEXT    NOT NULL,
        provider_ref  TEXT    NOT NULL,
        user_id       INTEGER,
        course_id     TEXT    NOT NULL,
        plan          TEXT    NOT NULL,
        status        TEXT    NOT NULL,
        amount_cents  INTEGER NOT NULL DEFAULT 0,
        currency      TEXT    NOT NULL DEFAULT 'USD',
        method        TEXT    NOT NULL DEFAULT '',
        email         TEXT    NOT NULL DEFAULT '',
        until         INTEGER,
        manage_url    TEXT    NOT NULL DEFAULT '',
        test_mode     INTEGER NOT NULL DEFAULT 0,
        created_at    INTEGER NOT NULL,
        updated_at    INTEGER NOT NULL,
        UNIQUE (provider, provider_ref)
    )",
    "CREATE INDEX IF NOT EXISTS payments_by_user ON payments(user_id)",
    "CREATE TABLE IF NOT EXISTS access_overrides (
        user_id     INTEGER NOT NULL,
        course_id   TEXT    NOT NULL,
        mode        TEXT    NOT NULL,
        created_at  INTEGER NOT NULL,
        PRIMARY KEY (user_id, course_id)
    )",
    "CREATE TABLE IF NOT EXISTS bans (
        user_id     INTEGER PRIMARY KEY,
        created_at  INTEGER NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS admin_log (
        id          INTEGER PRIMARY KEY,
        admin       TEXT    NOT NULL,
        action      TEXT    NOT NULL,
        target      TEXT    NOT NULL,
        detail      TEXT    NOT NULL,
        created_at  INTEGER NOT NULL
    )",
];

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Payment {
    pub id: i64,
    pub provider: String,
    pub provider_ref: String,
    pub user_id: Option<i64>,
    pub course_id: String,
    /// "lifetime" or "monthly".
    pub plan: String,
    pub status: String,
    pub amount_cents: i64,
    pub currency: String,
    pub method: String,
    pub email: String,
    pub until: Option<i64>,
    pub manage_url: String,
    pub test_mode: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Payment {
    /// Does this payment open its course right now?
    pub fn active(&self, now: i64) -> bool {
        match self.plan.as_str() {
            "lifetime" => matches!(self.status.as_str(), "paid" | "partial_refund"),
            "monthly" => {
                matches!(self.status.as_str(), "active" | "on_trial" | "past_due" | "cancelled")
                    && self.until.map_or(true, |u| u > now)
            }
            _ => false,
        }
    }
}

const PAYMENT_COLUMNS: &str = "id, provider, provider_ref, user_id, course_id, plan, status, amount_cents, currency, \
     method, email, until, manage_url, test_mode, created_at, updated_at";

fn payment_from_row(r: &[Value]) -> Payment {
    Payment {
        id: r[0].int(),
        provider: r[1].text(),
        provider_ref: r[2].text(),
        user_id: r[3].opt_int(),
        course_id: r[4].text(),
        plan: r[5].text(),
        status: r[6].text(),
        amount_cents: r[7].int(),
        currency: r[8].text(),
        method: r[9].text(),
        email: r[10].text(),
        until: r[11].opt_int(),
        manage_url: r[12].text(),
        test_mode: r[13].int() != 0,
        created_at: r[14].int(),
        updated_at: r[15].int(),
    }
}

/// Inserts or updates a payment by (provider, provider reference).
pub async fn upsert_payment(db: &Db, p: &Payment) -> DbResult<()> {
    db.execute(
        "INSERT INTO payments (provider, provider_ref, user_id, course_id, plan, status, amount_cents, currency,
                               method, email, until, manage_url, test_mode, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?14)
         ON CONFLICT(provider, provider_ref) DO UPDATE SET
            user_id = COALESCE(excluded.user_id, payments.user_id),
            course_id = CASE WHEN excluded.course_id = '' THEN payments.course_id ELSE excluded.course_id END,
            plan = excluded.plan,
            status = excluded.status,
            amount_cents = CASE WHEN excluded.amount_cents = 0 THEN payments.amount_cents ELSE excluded.amount_cents END,
            currency = excluded.currency,
            method = CASE WHEN excluded.method = '' THEN payments.method ELSE excluded.method END,
            email = CASE WHEN excluded.email = '' THEN payments.email ELSE excluded.email END,
            until = excluded.until,
            manage_url = CASE WHEN excluded.manage_url = '' THEN payments.manage_url ELSE excluded.manage_url END,
            test_mode = excluded.test_mode,
            updated_at = excluded.updated_at",
        args![
            p.provider.as_str(),
            p.provider_ref.as_str(),
            p.user_id,
            p.course_id.as_str(),
            p.plan.as_str(),
            p.status.as_str(),
            p.amount_cents,
            p.currency.as_str(),
            p.method.as_str(),
            p.email.as_str(),
            p.until,
            p.manage_url.as_str(),
            p.test_mode,
            p.created_at,
        ],
    )
    .await?;
    Ok(())
}

pub async fn payments_for_user(db: &Db, user_id: i64) -> DbResult<Vec<Payment>> {
    Ok(db
        .query(
            &format!("SELECT {PAYMENT_COLUMNS} FROM payments WHERE user_id = ?1 ORDER BY created_at"),
            args![user_id],
        )
        .await?
        .iter()
        .map(|r| payment_from_row(r))
        .collect())
}

pub async fn all_payments(db: &Db, limit: u32) -> DbResult<Vec<Payment>> {
    Ok(db
        .query(&format!("SELECT {PAYMENT_COLUMNS} FROM payments ORDER BY created_at DESC LIMIT ?1"), args![limit])
        .await?
        .iter()
        .map(|r| payment_from_row(r))
        .collect())
}

/// course id → "open" or "block".
pub async fn overrides(db: &Db, user_id: i64) -> DbResult<HashMap<String, String>> {
    Ok(db
        .query("SELECT course_id, mode FROM access_overrides WHERE user_id = ?1", args![user_id])
        .await?
        .into_iter()
        .map(|r| (r[0].text(), r[1].text()))
        .collect())
}

pub async fn all_overrides(db: &Db) -> DbResult<Vec<(i64, String, String)>> {
    Ok(db
        .query("SELECT user_id, course_id, mode FROM access_overrides", args![])
        .await?
        .into_iter()
        .map(|r| (r[0].int(), r[1].text(), r[2].text()))
        .collect())
}

/// `mode` "open" or "block"; None clears the manual setting.
pub async fn set_override(db: &Db, user_id: i64, course_id: &str, mode: Option<&str>, now: i64) -> DbResult<()> {
    match mode {
        Some(m) => {
            db.execute(
                "INSERT INTO access_overrides (user_id, course_id, mode, created_at) VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(user_id, course_id) DO UPDATE SET mode = excluded.mode, created_at = excluded.created_at",
                args![user_id, course_id, m, now],
            )
            .await?
        }
        None => {
            db.execute("DELETE FROM access_overrides WHERE user_id = ?1 AND course_id = ?2", args![user_id, course_id])
                .await?
        }
    };
    Ok(())
}

pub async fn is_banned(db: &Db, user_id: i64) -> DbResult<bool> {
    Ok(db.query_one("SELECT 1 FROM bans WHERE user_id = ?1", args![user_id]).await?.is_some())
}

/// Suspending an account also signs it out everywhere.
pub async fn set_ban(db: &Db, user_id: i64, banned: bool, now: i64) -> DbResult<()> {
    if banned {
        db.run(
            vec![
                Stmt::new(
                    "INSERT INTO bans (user_id, created_at) VALUES (?1, ?2) ON CONFLICT(user_id) DO NOTHING",
                    args![user_id, now],
                ),
                Stmt::new("DELETE FROM sessions WHERE user_id = ?1", args![user_id]),
            ],
            true,
        )
        .await?;
    } else {
        db.execute("DELETE FROM bans WHERE user_id = ?1", args![user_id]).await?;
    }
    Ok(())
}

/// The verified Google e-mail of an account, if it signs in with Google.
pub async fn user_email(db: &Db, user_id: i64) -> DbResult<Option<String>> {
    Ok(db
        .query_one("SELECT email FROM google_accounts WHERE user_id = ?1 AND email <> '' LIMIT 1", args![user_id])
        .await?
        .map(|r| r[0].text()))
}

pub async fn log(db: &Db, admin: &str, action: &str, target: &str, detail: &str, now: i64) -> DbResult<()> {
    db.execute(
        "INSERT INTO admin_log (admin, action, target, detail, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        args![admin, action, target, detail, now],
    )
    .await?;
    Ok(())
}

/// (at, admin, action, target, detail), newest first.
pub async fn recent_log(db: &Db, limit: u32) -> DbResult<Vec<(i64, String, String, String, String)>> {
    Ok(db
        .query(
            "SELECT created_at, admin, action, target, detail FROM admin_log ORDER BY id DESC LIMIT ?1",
            args![limit],
        )
        .await?
        .into_iter()
        .map(|r| (r[0].int(), r[1].text(), r[2].text(), r[3].text(), r[4].text()))
        .collect())
}

/// Removes an account and everything tied to it. Payment records are kept
/// (for accounting) but detached from the account.
pub async fn delete_user(db: &Db, user_id: i64) -> DbResult<()> {
    let tables = ["sessions", "completions", "xp_events", "google_accounts", "avatars", "access_overrides", "bans"];
    let mut stmts: Vec<Stmt> = tables
        .iter()
        .map(|t| Stmt::new(format!("DELETE FROM {t} WHERE user_id = ?1"), args![user_id]))
        .collect();
    stmts.push(Stmt::new("UPDATE payments SET user_id = NULL WHERE user_id = ?1", args![user_id]));
    stmts.push(Stmt::new("DELETE FROM users WHERE id = ?1", args![user_id]));
    db.run(stmts, true).await?;
    Ok(())
}

/// One row per account for the admin page.
#[derive(Debug, Clone)]
pub struct UserRow {
    pub id: i64,
    pub username: String,
    pub created_at: i64,
    pub xp: i64,
    pub last_active_day: Option<i64>,
    pub email: String,
    pub google: bool,
    pub lessons: i64,
    pub banned: bool,
}

/// Accounts, newest first; `search` matches the username or e-mail.
pub async fn user_rows(db: &Db, search: &str, limit: u32) -> DbResult<Vec<UserRow>> {
    let pattern = format!("%{}%", search.trim().to_lowercase());
    Ok(db
        .query(
            "SELECT u.id, u.username, u.created_at, u.xp, u.last_active_day,
                    COALESCE(g.email, ''), g.user_id IS NOT NULL,
                    (SELECT COUNT(*) FROM completions c WHERE c.user_id = u.id),
                    b.user_id IS NOT NULL
             FROM users u
             LEFT JOIN google_accounts g ON g.user_id = u.id
             LEFT JOIN bans b ON b.user_id = u.id
             WHERE ?1 = '%%' OR LOWER(u.username) LIKE ?1 OR LOWER(COALESCE(g.email, '')) LIKE ?1
             ORDER BY u.id DESC
             LIMIT ?2",
            args![pattern, limit],
        )
        .await?
        .into_iter()
        .map(|r| UserRow {
            id: r[0].int(),
            username: r[1].text(),
            created_at: r[2].int(),
            xp: r[3].int(),
            last_active_day: r[4].opt_int(),
            email: r[5].text(),
            google: r[6].int() != 0,
            lessons: r[7].int(),
            banned: r[8].int() != 0,
        })
        .collect())
}

/// Simple counters for the admin overview.
pub async fn counts(db: &Db, since: i64, since_day: i64) -> DbResult<HashMap<&'static str, i64>> {
    let queries: [(&'static str, &str, Vec<Value>); 6] = [
        ("users", "SELECT COUNT(*) FROM users", args![]),
        ("google_users", "SELECT COUNT(*) FROM google_accounts", args![]),
        ("new_users", "SELECT COUNT(*) FROM users WHERE created_at >= ?1", args![since]),
        ("active_users", "SELECT COUNT(*) FROM users WHERE last_active_day >= ?1", args![since_day]),
        ("lessons", "SELECT COUNT(*) FROM xp_events WHERE created_at >= ?1", args![since]),
        ("banned", "SELECT COUNT(*) FROM bans", args![]),
    ];
    let names: Vec<&'static str> = queries.iter().map(|q| q.0).collect();
    let results = db.run(queries.into_iter().map(|(_, sql, a)| Stmt::new(sql, a)).collect(), false).await?;
    Ok(names
        .into_iter()
        .zip(results)
        .map(|(n, r)| (n, r.rows.first().map_or(0, |row| row[0].int())))
        .collect())
}

/// course id → number of users who finished at least one of these lessons.
pub async fn learners_per_lesson_prefix(db: &Db) -> DbResult<Vec<(String, i64)>> {
    Ok(db
        .query("SELECT lesson_id, user_id FROM completions", args![])
        .await?
        .into_iter()
        .map(|r| (r[0].text(), r[1].int()))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    #[tokio::test]
    async fn payments_overrides_bans() {
        let c = Db::memory().unwrap();
        db::migrate(&c).await.unwrap();
        let id = db::create_user(&c, "ann", "!google", 1).await.unwrap().unwrap();
        db::link_google(&c, "g1", id, "ann@example.com").await.unwrap();

        let mut p = Payment {
            provider: "lemonsqueezy".into(),
            provider_ref: "ord_1".into(),
            user_id: Some(id),
            course_id: "python".into(),
            plan: "lifetime".into(),
            status: "paid".into(),
            amount_cents: 2500,
            currency: "USD".into(),
            created_at: 10,
            updated_at: 10,
            ..Default::default()
        };
        upsert_payment(&c, &p).await.unwrap();
        p.status = "refunded".into();
        p.amount_cents = 0; // keeps the stored amount
        upsert_payment(&c, &p).await.unwrap();
        let got = payments_for_user(&c, id).await.unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].status, "refunded");
        assert_eq!(got[0].amount_cents, 2500);
        assert!(!got[0].active(100));

        let sub = Payment { plan: "monthly".into(), status: "cancelled".into(), until: Some(500), ..Default::default() };
        assert!(sub.active(400));
        assert!(!sub.active(600));

        set_override(&c, id, "rust", Some("open"), 5).await.unwrap();
        set_override(&c, id, "rust", Some("block"), 6).await.unwrap();
        assert_eq!(overrides(&c, id).await.unwrap()["rust"], "block");
        set_override(&c, id, "rust", None, 7).await.unwrap();
        assert!(overrides(&c, id).await.unwrap().is_empty());

        db::create_session(&c, "t", id, 1).await.unwrap();
        set_ban(&c, id, true, 9).await.unwrap();
        assert!(is_banned(&c, id).await.unwrap());
        assert_eq!(db::user_for_token(&c, "t").await.unwrap(), None, "suspending signs the user out");

        assert_eq!(user_email(&c, id).await.unwrap().as_deref(), Some("ann@example.com"));
        let rows = user_rows(&c, "ANN@", 10).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert!(rows[0].banned && rows[0].google);

        log(&c, "admin@x", "ban", "ann", "", 11).await.unwrap();
        assert_eq!(recent_log(&c, 5).await.unwrap().len(), 1);
        let n = counts(&c, 0, 0).await.unwrap();
        assert_eq!(n["users"], 1);

        delete_user(&c, id).await.unwrap();
        assert!(db::get_user(&c, id).await.unwrap().is_none());
        assert_eq!(all_payments(&c, 10).await.unwrap()[0].user_id, None, "payment kept, detached");
    }
}

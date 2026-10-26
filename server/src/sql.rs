//! A small SQL layer with two engines:
//!
//! * **Remote** – a Turso / libSQL database reached over HTTPS (the "Hrana
//!   over HTTP" protocol, `POST /v2/pipeline`). This is what production
//!   uses, so accounts and progress survive restarts and redeploys.
//! * **Local** – an SQLite file through rusqlite, for development and tests.
//!
//! Both run the same SQL, so the queries in `db.rs` are written once.

use std::sync::Mutex;
use std::time::Duration;

use serde_json::{json, Value as Json};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Int(i64),
    Real(f64),
    Text(String),
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::Int(v)
    }
}
impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Value::Int(v as i64)
    }
}
impl From<u32> for Value {
    fn from(v: u32) -> Self {
        Value::Int(v as i64)
    }
}
impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Int(v as i64)
    }
}
impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::Text(v.to_string())
    }
}
impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::Text(v)
    }
}
impl<T: Into<Value>> From<Option<T>> for Value {
    fn from(v: Option<T>) -> Self {
        v.map_or(Value::Null, Into::into)
    }
}

impl Value {
    pub fn int(&self) -> i64 {
        match self {
            Value::Int(i) => *i,
            Value::Real(f) => *f as i64,
            Value::Text(t) => t.parse().unwrap_or(0),
            Value::Null => 0,
        }
    }
    pub fn opt_int(&self) -> Option<i64> {
        match self {
            Value::Null => None,
            v => Some(v.int()),
        }
    }
    pub fn text(&self) -> String {
        match self {
            Value::Text(t) => t.clone(),
            Value::Int(i) => i.to_string(),
            Value::Real(f) => f.to_string(),
            Value::Null => String::new(),
        }
    }
    pub fn opt_text(&self) -> Option<String> {
        match self {
            Value::Null => None,
            v => Some(v.text()),
        }
    }
}

/// `args![a, b, c]` builds the positional arguments of a statement.
#[macro_export]
macro_rules! args {
    ($($v:expr),* $(,)?) => { vec![$($crate::sql::Value::from($v)),*] };
}

#[derive(Debug, Clone)]
pub struct Stmt {
    pub sql: String,
    pub args: Vec<Value>,
}

impl Stmt {
    pub fn new(sql: impl Into<String>, args: Vec<Value>) -> Self {
        Stmt { sql: sql.into(), args }
    }
}

#[derive(Debug, Clone, Default)]
pub struct QueryResult {
    pub rows: Vec<Vec<Value>>,
    pub last_insert_rowid: Option<i64>,
    pub affected: u64,
}

#[derive(Debug, Clone)]
pub struct DbError(pub String);

impl DbError {
    pub fn is_unique_violation(&self) -> bool {
        self.0.contains("UNIQUE constraint failed")
    }
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for DbError {}

impl From<rusqlite::Error> for DbError {
    fn from(e: rusqlite::Error) -> Self {
        DbError(e.to_string())
    }
}

pub type DbResult<T> = Result<T, DbError>;

pub struct Remote {
    url: String,
    token: String,
    http: reqwest::Client,
}

pub enum Db {
    Local(Mutex<rusqlite::Connection>),
    Remote(Remote),
}

impl Db {
    pub fn local(path: &str) -> DbResult<Db> {
        Ok(Db::Local(Mutex::new(rusqlite::Connection::open(path)?)))
    }

    pub fn memory() -> DbResult<Db> {
        Ok(Db::Local(Mutex::new(rusqlite::Connection::open_in_memory()?)))
    }

    /// `url` as Turso shows it (`libsql://name-org.turso.io`) or https://.
    pub fn remote(url: &str, token: &str) -> DbResult<Db> {
        let url = url.trim().trim_end_matches('/');
        let url = match url.strip_prefix("libsql://") {
            Some(rest) => format!("https://{rest}"),
            None => url.to_string(),
        };
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .build()
            .map_err(|e| DbError(e.to_string()))?;
        Ok(Db::Remote(Remote { url, token: token.trim().to_string(), http }))
    }

    pub fn describe(&self) -> String {
        match self {
            Db::Local(_) => "local SQLite file".into(),
            Db::Remote(r) => format!("Turso ({})", r.url),
        }
    }

    /// Runs statements in order. With `atomic`, they all succeed or none
    /// are applied.
    pub async fn run(&self, stmts: Vec<Stmt>, atomic: bool) -> DbResult<Vec<QueryResult>> {
        match self {
            Db::Local(conn) => {
                let mut conn = conn.lock().map_err(|_| DbError("database lock poisoned".into()))?;
                local_run(&mut conn, &stmts, atomic)
            }
            Db::Remote(r) => r.run(&stmts, atomic).await,
        }
    }

    pub async fn execute(&self, sql: &str, args: Vec<Value>) -> DbResult<QueryResult> {
        let mut out = self.run(vec![Stmt::new(sql, args)], false).await?;
        Ok(out.pop().unwrap_or_default())
    }

    pub async fn query(&self, sql: &str, args: Vec<Value>) -> DbResult<Vec<Vec<Value>>> {
        Ok(self.execute(sql, args).await?.rows)
    }

    pub async fn query_one(&self, sql: &str, args: Vec<Value>) -> DbResult<Option<Vec<Value>>> {
        Ok(self.query(sql, args).await?.into_iter().next())
    }
}

// ------------------------------------------------------------------ local

fn to_sql(v: &Value) -> rusqlite::types::Value {
    match v {
        Value::Null => rusqlite::types::Value::Null,
        Value::Int(i) => rusqlite::types::Value::Integer(*i),
        Value::Real(f) => rusqlite::types::Value::Real(*f),
        Value::Text(t) => rusqlite::types::Value::Text(t.clone()),
    }
}

fn from_sql(v: rusqlite::types::ValueRef<'_>) -> Value {
    use rusqlite::types::ValueRef;
    match v {
        ValueRef::Null => Value::Null,
        ValueRef::Integer(i) => Value::Int(i),
        ValueRef::Real(f) => Value::Real(f),
        ValueRef::Text(t) => Value::Text(String::from_utf8_lossy(t).into_owned()),
        ValueRef::Blob(b) => Value::Text(String::from_utf8_lossy(b).into_owned()),
    }
}

fn local_one(conn: &rusqlite::Connection, s: &Stmt) -> DbResult<QueryResult> {
    let mut stmt = conn.prepare(&s.sql)?;
    let cols = stmt.column_count();
    let params: Vec<rusqlite::types::Value> = s.args.iter().map(to_sql).collect();
    let mut rows = stmt.query(rusqlite::params_from_iter(params.iter()))?;
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        let mut r = Vec::with_capacity(cols);
        for i in 0..cols {
            r.push(from_sql(row.get_ref(i)?));
        }
        out.push(r);
    }
    drop(rows);
    Ok(QueryResult { rows: out, last_insert_rowid: Some(conn.last_insert_rowid()), affected: conn.changes() as u64 })
}

fn local_run(conn: &mut rusqlite::Connection, stmts: &[Stmt], atomic: bool) -> DbResult<Vec<QueryResult>> {
    if atomic {
        let tx = conn.transaction()?;
        let mut out = Vec::new();
        for s in stmts {
            out.push(local_one(&tx, s)?);
        }
        tx.commit()?;
        Ok(out)
    } else {
        stmts.iter().map(|s| local_one(conn, s)).collect()
    }
}

// ------------------------------------------------------------------ remote

fn to_hrana(v: &Value) -> Json {
    match v {
        Value::Null => json!({ "type": "null" }),
        Value::Int(i) => json!({ "type": "integer", "value": i.to_string() }),
        Value::Real(f) => json!({ "type": "float", "value": f }),
        Value::Text(t) => json!({ "type": "text", "value": t }),
    }
}

fn from_hrana(v: &Json) -> Value {
    match v["type"].as_str().unwrap_or("null") {
        "integer" => Value::Int(
            v["value"].as_str().and_then(|s| s.parse().ok()).or_else(|| v["value"].as_i64()).unwrap_or(0),
        ),
        "float" => Value::Real(v["value"].as_f64().unwrap_or(0.0)),
        "text" => Value::Text(v["value"].as_str().unwrap_or("").to_string()),
        "blob" => Value::Text(v["base64"].as_str().unwrap_or("").to_string()),
        _ => Value::Null,
    }
}

fn hrana_stmt(s: &Stmt) -> Json {
    json!({ "sql": s.sql, "args": s.args.iter().map(to_hrana).collect::<Vec<_>>(), "want_rows": true })
}

fn hrana_result(r: &Json) -> QueryResult {
    QueryResult {
        rows: r["rows"]
            .as_array()
            .map(|rows| {
                rows.iter()
                    .map(|row| row.as_array().map(|c| c.iter().map(from_hrana).collect()).unwrap_or_default())
                    .collect()
            })
            .unwrap_or_default(),
        last_insert_rowid: r["last_insert_rowid"]
            .as_str()
            .and_then(|s| s.parse().ok())
            .or_else(|| r["last_insert_rowid"].as_i64()),
        affected: r["affected_row_count"].as_u64().unwrap_or(0),
    }
}

fn hrana_error(e: &Json) -> DbError {
    DbError(e["message"].as_str().unwrap_or("database error").to_string())
}

impl Remote {
    async fn pipeline(&self, requests: Vec<Json>) -> DbResult<Vec<Json>> {
        let body = json!({ "baton": null, "requests": requests });
        let mut last_err = DbError("database unreachable".into());
        // One retry: the first request after an idle period sometimes fails.
        for attempt in 0..2 {
            if attempt > 0 {
                tokio::time::sleep(Duration::from_millis(300)).await;
            }
            let resp = self
                .http
                .post(format!("{}/v2/pipeline", self.url))
                .bearer_auth(&self.token)
                .json(&body)
                .send()
                .await;
            match resp {
                Ok(r) if r.status().is_success() => {
                    let v: Json = r.json().await.map_err(|e| DbError(e.to_string()))?;
                    return Ok(v["results"].as_array().cloned().unwrap_or_default());
                }
                Ok(r) => {
                    let status = r.status();
                    let text = r.text().await.unwrap_or_default();
                    last_err = DbError(format!("database HTTP {status}: {}", text.chars().take(300).collect::<String>()));
                    if status.is_client_error() {
                        break; // bad token or bad request: retrying won't help
                    }
                }
                Err(e) => last_err = DbError(e.to_string()),
            }
        }
        Err(last_err)
    }

    async fn run(&self, stmts: &[Stmt], atomic: bool) -> DbResult<Vec<QueryResult>> {
        if !atomic {
            let mut requests: Vec<Json> =
                stmts.iter().map(|s| json!({ "type": "execute", "stmt": hrana_stmt(s) })).collect();
            requests.push(json!({ "type": "close" }));
            let results = self.pipeline(requests).await?;
            let mut out = Vec::with_capacity(stmts.len());
            for r in results.iter().take(stmts.len()) {
                match r["type"].as_str() {
                    Some("ok") => out.push(hrana_result(&r["response"]["result"])),
                    _ => return Err(hrana_error(&r["error"])),
                }
            }
            return Ok(out);
        }

        // Atomic: one batch. BEGIN, each statement only if the previous step
        // succeeded, COMMIT if the last one did, otherwise ROLLBACK.
        let n = stmts.len();
        let mut steps = vec![json!({ "stmt": { "sql": "BEGIN" } })];
        for (i, s) in stmts.iter().enumerate() {
            steps.push(json!({ "condition": { "type": "ok", "step": i }, "stmt": hrana_stmt(s) }));
        }
        steps.push(json!({ "condition": { "type": "ok", "step": n }, "stmt": { "sql": "COMMIT" } }));
        steps.push(json!({
            "condition": { "type": "not", "cond": { "type": "ok", "step": n + 1 } },
            "stmt": { "sql": "ROLLBACK" }
        }));
        let results = self
            .pipeline(vec![json!({ "type": "batch", "batch": { "steps": steps } }), json!({ "type": "close" })])
            .await?;
        let first = results.first().cloned().unwrap_or(Json::Null);
        if first["type"].as_str() != Some("ok") {
            return Err(hrana_error(&first["error"]));
        }
        let batch = &first["response"]["result"];
        let errors = batch["step_errors"].as_array().cloned().unwrap_or_default();
        if let Some(e) = errors.iter().take(n + 2).find(|e| !e.is_null()) {
            return Err(hrana_error(e));
        }
        let results = batch["step_results"].as_array().cloned().unwrap_or_default();
        Ok((1..=n).map(|i| results.get(i).map(hrana_result).unwrap_or_default()).collect())
    }
}

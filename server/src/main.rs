//! Code-Iot server: JSON API under /api plus the built web app.
//!
//! Environment variables (all optional):
//! * `ADDR`           – listen address, default `0.0.0.0:3000`
//! * `TURSO_DATABASE_URL`, `TURSO_AUTH_TOKEN` – keep all data in a Turso
//!   database (recommended for hosting: data survives restarts)
//! * `DATABASE_PATH`  – local SQLite file when Turso isn't set, default `codesprout.db`
//! * `WEB_DIST`       – built frontend folder, default `web/dist`
//! * `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET` – turn on "Continue with Google"
//! * `PUBLIC_URL`     – the site's address for Google's redirect (Render sets
//!   `RENDER_EXTERNAL_URL`, which is used when this is missing)

mod db;
mod error;
mod google;
mod routes;
mod sql;
mod store;
mod access;
mod payments;
mod admin;
mod run;
mod sketch;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

use crate::routes::{api_router, AppState};

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=info".into()),
        )
        .init();

    let db_path = env_or("DATABASE_PATH", "codesprout.db");
    let dist = PathBuf::from(env_or("WEB_DIST", "web/dist"));
    let addr: SocketAddr = env_or("ADDR", "0.0.0.0:3000").parse()?;

    let database = match (std::env::var("TURSO_DATABASE_URL"), std::env::var("TURSO_AUTH_TOKEN")) {
        (Ok(url), Ok(token)) if !url.trim().is_empty() && !token.trim().is_empty() => sql::Db::remote(&url, &token)?,
        _ => sql::Db::local(&db_path)?,
    };
    // Create the tables; with a remote database, retry while it wakes up.
    let mut attempt = 0;
    loop {
        match db::migrate(&database).await {
            Ok(()) => break,
            Err(e) if attempt < 4 => {
                attempt += 1;
                tracing::warn!("database not ready ({e}); retrying");
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
            Err(e) => return Err(e.into()),
        }
    }
    let db_label = database.describe();
    let state = AppState { db: Arc::new(database) };

    // Single-page app: unknown paths fall back to index.html so the
    // client-side router can handle them.
    let spa = ServeDir::new(&dist).fallback(ServeFile::new(dist.join("index.html")));

    // Paths under /api that match no route get a JSON 404, never the web page.
    let app = api_router(state)
        .fallback(move |req: axum::extract::Request| {
            let mut spa = spa.clone();
            async move {
                use axum::response::IntoResponse;
                if req.uri().path().starts_with("/api/") || req.uri().path() == "/api" {
                    return crate::error::ApiError::NotFound.into_response();
                }
                match spa.try_call(req).await {
                    Ok(resp) => {
                        let mut resp = resp.map(axum::body::Body::new).into_response();
                        // The page itself is always checked for a new version, so a
                        // phone never keeps an old copy after an update. (Its scripts
                        // and styles have versioned names and can be cached.)
                        let is_html = resp
                            .headers()
                            .get(axum::http::header::CONTENT_TYPE)
                            .and_then(|v| v.to_str().ok())
                            .is_some_and(|v| v.starts_with("text/html"));
                        if is_html {
                            resp.headers_mut().insert(
                                axum::http::header::CACHE_CONTROL,
                                axum::http::HeaderValue::from_static("no-cache"),
                            );
                        }
                        resp
                    }
                    Err(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response(),
                }
            }
        })
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    tracing::info!("Code-Iot listening on http://{addr} (db: {db_label}, web: {})", dist.display());
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use codesprout_core::api::ApiErrorBody;

#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    Unauthorized,
    NotFound,
    Conflict(String),
    Internal(String),
}

pub type ApiResult<T> = Result<T, ApiError>;

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            ApiError::BadRequest(m) => (StatusCode::BAD_REQUEST, m),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "Please log in".to_string()),
            ApiError::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
            ApiError::Conflict(m) => (StatusCode::CONFLICT, m),
            ApiError::Internal(m) => {
                tracing::error!("internal error: {m}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong".to_string())
            }
        };
        (status, Json(ApiErrorBody { error: msg })).into_response()
    }
}

impl From<crate::sql::DbError> for ApiError {
    fn from(e: crate::sql::DbError) -> Self {
        ApiError::Internal(e.to_string())
    }
}

impl From<tokio::task::JoinError> for ApiError {
    fn from(e: tokio::task::JoinError) -> Self {
        ApiError::Internal(e.to_string())
    }
}

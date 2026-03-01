use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error("rate limit exceeded")]
    RateLimited,
    #[error("queue is full")]
    QueueFull,
    #[error("execution not found")]
    NotFound,
    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for EngineError {
    fn into_response(self) -> Response {
        let status = match self {
            EngineError::Unauthorized => StatusCode::UNAUTHORIZED,
            EngineError::Forbidden => StatusCode::FORBIDDEN,
            EngineError::InvalidRequest(_) => StatusCode::BAD_REQUEST,
            EngineError::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            EngineError::QueueFull => StatusCode::SERVICE_UNAVAILABLE,
            EngineError::NotFound => StatusCode::NOT_FOUND,
            EngineError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = Json(ErrorBody {
            error: self.to_string(),
        });
        (status, body).into_response()
    }
}

impl From<anyhow::Error> for EngineError {
    fn from(value: anyhow::Error) -> Self {
        Self::Internal(value.to_string())
    }
}

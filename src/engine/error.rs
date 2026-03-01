use std::fmt::{Display, Formatter};

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Debug)]
pub enum EngineError {
    Unauthorized,
    Forbidden,
    InvalidRequest(String),
    RateLimited,
    QueueFull,
    NotFound,
    Internal(String),
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

impl Display for EngineError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineError::Unauthorized => write!(f, "unauthorized"),
            EngineError::Forbidden => write!(f, "forbidden"),
            EngineError::InvalidRequest(msg) => write!(f, "invalid request: {msg}"),
            EngineError::RateLimited => write!(f, "rate limit exceeded"),
            EngineError::QueueFull => write!(f, "queue is full"),
            EngineError::NotFound => write!(f, "execution not found"),
            EngineError::Internal(msg) => write!(f, "internal error: {msg}"),
        }
    }
}

impl std::error::Error for EngineError {}

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

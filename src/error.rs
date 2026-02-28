use axum::{
    Json,
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Serialize;

pub type GatewayResult<T> = Result<T, GatewayError>;

#[derive(Debug)]
pub enum GatewayError {
    Unauthorized,
    RateLimited { retry_after_secs: u64 },
    Validation(String),
    RouteNotFound,
    UpstreamUnavailable,
    Upstream(String),
    PayloadTooLarge,
    Internal(String),
}

#[derive(Debug, Serialize)]
struct ErrorBody<'a> {
    error: &'a str,
    message: String,
}

impl GatewayError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Unauthorized => "unauthorized",
            Self::RateLimited { .. } => "rate_limited",
            Self::Validation(_) => "validation_error",
            Self::RouteNotFound => "route_not_found",
            Self::UpstreamUnavailable => "upstream_unavailable",
            Self::Upstream(_) => "upstream_error",
            Self::PayloadTooLarge => "payload_too_large",
            Self::Internal(_) => "internal_error",
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::Unauthorized => "Invalid or missing API key".to_string(),
            Self::RateLimited { .. } => "Rate limit exceeded".to_string(),
            Self::Validation(msg) => msg.clone(),
            Self::RouteNotFound => "No route matched the request".to_string(),
            Self::UpstreamUnavailable => "No healthy upstream available".to_string(),
            Self::Upstream(msg) => msg.clone(),
            Self::PayloadTooLarge => "Request body exceeds configured limit".to_string(),
            Self::Internal(msg) => msg.clone(),
        }
    }

    pub fn status(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::RouteNotFound => StatusCode::NOT_FOUND,
            Self::UpstreamUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            Self::Upstream(_) => StatusCode::BAD_GATEWAY,
            Self::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for GatewayError {
    fn into_response(self) -> Response {
        let status = self.status();
        let mut response = (status, Json(ErrorBody {
            error: self.code(),
            message: self.message(),
        }))
            .into_response();

        if let Self::RateLimited { retry_after_secs } = self {
            if let Ok(v) = HeaderValue::from_str(&retry_after_secs.to_string()) {
                response.headers_mut().insert(header::RETRY_AFTER, v);
            }
        }

        if !response.headers().contains_key(header::CONTENT_TYPE) {
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            );
        }

        response
    }
}

impl From<anyhow::Error> for GatewayError {
    fn from(err: anyhow::Error) -> Self {
        Self::Internal(err.to_string())
    }
}

impl From<reqwest::Error> for GatewayError {
    fn from(err: reqwest::Error) -> Self {
        Self::Upstream(err.to_string())
    }
}

impl From<redis::RedisError> for GatewayError {
    fn from(err: redis::RedisError) -> Self {
        Self::Internal(err.to_string())
    }
}

use async_trait::async_trait;
use axum::{
    Json,
    body::Body,
    http::{HeaderName, HeaderValue, header},
    response::{IntoResponse, Response},
};
use serde::Serialize;

use crate::{
    context::RequestContext,
    error::{GatewayError, GatewayResult},
    middleware::{ControlFlow, GatewayMiddleware},
    ratelimit::RateLimiter,
};

#[derive(Serialize)]
struct RateLimitBody<'a> {
    error: &'a str,
    message: &'a str,
}

pub struct RateLimitMiddleware {
    limiter: RateLimiter,
    key_header: HeaderName,
    fail_open_on_error: bool,
}

impl RateLimitMiddleware {
    pub fn new(limiter: RateLimiter, key_header: String, fail_open_on_error: bool) -> Self {
        let key_header = HeaderName::from_bytes(key_header.as_bytes())
            .unwrap_or_else(|_| HeaderName::from_static("x-api-key"));

        Self {
            limiter,
            key_header,
            fail_open_on_error,
        }
    }

    fn resolve_key(&self, ctx: &RequestContext) -> String {
        if let Some(key) = ctx
            .headers
            .get(&self.key_header)
            .and_then(|v| v.to_str().ok())
            .filter(|v| !v.is_empty())
        {
            return key.to_string();
        }

        if let Some(ip) = ctx.client_ip {
            return ip.to_string();
        }

        "anonymous".to_string()
    }

    fn limited_response(&self, retry_after_secs: u64) -> Response<Body> {
        let mut response = (
            axum::http::StatusCode::TOO_MANY_REQUESTS,
            Json(RateLimitBody {
                error: "rate_limited",
                message: "Rate limit exceeded",
            }),
        )
            .into_response();

        if let Ok(value) = HeaderValue::from_str(&retry_after_secs.to_string()) {
            response.headers_mut().insert(header::RETRY_AFTER, value);
        }

        response
    }
}

#[async_trait]
impl GatewayMiddleware for RateLimitMiddleware {
    fn name(&self) -> &'static str {
        "rate-limit"
    }

    async fn on_request(&self, ctx: &mut RequestContext) -> GatewayResult<ControlFlow> {
        let key = self.resolve_key(ctx);
        let scope = format!("{}:{}", key, ctx.uri.path());

        match self.limiter.check(&scope, &ctx.request_id).await {
            Ok(decision) => {
                ctx.metadata.insert(
                    "ratelimit.remaining".to_string(),
                    decision.remaining.to_string(),
                );

                if decision.allowed {
                    Ok(ControlFlow::Continue)
                } else {
                    Ok(ControlFlow::ShortCircuit(
                        self.limited_response(decision.retry_after_secs),
                    ))
                }
            }
            Err(err) => {
                if self.fail_open_on_error {
                    tracing::warn!(
                        request_id = %ctx.request_id,
                        error = %err.message(),
                        "rate limiter backend failed; allowing request because fail-open is enabled"
                    );
                    Ok(ControlFlow::Continue)
                } else {
                    Err(GatewayError::Internal(
                        "rate limiter backend unavailable".to_string(),
                    ))
                }
            }
        }
    }

    async fn on_response(
        &self,
        ctx: &RequestContext,
        response: &mut Response<Body>,
    ) -> GatewayResult<()> {
        if let Some(remaining) = ctx.metadata.get("ratelimit.remaining")
            && let Ok(value) = HeaderValue::from_str(remaining)
        {
            response
                .headers_mut()
                .insert(HeaderName::from_static("x-ratelimit-remaining"), value);
        }

        Ok(())
    }
}

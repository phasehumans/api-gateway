use async_trait::async_trait;

use crate::{
    config::ValidationConfig,
    context::RequestContext,
    error::{GatewayError, GatewayResult},
    middleware::{ControlFlow, GatewayMiddleware},
};

pub struct RequestValidationMiddleware {
    cfg: ValidationConfig,
}

impl RequestValidationMiddleware {
    pub fn new(cfg: ValidationConfig) -> Self {
        Self { cfg }
    }
}

#[async_trait]
impl GatewayMiddleware for RequestValidationMiddleware {
    fn name(&self) -> &'static str {
        "request-validation"
    }

    async fn on_request(&self, ctx: &mut RequestContext) -> GatewayResult<ControlFlow> {
        if self.cfg.require_host_header && !ctx.headers.contains_key("host") {
            return Err(GatewayError::Validation(
                "Missing required Host header".to_string(),
            ));
        }

        if ctx.headers.len() > self.cfg.max_headers {
            return Err(GatewayError::Validation(format!(
                "Too many headers: {} > {}",
                ctx.headers.len(),
                self.cfg.max_headers
            )));
        }

        let method = ctx.method.as_str().to_ascii_uppercase();
        if !self.cfg.allowed_methods.iter().any(|m| m == &method) {
            return Err(GatewayError::Validation(format!(
                "Method {} is not allowed",
                ctx.method
            )));
        }

        if let Some(content_length) = ctx
            .headers
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<usize>().ok())
            && content_length != ctx.body.len()
        {
            return Err(GatewayError::Validation(
                "content-length does not match payload size".to_string(),
            ));
        }

        if ctx.body.len() > self.cfg.max_body_bytes {
            return Err(GatewayError::PayloadTooLarge);
        }

        Ok(ControlFlow::Continue)
    }
}

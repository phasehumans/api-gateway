use async_trait::async_trait;

use crate::{
    context::RequestContext,
    error::GatewayResult,
    middleware::{ControlFlow, GatewayMiddleware},
};

pub struct RequestLoggingMiddleware;

#[async_trait]
impl GatewayMiddleware for RequestLoggingMiddleware {
    fn name(&self) -> &'static str {
        "request-logging"
    }

    async fn on_request(&self, ctx: &mut RequestContext) -> GatewayResult<ControlFlow> {
        tracing::info!(
            request_id = %ctx.request_id,
            method = %ctx.method,
            path = %ctx.uri.path(),
            client_ip = ?ctx.client_ip,
            "incoming request"
        );
        Ok(ControlFlow::Continue)
    }

    async fn on_response(
        &self,
        ctx: &RequestContext,
        response: &mut axum::response::Response,
    ) -> GatewayResult<()> {
        let latency_ms = ctx.started_at.elapsed().as_millis();
        tracing::info!(
            request_id = %ctx.request_id,
            method = %ctx.method,
            path = %ctx.uri.path(),
            status = %response.status(),
            upstream = ?ctx.chosen_upstream,
            latency_ms = latency_ms,
            "request completed"
        );
        Ok(())
    }
}

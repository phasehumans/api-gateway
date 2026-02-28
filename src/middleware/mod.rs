pub mod auth;
pub mod logging;
pub mod rate_limit;
pub mod validation;

use async_trait::async_trait;
use axum::{
    body::Body,
    response::Response,
};

use crate::{
    context::RequestContext,
    error::GatewayResult,
};

pub enum ControlFlow {
    Continue,
    ShortCircuit(Response<Body>),
}

#[async_trait]
pub trait GatewayMiddleware: Send + Sync {
    fn name(&self) -> &'static str;

    async fn on_request(&self, ctx: &mut RequestContext) -> GatewayResult<ControlFlow>;

    async fn on_response(
        &self,
        _ctx: &RequestContext,
        _response: &mut Response<Body>,
    ) -> GatewayResult<()> {
        Ok(())
    }
}

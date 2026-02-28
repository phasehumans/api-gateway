use async_trait::async_trait;

use crate::{
    context::RequestContext,
    error::{GatewayError, GatewayResult},
    middleware::{ControlFlow, GatewayMiddleware},
};

pub struct ApiKeyAuthMiddleware {
    valid_keys: Vec<Vec<u8>>,
    exempt_prefixes: Vec<String>,
}

impl ApiKeyAuthMiddleware {
    pub fn new(valid_keys: Vec<String>, exempt_prefixes: Vec<String>) -> Self {
        Self {
            valid_keys: valid_keys.into_iter().map(|k| k.into_bytes()).collect(),
            exempt_prefixes,
        }
    }

    fn is_exempt_path(&self, path: &str) -> bool {
        self.exempt_prefixes.iter().any(|prefix| path.starts_with(prefix))
    }

    fn is_valid_key(&self, provided: &[u8]) -> bool {
        self.valid_keys
            .iter()
            .any(|expected| timing_safe_eq(expected, provided))
    }
}

#[async_trait]
impl GatewayMiddleware for ApiKeyAuthMiddleware {
    fn name(&self) -> &'static str {
        "api-key-auth"
    }

    async fn on_request(&self, ctx: &mut RequestContext) -> GatewayResult<ControlFlow> {
        if self.is_exempt_path(ctx.uri.path()) {
            return Ok(ControlFlow::Continue);
        }

        let provided = ctx
            .headers
            .get("x-api-key")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.as_bytes())
            .ok_or(GatewayError::Unauthorized)?;

        if !self.is_valid_key(provided) {
            return Err(GatewayError::Unauthorized);
        }

        Ok(ControlFlow::Continue)
    }
}

fn timing_safe_eq(a: &[u8], b: &[u8]) -> bool {
    let max = a.len().max(b.len());
    let mut diff = (a.len() ^ b.len()) as u8;

    for idx in 0..max {
        let av = *a.get(idx).unwrap_or(&0);
        let bv = *b.get(idx).unwrap_or(&0);
        diff |= av ^ bv;
    }

    diff == 0
}

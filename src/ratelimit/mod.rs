pub mod in_memory;
pub mod redis_backend;

use std::sync::Arc;

use async_trait::async_trait;

use crate::error::GatewayResult;

#[derive(Debug, Clone)]
pub enum RateLimitAlgorithm {
    TokenBucket {
        capacity: u32,
        refill_tokens_per_sec: f64,
    },
    SlidingWindow {
        window_seconds: u64,
        max_requests: u64,
    },
}

#[derive(Debug, Clone)]
pub struct RateLimitPolicy {
    pub algorithm: RateLimitAlgorithm,
}

#[derive(Debug, Clone, Default)]
pub struct RateLimitDecision {
    pub allowed: bool,
    pub remaining: u64,
    pub retry_after_secs: u64,
}

#[async_trait]
pub trait RateLimitBackend: Send + Sync {
    async fn check(
        &self,
        key: &str,
        policy: &RateLimitPolicy,
        request_id: &str,
    ) -> GatewayResult<RateLimitDecision>;
}

#[derive(Clone)]
pub struct RateLimiter {
    backend: Arc<dyn RateLimitBackend>,
    policy: RateLimitPolicy,
}

impl RateLimiter {
    pub fn new(backend: Arc<dyn RateLimitBackend>, policy: RateLimitPolicy) -> Self {
        Self { backend, policy }
    }

    pub async fn check(&self, key: &str, request_id: &str) -> GatewayResult<RateLimitDecision> {
        self.backend.check(key, &self.policy, request_id).await
    }
}

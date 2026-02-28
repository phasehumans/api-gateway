use std::{
    collections::VecDeque,
    sync::Arc,
    time::Instant,
};

use async_trait::async_trait;
use dashmap::DashMap;
use tokio::sync::Mutex;

use crate::{
    error::{GatewayError, GatewayResult},
    ratelimit::{RateLimitAlgorithm, RateLimitBackend, RateLimitDecision, RateLimitPolicy},
};

pub struct InMemoryRateLimitBackend {
    state: DashMap<String, Arc<Mutex<RateLimitState>>>,
}

enum RateLimitState {
    TokenBucket(TokenBucketState),
    SlidingWindow(SlidingWindowState),
}

struct TokenBucketState {
    tokens: f64,
    last_refill: Instant,
}

struct SlidingWindowState {
    entries: VecDeque<Instant>,
}

impl InMemoryRateLimitBackend {
    pub fn new() -> Self {
        Self {
            state: DashMap::new(),
        }
    }

    fn entry_for(&self, key: &str, policy: &RateLimitPolicy) -> Arc<Mutex<RateLimitState>> {
        self.state
            .entry(key.to_string())
            .or_insert_with(|| {
                Arc::new(Mutex::new(match &policy.algorithm {
                    RateLimitAlgorithm::TokenBucket { capacity, .. } => {
                        RateLimitState::TokenBucket(TokenBucketState {
                            tokens: *capacity as f64,
                            last_refill: Instant::now(),
                        })
                    }
                    RateLimitAlgorithm::SlidingWindow { .. } => {
                        RateLimitState::SlidingWindow(SlidingWindowState {
                            entries: VecDeque::new(),
                        })
                    }
                }))
            })
            .clone()
    }
}

#[async_trait]
impl RateLimitBackend for InMemoryRateLimitBackend {
    async fn check(
        &self,
        key: &str,
        policy: &RateLimitPolicy,
        _request_id: &str,
    ) -> GatewayResult<RateLimitDecision> {
        let state = self.entry_for(key, policy);
        let mut state = state.lock().await;

        match (&policy.algorithm, &mut *state) {
            (
                RateLimitAlgorithm::TokenBucket {
                    capacity,
                    refill_tokens_per_sec,
                },
                RateLimitState::TokenBucket(bucket),
            ) => {
                if *refill_tokens_per_sec <= 0.0 {
                    return Err(GatewayError::Internal(
                        "token bucket refill rate must be > 0".to_string(),
                    ));
                }

                let now = Instant::now();
                let elapsed = now.duration_since(bucket.last_refill).as_secs_f64();
                bucket.last_refill = now;
                bucket.tokens =
                    (bucket.tokens + elapsed * refill_tokens_per_sec).min(*capacity as f64);

                if bucket.tokens >= 1.0 {
                    bucket.tokens -= 1.0;
                    Ok(RateLimitDecision {
                        allowed: true,
                        remaining: bucket.tokens.floor() as u64,
                        retry_after_secs: 0,
                    })
                } else {
                    let needed = 1.0 - bucket.tokens;
                    let retry_after = (needed / refill_tokens_per_sec).ceil().max(1.0) as u64;
                    Ok(RateLimitDecision {
                        allowed: false,
                        remaining: 0,
                        retry_after_secs: retry_after,
                    })
                }
            }
            (
                RateLimitAlgorithm::SlidingWindow {
                    window_seconds,
                    max_requests,
                },
                RateLimitState::SlidingWindow(window),
            ) => {
                let now = Instant::now();
                while let Some(front) = window.entries.front() {
                    if now.duration_since(*front).as_secs() >= *window_seconds {
                        window.entries.pop_front();
                    } else {
                        break;
                    }
                }

                if (window.entries.len() as u64) < *max_requests {
                    window.entries.push_back(now);
                    Ok(RateLimitDecision {
                        allowed: true,
                        remaining: max_requests.saturating_sub(window.entries.len() as u64),
                        retry_after_secs: 0,
                    })
                } else {
                    let retry = window
                        .entries
                        .front()
                        .map(|t| {
                            window_seconds
                                .saturating_sub(now.duration_since(*t).as_secs())
                                .max(1)
                        })
                        .unwrap_or(1);

                    Ok(RateLimitDecision {
                        allowed: false,
                        remaining: 0,
                        retry_after_secs: retry,
                    })
                }
            }
            (RateLimitAlgorithm::TokenBucket { capacity, .. }, state) => {
                *state = RateLimitState::TokenBucket(TokenBucketState {
                    tokens: *capacity as f64,
                    last_refill: Instant::now(),
                });
                Ok(RateLimitDecision {
                    allowed: true,
                    remaining: (*capacity).saturating_sub(1) as u64,
                    retry_after_secs: 0,
                })
            }
            (RateLimitAlgorithm::SlidingWindow { .. }, state) => {
                *state = RateLimitState::SlidingWindow(SlidingWindowState {
                    entries: VecDeque::new(),
                });
                Ok(RateLimitDecision {
                    allowed: true,
                    remaining: 0,
                    retry_after_secs: 0,
                })
            }
        }
    }
}

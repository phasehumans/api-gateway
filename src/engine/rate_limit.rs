use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
struct TokenBucket {
    tokens: f64,
    capacity: f64,
    refill_per_sec: f64,
    last_refill: Instant,
}

impl TokenBucket {
    fn new(capacity: f64, refill_per_sec: f64, now: Instant) -> Self {
        Self {
            tokens: capacity,
            capacity,
            refill_per_sec,
            last_refill: now,
        }
    }

    fn try_take(&mut self, now: Instant) -> bool {
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.last_refill = now;
        self.tokens = (self.tokens + elapsed * self.refill_per_sec).min(self.capacity);
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

#[derive(Clone)]
pub struct TenantRateLimiter {
    state: std::sync::Arc<tokio::sync::Mutex<HashMap<String, TokenBucket>>>,
    burst: f64,
    refill_per_sec: f64,
    stale_after: Duration,
}

impl TenantRateLimiter {
    pub fn new(rate_per_minute: u32, burst: u32) -> Self {
        let refill = (rate_per_minute.max(1) as f64) / 60.0;
        Self {
            state: std::sync::Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            burst: burst.max(1) as f64,
            refill_per_sec: refill,
            stale_after: Duration::from_secs(30 * 60),
        }
    }

    pub async fn allow(&self, tenant_id: &str) -> bool {
        let now = Instant::now();
        let mut state = self.state.lock().await;
        state.retain(|_, bucket| now.duration_since(bucket.last_refill) < self.stale_after);
        let bucket = state
            .entry(tenant_id.to_string())
            .or_insert_with(|| TokenBucket::new(self.burst, self.refill_per_sec, now));
        bucket.try_take(now)
    }
}

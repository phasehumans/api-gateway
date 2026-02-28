use async_trait::async_trait;
use redis::{
    Script,
    aio::ConnectionManager,
};
use std::time::{
    SystemTime,
    UNIX_EPOCH,
};

use crate::{
    error::{GatewayError, GatewayResult},
    ratelimit::{RateLimitAlgorithm, RateLimitBackend, RateLimitDecision, RateLimitPolicy},
};

const TOKEN_BUCKET_LUA: &str = r#"
local key = KEYS[1]
local capacity = tonumber(ARGV[1])
local refill = tonumber(ARGV[2])
local now_ms = tonumber(ARGV[3])
local ttl = tonumber(ARGV[4])

local state = redis.call('HMGET', key, 'tokens', 'ts')
local tokens = tonumber(state[1])
local ts = tonumber(state[2])

if tokens == nil then
  tokens = capacity
  ts = now_ms
end

local delta_seconds = math.max(0, now_ms - ts) / 1000.0
tokens = math.min(capacity, tokens + (delta_seconds * refill))

local allowed = 0
local remaining = 0
local retry_after = 0

if tokens >= 1 then
  tokens = tokens - 1
  allowed = 1
  remaining = math.floor(tokens)
else
  local needed = 1 - tokens
  retry_after = math.max(1, math.ceil(needed / refill))
end

redis.call('HMSET', key, 'tokens', tokens, 'ts', now_ms)
redis.call('EXPIRE', key, ttl)

return {allowed, remaining, retry_after}
"#;

const SLIDING_WINDOW_LUA: &str = r#"
local key = KEYS[1]
local now_ms = tonumber(ARGV[1])
local window_ms = tonumber(ARGV[2])
local max_requests = tonumber(ARGV[3])
local member = ARGV[4]
local ttl = tonumber(ARGV[5])

redis.call('ZREMRANGEBYSCORE', key, 0, now_ms - window_ms)
local count = redis.call('ZCARD', key)

if count < max_requests then
  redis.call('ZADD', key, now_ms, member)
  redis.call('EXPIRE', key, ttl)
  return {1, max_requests - (count + 1), 0}
else
  local oldest = redis.call('ZRANGE', key, 0, 0, 'WITHSCORES')
  local retry_after = 1
  if oldest[2] then
    local oldest_score = tonumber(oldest[2])
    retry_after = math.max(1, math.ceil((oldest_score + window_ms - now_ms) / 1000.0))
  end
  return {0, 0, retry_after}
end
"#;

pub struct RedisRateLimitBackend {
    manager: ConnectionManager,
    key_prefix: String,
}

impl RedisRateLimitBackend {
    pub async fn new(url: String, key_prefix: String) -> GatewayResult<Self> {
        let client = redis::Client::open(url)?;
        let manager = client.get_connection_manager().await?;
        Ok(Self {
            manager,
            key_prefix,
        })
    }

    fn key(&self, key: &str) -> String {
        format!("{}:{}", self.key_prefix, key)
    }

    fn now_ms() -> GatewayResult<i64> {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| GatewayError::Internal(e.to_string()))?;
        Ok(duration.as_millis() as i64)
    }
}

#[async_trait]
impl RateLimitBackend for RedisRateLimitBackend {
    async fn check(
        &self,
        key: &str,
        policy: &RateLimitPolicy,
        request_id: &str,
    ) -> GatewayResult<RateLimitDecision> {
        let mut conn = self.manager.clone();
        let full_key = self.key(key);
        let now_ms = Self::now_ms()?;

        match &policy.algorithm {
            RateLimitAlgorithm::TokenBucket {
                capacity,
                refill_tokens_per_sec,
            } => {
                if *refill_tokens_per_sec <= 0.0 {
                    return Err(GatewayError::Internal(
                        "token bucket refill rate must be > 0".to_string(),
                    ));
                }

                let ttl = ((*capacity as f64 / refill_tokens_per_sec).ceil() as i64).max(1) * 2;
                let script = Script::new(TOKEN_BUCKET_LUA);
                let (allowed, remaining, retry_after): (i64, i64, i64) = script
                    .key(&full_key)
                    .arg(*capacity as i64)
                    .arg(*refill_tokens_per_sec)
                    .arg(now_ms)
                    .arg(ttl)
                    .invoke_async(&mut conn)
                    .await?;

                Ok(RateLimitDecision {
                    allowed: allowed == 1,
                    remaining: remaining.max(0) as u64,
                    retry_after_secs: retry_after.max(0) as u64,
                })
            }
            RateLimitAlgorithm::SlidingWindow {
                window_seconds,
                max_requests,
            } => {
                let ttl = (*window_seconds as i64 + 1).max(1);
                let member = format!("{}-{}", now_ms, request_id);
                let script = Script::new(SLIDING_WINDOW_LUA);
                let (allowed, remaining, retry_after): (i64, i64, i64) = script
                    .key(&full_key)
                    .arg(now_ms)
                    .arg((*window_seconds * 1000) as i64)
                    .arg(*max_requests as i64)
                    .arg(member)
                    .arg(ttl)
                    .invoke_async(&mut conn)
                    .await?;

                Ok(RateLimitDecision {
                    allowed: allowed == 1,
                    remaining: remaining.max(0) as u64,
                    retry_after_secs: retry_after.max(0) as u64,
                })
            }
        }
    }
}

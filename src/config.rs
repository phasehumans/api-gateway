use anyhow::{Context, Result, anyhow};
use std::{
    collections::HashSet,
    env,
    net::SocketAddr,
};

#[derive(Debug, Clone)]
pub struct GatewayConfig {
    pub bind_addr: SocketAddr,
    pub api_keys: HashSet<String>,
    pub auth_exempt_prefixes: Vec<String>,
    pub validation: ValidationConfig,
    pub rate_limit: RateLimitConfig,
    pub circuit_breaker: CircuitBreakerConfig,
    pub routing: RoutingConfig,
    pub upstreams: Vec<UpstreamConfig>,
    pub routes: Vec<RouteConfig>,
}

#[derive(Debug, Clone)]
pub struct ValidationConfig {
    pub max_body_bytes: usize,
    pub allowed_methods: Vec<String>,
    pub require_host_header: bool,
    pub max_headers: usize,
}

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub backend: RateLimitBackendConfig,
    pub policy: RateLimitPolicyConfig,
    pub key_header: String,
    pub fail_open_on_error: bool,
}

#[derive(Debug, Clone)]
pub enum RateLimitBackendConfig {
    InMemory,
    Redis { url: String, key_prefix: String },
}

#[derive(Debug, Clone)]
pub enum RateLimitPolicyConfig {
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
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub open_seconds: u64,
    pub half_open_max_requests: u32,
}

#[derive(Debug, Clone)]
pub struct RoutingConfig {
    pub prefer_low_latency: bool,
    pub in_flight_penalty: u64,
    pub failure_penalty: u64,
}

#[derive(Debug, Clone)]
pub struct UpstreamConfig {
    pub name: String,
    pub base_url: String,
    pub weight: u32,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone)]
pub struct RouteConfig {
    pub path_prefix: String,
    pub upstreams: Vec<String>,
}

impl GatewayConfig {
    pub fn from_env() -> Result<Self> {
        let bind_addr = env::var("BIND_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
            .parse::<SocketAddr>()
            .context("invalid BIND_ADDR")?;

        let api_keys = parse_csv("API_KEYS", "dev-key")
            .into_iter()
            .collect::<HashSet<_>>();

        let auth_exempt_prefixes = parse_csv("AUTH_EXEMPT_PREFIXES", "/health");

        let validation = ValidationConfig {
            max_body_bytes: parse_env("MAX_BODY_BYTES", 1_048_576usize),
            allowed_methods: parse_csv("ALLOWED_METHODS", "GET,POST,PUT,PATCH,DELETE,OPTIONS")
                .into_iter()
                .map(|m| m.to_ascii_uppercase())
                .collect(),
            require_host_header: parse_env("REQUIRE_HOST_HEADER", true),
            max_headers: parse_env("MAX_HEADERS", 128usize),
        };

        let rate_limit_algorithm = env::var("RATE_LIMIT_ALGORITHM")
            .unwrap_or_else(|_| "token_bucket".to_string())
            .to_ascii_lowercase();

        let policy = match rate_limit_algorithm.as_str() {
            "token_bucket" => RateLimitPolicyConfig::TokenBucket {
                capacity: parse_env("RATE_LIMIT_CAPACITY", 200u32),
                refill_tokens_per_sec: parse_env("RATE_LIMIT_REFILL_TPS", 100.0f64),
            },
            "sliding_window" => RateLimitPolicyConfig::SlidingWindow {
                window_seconds: parse_env("RATE_LIMIT_WINDOW_SECONDS", 60u64),
                max_requests: parse_env("RATE_LIMIT_MAX_REQUESTS", 600u64),
            },
            other => return Err(anyhow!("unsupported RATE_LIMIT_ALGORITHM: {other}")),
        };

        let backend = match env::var("RATE_LIMIT_BACKEND")
            .unwrap_or_else(|_| "memory".to_string())
            .to_ascii_lowercase()
            .as_str()
        {
            "memory" | "in_memory" => RateLimitBackendConfig::InMemory,
            "redis" => {
                let url = env::var("REDIS_URL")
                    .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
                let key_prefix = env::var("REDIS_KEY_PREFIX")
                    .unwrap_or_else(|_| "gateway:ratelimit".to_string());
                RateLimitBackendConfig::Redis { url, key_prefix }
            }
            other => return Err(anyhow!("unsupported RATE_LIMIT_BACKEND: {other}")),
        };

        let rate_limit = RateLimitConfig {
            enabled: parse_env("RATE_LIMIT_ENABLED", true),
            backend,
            policy,
            key_header: env::var("RATE_LIMIT_KEY_HEADER")
                .unwrap_or_else(|_| "x-api-key".to_string()),
            fail_open_on_error: parse_env("RATE_LIMIT_FAIL_OPEN", false),
        };

        let circuit_breaker = CircuitBreakerConfig {
            failure_threshold: parse_env("CB_FAILURE_THRESHOLD", 5u32),
            open_seconds: parse_env("CB_OPEN_SECONDS", 20u64),
            half_open_max_requests: parse_env("CB_HALF_OPEN_MAX", 1u32),
        };

        let routing = RoutingConfig {
            prefer_low_latency: parse_env("ROUTING_PREFER_LOW_LATENCY", true),
            in_flight_penalty: parse_env("ROUTING_IN_FLIGHT_PENALTY", 12u64),
            failure_penalty: parse_env("ROUTING_FAILURE_PENALTY", 250u64),
        };

        let upstreams = parse_upstreams(
            &env::var("UPSTREAMS")
                .unwrap_or_else(|_| "svc-a=http://127.0.0.1:9001,svc-b=http://127.0.0.1:9002".into()),
        )?;

        let routes = parse_routes(
            &env::var("ROUTES")
                .unwrap_or_else(|_| "/=svc-a|svc-b,/health=svc-a".into()),
        )?;

        Ok(Self {
            bind_addr,
            api_keys,
            auth_exempt_prefixes,
            validation,
            rate_limit,
            circuit_breaker,
            routing,
            upstreams,
            routes,
        })
    }
}

fn parse_upstreams(raw: &str) -> Result<Vec<UpstreamConfig>> {
    let mut out = Vec::new();
    for chunk in raw.split(',').filter(|c| !c.trim().is_empty()) {
        let mut parts = chunk.splitn(2, '=');
        let name = parts
            .next()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| anyhow!("invalid upstream entry: {chunk}"))?
            .to_string();
        let rhs = parts
            .next()
            .map(str::trim)
            .ok_or_else(|| anyhow!("invalid upstream entry: {chunk}"))?;

        let spec: Vec<&str> = rhs.split('@').collect();
        let base_url = spec
            .first()
            .map(|s| s.trim().trim_end_matches('/').to_string())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| anyhow!("invalid upstream URL for {name}"))?;
        let weight = spec
            .get(1)
            .map(|s| s.parse::<u32>().context("invalid upstream weight"))
            .transpose()?
            .unwrap_or(100)
            .max(1);
        let timeout_ms = spec
            .get(2)
            .map(|s| s.parse::<u64>().context("invalid upstream timeout"))
            .transpose()?
            .unwrap_or(3_000)
            .max(100);

        out.push(UpstreamConfig {
            name,
            base_url,
            weight,
            timeout_ms,
        });
    }

    if out.is_empty() {
        return Err(anyhow!("no upstreams configured"));
    }
    Ok(out)
}

fn parse_routes(raw: &str) -> Result<Vec<RouteConfig>> {
    let mut out = Vec::new();
    for chunk in raw.split(',').filter(|c| !c.trim().is_empty()) {
        let mut parts = chunk.splitn(2, '=');
        let path_prefix = parts
            .next()
            .map(str::trim)
            .filter(|s| s.starts_with('/'))
            .ok_or_else(|| anyhow!("invalid route entry: {chunk}"))?
            .to_string();

        let upstreams = parts
            .next()
            .map(str::trim)
            .ok_or_else(|| anyhow!("invalid route entry: {chunk}"))?
            .split('|')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        if upstreams.is_empty() {
            return Err(anyhow!("route has no upstreams: {chunk}"));
        }

        out.push(RouteConfig {
            path_prefix,
            upstreams,
        });
    }

    if out.is_empty() {
        return Err(anyhow!("no routes configured"));
    }
    Ok(out)
}

fn parse_csv(key: &str, default: &str) -> Vec<String> {
    env::var(key)
        .unwrap_or_else(|_| default.to_string())
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn parse_env<T>(key: &str, default: T) -> T
where
    T: std::str::FromStr,
{
    env::var(key)
        .ok()
        .and_then(|s| s.parse::<T>().ok())
        .unwrap_or(default)
}

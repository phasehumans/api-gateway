use axum::{
    body::Body,
    response::Response,
};
use dashmap::DashMap;
use http::header::HeaderName;
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{
            AtomicU64,
            Ordering,
        },
    },
    time::{
        Duration,
        Instant,
    },
};

use crate::{
    config::{RouteConfig, UpstreamConfig},
    context::RequestContext,
    error::{GatewayError, GatewayResult},
};

#[derive(Clone)]
pub struct UpstreamPool {
    client: reqwest::Client,
    services: HashMap<String, UpstreamConfig>,
    stats: DashMap<String, Arc<UpstreamStats>>,
}

#[derive(Default)]
struct UpstreamStats {
    in_flight: AtomicU64,
    consecutive_failures: AtomicU64,
    success_total: AtomicU64,
    failure_total: AtomicU64,
    avg_latency_micros: AtomicU64,
}

#[derive(Debug, Clone, Default)]
pub struct UpstreamSnapshot {
    pub in_flight: u64,
    pub consecutive_failures: u64,
    pub success_total: u64,
    pub failure_total: u64,
    pub avg_latency_ms: u64,
}

impl UpstreamPool {
    pub fn new(upstreams: Vec<UpstreamConfig>) -> GatewayResult<Self> {
        let mut services = HashMap::new();
        let stats = DashMap::new();

        for upstream in upstreams {
            stats.insert(upstream.name.clone(), Arc::new(UpstreamStats::default()));
            services.insert(upstream.name.clone(), upstream);
        }

        let client = reqwest::Client::builder()
            .pool_idle_timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(32)
            .tcp_nodelay(true)
            .build()
            .map_err(|e| GatewayError::Internal(e.to_string()))?;

        Ok(Self {
            client,
            services,
            stats,
        })
    }

    pub fn get(&self, name: &str) -> Option<UpstreamConfig> {
        self.services.get(name).cloned()
    }

    pub fn route_candidates(&self, route: &RouteConfig) -> Vec<UpstreamConfig> {
        route
            .upstreams
            .iter()
            .filter_map(|name| self.get(name))
            .collect()
    }

    pub fn snapshot(&self, name: &str) -> UpstreamSnapshot {
        self.stats
            .get(name)
            .map(|stats| UpstreamSnapshot {
                in_flight: stats.in_flight.load(Ordering::Relaxed),
                consecutive_failures: stats.consecutive_failures.load(Ordering::Relaxed),
                success_total: stats.success_total.load(Ordering::Relaxed),
                failure_total: stats.failure_total.load(Ordering::Relaxed),
                avg_latency_ms: stats.avg_latency_micros.load(Ordering::Relaxed) / 1_000,
            })
            .unwrap_or_default()
    }

    pub async fn forward(
        &self,
        ctx: &RequestContext,
        upstream: &UpstreamConfig,
    ) -> GatewayResult<Response<Body>> {
        let stats = self
            .stats
            .get(&upstream.name)
            .map(|s| s.clone())
            .ok_or_else(|| GatewayError::Internal("upstream stats unavailable".to_string()))?;

        stats.in_flight.fetch_add(1, Ordering::Relaxed);

        let path_and_query = ctx
            .uri
            .path_and_query()
            .map(|pq| pq.as_str())
            .unwrap_or(ctx.uri.path());
        let target_url = format!("{}{}", upstream.base_url.trim_end_matches('/'), path_and_query);

        let mut request = self
            .client
            .request(ctx.method.clone(), &target_url)
            .body(ctx.body.clone());

        for (name, value) in &ctx.headers {
            if should_forward_header(name) {
                request = request.header(name, value);
            }
        }

        request = request.header("x-request-id", ctx.request_id.clone());
        if let Some(client_ip) = ctx.client_ip {
            request = request.header("x-forwarded-for", client_ip.to_string());
        }

        let started = Instant::now();
        let response = request
            .timeout(Duration::from_millis(upstream.timeout_ms))
            .send()
            .await;

        stats.in_flight.fetch_sub(1, Ordering::Relaxed);

        match response {
            Ok(upstream_response) => {
                let status = upstream_response.status();
                let headers = upstream_response.headers().clone();
                let body = upstream_response.bytes().await?;
                let latency = started.elapsed();

                if status.is_server_error() {
                    stats.record_failure();
                } else {
                    stats.record_success(latency);
                }

                let mut builder = Response::builder().status(status);
                for (name, value) in &headers {
                    if should_forward_header(name) {
                        builder = builder.header(name, value);
                    }
                }

                builder
                    .body(Body::from(body))
                    .map_err(|e| GatewayError::Internal(e.to_string()))
            }
            Err(err) => {
                stats.record_failure();
                Err(GatewayError::Upstream(err.to_string()))
            }
        }
    }
}

impl UpstreamStats {
    fn record_success(&self, latency: Duration) {
        self.success_total.fetch_add(1, Ordering::Relaxed);
        self.consecutive_failures.store(0, Ordering::Relaxed);

        let latency_micros = latency.as_micros() as u64;
        let current = self.avg_latency_micros.load(Ordering::Relaxed);
        let updated = if current == 0 {
            latency_micros
        } else {
            (current * 7 + latency_micros) / 8
        };
        self.avg_latency_micros.store(updated, Ordering::Relaxed);
    }

    fn record_failure(&self) {
        self.failure_total.fetch_add(1, Ordering::Relaxed);
        self.consecutive_failures.fetch_add(1, Ordering::Relaxed);
    }
}

fn should_forward_header(name: &HeaderName) -> bool {
    let lowercase = name.as_str().to_ascii_lowercase();
    !matches!(
        lowercase.as_str(),
        "connection"
            | "keep-alive"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "te"
            | "trailer"
            | "transfer-encoding"
            | "upgrade"
            | "host"
            | "content-length"
    )
}

use std::{
    net::IpAddr,
    sync::Arc,
};

use axum::{
    body::{
        Body,
        to_bytes,
    },
    http::{
        HeaderName,
        HeaderValue,
        Request,
    },
    response::{
        IntoResponse,
        Response,
    },
};
use uuid::Uuid;

use crate::{
    circuit_breaker::CircuitBreaker,
    config::{
        GatewayConfig,
        RateLimitBackendConfig,
        RateLimitPolicyConfig,
        RouteConfig,
    },
    context::RequestContext,
    error::{GatewayError, GatewayResult},
    middleware::{
        ControlFlow,
        GatewayMiddleware,
        auth::ApiKeyAuthMiddleware,
        logging::RequestLoggingMiddleware,
        rate_limit::RateLimitMiddleware,
        validation::RequestValidationMiddleware,
    },
    ratelimit::{
        RateLimitAlgorithm,
        RateLimitBackend,
        RateLimitPolicy,
        RateLimiter,
        in_memory::InMemoryRateLimitBackend,
        redis_backend::RedisRateLimitBackend,
    },
    router::{
        IntelligentRouter,
        RoutingCandidate,
        RoutingStrategy,
    },
    upstream::UpstreamPool,
};

pub struct Gateway {
    middlewares: Vec<Arc<dyn GatewayMiddleware>>,
    routes: Vec<RouteConfig>,
    router: Arc<dyn RoutingStrategy>,
    upstream_pool: Arc<UpstreamPool>,
    circuit_breaker: CircuitBreaker,
    max_body_bytes: usize,
}

impl Gateway {
    pub async fn from_config(config: GatewayConfig) -> GatewayResult<Self> {
        let mut middlewares: Vec<Arc<dyn GatewayMiddleware>> = vec![
            Arc::new(RequestLoggingMiddleware),
            Arc::new(RequestValidationMiddleware::new(config.validation.clone())),
            Arc::new(ApiKeyAuthMiddleware::new(
                config.api_keys.iter().cloned().collect(),
                config.auth_exempt_prefixes.clone(),
            )),
        ];

        if config.rate_limit.enabled {
            let policy = match config.rate_limit.policy {
                RateLimitPolicyConfig::TokenBucket {
                    capacity,
                    refill_tokens_per_sec,
                } => RateLimitPolicy {
                    algorithm: RateLimitAlgorithm::TokenBucket {
                        capacity,
                        refill_tokens_per_sec,
                    },
                },
                RateLimitPolicyConfig::SlidingWindow {
                    window_seconds,
                    max_requests,
                } => RateLimitPolicy {
                    algorithm: RateLimitAlgorithm::SlidingWindow {
                        window_seconds,
                        max_requests,
                    },
                },
            };

            let backend: Arc<dyn RateLimitBackend> = match &config.rate_limit.backend {
                RateLimitBackendConfig::InMemory => Arc::new(InMemoryRateLimitBackend::new()),
                RateLimitBackendConfig::Redis { url, key_prefix } => {
                    Arc::new(RedisRateLimitBackend::new(url.clone(), key_prefix.clone()).await?)
                }
            };

            let limiter = RateLimiter::new(backend, policy);
            middlewares.push(Arc::new(RateLimitMiddleware::new(
                limiter,
                config.rate_limit.key_header.clone(),
                config.rate_limit.fail_open_on_error,
            )));
        }

        let router: Arc<dyn RoutingStrategy> = Arc::new(IntelligentRouter::new(config.routing.clone()));
        let upstream_pool = Arc::new(UpstreamPool::new(config.upstreams.clone())?);
        let circuit_breaker = CircuitBreaker::new(config.circuit_breaker.clone());

        Ok(Self {
            middlewares,
            routes: config.routes,
            router,
            upstream_pool,
            circuit_breaker,
            max_body_bytes: config.validation.max_body_bytes,
        })
    }

    pub async fn handle_http(
        &self,
        request: Request<Body>,
        client_ip: Option<IpAddr>,
    ) -> Response<Body> {
        let (parts, body) = request.into_parts();
        let max_capture = self.max_body_bytes.saturating_add(1);
        let body = match to_bytes(body, max_capture).await {
            Ok(body) => body,
            Err(_) => {
                let mut response = GatewayError::PayloadTooLarge.into_response();
                self.attach_hardening_headers("unknown", &mut response);
                return response;
            }
        };

        let request_id = parts
            .headers
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .map(ToString::to_string)
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let mut ctx = RequestContext::new(
            request_id,
            parts.method,
            parts.uri,
            parts.headers,
            body,
            client_ip,
        );

        let mut executed = Vec::new();

        for (idx, middleware) in self.middlewares.iter().enumerate() {
            match middleware.on_request(&mut ctx).await {
                Ok(ControlFlow::Continue) => executed.push(idx),
                Ok(ControlFlow::ShortCircuit(mut response)) => {
                    self.apply_response_middlewares(&executed, &ctx, &mut response)
                        .await;
                    self.attach_hardening_headers(&ctx.request_id, &mut response);
                    return response;
                }
                Err(err) => {
                    tracing::warn!(
                        request_id = %ctx.request_id,
                        middleware = middleware.name(),
                        error = %err.message(),
                        "middleware rejected request"
                    );
                    let mut response = err.into_response();
                    self.apply_response_middlewares(&executed, &ctx, &mut response)
                        .await;
                    self.attach_hardening_headers(&ctx.request_id, &mut response);
                    return response;
                }
            }
        }

        let route = match self.resolve_route(ctx.uri.path()) {
            Some(route) => route,
            None => {
                let mut response = GatewayError::RouteNotFound.into_response();
                self.apply_response_middlewares(&executed, &ctx, &mut response)
                    .await;
                self.attach_hardening_headers(&ctx.request_id, &mut response);
                return response;
            }
        };

        let candidates = self.upstream_pool.route_candidates(&route);
        if candidates.is_empty() {
            let mut response = GatewayError::UpstreamUnavailable.into_response();
            self.apply_response_middlewares(&executed, &ctx, &mut response)
                .await;
            self.attach_hardening_headers(&ctx.request_id, &mut response);
            return response;
        }

        let mut ranked_input = Vec::with_capacity(candidates.len());
        for upstream in &candidates {
            ranked_input.push(RoutingCandidate {
                breaker_open: self.circuit_breaker.is_open(&upstream.name).await,
                stats: self.upstream_pool.snapshot(&upstream.name),
                upstream: upstream.clone(),
            });
        }

        ctx.route = Some(route.clone());
        let ranked = self.router.rank(&route, &ranked_input);

        let mut last_error: Option<GatewayError> = None;

        for upstream_name in ranked {
            if !self.circuit_breaker.allow_request(&upstream_name).await {
                continue;
            }

            let Some(upstream) = self.upstream_pool.get(&upstream_name) else {
                continue;
            };

            ctx.chosen_upstream = Some(upstream_name.clone());

            match self.upstream_pool.forward(&ctx, &upstream).await {
                Ok(mut response) => {
                    if response.status().is_server_error() {
                        self.circuit_breaker.record_failure(&upstream_name).await;
                    } else {
                        self.circuit_breaker.record_success(&upstream_name).await;
                    }

                    self.apply_response_middlewares(&executed, &ctx, &mut response)
                        .await;
                    self.attach_hardening_headers(&ctx.request_id, &mut response);
                    return response;
                }
                Err(err) => {
                    self.circuit_breaker.record_failure(&upstream_name).await;
                    tracing::warn!(
                        request_id = %ctx.request_id,
                        upstream = %upstream_name,
                        error = %err.message(),
                        "upstream call failed; trying next candidate"
                    );
                    last_error = Some(err);
                }
            }
        }

        let mut response = last_error
            .unwrap_or(GatewayError::UpstreamUnavailable)
            .into_response();
        self.apply_response_middlewares(&executed, &ctx, &mut response)
            .await;
        self.attach_hardening_headers(&ctx.request_id, &mut response);
        response
    }

    fn resolve_route(&self, path: &str) -> Option<RouteConfig> {
        self.routes
            .iter()
            .filter(|route| path.starts_with(route.path_prefix.as_str()))
            .max_by_key(|route| route.path_prefix.len())
            .cloned()
    }

    async fn apply_response_middlewares(
        &self,
        executed: &[usize],
        ctx: &RequestContext,
        response: &mut Response<Body>,
    ) {
        for idx in executed.iter().rev() {
            let middleware = &self.middlewares[*idx];
            if let Err(err) = middleware.on_response(ctx, response).await {
                tracing::warn!(
                    request_id = %ctx.request_id,
                    middleware = middleware.name(),
                    error = %err.message(),
                    "middleware post-response hook failed"
                );
            }
        }
    }

    fn attach_hardening_headers(&self, request_id: &str, response: &mut Response<Body>) {
        if let Ok(value) = HeaderValue::from_str(request_id) {
            response
                .headers_mut()
                .insert(HeaderName::from_static("x-request-id"), value);
        }

        response.headers_mut().insert(
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        );
        response.headers_mut().insert(
            HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY"),
        );
        response.headers_mut().insert(
            HeaderName::from_static("referrer-policy"),
            HeaderValue::from_static("no-referrer"),
        );
    }
}

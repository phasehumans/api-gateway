use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use uuid::Uuid;

use crate::engine::{
    config::EngineConfig,
    error::EngineError,
    metrics::MetricsRegistry,
    models::{
        CreateExecutionResponse, ExecutionRecord, ExecutionRequest, ExecutionSummaryResponse,
    },
    queue::{QueuedJob, Scheduler},
    rate_limit::TenantRateLimiter,
    store::ExecutionStore,
};

#[derive(Clone)]
pub struct AppState {
    config: EngineConfig,
    store: Arc<ExecutionStore>,
    scheduler: Scheduler,
    metrics: Arc<MetricsRegistry>,
    rate_limiter: TenantRateLimiter,
}

pub fn routes(
    config: EngineConfig,
    store: Arc<ExecutionStore>,
    scheduler: Scheduler,
    metrics_registry: Arc<MetricsRegistry>,
) -> Router {
    let rate_limiter =
        TenantRateLimiter::new(config.rate_limit_per_minute, config.rate_limit_burst);
    let state = AppState {
        config,
        store,
        scheduler,
        metrics: metrics_registry,
        rate_limiter,
    };
    Router::new()
        .route("/healthz", get(health))
        .route("/metrics", get(metrics))
        .route("/v1/executions", post(submit_execution))
        .route("/v1/executions/{id}", get(get_execution))
        .route("/v1/executions/{id}/result", get(get_result))
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true }))
}

async fn metrics(State(state): State<AppState>) -> (StatusCode, String) {
    (StatusCode::OK, state.metrics.render_prometheus())
}

async fn submit_execution(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(mut request): Json<ExecutionRequest>,
) -> Result<(StatusCode, Json<CreateExecutionResponse>), EngineError> {
    let tenant_id = authenticate(&state.config, &headers)?;
    enforce_rate_limit(&state, &tenant_id).await?;

    validate_request(&request)?;
    if request.allow_network && !state.config.network_allowed_tenants.contains(&tenant_id) {
        return Err(EngineError::Forbidden);
    }
    if request.mode.is_none() {
        request.mode = Some(crate::engine::models::ExecutionMode::Human);
    }

    let id = Uuid::new_v4();
    let limits = request
        .limits
        .clone()
        .unwrap_or_else(|| state.config.default_limits.clone())
        .normalized();
    let record: ExecutionRecord =
        state
            .store
            .create_record(id, tenant_id.clone(), request.clone(), limits.clone());
    state.store.insert(record);

    if let Err(err) = state
        .scheduler
        .submit(QueuedJob {
            id,
            tenant_id,
            request,
            limits,
        })
        .await
    {
        state.store.remove(&id);
        return Err(err);
    }

    Ok((
        StatusCode::ACCEPTED,
        Json(CreateExecutionResponse {
            id,
            status: crate::engine::models::ExecutionStatus::Queued,
        }),
    ))
}

async fn get_execution(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<ExecutionSummaryResponse>, EngineError> {
    let tenant_id = authenticate(&state.config, &headers)?;
    let record = load_for_tenant(&state, id, &tenant_id)?;

    Ok(Json(ExecutionSummaryResponse {
        id: record.id,
        tenant_id: record.tenant_id,
        status: record.status,
        created_at_ms: record.created_at_ms,
        started_at_ms: record.started_at_ms,
        finished_at_ms: record.finished_at_ms,
    }))
}

async fn get_result(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<ExecutionRecord>, EngineError> {
    let tenant_id = authenticate(&state.config, &headers)?;
    let record = load_for_tenant(&state, id, &tenant_id)?;
    Ok(Json(record))
}

fn authenticate(config: &EngineConfig, headers: &HeaderMap) -> Result<String, EngineError> {
    let key = headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .ok_or(EngineError::Unauthorized)?;
    for (candidate_key, tenant_id) in &config.api_keys {
        if constant_time_eq(key.as_bytes(), candidate_key.as_bytes()) {
            return Ok(tenant_id.clone());
        }
    }
    Err(EngineError::Unauthorized)
}

async fn enforce_rate_limit(state: &AppState, tenant_id: &str) -> Result<(), EngineError> {
    if !state.rate_limiter.allow(tenant_id).await {
        return Err(EngineError::RateLimited);
    }
    Ok(())
}

fn validate_request(request: &ExecutionRequest) -> Result<(), EngineError> {
    if request.code.trim().is_empty() {
        return Err(EngineError::InvalidRequest("code is empty".to_string()));
    }
    if request.code.len() > 250_000 {
        return Err(EngineError::InvalidRequest("code too large".to_string()));
    }
    if request.args.len() > 16 {
        return Err(EngineError::InvalidRequest(
            "too many runtime args".to_string(),
        ));
    }
    if request.stdin.len() > 256_000 {
        return Err(EngineError::InvalidRequest("stdin too large".to_string()));
    }
    if request.test_cases.len() > 128 {
        return Err(EngineError::InvalidRequest(
            "too many test cases; max is 128".to_string(),
        ));
    }
    for case in &request.test_cases {
        if case.stdin.len() > 64_000 {
            return Err(EngineError::InvalidRequest(
                "test case stdin too large".to_string(),
            ));
        }
    }
    if let Some(limits) = &request.limits {
        if limits.timeout_ms == 0 || limits.memory_mb == 0 || limits.max_output_bytes == 0 {
            return Err(EngineError::InvalidRequest(
                "limits must be greater than zero".to_string(),
            ));
        }
    }
    Ok(())
}

fn load_for_tenant(
    state: &AppState,
    id: Uuid,
    tenant_id: &str,
) -> Result<ExecutionRecord, EngineError> {
    let record = state.store.get(&id).ok_or(EngineError::NotFound)?;
    if record.tenant_id != tenant_id {
        return Err(EngineError::Forbidden);
    }
    Ok(record)
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut out = 0u8;
    for (l, r) in a.iter().zip(b.iter()) {
        out |= l ^ r;
    }
    out == 0
}

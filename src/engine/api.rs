use std::{collections::HashMap, sync::Arc};

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
    store::ExecutionStore,
};

#[derive(Clone)]
pub struct AppState {
    config: EngineConfig,
    store: Arc<ExecutionStore>,
    scheduler: Scheduler,
    metrics: Arc<MetricsRegistry>,
    ratelimit_state: Arc<tokio::sync::Mutex<HashMap<String, std::time::Instant>>>,
}

pub fn routes(
    config: EngineConfig,
    store: Arc<ExecutionStore>,
    scheduler: Scheduler,
    metrics_registry: Arc<MetricsRegistry>,
) -> Router {
    let state = AppState {
        config,
        store,
        scheduler,
        metrics: metrics_registry,
        ratelimit_state: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
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
    if request.mode.is_none() {
        request.mode = Some(crate::engine::models::ExecutionMode::Human);
    }

    let id = Uuid::new_v4();
    let limits = request
        .limits
        .clone()
        .unwrap_or_else(|| state.config.default_limits.clone());
    let record: ExecutionRecord =
        state
            .store
            .create_record(id, tenant_id.clone(), request.clone(), limits.clone());
    state.store.insert(record);

    state
        .scheduler
        .submit(QueuedJob {
            id,
            tenant_id,
            request,
            limits,
        })
        .await?;

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
        created_at: record.created_at,
        started_at: record.started_at,
        finished_at: record.finished_at,
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
    config
        .api_keys
        .get(key)
        .cloned()
        .ok_or(EngineError::Unauthorized)
}

async fn enforce_rate_limit(state: &AppState, tenant_id: &str) -> Result<(), EngineError> {
    let mut map = state.ratelimit_state.lock().await;
    let now = std::time::Instant::now();
    let allowance = std::time::Duration::from_secs(
        (60f64 / state.config.rate_limit_per_minute.max(1) as f64).ceil() as u64,
    );
    if let Some(last) = map.get(tenant_id) {
        if now.duration_since(*last) < allowance {
            return Err(EngineError::RateLimited);
        }
    }
    map.insert(tenant_id.to_string(), now);
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
    Ok(())
}

fn load_for_tenant(state: &AppState, id: Uuid, tenant_id: &str) -> Result<ExecutionRecord, EngineError> {
    let record = state.store.get(&id).ok_or(EngineError::NotFound)?;
    if record.tenant_id != tenant_id {
        return Err(EngineError::Forbidden);
    }
    Ok(record)
}

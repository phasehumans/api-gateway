pub mod api;
pub mod config;
pub mod error;
pub mod metrics;
pub mod models;
pub mod queue;
pub mod rate_limit;
pub mod sandbox;
pub mod store;
pub mod worker;

use std::{net::SocketAddr, sync::Arc};

use anyhow::Context;
use axum::Router;

use crate::engine::{
    api::routes, config::EngineConfig, metrics::MetricsRegistry, queue::Scheduler,
    sandbox::SandboxFactory, store::ExecutionStore, worker::spawn_worker_pool,
};

pub async fn run() -> anyhow::Result<()> {
    let config = EngineConfig::from_env();
    init_tracing(&config);

    let store = Arc::new(ExecutionStore::new(config.persistence_path.clone()));
    let metrics = Arc::new(MetricsRegistry::new());
    let scheduler = Scheduler::new(config.queue_capacity, metrics.clone());
    let sandbox = SandboxFactory::from_config(&config).context("sandbox backend init failed")?;

    spawn_worker_pool(
        config.worker_count.max(1),
        scheduler.receiver(),
        store.clone(),
        metrics.clone(),
        sandbox,
    );

    let app: Router = routes(config.clone(), store, scheduler, metrics);
    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
    let local = listener
        .local_addr()
        .unwrap_or(SocketAddr::from(([0, 0, 0, 0], 0)));
    tracing::info!(bind = %local, "sandbox execution engine ready");
    axum::serve(listener, app).await?;
    Ok(())
}

fn init_tracing(config: &EngineConfig) {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(config.log_level.clone()));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .json()
        .with_current_span(false)
        .with_span_list(false)
        .init();
}

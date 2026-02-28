mod circuit_breaker;
mod config;
mod context;
mod error;
mod gateway;
mod middleware;
mod ratelimit;
mod router;
mod upstream;

use std::{
    net::SocketAddr,
    sync::Arc,
};

use anyhow::Context;
use axum::{
    Router,
    body::Body,
    extract::{
        ConnectInfo,
        State,
    },
    http::Request,
    routing::any,
};
use gateway::Gateway;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

use crate::config::GatewayConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let cfg = GatewayConfig::from_env().context("failed to build gateway config")?;
    let bind_addr = cfg.bind_addr;

    let gateway = Arc::new(Gateway::from_config(cfg).await?);

    let app = Router::new().fallback(any(proxy_handler)).with_state(gateway);

    let listener = TcpListener::bind(bind_addr)
        .await
        .context("failed to bind listener")?;

    tracing::info!(addr = %bind_addr, "API gateway listening");

    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .context("gateway server error")?;

    Ok(())
}

async fn proxy_handler(
    State(gateway): State<Arc<Gateway>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<Body>,
) -> axum::response::Response {
    gateway.handle_http(request, Some(addr.ip())).await
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("info,hyper=warn,reqwest=warn,tower_http=warn")
    });

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .init();
}

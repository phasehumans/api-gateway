mod docker;
mod language;
mod process;

use std::sync::Arc;

use async_trait::async_trait;

use crate::engine::{
    config::{EngineConfig, SandboxBackendKind},
    models::ExecutionRequest,
    queue::QueuedJob,
};

pub use docker::DockerSandbox;
pub use language::LanguageSpec;
pub use process::ProcessSandbox;

#[derive(Debug, Clone)]
pub struct SandboxResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration_ms: u128,
    pub timed_out: bool,
}

#[derive(Debug, Clone)]
pub struct RunSpec {
    pub request: ExecutionRequest,
    pub limits: crate::engine::models::ExecutionLimits,
    pub id: uuid::Uuid,
}

impl From<QueuedJob> for RunSpec {
    fn from(value: QueuedJob) -> Self {
        Self {
            request: value.request,
            limits: value.limits,
            id: value.id,
        }
    }
}

#[async_trait]
pub trait SandboxBackend: Send + Sync {
    fn name(&self) -> &'static str;
    async fn execute(&self, spec: RunSpec) -> anyhow::Result<SandboxResult>;
}

pub struct SandboxFactory;

impl SandboxFactory {
    pub fn from_config(config: &EngineConfig) -> anyhow::Result<Arc<dyn SandboxBackend>> {
        match config.sandbox_backend {
            SandboxBackendKind::Docker => Ok(Arc::new(DockerSandbox::new()?)),
            SandboxBackendKind::Process => Ok(Arc::new(ProcessSandbox::new())),
        }
    }
}

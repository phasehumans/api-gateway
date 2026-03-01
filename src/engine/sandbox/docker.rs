use async_trait::async_trait;

use crate::engine::sandbox::{RunSpec, SandboxBackend, SandboxResult};

pub struct DockerSandbox;

impl DockerSandbox {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self)
    }
}

#[async_trait]
impl SandboxBackend for DockerSandbox {
    fn name(&self) -> &'static str {
        "docker"
    }

    async fn execute(&self, _spec: RunSpec) -> anyhow::Result<SandboxResult> {
        anyhow::bail!("docker sandbox implementation pending");
    }
}

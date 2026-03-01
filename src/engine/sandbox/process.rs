use std::time::Instant;

use async_trait::async_trait;

use crate::engine::sandbox::{RunSpec, SandboxBackend, SandboxResult};

pub struct ProcessSandbox;

impl ProcessSandbox {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl SandboxBackend for ProcessSandbox {
    fn name(&self) -> &'static str {
        "process"
    }

    async fn execute(&self, _spec: RunSpec) -> anyhow::Result<SandboxResult> {
        let started = Instant::now();
        Ok(SandboxResult {
            stdout: String::new(),
            stderr: "process backend not implemented yet".to_string(),
            exit_code: 1,
            duration_ms: started.elapsed().as_millis(),
            timed_out: false,
        })
    }
}

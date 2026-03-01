use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    Python,
    JavaScript,
    Rust,
    C,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    Human,
    AgentOptimized,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub stdin: String,
    pub expected_stdout: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLimits {
    pub cpu_cores: f32,
    pub memory_mb: u64,
    pub timeout_ms: u64,
    pub max_processes: u64,
    pub max_file_size_bytes: u64,
    pub max_output_bytes: usize,
}

impl ExecutionLimits {
    pub fn normalized(mut self) -> Self {
        self.cpu_cores = self.cpu_cores.clamp(0.1, 4.0);
        self.memory_mb = self.memory_mb.clamp(32, 8192);
        self.timeout_ms = self.timeout_ms.clamp(50, 120_000);
        self.max_processes = self.max_processes.clamp(1, 256);
        self.max_file_size_bytes = self.max_file_size_bytes.clamp(1024, 100 * 1024 * 1024);
        self.max_output_bytes = self.max_output_bytes.clamp(1024, 4 * 1024 * 1024);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    pub language: Language,
    pub code: String,
    #[serde(default)]
    pub stdin: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub allow_network: bool,
    pub limits: Option<ExecutionLimits>,
    #[serde(default)]
    pub mode: Option<ExecutionMode>,
    #[serde(default)]
    pub test_cases: Vec<TestCase>,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    TimedOut,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCaseResult {
    pub stdin: String,
    pub stdout: String,
    pub stderr: String,
    pub passed: Option<bool>,
    pub exit_code: i32,
    pub duration_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration_ms: u128,
    pub sandbox_backend: String,
    #[serde(default)]
    pub test_results: Vec<TestCaseResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub id: Uuid,
    pub tenant_id: String,
    pub status: ExecutionStatus,
    pub request: ExecutionRequest,
    pub limits: ExecutionLimits,
    pub output: Option<ExecutionOutput>,
    pub error: Option<String>,
    #[serde(default)]
    pub events: Vec<ExecutionEvent>,
    pub created_at_ms: u64,
    pub started_at_ms: Option<u64>,
    pub finished_at_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionEvent {
    pub ts_ms: u64,
    pub stage: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateExecutionResponse {
    pub id: Uuid,
    pub status: ExecutionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSummaryResponse {
    pub id: Uuid,
    pub tenant_id: String,
    pub status: ExecutionStatus,
    pub created_at_ms: u64,
    pub started_at_ms: Option<u64>,
    pub finished_at_ms: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::ExecutionLimits;

    #[test]
    fn normalizes_limits_to_safe_bounds() {
        let normalized = ExecutionLimits {
            cpu_cores: 0.01,
            memory_mb: 1,
            timeout_ms: 1,
            max_processes: 999,
            max_file_size_bytes: 1,
            max_output_bytes: 99_000_000,
        }
        .normalized();

        assert_eq!(normalized.cpu_cores, 0.1);
        assert_eq!(normalized.memory_mb, 32);
        assert_eq!(normalized.timeout_ms, 50);
        assert_eq!(normalized.max_processes, 256);
        assert_eq!(normalized.max_file_size_bytes, 1024);
        assert_eq!(normalized.max_output_bytes, 4 * 1024 * 1024);
    }
}

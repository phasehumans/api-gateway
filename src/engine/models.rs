use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
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
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
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
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
}

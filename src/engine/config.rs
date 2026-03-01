use std::{
    collections::HashMap,
    env,
    net::SocketAddr,
    path::PathBuf,
    str::FromStr,
};

use crate::engine::models::ExecutionLimits;

#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub bind_addr: SocketAddr,
    pub worker_count: usize,
    pub queue_capacity: usize,
    pub sandbox_backend: SandboxBackendKind,
    pub default_limits: ExecutionLimits,
    pub api_keys: HashMap<String, String>,
    pub rate_limit_per_minute: u32,
    pub persistence_path: Option<PathBuf>,
    pub log_level: String,
}

impl EngineConfig {
    pub fn from_env() -> Self {
        Self {
            bind_addr: env_parse("BIND_ADDR", "0.0.0.0:8080"),
            worker_count: env_parse("WORKER_COUNT", 4usize),
            queue_capacity: env_parse("QUEUE_CAPACITY", 1024usize),
            sandbox_backend: env_parse("SANDBOX_BACKEND", SandboxBackendKind::Docker),
            default_limits: ExecutionLimits {
                cpu_cores: env_parse("DEFAULT_CPU_CORES", 0.5),
                memory_mb: env_parse("DEFAULT_MEMORY_MB", 256),
                timeout_ms: env_parse("DEFAULT_TIMEOUT_MS", 3000),
                max_processes: env_parse("DEFAULT_MAX_PROCESSES", 32),
                max_file_size_bytes: env_parse("DEFAULT_MAX_FILE_SIZE_BYTES", 1024 * 1024),
                max_output_bytes: env_parse("DEFAULT_MAX_OUTPUT_BYTES", 64 * 1024),
            },
            api_keys: parse_api_keys(
                &env::var("API_KEYS").unwrap_or_else(|_| "default:dev-key".to_string()),
            ),
            rate_limit_per_minute: env_parse("RATE_LIMIT_PER_MINUTE", 120u32),
            persistence_path: env::var("PERSIST_RESULTS_PATH").ok().map(PathBuf::from),
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum SandboxBackendKind {
    #[default]
    Docker,
    Process,
}

impl FromStr for SandboxBackendKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "docker" => Ok(Self::Docker),
            "process" => Ok(Self::Process),
            _ => Err(format!("unsupported sandbox backend: {s}")),
        }
    }
}

fn parse_api_keys(input: &str) -> HashMap<String, String> {
    let mut keys = HashMap::new();
    for raw in input.split(',') {
        let entry = raw.trim();
        if entry.is_empty() {
            continue;
        }
        if let Some((tenant, key)) = entry.split_once(':') {
            keys.insert(key.to_string(), tenant.to_string());
        }
    }
    if keys.is_empty() {
        keys.insert("dev-key".to_string(), "default".to_string());
    }
    keys
}

fn env_parse<T>(key: &str, default: T) -> T
where
    T: FromStr,
{
    env::var(key)
        .ok()
        .and_then(|raw| raw.parse().ok())
        .unwrap_or(default)
}

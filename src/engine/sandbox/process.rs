use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    path::PathBuf,
    process::Stdio,
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::Context;
use async_trait::async_trait;
use dashmap::DashMap;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
};

use crate::engine::sandbox::{LanguageSpec, RunSpec, SandboxBackend, SandboxResult};

pub struct ProcessSandbox {
    compile_cache: Arc<DashMap<u64, PathBuf>>,
}

impl ProcessSandbox {
    pub fn new() -> Self {
        Self {
            compile_cache: Arc::new(DashMap::new()),
        }
    }
}

#[async_trait]
impl SandboxBackend for ProcessSandbox {
    fn name(&self) -> &'static str {
        "process"
    }

    async fn execute(&self, spec: RunSpec) -> anyhow::Result<SandboxResult> {
        if spec.request.code.as_bytes().len() as u64 > spec.limits.max_file_size_bytes {
            anyhow::bail!("source exceeds configured file size limit");
        }

        let lang = LanguageSpec::for_language(&spec.request.language);
        let work_dir = std::env::temp_dir().join(format!(
            "unsafe-process-{}-{}",
            spec.id.as_simple(),
            now_nanos()
        ));
        tokio::fs::create_dir_all(&work_dir).await?;

        let started = Instant::now();
        let source_path = lang.source_path(&work_dir);
        tokio::fs::write(&source_path, spec.request.code.as_bytes()).await?;

        let mut cmd = if let Some(interpreter) = lang.process_interpreted_cmd {
            let mut cmd = Command::new(interpreter);
            cmd.arg(&source_path);
            cmd.args(&spec.request.args);
            cmd
        } else {
            let bin_path = self
                .compile_or_get_cached(&spec, &lang, &source_path)
                .await?;
            let mut cmd = Command::new(bin_path);
            cmd.args(&spec.request.args);
            cmd
        };

        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .context("failed to spawn process backend command")?;
        if let Some(mut stdin) = child.stdin.take() {
            let stdin_bytes = spec.request.stdin.into_bytes();
            tokio::spawn(async move {
                let _ = stdin.write_all(&stdin_bytes).await;
            });
        }

        let stdout = child.stdout.take().context("missing stdout pipe")?;
        let stderr = child.stderr.take().context("missing stderr pipe")?;
        let limit = spec.limits.max_output_bytes;
        let stdout_task = tokio::spawn(async move { read_limited(stdout, limit).await });
        let stderr_task = tokio::spawn(async move { read_limited(stderr, limit).await });

        let wait_result =
            tokio::time::timeout(Duration::from_millis(spec.limits.timeout_ms), child.wait()).await;

        let (status_code, timed_out) = match wait_result {
            Ok(Ok(status)) => (status.code().unwrap_or(-1), false),
            Ok(Err(err)) => {
                cleanup_dir(&work_dir).await;
                return Err(err).context("process backend command wait failed");
            }
            Err(_) => {
                let _ = child.kill().await;
                (-1, true)
            }
        };

        let stdout = stdout_task.await.unwrap_or_default();
        let stderr = stderr_task.await.unwrap_or_default();
        cleanup_dir(&work_dir).await;

        Ok(SandboxResult {
            stdout: String::from_utf8_lossy(&stdout).to_string(),
            stderr: String::from_utf8_lossy(&stderr).to_string(),
            exit_code: status_code,
            duration_ms: started.elapsed().as_millis(),
            timed_out,
        })
    }
}

impl ProcessSandbox {
    async fn compile_or_get_cached(
        &self,
        spec: &RunSpec,
        lang: &LanguageSpec,
        source_path: &std::path::Path,
    ) -> anyhow::Result<PathBuf> {
        let mut hasher = DefaultHasher::new();
        lang.source_name.hash(&mut hasher);
        spec.request.code.hash(&mut hasher);
        let key = hasher.finish();

        if let Some(cached) = self.compile_cache.get(&key) {
            if cached.exists() {
                return Ok(cached.value().clone());
            }
        }

        let cache_dir = std::env::temp_dir().join("unsafe-process-compile-cache");
        tokio::fs::create_dir_all(&cache_dir).await?;
        let bin_path = cache_dir.join(format!("compiled-{}", key));
        let compiler = lang
            .process_compile_cmd
            .context("compile command missing for compiled language")?;

        let mut compile = Command::new(compiler);
        compile.arg(source_path);
        if compiler == "rustc" {
            compile.args(["-O", "-o"]);
        } else {
            compile.args(["-O2", "-o"]);
        }
        compile.arg(&bin_path);
        let output = compile.output().await?;
        if !output.status.success() {
            anyhow::bail!("{}", String::from_utf8_lossy(&output.stderr).to_string());
        }
        self.compile_cache.insert(key, bin_path.clone());
        Ok(bin_path)
    }
}

fn now_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

async fn cleanup_dir(path: &std::path::Path) {
    let _ = tokio::fs::remove_dir_all(path).await;
}

async fn read_limited<R>(mut reader: R, limit: usize) -> Vec<u8>
where
    R: tokio::io::AsyncRead + Unpin,
{
    let mut out = Vec::with_capacity(limit.min(8192));
    let mut chunk = [0u8; 4096];
    loop {
        match reader.read(&mut chunk).await {
            Ok(0) => break,
            Ok(n) => {
                if out.len() < limit {
                    let remaining = limit - out.len();
                    out.extend_from_slice(&chunk[..remaining.min(n)]);
                }
            }
            Err(_) => break,
        }
    }
    out
}

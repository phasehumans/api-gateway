use std::{
    path::{Path, PathBuf},
    process::Stdio,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::Context;
use async_trait::async_trait;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
};

use crate::engine::sandbox::{LanguageSpec, RunSpec, SandboxBackend, SandboxResult};

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

    async fn execute(&self, spec: RunSpec) -> anyhow::Result<SandboxResult> {
        if spec.request.code.as_bytes().len() as u64 > spec.limits.max_file_size_bytes {
            anyhow::bail!("source exceeds configured file size limit");
        }

        let lang = LanguageSpec::for_language(&spec.request.language);
        let work_dir = make_work_dir(spec.id)?;
        write_source(&work_dir, &lang, &spec.request.code).await?;

        let container_name = format!("exec-{}-{}", spec.id.as_simple(), now_nanos() % 1_000_000);

        let mut args: Vec<String> = vec![
            "run".to_string(),
            "--name".to_string(),
            container_name.clone(),
            "--rm".to_string(),
            "--init".to_string(),
            "--cpus".to_string(),
            spec.limits.cpu_cores.to_string(),
            "--memory".to_string(),
            format!("{}m", spec.limits.memory_mb),
            "--pids-limit".to_string(),
            spec.limits.max_processes.to_string(),
            "--ulimit".to_string(),
            format!("nproc={}", spec.limits.max_processes),
            "--ulimit".to_string(),
            format!("fsize={}", spec.limits.max_file_size_bytes),
            "--read-only".to_string(),
            "--tmpfs".to_string(),
            "/tmp:rw,nosuid,nodev,noexec,size=64m".to_string(),
            "--mount".to_string(),
            format!(
                "type=bind,src={},dst=/workspace,readonly",
                normalize_mount_path(&work_dir)
            ),
            "-w".to_string(),
            "/workspace".to_string(),
            "--security-opt".to_string(),
            "no-new-privileges".to_string(),
            "--cap-drop".to_string(),
            "ALL".to_string(),
        ];
        if !spec.request.allow_network {
            args.push("--network".to_string());
            args.push("none".to_string());
        }

        args.push(lang.docker_image.to_string());
        args.push("sh".to_string());
        args.push("-lc".to_string());
        args.push(lang.docker_script.to_string());
        args.push("--".to_string());
        args.extend(spec.request.args.iter().cloned());

        let started = Instant::now();
        let mut cmd = Command::new("docker");
        cmd.args(args);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn().context("failed to spawn docker run")?;
        if let Some(mut stdin) = child.stdin.take() {
            let stdin_bytes = spec.request.stdin.into_bytes();
            tokio::spawn(async move {
                let _ = stdin.write_all(&stdin_bytes).await;
            });
        }

        let stdout = child.stdout.take().context("missing stdout pipe")?;
        let stderr = child.stderr.take().context("missing stderr pipe")?;
        let out_limit = spec.limits.max_output_bytes;
        let stdout_task = tokio::spawn(async move { read_limited(stdout, out_limit).await });
        let stderr_task = tokio::spawn(async move { read_limited(stderr, out_limit).await });

        let wait_result =
            tokio::time::timeout(Duration::from_millis(spec.limits.timeout_ms), child.wait()).await;

        let (status_code, timed_out) = match wait_result {
            Ok(Ok(status)) => (status.code().unwrap_or(-1), false),
            Ok(Err(err)) => {
                cleanup_container(&container_name).await;
                cleanup_dir(&work_dir).await;
                return Err(err).context("docker wait failed");
            }
            Err(_) => {
                let _ = child.kill().await;
                cleanup_container(&container_name).await;
                (-1, true)
            }
        };

        let stdout_bytes = stdout_task.await.unwrap_or_default();
        let stderr_bytes = stderr_task.await.unwrap_or_default();

        cleanup_dir(&work_dir).await;

        Ok(SandboxResult {
            stdout: String::from_utf8_lossy(&stdout_bytes).to_string(),
            stderr: String::from_utf8_lossy(&stderr_bytes).to_string(),
            exit_code: status_code,
            duration_ms: started.elapsed().as_millis(),
            timed_out,
        })
    }
}

async fn write_source(work_dir: &Path, lang: &LanguageSpec, source: &str) -> anyhow::Result<()> {
    tokio::fs::create_dir_all(work_dir)
        .await
        .with_context(|| format!("failed to create work dir {}", work_dir.display()))?;
    tokio::fs::write(lang.source_path(work_dir), source.as_bytes())
        .await
        .context("failed to write source file")?;
    Ok(())
}

fn make_work_dir(id: uuid::Uuid) -> anyhow::Result<PathBuf> {
    let dir = std::env::temp_dir().join(format!("sandbox-{}-{}", id.as_simple(), now_nanos()));
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create work dir {}", dir.display()))?;
    Ok(dir)
}

fn normalize_mount_path(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}

fn now_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

async fn cleanup_container(name: &str) {
    let _ = Command::new("docker")
        .arg("rm")
        .arg("-f")
        .arg(name)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await;
}

async fn cleanup_dir(path: &Path) {
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

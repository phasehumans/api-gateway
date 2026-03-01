use std::sync::Arc;

use tokio::sync::{Mutex, mpsc::Receiver};
// worker pools

use crate::engine::{
    metrics::MetricsRegistry,
    models::{ExecutionStatus, TestCaseResult},
    sandbox::{RunSpec, SandboxBackend, SandboxResult},
    store::ExecutionStore,
};

pub fn spawn_worker_pool(
    workers: usize,
    receiver: Arc<Mutex<Receiver<crate::engine::queue::QueuedJob>>>,
    store: Arc<ExecutionStore>,
    metrics: Arc<MetricsRegistry>,
    sandbox: Arc<dyn SandboxBackend>,
) {
    for worker_id in 0..workers {
        let receiver = receiver.clone();
        let store = store.clone();
        let metrics = metrics.clone();
        let sandbox = sandbox.clone();
        tokio::spawn(async move {
            worker_loop(worker_id, receiver, store, metrics, sandbox).await;
        });
    }
}

async fn worker_loop(
    worker_id: usize,
    receiver: Arc<Mutex<Receiver<crate::engine::queue::QueuedJob>>>,
    store: Arc<ExecutionStore>,
    metrics: Arc<MetricsRegistry>,
    sandbox: Arc<dyn SandboxBackend>,
) {
    loop {
        let job = {
            let mut locked = receiver.lock().await;
            locked.recv().await
        };
        let Some(job) = job else {
            tracing::info!(worker_id, "job queue closed, worker exiting");
            break;
        };

        tracing::info!(worker_id, execution_id = %job.id, "starting execution");
        metrics.started();
        store.mark_running(job.id);
        store.append_event(job.id, "worker", format!("worker-{worker_id} claimed job"));

        let job_id = job.id;
        let request = job.request.clone();
        let limits = job.limits.clone();
        let base_spec = RunSpec::from(job);

        let result = if request.test_cases.is_empty() {
            sandbox
                .execute(base_spec)
                .await
                .map(|single| (single, Vec::new()))
        } else {
            execute_test_cases(job_id, request, limits, sandbox.clone()).await
        };

        match result {
            Ok((result, test_results)) => {
                let status = if result.timed_out {
                    metrics.timed_out();
                    ExecutionStatus::TimedOut
                } else if result.exit_code == 0 {
                    ExecutionStatus::Succeeded
                } else {
                    metrics.failed();
                    ExecutionStatus::Failed
                };

                metrics.completed();
                store
                    .mark_finished(
                        job_id,
                        status,
                        Some(crate::engine::models::ExecutionOutput {
                            stdout: result.stdout,
                            stderr: result.stderr,
                            exit_code: result.exit_code,
                            duration_ms: result.duration_ms,
                            sandbox_backend: sandbox.name().to_string(),
                            test_results,
                        }),
                        None,
                    )
                    .await;
            }
            Err(err) => {
                store.append_event(job_id, "sandbox_error", err.to_string());
                metrics.failed();
                store
                    .mark_finished(job_id, ExecutionStatus::Failed, None, Some(err.to_string()))
                    .await;
            }
        }
    }
}

async fn execute_test_cases(
    id: uuid::Uuid,
    request: crate::engine::models::ExecutionRequest,
    limits: crate::engine::models::ExecutionLimits,
    sandbox: Arc<dyn SandboxBackend>,
) -> anyhow::Result<(SandboxResult, Vec<TestCaseResult>)> {
    let test_cases = request.test_cases.clone();
    let mut test_results = Vec::with_capacity(test_cases.len());
    let mut final_result = None;

    for case in test_cases {
        let mut request_for_case = request.clone();
        request_for_case.stdin = case.stdin.clone();
        request_for_case.test_cases.clear();
        let spec = RunSpec {
            request: request_for_case,
            limits: limits.clone(),
            id,
        };
        let out = sandbox.execute(spec).await?;
        let passed = case
            .expected_stdout
            .as_ref()
            .map(|expected| out.stdout.trim() == expected.trim());
        test_results.push(TestCaseResult {
            stdin: case.stdin,
            stdout: out.stdout.clone(),
            stderr: out.stderr.clone(),
            passed,
            exit_code: out.exit_code,
            duration_ms: out.duration_ms,
        });
        final_result = Some(out.clone());
        if out.timed_out {
            break;
        }
    }

    let fallback = SandboxResult {
        stdout: String::new(),
        stderr: String::new(),
        exit_code: 0,
        duration_ms: 0,
        timed_out: false,
    };

    Ok((final_result.unwrap_or(fallback), test_results))
}

use std::sync::Arc;

use tokio::sync::{Mutex, mpsc::Receiver};

use crate::engine::{
    metrics::MetricsRegistry,
    models::ExecutionStatus,
    sandbox::{RunSpec, SandboxBackend},
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

        let job_id = job.id;
        match sandbox.execute(RunSpec::from(job)).await {
            Ok(result) => {
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
                store.mark_finished(
                    job_id,
                    status,
                    Some(crate::engine::models::ExecutionOutput {
                        stdout: result.stdout,
                        stderr: result.stderr,
                        exit_code: result.exit_code,
                        duration_ms: result.duration_ms,
                        sandbox_backend: sandbox.name().to_string(),
                        test_results: Vec::new(),
                    }),
                    None,
                )
                .await;
            }
            Err(err) => {
                metrics.failed();
                store.mark_finished(
                    job_id,
                    ExecutionStatus::Failed,
                    None,
                    Some(err.to_string()),
                )
                .await;
            }
        }
    }
}

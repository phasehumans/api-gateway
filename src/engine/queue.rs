use std::sync::Arc;

use tokio::sync::{
    Mutex,
    mpsc::{self, Receiver, Sender},
};
use uuid::Uuid;

use crate::engine::{
    error::EngineError,
    metrics::MetricsRegistry,
    models::{ExecutionLimits, ExecutionRequest},
};

#[derive(Debug, Clone)]
pub struct QueuedJob {
    pub id: Uuid,
    pub tenant_id: String,
    pub request: ExecutionRequest,
    pub limits: ExecutionLimits,
}

#[derive(Clone)]
pub struct Scheduler {
    sender: Sender<QueuedJob>,
    receiver: Arc<Mutex<Receiver<QueuedJob>>>,
    metrics: Arc<MetricsRegistry>,
}

impl Scheduler {
    pub fn new(capacity: usize, metrics: Arc<MetricsRegistry>) -> Self {
        let (sender, receiver) = mpsc::channel(capacity);
        Self {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
            metrics,
        }
    }

    pub async fn submit(&self, job: QueuedJob) -> Result<(), EngineError> {
        self.sender
            .send(job)
            .await
            .map_err(|_| EngineError::QueueFull)?;
        self.metrics.submitted();
        Ok(())
    }

    pub fn receiver(&self) -> Arc<Mutex<Receiver<QueuedJob>>> {
        self.receiver.clone()
    }
}

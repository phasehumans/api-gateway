use std::{path::PathBuf, sync::Arc};

use dashmap::DashMap;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::engine::models::{
    ExecutionEvent, ExecutionOutput, ExecutionRecord, ExecutionRequest, ExecutionStatus,
};

#[derive(Clone)]
pub struct ExecutionStore {
    records: Arc<DashMap<Uuid, ExecutionRecord>>,
    persistence_path: Option<PathBuf>,
    write_lock: Arc<Mutex<()>>,
}

impl ExecutionStore {
    pub fn new(persistence_path: Option<PathBuf>) -> Self {
        Self {
            records: Arc::new(DashMap::new()),
            persistence_path,
            write_lock: Arc::new(Mutex::new(())),
        }
    }

    pub fn insert(&self, record: ExecutionRecord) {
        self.records.insert(record.id, record);
    }

    pub fn get(&self, id: &Uuid) -> Option<ExecutionRecord> {
        self.records.get(id).map(|e| e.value().clone())
    }

    pub fn remove(&self, id: &Uuid) {
        self.records.remove(id);
    }

    pub fn mark_running(&self, id: Uuid) {
        if let Some(mut entry) = self.records.get_mut(&id) {
            let now = now_ms();
            entry.status = ExecutionStatus::Running;
            entry.started_at_ms = Some(now);
            entry.events.push(ExecutionEvent {
                ts_ms: now,
                stage: "running".to_string(),
                message: "worker started execution".to_string(),
            });
        }
    }

    pub fn append_event(&self, id: Uuid, stage: impl Into<String>, message: impl Into<String>) {
        if let Some(mut entry) = self.records.get_mut(&id) {
            let now = now_ms();
            entry.events.push(ExecutionEvent {
                ts_ms: now,
                stage: stage.into(),
                message: message.into(),
            });
        }
    }

    pub async fn mark_finished(
        &self,
        id: Uuid,
        status: ExecutionStatus,
        output: Option<ExecutionOutput>,
        error: Option<String>,
    ) {
        let snapshot = if let Some(mut entry) = self.records.get_mut(&id) {
            let now = now_ms();
            entry.status = status;
            entry.output = output;
            entry.error = error;
            entry.finished_at_ms = Some(now);
            entry.events.push(ExecutionEvent {
                ts_ms: now,
                stage: "finished".to_string(),
                message: "execution finalized".to_string(),
            });
            Some(entry.clone())
        } else {
            None
        };

        if let (Some(path), Some(record)) = (&self.persistence_path, snapshot) {
            let _guard = self.write_lock.lock().await;
            let line = match serde_json::to_string(&record) {
                Ok(line) => line,
                Err(_) => return,
            };
            let mut options = tokio::fs::OpenOptions::new();
            options.create(true).append(true);
            if let Ok(mut file) = options.open(path).await {
                let _ = file.write_all(line.as_bytes()).await;
                let _ = file.write_all(b"\n").await;
            }
        }
    }

    pub fn create_record(
        &self,
        id: Uuid,
        tenant_id: String,
        request: ExecutionRequest,
        limits: crate::engine::models::ExecutionLimits,
    ) -> ExecutionRecord {
        let now = now_ms();
        ExecutionRecord {
            id,
            tenant_id,
            status: ExecutionStatus::Queued,
            request,
            limits,
            output: None,
            error: None,
            events: vec![ExecutionEvent {
                ts_ms: now,
                stage: "queued".to_string(),
                message: "execution accepted and queued".to_string(),
            }],
            created_at_ms: now,
            started_at_ms: None,
            finished_at_ms: None,
        }
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

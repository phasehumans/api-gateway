use std::{path::PathBuf, sync::Arc};

use chrono::Utc;
use dashmap::DashMap;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::engine::models::{
    ExecutionOutput, ExecutionRecord, ExecutionRequest, ExecutionStatus,
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

    pub fn mark_running(&self, id: Uuid) {
        if let Some(mut entry) = self.records.get_mut(&id) {
            entry.status = ExecutionStatus::Running;
            entry.started_at = Some(Utc::now());
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
            entry.status = status;
            entry.output = output;
            entry.error = error;
            entry.finished_at = Some(Utc::now());
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
                let _ = tokio::io::AsyncWriteExt::write_all(&mut file, line.as_bytes()).await;
                let _ = tokio::io::AsyncWriteExt::write_all(&mut file, b"\n").await;
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
        ExecutionRecord {
            id,
            tenant_id,
            status: ExecutionStatus::Queued,
            request,
            limits,
            output: None,
            error: None,
            created_at: Utc::now(),
            started_at: None,
            finished_at: None,
        }
    }
}

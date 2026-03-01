use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Default)]
pub struct MetricsRegistry {
    submitted_total: AtomicU64,
    started_total: AtomicU64,
    completed_total: AtomicU64,
    failed_total: AtomicU64,
    timed_out_total: AtomicU64,
    queue_depth: AtomicU64,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn submitted(&self) {
        self.submitted_total.fetch_add(1, Ordering::Relaxed);
        self.queue_depth.fetch_add(1, Ordering::Relaxed);
    }

    pub fn started(&self) {
        self.started_total.fetch_add(1, Ordering::Relaxed);
        self.queue_depth.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn completed(&self) {
        self.completed_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn failed(&self) {
        self.failed_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn timed_out(&self) {
        self.timed_out_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn render_prometheus(&self) -> String {
        format!(
            concat!(
                "# TYPE execution_submitted_total counter\n",
                "execution_submitted_total {}\n",
                "# TYPE execution_started_total counter\n",
                "execution_started_total {}\n",
                "# TYPE execution_completed_total counter\n",
                "execution_completed_total {}\n",
                "# TYPE execution_failed_total counter\n",
                "execution_failed_total {}\n",
                "# TYPE execution_timed_out_total counter\n",
                "execution_timed_out_total {}\n",
                "# TYPE execution_queue_depth gauge\n",
                "execution_queue_depth {}\n"
            ),
            self.submitted_total.load(Ordering::Relaxed),
            self.started_total.load(Ordering::Relaxed),
            self.completed_total.load(Ordering::Relaxed),
            self.failed_total.load(Ordering::Relaxed),
            self.timed_out_total.load(Ordering::Relaxed),
            self.queue_depth.load(Ordering::Relaxed),
        )
    }
}

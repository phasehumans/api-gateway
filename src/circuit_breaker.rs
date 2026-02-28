use dashmap::DashMap;
use std::{
    sync::Arc,
    time::{
        Duration,
        Instant,
    },
};
use tokio::sync::Mutex;

use crate::config::CircuitBreakerConfig;

#[derive(Clone)]
pub struct CircuitBreaker {
    cfg: CircuitBreakerConfig,
    states: Arc<DashMap<String, Arc<Mutex<BreakerState>>>>,
}

#[derive(Debug)]
struct BreakerState {
    phase: BreakerPhase,
    consecutive_failures: u32,
    half_open_in_flight: u32,
}

#[derive(Debug)]
enum BreakerPhase {
    Closed,
    Open { until: Instant },
    HalfOpen,
}

impl CircuitBreaker {
    pub fn new(cfg: CircuitBreakerConfig) -> Self {
        Self {
            cfg,
            states: Arc::new(DashMap::new()),
        }
    }

    fn state_for(&self, service: &str) -> Arc<Mutex<BreakerState>> {
        self.states
            .entry(service.to_string())
            .or_insert_with(|| {
                Arc::new(Mutex::new(BreakerState {
                    phase: BreakerPhase::Closed,
                    consecutive_failures: 0,
                    half_open_in_flight: 0,
                }))
            })
            .clone()
    }

    pub async fn allow_request(&self, service: &str) -> bool {
        let state = self.state_for(service);
        let mut state = state.lock().await;
        let now = Instant::now();

        match state.phase {
            BreakerPhase::Closed => true,
            BreakerPhase::Open { until } => {
                if now >= until {
                    state.phase = BreakerPhase::HalfOpen;
                    state.half_open_in_flight = 1;
                    true
                } else {
                    false
                }
            }
            BreakerPhase::HalfOpen => {
                if state.half_open_in_flight < self.cfg.half_open_max_requests {
                    state.half_open_in_flight += 1;
                    true
                } else {
                    false
                }
            }
        }
    }

    pub async fn record_success(&self, service: &str) {
        let state = self.state_for(service);
        let mut state = state.lock().await;
        state.phase = BreakerPhase::Closed;
        state.consecutive_failures = 0;
        state.half_open_in_flight = 0;
    }

    pub async fn record_failure(&self, service: &str) {
        let state = self.state_for(service);
        let mut state = state.lock().await;

        match state.phase {
            BreakerPhase::Closed => {
                state.consecutive_failures += 1;
                if state.consecutive_failures >= self.cfg.failure_threshold {
                    state.phase = BreakerPhase::Open {
                        until: Instant::now() + Duration::from_secs(self.cfg.open_seconds),
                    };
                    state.consecutive_failures = 0;
                    state.half_open_in_flight = 0;
                }
            }
            BreakerPhase::HalfOpen => {
                state.phase = BreakerPhase::Open {
                    until: Instant::now() + Duration::from_secs(self.cfg.open_seconds),
                };
                state.consecutive_failures = 0;
                state.half_open_in_flight = 0;
            }
            BreakerPhase::Open { .. } => {}
        }
    }

    pub async fn is_open(&self, service: &str) -> bool {
        let state = self.state_for(service);
        let mut state = state.lock().await;

        match state.phase {
            BreakerPhase::Open { until } => {
                if Instant::now() >= until {
                    state.phase = BreakerPhase::HalfOpen;
                    false
                } else {
                    true
                }
            }
            _ => false,
        }
    }
}

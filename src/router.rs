use std::sync::atomic::{
    AtomicU64,
    Ordering,
};

use crate::{
    config::{RouteConfig, RoutingConfig, UpstreamConfig},
    upstream::UpstreamSnapshot,
};

#[derive(Debug, Clone)]
pub struct RoutingCandidate {
    pub upstream: UpstreamConfig,
    pub stats: UpstreamSnapshot,
    pub breaker_open: bool,
}

pub trait RoutingStrategy: Send + Sync {
    fn rank(&self, route: &RouteConfig, candidates: &[RoutingCandidate]) -> Vec<String>;
}

pub struct IntelligentRouter {
    cfg: RoutingConfig,
    rr_counter: AtomicU64,
}

impl IntelligentRouter {
    pub fn new(cfg: RoutingConfig) -> Self {
        Self {
            cfg,
            rr_counter: AtomicU64::new(0),
        }
    }

    fn score(&self, candidate: &RoutingCandidate, seed: u64) -> i64 {
        if candidate.breaker_open {
            return -1_000_000;
        }

        let base = candidate.upstream.weight.max(1) as i64 * 1_000;
        let in_flight_penalty =
            candidate.stats.in_flight as i64 * self.cfg.in_flight_penalty as i64;
        let failure_penalty =
            candidate.stats.consecutive_failures as i64 * self.cfg.failure_penalty as i64;
        let latency_penalty = if self.cfg.prefer_low_latency {
            candidate.stats.avg_latency_ms as i64
        } else {
            0
        };

        // Lightweight bias to avoid repeatedly selecting same node when scores are close.
        let rr_bias = ((seed % candidate.upstream.weight.max(1) as u64) as i64) * 8;

        base + rr_bias - in_flight_penalty - failure_penalty - latency_penalty
    }
}

impl RoutingStrategy for IntelligentRouter {
    fn rank(&self, _route: &RouteConfig, candidates: &[RoutingCandidate]) -> Vec<String> {
        let seed = self.rr_counter.fetch_add(1, Ordering::Relaxed);

        let mut scored = candidates
            .iter()
            .enumerate()
            .map(|(idx, c)| (self.score(c, seed + idx as u64), c.upstream.name.clone()))
            .collect::<Vec<_>>();

        scored.sort_by(|a, b| b.0.cmp(&a.0));

        scored.into_iter().map(|(_, name)| name).collect()
    }
}

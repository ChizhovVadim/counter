use crate::domain::{CancelToken, LimitsType, SearchInfo};
use std::time::{Duration, Instant};

pub struct TimeManager {
    start: Instant,
    limits: LimitsType,
    cancel: CancelToken,
}

impl TimeManager {
    pub fn new(limits: LimitsType, cancel: CancelToken) -> Self {
        return TimeManager {
            start: Instant::now(),
            limits: limits,
            cancel,
        };
    }

    pub fn check_timeout(&self, nodes: u64) -> bool {
        if let Some(fixed_time) = self.limits.fixed_time {
            if self.start.elapsed() >= fixed_time {
                self.cancel.cancel();
            }
        }
        if let Some(fixed_nodes) = self.limits.fixed_nodes {
            if nodes >= fixed_nodes {
                self.cancel.cancel();
            }
        }
        return self.cancel.is_cancelled();
    }

    pub fn elapsed(&self) -> Duration {
        return self.start.elapsed();
    }

    pub fn iteration_complete(&mut self, si: &SearchInfo) {
        if self.limits.infinite {
            return;
        }
        if let Some(fixed_depth) = self.limits.fixed_depth {
            if si.depth >= fixed_depth as usize {
                self.cancel.cancel();
            }
        }
    }
}

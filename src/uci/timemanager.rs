use crate::types;
use crate::uci;
use std::sync;
use std::sync::atomic;
use std::time::{Duration, Instant};

pub struct TimeManager {
    start: Instant,
    abort: sync::Arc<sync::atomic::AtomicBool>,
    limits: uci::LimitsType,
}

impl TimeManager {
    pub fn default() -> Self {
        return TimeManager {
            start: Instant::now(),
            limits: uci::LimitsType::Infinite,
            abort: sync::Arc::new(sync::atomic::AtomicBool::new(false)),
        };
    }
    pub fn new(limits: uci::LimitsType, abort: sync::Arc<sync::atomic::AtomicBool>) -> Self {
        return TimeManager {
            start: Instant::now(),
            limits: limits,
            abort: abort,
        };
    }

    fn cancel(&self) {
        self.abort.store(true, atomic::Ordering::SeqCst);
    }
}

impl types::ITimeManager for TimeManager {
    fn elapsed(&self) -> Duration {
        return self.start.elapsed();
    }
    fn check_timeout(&self) -> bool {
        match self.limits {
            uci::LimitsType::FixedTime(fixed_time) => {
                if self.start.elapsed() >= fixed_time {
                    self.cancel();
                }
            }
            _ => (),
        }
        return self.abort.load(sync::atomic::Ordering::Relaxed);
    }
    fn iteration_complete(&mut self, si: &types::SearchInfo) {
        if let uci::LimitsType::FixedDepth(fixed_depth) = self.limits {
            if si.depth>=fixed_depth {
                self.cancel();
            }
        }
        if si.nodes >= 500_000 {
            uci::parse::print_uci_search_info(&si);
        }
    }
}

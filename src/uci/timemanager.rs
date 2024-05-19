use crate::types;
use crate::uci;
use std::sync;
use std::sync::atomic;
use std::time::{Duration, Instant};

pub struct TimeManager {
    start: Instant,
    abort: sync::Arc<sync::atomic::AtomicBool>,
    fixed_time: Option<Duration>,
    fixed_depth: Option<usize>,
    fixed_nodes: Option<u64>,
}

impl TimeManager {
    pub fn default() -> Self {
        return TimeManager {
            start: Instant::now(),
            abort: sync::Arc::new(sync::atomic::AtomicBool::new(false)),
            fixed_time: None,
            fixed_depth: None,
            fixed_nodes: None,
        };
    }
    pub fn new(
        abort: sync::Arc<sync::atomic::AtomicBool>,
        fixed_time: Option<Duration>,
        fixed_depth: Option<usize>,
        fixed_nodes: Option<u64>,
    ) -> Self {
        return TimeManager {
            start: Instant::now(),
            abort: abort,
            fixed_time: fixed_time,
            fixed_depth: fixed_depth,
            fixed_nodes: fixed_nodes,
        };
    }
    pub fn tournament(
        abort: sync::Arc<sync::atomic::AtomicBool>,
        main: Duration,
        inc: Option<Duration>,
        moves: Option<usize>,
    ) -> Self {
        let fixed_time = compute_time_limit(main, inc, moves);
        return TimeManager::new(abort, Some(fixed_time), None, None);
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
        if let Some(fixed_time) = self.fixed_time {
            if self.start.elapsed() >= fixed_time {
                self.cancel();
            }
        }
        return self.abort.load(sync::atomic::Ordering::Relaxed);
    }
    fn iteration_complete(&mut self, si: &types::SearchInfo) {
        if let Some(fixed_depth) = self.fixed_depth {
            if si.depth >= fixed_depth {
                self.cancel();
            }
        }
        if si.nodes >= 500_000 {
            uci::parse::print_uci_search_info(&si);
        }
    }
}

fn compute_time_limit(main: Duration, inc: Option<Duration>, moves: Option<usize>) -> Duration {
    const MOVE_OVERHEAD_MS: u128 = 20;
    const DEFAULT_MOVES_TO_GO: usize = 40;

    let main = main.as_millis();
    let inc = inc.unwrap_or(Duration::ZERO).as_millis();
    let time_reserve = ((main + inc) / 10).min(1_000).max(MOVE_OVERHEAD_MS);

    let moves = moves.unwrap_or(DEFAULT_MOVES_TO_GO) as u128;
    let time_limit = (main / moves + inc - MOVE_OVERHEAD_MS)
        .min(main - time_reserve)
        .max(MOVE_OVERHEAD_MS / 2);

    return Duration::from_millis(time_limit as u64);
}

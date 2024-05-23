use crate::types;
use crate::uci;
use std::sync;
use std::sync::atomic;
use std::time::{Duration, Instant};

pub struct FixedTimeManager {
    start: Instant,
    abort: sync::Arc<sync::atomic::AtomicBool>,
    fixed_time: Option<Duration>,
    fixed_depth: Option<usize>,
    fixed_nodes: Option<u64>,
}

impl FixedTimeManager {
    pub fn default() -> Self {
        return FixedTimeManager {
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
        return FixedTimeManager {
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
        return FixedTimeManager::new(abort, Some(fixed_time), None, None);
    }
    fn cancel(&self) {
        self.abort.store(true, atomic::Ordering::SeqCst);
    }
}

impl types::ITimeManager for FixedTimeManager {
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
        uci::show_search_progress(si);
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

//----------------------

pub struct SimpleTimeManager {
    start: Instant,
    abort: sync::Arc<sync::atomic::AtomicBool>,
    max_limit: Duration,
    ideal_limit: Duration,
}

impl SimpleTimeManager {
    pub fn new(
        abort: sync::Arc<sync::atomic::AtomicBool>,
        main: Duration,
        inc: Option<Duration>,
        moves: Option<usize>,
    ) -> Self {
        let (ideal_limit, max_limit) = compute_time_limits(main, inc, moves);
        return SimpleTimeManager {
            start: Instant::now(),
            abort: abort,
            max_limit: max_limit,
            ideal_limit: ideal_limit,
        };
    }
    fn cancel(&self) {
        self.abort.store(true, atomic::Ordering::SeqCst);
    }
}

impl types::ITimeManager for SimpleTimeManager {
    fn elapsed(&self) -> Duration {
        return self.start.elapsed();
    }
    fn check_timeout(&self) -> bool {
        if self.start.elapsed() >= self.max_limit {
            self.cancel();
        }
        return self.abort.load(sync::atomic::Ordering::Relaxed);
    }
    fn iteration_complete(&mut self, si: &types::SearchInfo) {
        if self.start.elapsed() >= self.ideal_limit {
            self.cancel();
        }
        uci::show_search_progress(si);
    }
}

fn compute_time_limits(
    main: Duration,
    inc: Option<Duration>,
    moves: Option<usize>,
) -> (Duration, Duration) {
    const MOVE_OVERHEAD_MS: u128 = 20;
    const DEFAULT_MOVES_TO_GO: usize = 30;

    let main = main.as_millis();
    let inc = inc.unwrap_or(Duration::ZERO).as_millis();
    let time_reserve = (main / 10).min(1_000).max(100);
    let moves = moves.unwrap_or(DEFAULT_MOVES_TO_GO) as u128;

    let ideal_limit = (main / (moves + 1) * 2 / 3 + inc / 2 - MOVE_OVERHEAD_MS)
        .min(main - time_reserve)
        .max(MOVE_OVERHEAD_MS / 2);
    let max_limit = (main / (moves + 1) * 2 + inc - MOVE_OVERHEAD_MS)
        .min(main - time_reserve)
        .max(MOVE_OVERHEAD_MS / 2);

    return (
        Duration::from_millis(ideal_limit as u64),
        Duration::from_millis(max_limit as u64),
    );
}

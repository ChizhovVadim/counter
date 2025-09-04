use crate::chess::Side;
use crate::domain::{CancelToken, LimitsType, SearchInfo};
use std::time::{Duration, Instant};

pub struct TimeManager {
    cancel: CancelToken,
    start: Instant,
    fixed_nodes: Option<u64>,
    fixed_depth: Option<u32>,
    max_usage: Option<Duration>,
    ideal_usage: Option<Duration>,
}

impl Default for TimeManager {
    fn default() -> Self {
        TimeManager {
            cancel: CancelToken::new(),
            start: Instant::now(),
            fixed_nodes: None,
            fixed_depth: None,
            max_usage: None,
            ideal_usage: None,
        }
    }
}

impl TimeManager {
    pub fn new(limits: LimitsType, cancel: CancelToken, side: Side) -> Self {
        let start = Instant::now();

        let tl = limits.tournament;
        if tl.white_time.is_none() || tl.black_time.is_none() {
            return TimeManager {
                cancel,
                start,
                fixed_depth: limits.fixed_depth,
                fixed_nodes: limits.fixed_nodes,
                max_usage: limits.fixed_time,
                ideal_usage: None,
            };
        }

        let (main, inc) = if side == Side::WHITE {
            (tl.white_time.unwrap(), tl.white_increment)
        } else {
            (tl.black_time.unwrap(), tl.black_increment)
        };

        let reserve = (main / 20).clamp(100, 1_000);
        let main = (main - reserve).max(0);

        let (max_usage, ideal_usage) = if let Some(moves) = tl.moves {
            let mut moves = moves as u64;
            if moves > 1 {
                //moves = moves + 5;
                moves = moves + (moves * 4 / 10).max(5);
                moves = moves.min(50);
            }
            let ideal_usage = main / moves + inc;
            let max_usage = 5 * main / moves + inc;
            (max_usage, ideal_usage)
        } else {
            let ideal_usage = main / 50 + inc;
            let max_usage = main / 10 + inc;
            (max_usage, ideal_usage)
        };
        let ideal_usage = ideal_usage.clamp(0, main);
        let max_usage = max_usage.clamp(0, main);

        return TimeManager {
            cancel,
            start,
            fixed_depth: None,
            fixed_nodes: None,
            max_usage: Some(Duration::from_millis(max_usage)),
            ideal_usage: Some(Duration::from_millis(ideal_usage)),
        };
    }

    pub fn check_timeout(&self, nodes: u64) -> bool {
        if let Some(max_usage) = self.max_usage {
            if self.start.elapsed() >= max_usage {
                self.cancel.cancel();
            }
        }
        if let Some(fixed_nodes) = self.fixed_nodes {
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
        if let Some(ideal_usage) = self.ideal_usage {
            if self.start.elapsed() >= ideal_usage {
                self.cancel.cancel();
            }
        }
        if let Some(fixed_depth) = self.fixed_depth {
            if si.depth >= fixed_depth as usize {
                self.cancel.cancel();
            }
        }
    }
}

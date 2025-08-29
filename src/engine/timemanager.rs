use crate::chess::Side;
use crate::domain::{CancelToken, LimitsType, SearchInfo, TournamentLimit};
use std::time::{Duration, Instant};

pub struct TimeManager {
    start: Instant,
    limits: LimitsType,
    cancel: CancelToken,
}

impl Default for TimeManager {
    fn default() -> Self {
        TimeManager::new(LimitsType::default(), CancelToken::new(), Side::WHITE)
    }
}

impl TimeManager {
    pub fn new(mut limits: LimitsType, cancel: CancelToken, side: Side) -> Self {
        // temporary use fixed time
        if limits.tournament.white_time.is_some() || limits.tournament.black_time.is_some() {
            limits = LimitsType::fixed_time(calc_fixed_time(limits.tournament, side))
        }

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

fn calc_fixed_time(tl: TournamentLimit, side: Side) -> Duration {
    let (main, inc) = if side == Side::WHITE {
        (tl.white_time.unwrap(), tl.white_increment)
    } else {
        (tl.black_time.unwrap(), tl.black_increment)
    };
    const DEFAULT_MOVES_TO_GO: u32 = 40;
    let moves = tl
        .moves
        .unwrap_or(DEFAULT_MOVES_TO_GO)
        .min(DEFAULT_MOVES_TO_GO) as u64;
    let reserve = (main / 20).clamp(100, 1_000);
    let main = (main - reserve).max(0);
    let total = inc + (main - inc) / moves;
    let total = total.clamp(0, main);
    return Duration::from_millis(total);
}

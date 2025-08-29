use crate::chess::{Move, Position};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

pub trait IEvaluator {
    fn init(&mut self, pos: &Position);
    fn make_move(&mut self, pos: &Position, mv: Move);
    fn unmake_move(&mut self);
    fn quik_evaluate(&mut self, pos: &Position) -> isize;

    fn evaluate(&mut self, pos: &Position) -> isize {
        self.init(pos);
        return self.quik_evaluate(pos);
    }
}

pub trait IEngine {
    fn get_options(&self) -> Vec<EngineOption>;
    fn set_option(&mut self, name: &str, value: &str);
    fn clear(&mut self);
    fn search(&mut self, search_params: SearchParams) -> SearchInfo;
}

#[derive(Debug, Default)]
pub struct TournamentLimit {
    pub white_time: Option<u64>,
    pub black_time: Option<u64>,
    pub white_increment: u64,
    pub black_increment: u64,
    pub moves: Option<u32>,
}

#[derive(Debug, Default)]
pub struct LimitsType {
    pub infinite: bool,
    pub fixed_nodes: Option<u64>,
    pub fixed_time: Option<Duration>,
    pub fixed_depth: Option<u32>,
    pub tournament: TournamentLimit,
}

impl LimitsType {
    pub fn fixed_time(d: Duration) -> LimitsType {
        return LimitsType {
            fixed_time: Some(d),
            ..Default::default()
        };
    }
}

pub struct SearchParams {
    pub position: Position,
    pub repeats: Vec<u64>,
    pub limits: LimitsType,
    pub cancel: CancelToken,
    pub progress: Box<dyn FnMut(&SearchInfo)>,
}

#[derive(Debug, Clone, Copy)]
pub enum UciScore {
    Mate(isize),
    Centipawns(isize),
}

impl Default for UciScore {
    fn default() -> Self {
        UciScore::Centipawns(0)
    }
}

#[derive(Debug, Default)]
pub struct SearchInfo {
    pub depth: usize,
    pub score: UciScore,
    pub nodes: u64,
    pub duration: Duration,
    pub main_line: Vec<Move>,
}

#[derive(Debug)]
pub struct EngineOption {
    pub name: &'static str,
    pub value: OptionValue,
}

#[derive(Debug)]
pub enum OptionValue {
    Bool(bool),
    Int {
        min: isize,
        max: isize,
        value: isize,
    },
    String(String),
}

#[derive(Clone)]
pub struct CancelToken(Arc<AtomicBool>);

impl CancelToken {
    pub fn new() -> Self {
        CancelToken(Arc::new(AtomicBool::new(false)))
    }

    pub fn is_cancelled(&self) -> bool {
        return self.0.load(Ordering::Relaxed);
    }

    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
}

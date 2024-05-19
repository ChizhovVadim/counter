use crate::chess;
use std::time::Duration;

pub type Score = i16;

pub trait IEvaluator {
    fn init(&mut self, pos: &chess::Position);
    fn make_move(&mut self, history: &chess::History);
    fn unmake_move(&mut self);
    fn quik_evaluate(&mut self, pos: &chess::Position) -> isize;

    fn evaluate(&mut self, pos: &chess::Position) -> isize {
        self.init(pos);
        return self.quik_evaluate(pos);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UciScore {
    Mate(isize),
    Centipawns(isize),
}

#[derive(Debug, Clone)]
pub struct SearchInfo {
    pub depth: usize,
    pub score: UciScore,
    pub nodes: u64,
    pub duration: Duration,
    pub main_line: Vec<chess::Move>,
}

pub trait ITimeManager : Send {
    fn elapsed(&self) -> Duration;
    fn check_timeout(&self) -> bool;
    fn iteration_complete(&mut self, si: &SearchInfo);
}

pub struct SearchParams {
    pub position: chess::Position,
    pub repeats: Vec<u64>,
    pub time_manager: Box<dyn ITimeManager>,
}

pub trait IEngine {
    fn get_options(&self) -> Vec<EngineOption>;
    fn set_option(&mut self, name: &str, value: &str);
    fn prepare(&mut self);
    fn clear(&mut self);
    fn search(&mut self, search_params: SearchParams) -> SearchInfo;
}

#[derive(Copy, Clone, Debug)]
pub struct EngineOption {
    pub name: &'static str,
    pub value: OptionValue,
}

#[derive(Copy, Clone, Debug)]
pub enum OptionValue {
    Bool(bool),
    Int {
        min: isize,
        max: isize,
        value: isize,
    },
}

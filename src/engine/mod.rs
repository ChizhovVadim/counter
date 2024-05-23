use std::time::Duration;

use crate::chess::{self, Move, Position};
use crate::types;
use crate::uci;
pub use history::HistoryTable;
pub use moveiter::MovePicker;
use std::sync;
pub use transtable::TransTable;
use utils::*;

mod history;
mod moveiter;
mod search;
mod see;
mod transtable;
mod utils;

pub struct Engine {
    experiment: bool,
    evaluator: Box<dyn types::IEvaluator>,
    trans_table: transtable::TransTable,
    time_manager: Box<dyn types::ITimeManager>,
    repeats: Vec<u64>,
    nodes: u64,
    root_depth: usize,
    stack: [SearchStack; utils::STACK_SIZE],
    reductions: [[i8; 64]; 64],
    history: history::HistoryTable,
}

#[derive(Clone, Copy)]
struct SearchStack {
    static_eval: isize,
    killer1: chess::Move,
    killer2: chess::Move,
    current_mv: chess::Move,
    key: u64,
    pv: [chess::Move; STACK_SIZE],
    pv_size: usize,
}

impl types::IEngine for Engine {
    fn get_options(&self) -> Vec<types::EngineOption> {
        return vec![
            types::EngineOption {
                name: "Hash",
                value: types::OptionValue::Int {
                    min: 4,
                    max: 1 << 16,
                    value: self.trans_table.size() as isize,
                },
            },
            types::EngineOption {
                name: "ExperimentSettings",
                value: types::OptionValue::Bool(self.experiment),
            },
        ];
    }
    fn set_option(&mut self, name: &str, value: &str) {
        match name {
            "Hash" => {
                if let Ok(size) = value.parse::<u32>() {
                    self.trans_table.resize(size as usize);
                }
            }
            "ExperimentSettings" => {
                self.experiment = value == "true";
            }
            _ => (),
        }
    }
    fn prepare(&mut self) {}
    fn clear(&mut self) {
        self.trans_table.clear();
        self.history.clear();
        // TODO clear killers
    }
    fn search(&mut self, search_params: types::SearchParams) -> types::SearchInfo {
        self.time_manager = search_params.time_manager;
        self.repeats = search_params.repeats;
        self.trans_table.inc_date();
        self.evaluator.init(&search_params.position);
        self.stack[0].key = search_params.position.key;
        self.nodes = 0;
        return self.iterative_deepening(&search_params.position);
    }
}

impl Engine {
    pub fn new(evaluator: Box<dyn types::IEvaluator>) -> Box<Self> {
        let mut eng = Box::new(Engine {
            experiment: false,
            evaluator: evaluator,
            trans_table: transtable::TransTable::new(128),
            time_manager: Box::new(uci::FixedTimeManager::default()),
            repeats: Vec::new(),
            nodes: 0,
            root_depth: 0,
            stack: [SearchStack {
                static_eval: 0,
                killer1: Move::EMPTY,
                killer2: Move::EMPTY,
                current_mv: Move::EMPTY,
                key: 0_u64,
                pv_size: 0,
                pv: [Move::EMPTY; STACK_SIZE],
            }; utils::STACK_SIZE],
            reductions: [[0_i8; 64]; 64],
            history: history::HistoryTable::new(),
        });

        let k = 2_f64 / 5_f64.ln() / 22_f64.ln();
        for d in 1..64 {
            for m in 1..64 {
                let r = k * (d as f64).ln() * (m as f64).ln();
                eng.reductions[d][m] = r as i8;
            }
        }

        return eng;
    }
}

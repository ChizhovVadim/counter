mod history;
mod moveorder;
//mod search;
mod search_counter55;
mod see;
mod timemanager;
mod transtable;
mod utils;

use crate::chess::{Move, Position};
use crate::domain::{
    CancelToken, EngineOption, IEngine, IEvaluator, LimitsType, OptionValue, SearchInfo,
    SearchParams,
};
use crate::eval;
use history::HistoryTable;
use timemanager::TimeManager;
use transtable::TransTable;

pub struct Engine {
    experiment: bool,
    nodes: u64,
    stack: Vec<SearchStack>,
    evaluator: Box<dyn IEvaluator>,
    time_manager: TimeManager,
    repeats: Vec<u64>,
    trans_table: TransTable,
    reductions: utils::Reductions,
    history: HistoryTable,
    progress: Box<dyn FnMut(&SearchInfo)>,
}

#[derive(Clone)]
struct SearchStack {
    position: Position,
    static_eval: isize,
    killer1: Move,
    killer2: Move,
    current_mv: Move,
    pv: [Move; utils::STACK_SIZE],
    pv_size: usize,
}

impl Engine {
    pub fn new() -> Self {
        return Engine {
            experiment: false,
            nodes: 0,
            evaluator: eval::make_eval("").unwrap(),
            time_manager: TimeManager::new(LimitsType::default(), CancelToken::new()),
            repeats: Vec::new(),
            trans_table: TransTable::new(64),
            reductions: utils::Reductions::new(utils::lmr_main),
            history: HistoryTable::new(),
            progress: Box::new(|_| {}),
            stack: vec![unsafe { std::mem::zeroed() }; utils::STACK_SIZE],
        };
    }
}

impl IEngine for Engine {
    fn clear(&mut self) {
        self.trans_table.clear();
        self.history.clear();
        for stack in &mut self.stack {
            stack.killer1 = Move::NONE;
            stack.killer2 = Move::NONE;
        }
        eprintln!("engine clear");
    }

    fn get_options(&self) -> Vec<EngineOption> {
        return vec![
            EngineOption {
                name: "Hash",
                value: OptionValue::Int {
                    min: 4,
                    max: 1 << 16,
                    value: self.trans_table.size() as isize,
                },
            },
            EngineOption {
                name: "ExperimentSettings",
                value: OptionValue::Bool(self.experiment),
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
                self.experiment = value.eq_ignore_ascii_case("true");
            }
            _ => (),
        }
    }

    fn search(&mut self, search_params: SearchParams) -> SearchInfo {
        self.time_manager = TimeManager::new(search_params.limits, search_params.cancel);
        self.repeats = search_params.repeats;
        self.progress = search_params.progress;
        self.stack[0].position = search_params.position.clone();
        self.evaluator.init(&self.stack[0].position);

        self.trans_table.inc_date();
        self.nodes = 0;
        return search_counter55::iterative_deepening(self);
    }
}

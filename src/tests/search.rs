use crate::chess;
use crate::types;
use crate::tests::make_test_engine;
use crate::uci;
use crate::types::IEngine;
use std::sync;
use std::time::Duration;

pub fn search_handler() {
    let mut eng = make_test_engine();
    let pos = chess::Position::from_fen("8/p3q1kp/1p2Pnp1/3pQ3/2pP4/1nP3N1/1B4PP/6K1 w - - 5 30")
        .unwrap();
    let abort = sync::Arc::new(sync::atomic::AtomicBool::new(false));
    let fixed_time = Duration::from_secs(30);
    let tm = Box::new(uci::FixedTimeManager::new(abort, Some(fixed_time), None, None));
    let res = eng.search(types::SearchParams{
        position: pos,
        repeats: Vec::new(),
        time_manager: tm,
    });
    uci::print_uci_search_info(&res);
}

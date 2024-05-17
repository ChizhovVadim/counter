#![allow(unused, invalid_value, internal_features)]
#![feature(core_intrinsics)]

mod chess;
mod engine;
mod eval;
mod tests;
mod types;
mod uci;

fn main() {
    chess::init();
    if let Some(commnad_name) = std::env::args().nth(1) {
        match commnad_name.as_str() {
            "search" => tests::search::search_handler(),
            "eval" => tests::eval::eval_handler(),
            "perft" => tests::perft::perft_handler(),
            "tactic" => tests::tactic::tactic_handler(),
            _ => panic!("command not found {}", commnad_name),
        }
    } else {
        uci_handler();
    }
}

fn uci_handler() {
    let eval_service = eval::make_eval("");
    let mut eng = engine::Engine::new(eval_service);
    uci::run(eng);
}

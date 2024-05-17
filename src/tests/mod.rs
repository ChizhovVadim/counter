pub mod eval;
pub mod perft;
pub mod search;
pub mod tactic;

use crate::types;
use crate::types::IEngine;

fn make_test_engine() -> Box<dyn types::IEngine> {
    //TODO can parse engine options line "-experimentsettings"
    let mut eval = String::from("");
    {
        let mut args = std::env::args().skip(1);
        while let Some(flg) = args.next() {
            if flg == "-eval" {
                if let Some(eval_arg) = args.next() {
                    eval = eval_arg;
                }
            }
        }
    }
    eprintln!("eval: {}", eval);
    let eval_service = crate::eval::make_eval(&eval);
    let mut eng = crate::engine::Engine::new(eval_service);
    eng.prepare();
    return eng;
}

mod eval;
mod perft;
mod tactic;
use std::path::PathBuf;

pub fn test_handler() -> bool {
    if let Some(commnad_name) = std::env::args().nth(1) {
        match commnad_name.as_str() {
            "perft" => {
                perft::perft_handler();
                return true;
            }
            "tactic" => {
                tactic::tactic_handler();
                return true;
            }
            "eval" => {
                eval::eval_handler();
                return true;
            }
            _ => return false,
        }
    }
    return false;
}

fn map_path(path: &str) -> PathBuf {
    if let Some(home_dir) = std::env::home_dir() {
        return home_dir.join(path);
    }
    return path.into();
}

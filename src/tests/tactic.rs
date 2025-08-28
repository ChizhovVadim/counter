use crate::chess::{Move, Position};
use crate::domain::{CancelToken, IEngine, LimitsType, SearchInfo, SearchParams};
use crate::engine::Engine;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::time::Duration;
use std::time::Instant;

//#[derive(Debug)]
struct TestEntry {
    content: String,
    position: Position,
    best_moves: Vec<Move>,
}

pub fn tactic_handler() {
    let path = super::map_path("chess/tests/tests.epd");
    let tests = load_tests(&path).expect("load tactic tests failed");
    eprintln!("loaded {} tests.", tests.len());

    let mut eng = Engine::new();
    let mut total = 0;
    let mut passed = 0;
    let start = Instant::now();
    for test in tests.iter() {
        let cancel = CancelToken::new();
        let search_res = eng.search(SearchParams {
            position: test.position.clone(),
            repeats: Vec::new(),
            limits: LimitsType::fixed_time(Duration::from_secs(3)),
            cancel: cancel.clone(),
            progress: Box::new(solve_tactic_progress(cancel, test.best_moves.clone())),
        });
        total += 1;
        if test.best_moves.contains(&search_res.main_line[0]) {
            passed += 1;
        } else {
            println!("{}", test.content);
            println!("{:?}", search_res);
        }
        println!("Solved: {}, Total: {}", passed, total);
    }
    println!("Test finished. Elapsed: {:?}", start.elapsed());
}

fn solve_tactic_progress(cancel: CancelToken, best_moves: Vec<Move>) -> impl FnMut(&SearchInfo) {
    let mut solve_count = 0;
    move |si| {
        if best_moves.contains(&si.main_line[0]) {
            solve_count += 1;
            // знатоки дают досрочный ответ
            if solve_count >= 3 {
                cancel.cancel();
            }
        } else {
            solve_count = 0;
        }
    }
}

fn load_tests(filename: &std::path::Path) -> Result<Vec<TestEntry>, Box<dyn std::error::Error>> {
    let file = File::open(filename)?;
    let lines = (io::BufReader::new(file)).lines();
    let mut result: Vec<TestEntry> = Vec::new();
    for line in lines {
        let line = line?;
        if let Some(test) = parse_test(line) {
            result.push(test);
        }
    }
    return Ok(result);
}

fn parse_test(s: String) -> Option<TestEntry> {
    let bm_begin = s.find("bm")?;
    let bm_end = s.find(";")?;
    let fen = s[..bm_begin].trim();
    let pos = Position::from_fen(fen)?;
    let mut best_moves: Vec<Move> = Vec::new();

    let sbest_moves = s[bm_begin..bm_end].split_ascii_whitespace().skip(1);
    for smove in sbest_moves {
        if let Some(m) = Move::parse_san(&pos, smove) {
            best_moves.push(m);
        } else {
            eprintln!("{}", s);
        }
    }

    if best_moves.is_empty() {
        eprintln!("{}", s);
        return None;
    }
    return Some(TestEntry {
        position: pos,
        content: s,
        best_moves: best_moves,
    });
}

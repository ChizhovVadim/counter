use crate::chess;
use crate::tests::make_test_engine;
use crate::types;
use crate::types::IEngine;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::sync;
use std::time::Duration;
use std::time::Instant;

#[derive(Debug)]
struct TestEntry {
    content: String,
    position: chess::Position,
    best_moves: Vec<chess::Move>,
}

pub fn tactic_handler() {
    //TODO ~/chess/tests/tests.epd
    let tests = load_tests(std::path::Path::new(
        "/Users/vadimchizhov/chess/tests/tests.epd",
    ))
    .expect("load tactic tests failed");
    eprintln!("loaded {} tests.", tests.len());

    let mut eng = make_test_engine();

    let mut total = 0;
    let mut passed = 0;
    let start = Instant::now();
    for test in tests.iter() {
        let tm = Box::new(TacticSolveTimeManager::new(
            Duration::from_secs(3),
            test.best_moves.clone(),
        ));
        let search_res = eng.search(types::SearchParams {
            position: test.position,
            repeats: Vec::new(),
            time_manager: tm,
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

fn load_tests(filename: &std::path::Path) -> Result<Vec<TestEntry>, Box<dyn std::error::Error>> {
    let file = File::open(filename)?;
    let lines = (io::BufReader::new(file)).lines();
    let mut result: Vec<TestEntry> = Vec::new();
    for line in lines.flatten() {
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
    let pos = chess::Position::from_fen(fen)?;
    let mut best_moves: Vec<chess::Move> = Vec::new();

    let sbest_moves = s[bm_begin..bm_end].split_ascii_whitespace().skip(1);
    for smove in sbest_moves {
        if let Some(m) = chess::Move::parse_san(&pos, smove) {
            best_moves.push(m);
        } else {
            eprintln!("{}", s);
        }
    }

    if best_moves.len() == 0 {
        eprintln!("{}", s);
        return None;
    }
    return Some(TestEntry {
        position: pos,
        content: s,
        best_moves: best_moves,
    });
}

struct TacticSolveTimeManager {
    max_time: Duration,
    best_moves: Vec<chess::Move>,
    start: Instant,
    abort: sync::Arc<sync::atomic::AtomicBool>,
    solve_count: usize,
}

impl types::ITimeManager for TacticSolveTimeManager {
    fn elapsed(&self) -> Duration {
        return self.start.elapsed();
    }
    fn check_timeout(&self) -> bool {
        if self.start.elapsed() >= self.max_time {
            self.cancel();
        }
        return self.abort.load(sync::atomic::Ordering::Relaxed);
    }
    fn iteration_complete(&mut self, si: &types::SearchInfo) {
        if self.best_moves.contains(&si.main_line[0]) {
            self.solve_count += 1;
        } else {
            self.solve_count = 0;
        }
        // считаем, что тест решен, если последние 3 итерации нашли лучший ход и достигнута минимальная глубина
        if self.solve_count >= 3 && si.depth >= 12 {
            self.cancel();
        }
    }
}

impl TacticSolveTimeManager {
    pub fn new(max_time: Duration, best_moves: Vec<chess::Move>) -> Self {
        return TacticSolveTimeManager {
            max_time: max_time,
            best_moves: best_moves,
            start: Instant::now(),
            abort: sync::Arc::new(sync::atomic::AtomicBool::new(false)),
            solve_count: 0,
        };
    }

    pub fn cancel(&self) {
        self.abort.store(true, sync::atomic::Ordering::SeqCst);
    }
}

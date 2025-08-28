use crate::chess::Position;
use crate::eval;
use std::fs::File;
use std::io::{self, BufRead};

struct DatasetItem {
    fen: String,
    target: f64,
}

pub fn eval_handler() {
    let dataset_path = super::map_path("chess/tuner/quiet-labeled.epd");
    let sigmoid_scale = 3.5 / 512.0;
    let mut evaluator = eval::make_eval("").unwrap();

    let mut total_cost = 0.0;
    let mut count = 0;

    let file = File::open(dataset_path).unwrap();
    let reader = io::BufReader::new(file);

    for line_result in reader.lines() {
        let line = line_result.unwrap();
        let item = parse_item(line);
        let pos = Position::from_fen(&item.fen).unwrap();
        let eval = evaluator.evaluate(&pos);
        let prob = sigmoid(sigmoid_scale * (eval as f64));
        let diff = prob - item.target;
        total_cost += diff * diff;
        count += 1;
    }

    let average_cost = total_cost / count as f64;
    eprintln!("Average cost: {}", average_cost);
}

fn sigmoid(x: f64) -> f64 {
    return 1.0 / (1.0 + (-x).exp());
}

//rnb1kbnr/pp1pppp1/7p/2q5/5P2/N1P1P3/P2P2PP/R1BQKBNR w KQkq - c9 "1/2-1/2";
fn parse_item(s: String) -> DatasetItem {
    let index = s.find('"').unwrap();
    let fen = &s[..index];
    let score = &s[index + 1..];
    let prob: f64;
    if score.starts_with("1/2-1/2") {
        prob = 0.5;
    } else if score.starts_with("1-0") {
        prob = 1.0;
    } else if score.starts_with("0-1") {
        prob = 0.0;
    } else {
        panic!("parse_item failed");
    }
    return DatasetItem {
        fen: String::from(fen),
        target: prob,
    };
}

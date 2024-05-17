use crate::eval;
use crate::chess;

pub fn eval_handler() {
    let mut eval_service = eval::make_eval("nnue");
    let pos = chess::Position::from_fen("8/p3q1kp/1p2Pnp1/3pQ3/2pP4/1nP3N1/1B4PP/6K1 w - - 5 30")
        .unwrap();
    let score = eval_service.evaluate(&pos);
    println!("score: {}", score)
}

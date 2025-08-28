use crate::chess::{Move, Position, bitboard};

// Удобен, чтобы оценить performance движка без затрат на оценку.
// Тактические тесты решает даже на такой оценочной функции.
pub struct MaterialEvaluationService {}

impl MaterialEvaluationService {
    pub fn new() -> Self {
        return MaterialEvaluationService {};
    }
}

#[allow(unused_variables)]
impl crate::domain::IEvaluator for MaterialEvaluationService {
    fn init(&mut self, pos: &Position) {}
    fn make_move(&mut self, pos: &Position, mv: Move) {}
    fn unmake_move(&mut self) {}
    fn quik_evaluate(&mut self, pos: &Position) -> isize {
        let val = 100
            * (bitboard::pop_count(pos.pawns & pos.white)
                - bitboard::pop_count(pos.pawns & pos.black))
            + 400
                * (bitboard::pop_count(pos.knights & pos.white)
                    - bitboard::pop_count(pos.knights & pos.black))
            + 400
                * (bitboard::pop_count(pos.bishops & pos.white)
                    - bitboard::pop_count(pos.bishops & pos.black))
            + 600
                * (bitboard::pop_count(pos.rooks & pos.white)
                    - bitboard::pop_count(pos.rooks & pos.black))
            + 1200
                * (bitboard::pop_count(pos.queens & pos.white)
                    - bitboard::pop_count(pos.queens & pos.black));
        return val;
    }
}

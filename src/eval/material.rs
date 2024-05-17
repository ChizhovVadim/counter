use crate::chess::{bitboard::pop_count, *};
use crate::types;
pub struct MaterialEvaluationService {}

impl MaterialEvaluationService{
    pub fn new() -> Self{
        return MaterialEvaluationService{};
    }
}

impl types::IEvaluator for MaterialEvaluationService {
    fn init(&mut self, _: &Position) {}
    fn make_move(&mut self, _: &History) {}
    fn unmake_move(&mut self) {}

    fn quik_evaluate(&mut self, pos: &Position) -> isize {
        let val = 100 * (pop_count(pos.pawns & pos.white) - pop_count(pos.pawns & pos.black))
            + 400 * (pop_count(pos.knights & pos.white) - pop_count(pos.knights & pos.black))
            + 400 * (pop_count(pos.bishops & pos.white) - pop_count(pos.bishops & pos.black))
            + 600 * (pop_count(pos.rooks & pos.white) - pop_count(pos.rooks & pos.black))
            + 1200 * (pop_count(pos.queens & pos.white) - pop_count(pos.queens & pos.black));
        return val;
    }
}

use crate::chess::{
    bitboard::pop_count,
    square::*,
    *,
};
use crate::types;

mod weights;

#[derive(Clone, Copy, Debug)]
struct Score {
    mg: i32,
    eg: i32,
}

struct Weights {
    bishop_pair: Score,
    psq: [Score; SIDE_NB * PIECE_NB * SQUARE_NB],
}

pub struct PestoEvaluationService {
    index: usize,
    stack: [Score; 256],
    weights: Weights,
}

impl Score {
    pub fn default() -> Score {
        return Score { mg: 0, eg: 0 };
    }

    pub fn add(&mut self, s: Score, n: i32) {
        self.mg += s.mg * n;
        self.eg += s.eg * n;
    }
}

impl PestoEvaluationService {
    pub fn new() -> Self {
        let mut result: PestoEvaluationService =
            unsafe { std::mem::MaybeUninit::uninit().assume_init() };
        result.index = 0;
        result.stack[result.index] = Score::default();
        crate::eval::pesto::weights::init_weights(&mut result.weights);
        return result;
    }
}

impl types::IEvaluator for PestoEvaluationService {
    fn init(&mut self, pos: &Position) {
        let updates = pos.to_updates();
        let score = eval_updates(&self.weights, &updates);
        self.index = 0;
        self.stack[self.index] = score;
    }

    fn make_move(&mut self, history: &History) {
        let updates_score = eval_updates(&self.weights, &history.updates[..history.update_size]);

        let mut score = self.stack[self.index];
        score.add(updates_score, 1);

        self.stack[self.index + 1] = score;
        self.index += 1;
    }

    fn unmake_move(&mut self) {
        self.index -= 1;
    }

    fn quik_evaluate(&mut self, pos: &Position) -> isize {
        //let mut score = eval_slow(self, pos);
        let mut score = self.stack[self.index];

        if pop_count(pos.bishops & pos.white) >= 2 {
            score.add(self.weights.bishop_pair, 1);
        }
        if pop_count(pos.bishops & pos.black) >= 2 {
            score.add(self.weights.bishop_pair, -1);
        }

        const TOTAL_PHASE: isize = 24;
        let phase = (pop_count(pos.knights | pos.bishops)
            + 2 * pop_count(pos.rooks)
            + 4 * pop_count(pos.queens))
        .min(TOTAL_PHASE);

        let total = ((score.mg as isize) * phase + (score.eg as isize) * (TOTAL_PHASE - phase))
            / TOTAL_PHASE;
        return total;
    }
}

/*fn eval_slow(e: &PestoEvaluationService, pos: &Position) -> Score {
    let mut result = Score::default();
    let mut bb = pos.all_pieces();
    while bb != 0 {
        let sq = first_one(bb);
        let side = if pos.white & square_mask(sq) != 0 {
            SIDE_WHITE
        } else {
            SIDE_BLACK
        };
        let piece = pos.piece_on_square(sq);
        let index = side_piece_sq_index(side, piece, sq);
        let val = e.weights.psq[index];
        result.add(val, 1);
        bb &= bb - 1;
    }
    return result;
}*/

fn eval_updates(weights: &Weights, updates: &[Update]) -> Score {
    let mut result = Score::default();
    for h in updates {
        let index = side_piece_sq_index(h.side, h.piece, h.square);
        let val = weights.psq[index];
        if h.action == UPDATE_ACTION_ADD {
            result.add(val, 1);
        } else {
            result.add(val, -1);
        }
    }
    return result;
}

fn side_piece_sq_index(side: usize, piece: Piece, square: Square) -> usize {
    (side << 9) ^ (piece << 6) ^ square
}

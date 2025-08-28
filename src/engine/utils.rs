use crate::chess::{Move, Piece, Position, bitboard};
use crate::domain::UciScore;

pub const STACK_SIZE: usize = 128;
pub const MAX_HEIGHT: usize = STACK_SIZE - 1;
pub const VALUE_DRAW: isize = 0;
pub const VALUE_MATE: isize = 30_000;
pub const VALUE_INFINITY: isize = VALUE_MATE + 1;
pub const VALUE_WIN: isize = VALUE_MATE - 2 * MAX_HEIGHT as isize;
pub const VALUE_LOSS: isize = -VALUE_WIN;

pub fn win_in(height: usize) -> isize {
    return VALUE_MATE - height as isize;
}

pub fn loss_in(height: usize) -> isize {
    return -VALUE_MATE + height as isize;
}

pub fn value_to_tt(v: isize, height: usize) -> isize {
    if v >= VALUE_WIN {
        return v + height as isize;
    }
    if v <= VALUE_LOSS {
        return v - height as isize;
    }
    return v;
}

pub fn value_from_tt(v: isize, height: usize) -> isize {
    if v >= VALUE_WIN {
        return v - height as isize;
    }
    if v <= VALUE_LOSS {
        return v + height as isize;
    }
    return v;
}

pub fn is_draw(p: &Position) -> bool {
    if p.rule50 > 100 {
        return true;
    }

    if (p.pawns | p.rooks | p.queens) == 0 && !bitboard::multiple(p.knights | p.bishops) {
        return true;
    }

    return false;
}

pub fn make_uci_score(v: isize) -> UciScore {
    if v >= VALUE_WIN {
        return UciScore::Mate((VALUE_MATE - v + 1) / 2);
    } else if v <= VALUE_LOSS {
        return UciScore::Mate((-VALUE_MATE - v) / 2);
    } else {
        return UciScore::Centipawns(v);
    }
}

pub fn is_capture_or_promotion(mv: Move) -> bool {
    return mv.captured_piece() != Piece::NONE || mv.promotion() != Piece::NONE;
}

pub fn allow_nmp(p: &Position) -> bool {
    let own_pieces = p.colours(p.side_to_move);
    return ((p.rooks | p.queens) & own_pieces) != 0
        || bitboard::multiple((p.knights | p.bishops) & own_pieces);
}

pub struct Reductions([[i8; 64]; 64]);

impl Reductions {
    pub fn new(f: fn(d: f64, m: f64) -> f64) -> Self {
        let mut res = Reductions([[0_i8; 64]; 64]);
        for d in 1..64 {
            for m in 1..64 {
                res.0[d][m] = f(d as f64, m as f64) as i8;
            }
        }
        return res;
    }

    pub fn get(&self, depth: isize, move_index: isize) -> isize {
        return self.0[depth.min(63) as usize][move_index.min(63) as usize] as isize;
    }
}

pub fn lmr_main(d: f64, m: f64) -> f64 {
    return 0.75 + d.ln() * m.ln() / 2.25;
}

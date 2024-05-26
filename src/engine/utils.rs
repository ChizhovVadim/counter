use crate::chess;
use crate::chess::{bitboard::more_than_one, Move, Position, PIECE_EMPTY};
use crate::types;

pub const STACK_SIZE: usize = 128;
pub const MAX_HEIGHT: usize = STACK_SIZE - 1;
pub const VALUE_DRAW: isize = 0;
pub const VALUE_MATE: isize = 30_000;
pub const VALUE_INFINITY: isize = VALUE_MATE + 1;
pub const VALUE_WIN: isize = VALUE_MATE - 2 * MAX_HEIGHT as isize;
pub const VALUE_LOSS: isize = -VALUE_WIN;

pub fn is_cap_or_prom(m: Move) -> bool {
    return m.captured_piece() != PIECE_EMPTY || m.promotion() != PIECE_EMPTY;
}

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

    if (p.pawns | p.rooks | p.queens) == 0 && !more_than_one(p.knights | p.bishops) {
        return true;
    }

    return false;
}

pub fn allow_nmp(p: &Position) -> bool {
    let own_pieces = p.colours(p.side_to_move);
    return ((p.rooks | p.queens) & own_pieces) != 0
        || more_than_one((p.knights | p.bishops) & own_pieces);
}

pub fn make_uci_score(v: isize) -> types::UciScore {
    if v >= VALUE_WIN {
        return types::UciScore::Mate((VALUE_MATE - v + 1) / 2);
    } else if v <= VALUE_LOSS {
        return types::UciScore::Mate((-VALUE_MATE - v) / 2);
    } else {
        return types::UciScore::Centipawns(v);
    }
}

pub fn is_pawn_advance(mv: Move, side: usize) -> bool {
    if mv.moving_piece() != PIECE_EMPTY {
        return false;
    }
    let rank = chess::square::rank(mv.to());
    if side == chess::SIDE_WHITE {
        return rank >= chess::square::RANK_6;
    } else {
        return rank <= chess::square::RANK_3;
    }
}

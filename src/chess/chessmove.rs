use chessmove::square::*;
use std::{fmt, iter::RepeatWith, ops::Index};

use super::*;

#[derive(PartialEq, Copy, Clone)]
pub struct Move(u32);

impl Move {
    pub const EMPTY: Move = Move(0_u32);

    pub fn make(from: Square, to: Square, moving_piece: Piece, captured_piece: Piece) -> Move {
        let val = from ^ (to << 6) ^ (moving_piece << 12) ^ (captured_piece << 15);
        Move(val as u32)
    }

    pub fn with_promotion(self, promotion: Piece) -> Move {
        return Move(self.0 ^ ((promotion as u32) << 18));
    }

    pub fn from(self) -> Square {
        (self.0 & 63) as Square
    }

    pub fn to(self) -> Square {
        ((self.0 >> 6) & 63) as Square
    }

    pub fn moving_piece(self) -> Piece {
        return ((self.0 >> 12) & 7) as Piece;
    }

    pub fn captured_piece(self) -> Piece {
        return ((self.0 >> 15) & 7) as Piece;
    }

    pub fn promotion(self) -> Piece {
        return ((self.0 >> 18) & 7) as Piece;
    }

    //TODO place to fmt::Display and remove
    pub fn name(self) -> String {
        if self == Move::EMPTY {
            return String::from("0000");
        }
        let mut s = String::new();
        s.push_str(&square::square_name(self.from()));
        s.push_str(&square::square_name(self.to()));
        match self.promotion() {
            PIECE_KNIGHT => s.push('n'),
            PIECE_BISHOP => s.push('b'),
            PIECE_ROOK => s.push('r'),
            PIECE_QUEEN => s.push('q'),
            _ => (),
        }
        return s;
    }

    pub fn parse_lan(pos: &Position, s: &str) -> Option<Move> {
        let mut ml = MoveList::new();
        generate_moves(pos, &mut ml);
        for m in ml.moves[..ml.size].iter() {
            if m.mv.name() == s {
                return Some(m.mv);
            }
        }
        return None;
    }

    pub fn parse_san(pos: &Position, san: &str) -> Option<Move> {
        let pat = &['+', '#', '?', '!'];
        let san = san.trim_end_matches(pat);
        let mut ml = MoveList::new();
        generate_legal_moves(pos, &mut ml);
        for item in ml.moves[..ml.size].iter() {
            if san == move_to_san(pos, &ml, item.mv) {
                return Some(item.mv);
            }
        }
        return None;
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //TODO direct
        write!(f, "{}", &self.name())
    }
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.name())
    }
}

fn write_san_piece_name(piece: Piece, sb: &mut String) {
    match piece {
        PIECE_KNIGHT => sb.push('N'),
        PIECE_BISHOP => sb.push('B'),
        PIECE_ROOK => sb.push('R'),
        PIECE_QUEEN => sb.push('Q'),
        PIECE_KING => sb.push('K'),
        _ => (),
    }
}

fn move_to_san(pos: &Position, ml: &MoveList, mv: Move) -> String {
    if mv.moving_piece() == PIECE_KING {
        //SAN kingside castling is indicated by the sequence O-O; queenside castling is indicated by the sequence O-O-O (note that these are capital Os, not zeroes, contrary to the FIDE standard for notation).
        if mv.from() == SQUARE_E1 && mv.to() == SQUARE_G1
            || mv.from() == SQUARE_E8 && mv.to() == SQUARE_G8
        {
            return String::from("O-O");
        }
        if mv.from() == SQUARE_E1 && mv.to() == SQUARE_C1
            || mv.from() == SQUARE_E8 && mv.to() == SQUARE_C8
        {
            return String::from("O-O-O");
        }
    }
    let mut str_piece = String::new();
    let mut str_capture = String::new();
    let mut str_from = String::new();
    let mut str_promotion = String::new();
    write_san_piece_name(mv.moving_piece(), &mut str_piece);
    let str_to = square_name(mv.to());
    if mv.captured_piece() != PIECE_EMPTY {
        str_capture.push('x');
        if mv.moving_piece() == PIECE_PAWN {
            str_from.push(file_name(mv.from()));
        }
    }
    if mv.promotion() != PIECE_EMPTY {
        str_promotion.push('=');
        write_san_piece_name(mv.promotion(), &mut str_promotion);
    }
    let mut ambiguity = false;
    let mut uniq_col = true;
    let mut uniq_row = true;
    for mv1 in ml.moves[..ml.size].iter().map(|x| x.mv) {
        if mv1.from() == mv.from() {
			continue
		}
		if mv1.to() != mv.to() {
			continue
		}
		if mv1.moving_piece() != mv.moving_piece() {
			continue
		}
		ambiguity = true;
		if file(mv1.from()) == file(mv.from()) {
			uniq_col = false;
		}
		if rank(mv1.from()) == rank(mv.from()) {
			uniq_row = false;
		}
    }
    if ambiguity {
		if uniq_col {
			str_from.push(file_name(mv.from()));
		} else if uniq_row {
            str_from.push(rank_name(mv.from()));
		} else {
			str_from = square_name(mv.from());
		}
	}
    return format!(
        "{}{}{}{}{}",
        str_piece, str_from, str_capture, str_to, str_promotion
    );
}

use super::{MoveList, Piece, Position, Square, square};
use std::fmt;
use std::fmt::Write;

#[derive(Copy, Clone, Default, PartialEq)]
pub struct Move(u32);

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}{:?}", self.from(), self.to())?;
        if let Some(promotion) = promotion_char(self.promotion()) {
            write!(f, "{}", promotion)?;
        }
        Ok(())
    }
}

fn promotion_char(piece: Piece) -> Option<char> {
    match piece {
        Piece::KNIGHT => Some('n'),
        Piece::BISHOP => Some('b'),
        Piece::ROOK => Some('r'),
        Piece::QUEEN => Some('q'),
        _ => None,
    }
}

impl Move {
    pub const NONE: Move = Move(0);
    pub const NULL: Move = Move(1);

    pub fn is_null(self) -> bool {
        return self == Move::NULL;
    }

    pub fn make(from: Square, to: Square, moving_piece: Piece, captured_piece: Piece) -> Move {
        Move(
            (from.0 as u32)
                ^ ((to.0 as u32) << 6)
                ^ ((moving_piece as u32) << 12)
                ^ ((captured_piece as u32) << 15),
        )
    }

    pub fn with_promotion(self, promotion: Piece) -> Move {
        return Move(self.0 ^ ((promotion as u32) << 18));
    }

    pub fn from(self) -> Square {
        Square((self.0 & 63) as u8)
    }

    pub fn to(self) -> Square {
        Square(((self.0 >> 6) & 63) as u8)
    }

    pub fn moving_piece(self) -> Piece {
        let value = (self.0 >> 12) & 7;
        unsafe { std::mem::transmute(value as u8) }
    }

    pub fn captured_piece(self) -> Piece {
        let value = (self.0 >> 15) & 7;
        unsafe { std::mem::transmute(value as u8) }
    }

    pub fn promotion(self) -> Piece {
        let value = (self.0 >> 18) & 7;
        unsafe { std::mem::transmute(value as u8) }
    }

    pub fn parse_lan(pos: &Position, s: &str) -> Option<Move> {
        let mut ml = MoveList::new();
        ml.gen_moves(pos);
        for m in &ml.moves[..ml.size] {
            let lan = format!("{:?}", m.mv);
            if lan.eq_ignore_ascii_case(s) {
                //if lan == s {
                return Some(m.mv);
            }
        }
        return None;
    }

    pub fn parse_san(pos: &Position, san: &str) -> Option<Move> {
        let pat = &['+', '#', '?', '!'];
        let san = san.trim_end_matches(pat);
        let mut ml = MoveList::new();
        ml.gen_legal_moves(pos);
        for item in ml.moves[..ml.size].iter() {
            if san == move_to_san(pos, &ml, item.mv) {
                return Some(item.mv);
            }
        }
        return None;
    }
}

#[allow(unused_must_use, unused_variables)]
fn move_to_san(pos: &Position, ml: &MoveList, mv: Move) -> String {
    if mv.moving_piece() == Piece::KING {
        //SAN kingside castling is indicated by the sequence O-O; queenside castling is indicated by the sequence O-O-O (note that these are capital Os, not zeroes, contrary to the FIDE standard for notation).
        if mv.from() == Square::E1 && mv.to() == Square::G1
            || mv.from() == Square::E8 && mv.to() == Square::G8
        {
            return String::from("O-O");
        }
        if mv.from() == Square::E1 && mv.to() == Square::C1
            || mv.from() == Square::E8 && mv.to() == Square::C8
        {
            return String::from("O-O-O");
        }
    }

    let mut ambiguity = false;
    let mut uniq_col = true;
    let mut uniq_row = true;
    for mv1 in ml.moves[..ml.size].iter().map(|x| x.mv) {
        if mv1.from() == mv.from() {
            continue;
        }
        if mv1.to() != mv.to() {
            continue;
        }
        if mv1.moving_piece() != mv.moving_piece() {
            continue;
        }
        ambiguity = true;
        if mv1.from().file() == mv.from().file() {
            uniq_col = false;
        }
        if mv1.from().rank() == mv.from().rank() {
            uniq_row = false;
        }
    }

    let mut res = String::new();

    write_san_piece_name(mv.moving_piece(), &mut res);
    if mv.captured_piece() != Piece::NONE && mv.moving_piece() == Piece::PAWN {
        res.push(square::FILE_NAMES2[mv.from().file() as usize]);
    }
    if ambiguity {
        if uniq_col {
            res.push(square::FILE_NAMES2[mv.from().file() as usize]);
        } else if uniq_row {
            res.push(square::RANK_NAMES2[mv.from().rank() as usize]);
        } else {
            write!(res, "{:?}", mv.from());
        }
    }
    if mv.captured_piece() != Piece::NONE {
        res.push('x');
    }
    write!(res, "{:?}", mv.to());
    if mv.promotion() != Piece::NONE {
        res.push('=');
        write_san_piece_name(mv.promotion(), &mut res);
    }

    return res;
}

fn write_san_piece_name(piece: Piece, sb: &mut String) {
    match piece {
        Piece::KNIGHT => sb.push('N'),
        Piece::BISHOP => sb.push('B'),
        Piece::ROOK => sb.push('R'),
        Piece::QUEEN => sb.push('Q'),
        Piece::KING => sb.push('K'),
        _ => (),
    }
}

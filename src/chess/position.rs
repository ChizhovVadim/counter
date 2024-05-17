use std::fmt;
use std::fmt::write;
use std::fmt::Write;
use std::isize;

use super::bitboard::*;
use super::square::*;
use super::*;

pub const INITIAL_POSITION_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub const WHITE_KING_SIDE: usize = 1;
pub const WHITE_QUEEN_SIDE: usize = 2;
pub const BLACK_KING_SIDE: usize = 4;
pub const BLACK_QUEEN_SIDE: usize = 8;

pub const UPDATE_ACTION_ADD: usize = 1;
pub const UPDATE_ACTION_REMOVE: usize = 2;

#[derive(Clone, Copy, Debug)]
pub struct Update {
    pub action: usize,
    pub side: usize,
    pub piece: Piece,
    pub square: Square,
}

pub struct History {
    pub key: u64,
    pub checkers: u64,
    pub mv: Move,
    pub ep_square: Square,
    pub rule50: isize,
    pub castling_rights: usize,
    pub updates: [Update; 4],
    pub update_size: usize,
}

#[derive(Clone, Copy)]
pub struct Position {
    pub white: u64,
    pub black: u64,
    pub pawns: u64,
    pub knights: u64,
    pub bishops: u64,
    pub rooks: u64,
    pub queens: u64,
    pub kings: u64,
    pub checkers: u64,
    pub side_to_move: usize,
    pub castling_rights: usize,
    pub rule50: isize,
    pub ep_square: Square, //TODO Option<Square>?
    pub key: u64,
}

impl History {
    pub fn new() -> History {
        return unsafe { std::mem::MaybeUninit::uninit().assume_init() };
    }

    pub fn clear_updates(&mut self) {
        self.update_size = 0;
    }

    pub fn add(&mut self, u: Update) {
        self.updates[self.update_size] = u;
        self.update_size += 1;
    }
}

impl Position {
    pub fn all_pieces(&self) -> u64 {
        return self.white | self.black;
    }

    pub fn is_check(&self) -> bool {
        return self.checkers != 0;
    }

    pub fn colours(&self, side: usize) -> u64 {
        if side == SIDE_WHITE {
            return self.white;
        }
        return self.black;
    }

    pub fn king_sq(&self, side: usize) -> Square {
        return first_one(self.kings & self.colours(side));
    }

    pub fn attackers_by_side(&self, side: usize, sq: Square) -> u64 {
        return self.colours(side) & self.attackers_to(sq);
    }

    fn compute_checkers(&self) -> u64 {
        return self.attackers_by_side(self.side_to_move ^ 1, self.king_sq(self.side_to_move));
    }

    fn is_legal(&self) -> bool {
        return self.attackers_by_side(self.side_to_move, self.king_sq(self.side_to_move ^ 1)) == 0;
    }

    fn new(
        pieces: &[Update],
        side_to_move: usize,
        castle_rights: usize,
        ep_square: Square,
        fifty: isize,
    ) -> Option<Position> {
        let mut pos = Position {
            white: 0,
            black: 0,
            pawns: 0,
            knights: 0,
            bishops: 0,
            rooks: 0,
            queens: 0,
            kings: 0,
            checkers: 0,
            side_to_move: side_to_move,
            key: 0,
            castling_rights: castle_rights,
            ep_square: ep_square,
            rule50: fifty,
        };
        if side_to_move == SIDE_WHITE {
            pos.key ^= unsafe { SIDE_KEY };
        }
        pos.key ^= unsafe { CASTLING_KEY[castle_rights] };
        if ep_square != SQUARE_NONE {
            pos.key ^= unsafe { ENPASSANT_KEY[file(ep_square)] };
        }
        pos.apply_updates(pieces);
        pos.checkers = pos.compute_checkers();
        if !pos.is_legal() {
            return None;
        }
        return Some(pos);
    }

    pub fn from_fen(fen: &str) -> Option<Position> {
        let tokens: Vec<_> = fen.split(' ').collect();
        if tokens.len() <= 3 {
            return None;
        }

        let mut pieces: Vec<Update> = Vec::new();
        let mut i = 0;
        for ch in tokens[0].chars() {
            if ch.is_ascii_digit() {
                let n: usize = ch.to_string().parse().unwrap();
                i += n;
            } else if ch.is_ascii_alphabetic() {
                let side = if ch.is_ascii_uppercase() {
                    SIDE_WHITE
                } else {
                    SIDE_BLACK
                };
                let piece = char_to_piece(ch.to_ascii_lowercase());
                if piece != PIECE_EMPTY {
                    pieces.push(Update {
                        action: UPDATE_ACTION_ADD,
                        side: side,
                        piece: piece,
                        square: flip_square(i),
                    });
                    i += 1;
                };
            }
        }

        let side_to_move = if tokens[1] == "w" {
            SIDE_WHITE
        } else {
            SIDE_BLACK
        };

        let s_castle_rights = tokens[2];
        let mut cr: usize = 0;
        if s_castle_rights.contains("K") {
            cr |= WHITE_KING_SIDE;
        }
        if s_castle_rights.contains("Q") {
            cr |= WHITE_QUEEN_SIDE;
        }
        if s_castle_rights.contains("k") {
            cr |= BLACK_KING_SIDE;
        }
        if s_castle_rights.contains("q") {
            cr |= BLACK_QUEEN_SIDE;
        }

        let ep_square = parse_square(tokens[3]);

        let mut rule50: isize = 0;
        if tokens.len() > 4 {
            rule50 = tokens[4].parse::<isize>().unwrap_or(0);
        }

        return Position::new(&pieces, side_to_move, cr, ep_square, rule50);
    }

    pub fn to_updates(self: &Position) -> Vec<Update> {
        let mut result = Vec::with_capacity(32);
        let mut bb = self.all_pieces();
        while bb != 0 {
            let sq = first_one(bb);
            let side = if self.white & square_mask(sq) != 0 {
                SIDE_WHITE
            } else {
                SIDE_BLACK
            };
            let piece = self.piece_on_square(sq);
            result.push(Update {
                action: UPDATE_ACTION_ADD,
                side: side,
                piece: piece,
                square: sq,
            });
            bb &= bb - 1;
        }
        return result;
    }

    pub fn piece_on_square(self: &Position, sq: Square) -> Piece {
        let b = square_mask(sq);
        if (self.white | self.black) & b == 0 {
            return PIECE_EMPTY;
        } else if self.pawns & b != 0 {
            return PIECE_PAWN;
        } else if self.knights & b != 0 {
            return PIECE_KNIGHT;
        } else if self.bishops & b != 0 {
            return PIECE_BISHOP;
        } else if self.rooks & b != 0 {
            return PIECE_ROOK;
        } else if self.queens & b != 0 {
            return PIECE_QUEEN;
        } else if self.kings & b != 0 {
            return PIECE_KING;
        }
        return PIECE_EMPTY;
    }

    pub fn side_piece_on_square(self: &Position, sq: Square) -> (usize, usize) {
        let piece = self.piece_on_square(sq);
        let side = if square_mask(sq) & self.white != 0 {
            SIDE_WHITE
        } else {
            SIDE_BLACK
        };
        return (side, piece);
    }

    fn compute_ley(&self) -> u64 {
        unsafe {
            let mut result = 0_u64;
            if self.side_to_move == SIDE_WHITE {
                result ^= SIDE_KEY;
            }
            result ^= CASTLING_KEY[self.castling_rights];
            if self.ep_square != SQUARE_NONE {
                result ^= ENPASSANT_KEY[file(self.ep_square)];
            }

            let mut bb = self.all_pieces();
            while bb != 0 {
                let sq = first_one(bb);
                bb &= bb - 1;
                let (side, piece) = self.side_piece_on_square(sq);
                let psq_key_index = sq ^ (piece << 6) ^ (side << 9);
                result ^= PSQ_KEY[psq_key_index];
            }

            return result;
        }
    }

    pub fn make_move(self: &mut Position, m: Move, history: &mut History) -> bool {
        history.mv = m;
        history.key = self.key;
        history.checkers = self.checkers;
        history.castling_rights = self.castling_rights;
        history.rule50 = self.rule50;
        history.ep_square = self.ep_square;
        history.clear_updates();

        if m == Move::EMPTY {
            self.rule50 += 1;
            self.side_to_move ^= 1;
            unsafe {
                self.key ^= SIDE_KEY;
            }
            self.ep_square = SQUARE_NONE;
            if history.ep_square != SQUARE_NONE {
                unsafe {
                    self.key ^= ENPASSANT_KEY[file(history.ep_square)];
                }
            }
            self.checkers = 0;
            return true;
        }

        let side = self.side_to_move;
        let from = m.from();
        let to = m.to();
        let moving_piece = m.moving_piece();
        let captured_piece = m.captured_piece();

        let mut piece_after_move = moving_piece;
        let mut capture_square = SQUARE_NONE;
        let mut is_castling = false;
        let mut rook_remove_sq = SQUARE_NONE;
        let mut rook_add_sq = SQUARE_NONE;

        unsafe {
            self.castling_rights &= CASTLE_MASK[from] & CASTLE_MASK[to];
            self.key ^= CASTLING_KEY[history.castling_rights] ^ CASTLING_KEY[self.castling_rights];
        }

        if moving_piece == PIECE_PAWN || captured_piece != PIECE_EMPTY {
            self.rule50 = 0;
        } else {
            self.rule50 += 1;
        }

        self.ep_square = SQUARE_NONE;
        if history.ep_square != SQUARE_NONE {
            unsafe {
                self.key ^= ENPASSANT_KEY[file(history.ep_square)];
            }
        }

        if m.promotion() != PIECE_EMPTY {
            piece_after_move = m.promotion();
        }
        if captured_piece != PIECE_EMPTY {
            capture_square = to;
        }

        if moving_piece == PIECE_PAWN {
            if to == history.ep_square {
                capture_square = if side == SIDE_WHITE { to - 8 } else { to + 8 };
            }
            if side == SIDE_WHITE {
                if to == from + 16 {
                    self.ep_square = from + 8;
                    unsafe {
                        self.key ^= ENPASSANT_KEY[file(from + 8)];
                    }
                }
            } else {
                if from == to + 16 {
                    self.ep_square = from - 8;
                    unsafe {
                        self.key ^= ENPASSANT_KEY[file(from - 8)];
                    }
                }
            }
        } else if moving_piece == PIECE_KING {
            if side == SIDE_WHITE {
                if from == SQUARE_E1 && to == SQUARE_G1 {
                    is_castling = true;
                    rook_remove_sq = SQUARE_H1;
                    rook_add_sq = SQUARE_F1;
                }
                if from == SQUARE_E1 && to == SQUARE_C1 {
                    is_castling = true;
                    rook_remove_sq = SQUARE_A1;
                    rook_add_sq = SQUARE_D1;
                }
            } else {
                if from == SQUARE_E8 && to == SQUARE_G8 {
                    is_castling = true;
                    rook_remove_sq = SQUARE_H8;
                    rook_add_sq = SQUARE_F8;
                }
                if from == SQUARE_E8 && to == SQUARE_C8 {
                    is_castling = true;
                    rook_remove_sq = SQUARE_A8;
                    rook_add_sq = SQUARE_D8;
                }
            }
        }

        // delete moving piece
        history.add(Update {
            action: UPDATE_ACTION_REMOVE,
            side: side,
            piece: moving_piece,
            square: from,
        });

        // delete captured piece
        if capture_square != SQUARE_NONE {
            history.add(Update {
                action: UPDATE_ACTION_REMOVE,
                side: side ^ 1,
                piece: captured_piece,
                square: capture_square,
            });
        }

        // add moving piece
        history.add(Update {
            action: UPDATE_ACTION_ADD,
            side: side,
            piece: piece_after_move,
            square: to,
        });

        // move rook
        if is_castling {
            history.add(Update {
                action: UPDATE_ACTION_REMOVE,
                side: side,
                piece: PIECE_ROOK,
                square: rook_remove_sq,
            });
            history.add(Update {
                action: UPDATE_ACTION_ADD,
                side: side,
                piece: PIECE_ROOK,
                square: rook_add_sq,
            });
        }

        self.apply_updates(&history.updates[0..history.update_size]);
        self.side_to_move ^= 1;
        unsafe {
            self.key ^= SIDE_KEY;
        }

        if !self.is_legal() {
            //self.unmake_move(&history);
            return false;
        }

        self.checkers = self.compute_checkers(); // SLOW!
        debug_assert!(self.key == self.compute_ley());
        return true;
    }

    /*pub fn unmake_move(self: &mut Position, history: &History) {
        for i in (0..history.update_size).rev() {
            let update = history.updates[i];
            let b = square_mask(update.square);
            if update.side == SIDE_WHITE {
                self.white ^= b;
            } else {
                self.black ^= b;
            }
            match update.piece {
                PIECE_PAWN => self.pawns ^= b,
                PIECE_KNIGHT => self.knights ^= b,
                PIECE_BISHOP => self.bishops ^= b,
                PIECE_ROOK => self.rooks ^= b,
                PIECE_QUEEN => self.queens ^= b,
                PIECE_KING => self.kings ^= b,
                _ => (),
            }
        }
        self.side_to_move ^= 1;
        self.key = history.key;
        self.checkers = history.checkers;
        self.castling_rights = history.castling_rights;
        self.rule50 = history.rule50;
        self.ep_square = history.ep_square;
    }*/

    fn apply_updates(&mut self, updates: &[Update]) {
        for update in updates {
            let b = square_mask(update.square);
            if update.side == SIDE_WHITE {
                self.white ^= b;
            } else {
                self.black ^= b;
            }
            match update.piece {
                PIECE_PAWN => self.pawns ^= b,
                PIECE_KNIGHT => self.knights ^= b,
                PIECE_BISHOP => self.bishops ^= b,
                PIECE_ROOK => self.rooks ^= b,
                PIECE_QUEEN => self.queens ^= b,
                PIECE_KING => self.kings ^= b,
                _ => (),
            }
            let psq_key_index = update.square ^ (update.piece << 6) ^ (update.side << 9);
            unsafe {
                self.key ^= PSQ_KEY[psq_key_index];
            }
        }
    }

    fn attackers_to(&self, sq: Square) -> u64 {
        let occ = self.white | self.black;
        return (pawn_attacks(SIDE_BLACK, sq) & self.pawns & self.white)
            | (pawn_attacks(SIDE_WHITE, sq) & self.pawns & self.black)
            | (knight_attacks(sq) & self.knights)
            | (bishop_attacks(sq, occ) & (self.bishops | self.queens))
            | (rook_attacks(sq, occ) & (self.rooks | self.queens))
            | (king_attacks(sq) & self.kings);
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", fen(self))
    }
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", fen(self))
    }
}

fn char_to_piece(ch: char) -> Piece {
    match ch {
        'p' => PIECE_PAWN,
        'n' => PIECE_KNIGHT,
        'b' => PIECE_BISHOP,
        'r' => PIECE_ROOK,
        'q' => PIECE_QUEEN,
        'k' => PIECE_KING,
        _ => PIECE_EMPTY,
    }
}

static mut CASTLE_MASK: [usize; SQUARE_NB] = [0_usize; SQUARE_NB];
static mut SIDE_KEY: u64 = 0;
static mut ENPASSANT_KEY: [u64; FILE_NB] = [0_u64; FILE_NB];
static mut CASTLING_KEY: [u64; 16] = [0_u64; 16];
static mut PSQ_KEY: [u64; SIDE_NB * PIECE_NB * SQUARE_NB] = [0_u64; SIDE_NB * PIECE_NB * SQUARE_NB];

fn init_keys() {
    let mut rng = crate::chess::rand::XorshiftRng::new();

    unsafe {
        SIDE_KEY = rng.gen();

        for item in ENPASSANT_KEY.iter_mut() {
            *item = rng.gen();
        }

        for item in PSQ_KEY.iter_mut() {
            *item = rng.gen();
        }

        let mut castle_keys = [0_u64; 4];
        for i in 0..castle_keys.len() {
            castle_keys[i] = rng.gen();
        }

        for i in 0..CASTLING_KEY.len() {
            for j in 0..castle_keys.len() {
                if i & (1 << j) != 0 {
                    CASTLING_KEY[i] ^= castle_keys[j];
                }
            }
        }
    }
}

pub(super) fn init_position() {
    init_keys();
    unsafe {
        for sq in 0..SQUARE_NB {
            CASTLE_MASK[sq] =
                WHITE_KING_SIDE | WHITE_QUEEN_SIDE | BLACK_KING_SIDE | BLACK_QUEEN_SIDE;
        }

        CASTLE_MASK[SQUARE_A1] &= !WHITE_QUEEN_SIDE;
        CASTLE_MASK[SQUARE_E1] &= !(WHITE_KING_SIDE | WHITE_QUEEN_SIDE);
        CASTLE_MASK[SQUARE_H1] &= !WHITE_KING_SIDE;

        CASTLE_MASK[SQUARE_A8] &= !BLACK_QUEEN_SIDE;
        CASTLE_MASK[SQUARE_E8] &= !(BLACK_KING_SIDE | BLACK_QUEEN_SIDE);
        CASTLE_MASK[SQUARE_H8] &= !BLACK_KING_SIDE;
    }
}

fn fen(p: &Position) -> String {
    let mut sb = String::new();

    let mut empty_count = 0;

    for i in 0..64 {
        let sq = square::flip_square(i);
        let (side, piece) = p.side_piece_on_square(sq);
        if piece == PIECE_EMPTY {
            empty_count += 1;
        } else {
            if empty_count != 0 {
                write!(sb, "{}", empty_count);
                empty_count = 0;
            }
            let piece_names = ['p','n', 'b','r','q','k'];
            let mut spiece = piece_names[piece-PIECE_PAWN];
            if side==SIDE_WHITE{
                spiece= spiece.to_ascii_uppercase();
            }
            sb.push(spiece);
        }

        if file(sq) == FILE_H {
            if empty_count != 0 {
                write!(sb, "{}", empty_count);
                empty_count = 0;
            }
            if rank(sq) != RANK_1 {
                sb.push_str("/");
            }
        }
    }
    sb.push_str(" ");

    if p.side_to_move == SIDE_WHITE {
        sb.push_str("w");
    } else {
        sb.push_str("b");
    }
    sb.push_str(" ");

    if p.castling_rights == 0 {
        sb.push_str("-");
    } else {
        if (p.castling_rights & WHITE_KING_SIDE) != 0 {
            sb.push_str("K");
        }
        if (p.castling_rights & WHITE_QUEEN_SIDE) != 0 {
            sb.push_str("Q");
        }
        if (p.castling_rights & BLACK_KING_SIDE) != 0 {
            sb.push_str("k");
        }
        if (p.castling_rights & BLACK_QUEEN_SIDE) != 0 {
            sb.push_str("q");
        }
    }
    sb.push_str(" ");

    if p.ep_square == SQUARE_NONE {
        sb.push_str("-");
    } else {
        sb.push_str(&square_name(p.ep_square));
    }
    write!(sb, " {} {}", p.rule50, p.rule50 / 2 + 1);
    return sb;
}

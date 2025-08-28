use super::rand::XorshiftRng;
use super::{Move, Piece, Side, Square, bitboard};

pub const CR_WHITE_KING_SIDE: u8 = 1;
pub const CR_WHITE_QUEEN_SIDE: u8 = 2;
pub const CR_BLACK_KING_SIDE: u8 = 4;
pub const CR_BLACK_QUEEN_SIDE: u8 = 8;

struct PieceInfo {
    side: Side,
    piece: Piece,
    square: Square,
}

#[derive(Clone)]
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
    pub side_to_move: Side,
    pub castling_rights: u8,
    pub rule50: isize,
    pub ep_square: Option<Square>,
    piece_key: u64,
    pub key: u64,
}

impl Position {
    pub const INITIAL_POSITION_FEN: &str =
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    pub fn all_pieces(&self) -> u64 {
        return self.white | self.black;
    }

    pub fn is_check(&self) -> bool {
        return self.checkers != 0;
    }

    pub fn colours(&self, side: Side) -> u64 {
        match side {
            Side::WHITE => self.white,
            Side::BLACK => self.black,
        }
    }

    pub fn king_sq(&self, side: Side) -> Square {
        return bitboard::first_one(self.kings & self.colours(side));
    }

    fn attackers_to(&self, sq: Square) -> u64 {
        let occ = self.white | self.black;
        return (bitboard::pawn_attacks(Side::BLACK, sq) & self.pawns & self.white)
            | (bitboard::pawn_attacks(Side::WHITE, sq) & self.pawns & self.black)
            | (bitboard::knight_attacks(sq) & self.knights)
            | (bitboard::bishop_attacks(sq, occ) & (self.bishops | self.queens))
            | (bitboard::rook_attacks(sq, occ) & (self.rooks | self.queens))
            | (bitboard::king_attacks(sq) & self.kings);
    }

    pub fn attackers_by_side(&self, side: Side, sq: Square) -> u64 {
        return self.colours(side) & self.attackers_to(sq);
    }

    fn compute_checkers(&self) -> u64 {
        return self.attackers_by_side(self.side_to_move.opp(), self.king_sq(self.side_to_move));
    }

    fn is_legal(&self) -> bool {
        return self.attackers_by_side(self.side_to_move, self.king_sq(self.side_to_move.opp()))
            == 0;
    }

    pub fn side_piece_on_square(&self, sq: Square) -> Option<(Side, Piece)> {
        let piece = self.piece_on_square(sq);
        if piece == Piece::NONE {
            return None;
        }
        if sq.to_bitboard() & self.white != 0 {
            Some((Side::WHITE, piece))
        } else {
            Some((Side::BLACK, piece))
        }
    }

    pub fn piece_on_square(&self, sq: Square) -> Piece {
        let b = sq.to_bitboard();
        if (self.white | self.black) & b == 0 {
            return Piece::NONE;
        } else if self.pawns & b != 0 {
            return Piece::PAWN;
        } else if self.knights & b != 0 {
            return Piece::KNIGHT;
        } else if self.bishops & b != 0 {
            return Piece::BISHOP;
        } else if self.rooks & b != 0 {
            return Piece::ROOK;
        } else if self.queens & b != 0 {
            return Piece::QUEEN;
        } else if self.kings & b != 0 {
            return Piece::KING;
        }
        return Piece::NONE;
    }

    fn new(
        pieces: &[PieceInfo],
        side_to_move: Side,
        castle_rights: u8,
        ep_square: Option<Square>,
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
            piece_key: 0,
            key: 0,
            castling_rights: castle_rights,
            ep_square: ep_square,
            rule50: fifty,
        };
        for piece_info in pieces.iter() {
            pos.xor_piece(piece_info.side, piece_info.piece, piece_info.square);
        }
        pos.checkers = pos.compute_checkers();
        if !pos.is_legal() {
            return None;
        }
        pos.update_key();
        return Some(pos);
    }

    pub fn from_fen(fen: &str) -> Option<Position> {
        let tokens: Vec<_> = fen.split(' ').collect();
        if tokens.len() <= 3 {
            return None;
        }

        let mut pieces: Vec<PieceInfo> = Vec::with_capacity(32);
        let mut i = 0;
        for ch in tokens[0].chars() {
            if ch.is_ascii_digit() {
                let n: usize = ch.to_string().parse().ok()?;
                i += n;
            } else if ch.is_ascii_alphabetic() {
                let side = if ch.is_ascii_uppercase() {
                    Side::WHITE
                } else {
                    Side::BLACK
                };
                let piece = char_to_piece(ch.to_ascii_lowercase());
                if let Some(piece) = piece {
                    pieces.push(PieceInfo {
                        side: side,
                        piece: piece,
                        square: Square(i as u8).flip(),
                    });
                    i += 1;
                };
            }
        }

        let side_to_move = if tokens[1] == "w" {
            Side::WHITE
        } else {
            Side::BLACK
        };

        let s_castle_rights = tokens[2];
        let mut cr: u8 = 0;
        if s_castle_rights.contains("K") {
            cr |= CR_WHITE_KING_SIDE;
        }
        if s_castle_rights.contains("Q") {
            cr |= CR_WHITE_QUEEN_SIDE;
        }
        if s_castle_rights.contains("k") {
            cr |= CR_BLACK_KING_SIDE;
        }
        if s_castle_rights.contains("q") {
            cr |= CR_BLACK_QUEEN_SIDE;
        }

        let ep_square = Square::parse(tokens[3]);

        let mut rule50: isize = 0;
        if tokens.len() > 4 {
            rule50 = tokens[4].parse::<isize>().unwrap_or(0);
        }

        return Position::new(&pieces, side_to_move, cr, ep_square, rule50);
    }

    pub fn make_move(&self, m: Move, result: &mut Position) -> bool {
        let from = m.from();
        let to = m.to();
        let moving_piece = m.moving_piece();
        let captured_piece = m.captured_piece();

        result.pawns = self.pawns;
        result.knights = self.knights;
        result.bishops = self.bishops;
        result.rooks = self.rooks;
        result.queens = self.queens;
        result.kings = self.kings;
        result.white = self.white;
        result.black = self.black;
        result.side_to_move = self.side_to_move.opp();
        result.piece_key = self.piece_key;
        result.ep_square = None;

        result.castling_rights =
            self.castling_rights & CASTLE_MASK[from.index()] & CASTLE_MASK[to.index()];

        if moving_piece == Piece::PAWN || captured_piece != Piece::NONE {
            result.rule50 = 0;
        } else {
            result.rule50 = self.rule50 + 1;
        }

        if captured_piece != Piece::NONE {
            if captured_piece == Piece::PAWN && Some(to) == self.ep_square {
                result.xor_piece(
                    result.side_to_move,
                    Piece::PAWN,
                    to.with_step(if self.side_to_move == Side::WHITE {
                        -8
                    } else {
                        8
                    }),
                );
            } else {
                result.xor_piece(result.side_to_move, captured_piece, to);
            }
        }

        result.move_piece(self.side_to_move, moving_piece, from, to);

        if moving_piece == Piece::PAWN {
            if self.side_to_move == Side::WHITE {
                if to == from.with_step(16) {
                    result.ep_square = Some(from.with_step(8));
                }
                if to.rank() == Square::RANK_8 {
                    result.xor_piece(Side::WHITE, Piece::PAWN, to);
                    result.xor_piece(Side::WHITE, m.promotion(), to);
                }
            } else {
                // overflow: if to == from - 16 {
                if from == to.with_step(16) {
                    result.ep_square = Some(from.with_step(-8));
                }
                if to.rank() == Square::RANK_1 {
                    result.xor_piece(Side::BLACK, Piece::PAWN, to);
                    result.xor_piece(Side::BLACK, m.promotion(), to);
                }
            }
        } else if moving_piece == Piece::KING {
            if self.side_to_move == Side::WHITE {
                if from == Square::E1 && to == Square::G1 {
                    result.move_piece(Side::WHITE, Piece::ROOK, Square::H1, Square::F1);
                }
                if from == Square::E1 && to == Square::C1 {
                    result.move_piece(Side::WHITE, Piece::ROOK, Square::A1, Square::D1);
                }
            } else {
                if from == Square::E8 && to == Square::G8 {
                    result.move_piece(Side::BLACK, Piece::ROOK, Square::H8, Square::F8);
                }
                if from == Square::E8 && to == Square::C8 {
                    result.move_piece(Side::BLACK, Piece::ROOK, Square::A8, Square::D8);
                }
            }
        }

        if !result.is_legal() {
            return false;
        }
        result.checkers = result.compute_checkers();
        result.update_key();
        debug_assert!(result.key == result.compute_key_slow());
        return true;
    }

    pub fn make_null_move(&self, result: &mut Position) {
        debug_assert!(self.checkers == 0);

        result.white = self.white;
        result.black = self.black;
        result.pawns = self.pawns;
        result.knights = self.knights;
        result.bishops = self.bishops;
        result.rooks = self.rooks;
        result.queens = self.queens;
        result.kings = self.kings;

        result.checkers = 0;
        result.side_to_move = self.side_to_move.opp();
        result.castling_rights = self.castling_rights;
        result.rule50 = self.rule50 + 1;
        result.ep_square = None;
        result.piece_key = self.piece_key;
        result.update_key();

        debug_assert!(result.key == result.compute_key_slow());
    }

    fn update_key(&mut self) {
        let mut key = self.piece_key;
        if self.side_to_move == Side::WHITE {
            key ^= HASH_KEYS.side;
        }
        key ^= HASH_KEYS.castling[self.castling_rights as usize];
        //TODO можно проверять, что у противника есть пешка, кот может взять на проходе
        if let Some(ep_square) = self.ep_square {
            key ^= HASH_KEYS.enpassant[ep_square.file() as usize];
        }
        self.key = key;
    }

    fn compute_key_slow(&self) -> u64 {
        let mut res = 0_u64;

        let mut bb = self.all_pieces();
        while bb != 0 {
            let sq = bitboard::first_one(bb);
            bb &= bb - 1;
            let (side, piece) = self.side_piece_on_square(sq).unwrap();
            res ^= HASH_KEYS.psq_key(side, piece, sq);
        }

        if self.side_to_move == Side::WHITE {
            res ^= HASH_KEYS.side;
        }
        res ^= HASH_KEYS.castling[self.castling_rights as usize];
        if let Some(ep_square) = self.ep_square {
            res ^= HASH_KEYS.enpassant[ep_square.file() as usize];
        }

        return res;
    }

    fn xor_piece(&mut self, side: Side, piece: Piece, sq: Square) {
        let b = sq.to_bitboard();
        match side {
            Side::WHITE => self.white ^= b,
            Side::BLACK => self.black ^= b,
        }
        match piece {
            Piece::NONE => debug_assert!(false),
            Piece::PAWN => self.pawns ^= b,
            Piece::KNIGHT => self.knights ^= b,
            Piece::BISHOP => self.bishops ^= b,
            Piece::ROOK => self.rooks ^= b,
            Piece::QUEEN => self.queens ^= b,
            Piece::KING => self.kings ^= b,
        }
        self.piece_key ^= HASH_KEYS.psq_key(side, piece, sq);
    }

    fn move_piece(&mut self, side: Side, piece: Piece, from: Square, to: Square) {
        let b = from.to_bitboard() ^ to.to_bitboard();
        match side {
            Side::WHITE => self.white ^= b,
            Side::BLACK => self.black ^= b,
        }
        match piece {
            Piece::NONE => debug_assert!(false),
            Piece::PAWN => self.pawns ^= b,
            Piece::KNIGHT => self.knights ^= b,
            Piece::BISHOP => self.bishops ^= b,
            Piece::ROOK => self.rooks ^= b,
            Piece::QUEEN => self.queens ^= b,
            Piece::KING => self.kings ^= b,
        }
        self.piece_key ^= HASH_KEYS.psq_key(side, piece, from) ^ HASH_KEYS.psq_key(side, piece, to);
    }
}

fn char_to_piece(ch: char) -> Option<Piece> {
    match ch {
        'p' => Some(Piece::PAWN),
        'n' => Some(Piece::KNIGHT),
        'b' => Some(Piece::BISHOP),
        'r' => Some(Piece::ROOK),
        'q' => Some(Piece::QUEEN),
        'k' => Some(Piece::KING),
        _ => None,
    }
}

static CASTLE_MASK: [u8; Square::SQUARE_NB] = init_castle_mask();

const fn init_castle_mask() -> [u8; 64] {
    const CASTLEMASK_ALL: u8 =
        CR_WHITE_KING_SIDE | CR_WHITE_QUEEN_SIDE | CR_BLACK_KING_SIDE | CR_BLACK_QUEEN_SIDE;
    let mut res = [CASTLEMASK_ALL; 64];

    res[Square::A1.index()] &= !CR_WHITE_QUEEN_SIDE;
    res[Square::E1.index()] &= !(CR_WHITE_KING_SIDE | CR_WHITE_QUEEN_SIDE);
    res[Square::H1.index()] &= !CR_WHITE_KING_SIDE;

    res[Square::A8.index()] &= !CR_BLACK_QUEEN_SIDE;
    res[Square::E8.index()] &= !(CR_BLACK_KING_SIDE | CR_BLACK_QUEEN_SIDE);
    res[Square::H8.index()] &= !CR_BLACK_KING_SIDE;

    return res;
}

struct HashKeys {
    side: u64,
    enpassant: [u64; 8],
    castling: [u64; 16],
    psq: [u64; 1_024],
}

impl HashKeys {
    fn psq_key(&self, side: Side, piece: Piece, sq: Square) -> u64 {
        let index = ((side as usize) << 9) ^ ((piece as usize) << 6) ^ (sq.index());
        return self.psq[index];
    }

    const fn new() -> Self {
        let mut rng = XorshiftRng::new();

        let mut castle_keys = [0_u64; 4];
        let mut i = 0;
        while i < castle_keys.len() {
            castle_keys[i] = rng.next();
            i += 1;
        }

        let mut res = HashKeys {
            side: rng.next(),
            enpassant: [0_u64; 8],
            castling: [0_u64; 16],
            psq: [0_u64; 1_024],
        };

        let mut i = 0;
        while i < res.enpassant.len() {
            res.enpassant[i] = rng.next();
            i += 1;
        }

        let mut i = 0;
        while i < res.castling.len() {
            let mut j = 0;
            while j < castle_keys.len() {
                if (i & (1 << j)) != 0 {
                    res.castling[i] ^= castle_keys[j];
                }
                j += 1;
            }
            i += 1;
        }

        let mut i = 0;
        while i < res.psq.len() {
            res.psq[i] = rng.next();
            i += 1;
        }

        return res;
    }
}

static HASH_KEYS: HashKeys = HashKeys::new();

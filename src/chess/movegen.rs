use crate::chess::Side;
use std::fmt;

use super::{Move, Piece, Position, Square, bitboard, position};

pub const MAX_MOVES: usize = 256;

const RANK2_MASK: u64 = bitboard::RANKMASK[Square::RANK_2 as usize];
const RANK3_MASK: u64 = bitboard::RANKMASK[Square::RANK_3 as usize];
const RANK6_MASK: u64 = bitboard::RANKMASK[Square::RANK_6 as usize];
const RANK7_MASK: u64 = bitboard::RANKMASK[Square::RANK_7 as usize];

const F1G1_MASK: u64 = Square::F1.to_bitboard() | Square::G1.to_bitboard();
const B1D1_MASK: u64 =
    Square::B1.to_bitboard() | Square::C1.to_bitboard() | Square::D1.to_bitboard();

const F8G8_MASK: u64 = Square::F8.to_bitboard() | Square::G8.to_bitboard();
const B8D8_MASK: u64 =
    Square::B8.to_bitboard() | Square::C8.to_bitboard() | Square::D8.to_bitboard();

#[derive(Copy, Clone)]
pub struct OrderedMove {
    pub mv: Move,
    pub key: i32,
}

#[derive(Clone)]
pub struct MoveList {
    pub moves: [OrderedMove; MAX_MOVES],
    pub size: usize,
}

impl fmt::Debug for MoveList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for x in &self.moves[..self.size] {
            write!(f, "{:?},", x.mv)?;
        }
        writeln!(f)
    }
}

impl MoveList {
    #[allow(invalid_value)]
    pub fn new() -> MoveList {
        return MoveList {
            moves: unsafe { std::mem::MaybeUninit::uninit().assume_init() },
            size: 0,
        };
    }

    pub fn sort(&mut self) {
        self.moves[..self.size].sort_unstable_by_key(|x| -x.key);
    }

    pub fn gen_moves(&mut self, pos: &Position) {
        self.size = 0;

        let own_pieces = pos.colours(pos.side_to_move);
        let own_pawns = pos.pawns & own_pieces;
        let all_pieces = pos.all_pieces();
        let quiet_target = !all_pieces;
        let noisy_target = pos.colours(pos.side_to_move.opp());

        if let Some(ep_sq) = pos.ep_square {
            let mut from_bb = bitboard::pawn_attacks(pos.side_to_move.opp(), ep_sq) & own_pawns;
            while from_bb != 0 {
                let from = bitboard::first_one(from_bb);
                self.add(Move::make(from, ep_sq, Piece::PAWN, Piece::PAWN));
                from_bb &= from_bb - 1;
            }
        }

        if pos.side_to_move == Side::WHITE {
            let normal_moves = bitboard::up(own_pawns) & !all_pieces;
            self.gen_pawn_moves(pos, normal_moves & quiet_target, 8, true);
            self.gen_pawn_moves(
                pos,
                bitboard::up(normal_moves & RANK3_MASK) & !all_pieces & quiet_target,
                16,
                true,
            );
            self.gen_pawn_moves(pos, bitboard::up_left(own_pawns) & noisy_target, 7, true);
            self.gen_pawn_moves(pos, bitboard::up_right(own_pawns) & noisy_target, 9, true);
        } else {
            let normal_moves = bitboard::down(own_pawns) & !all_pieces;
            self.gen_pawn_moves(pos, normal_moves & quiet_target, -8, true);
            self.gen_pawn_moves(
                pos,
                bitboard::down(normal_moves & RANK6_MASK) & !all_pieces & quiet_target,
                -16,
                true,
            );
            self.gen_pawn_moves(pos, bitboard::down_left(own_pawns) & noisy_target, -9, true);
            self.gen_pawn_moves(
                pos,
                bitboard::down_right(own_pawns) & noisy_target,
                -7,
                true,
            );
        }

        self.gen_piece_moves(pos, own_pieces, !own_pieces);
        self.gen_king_moves(pos, !own_pieces);

        if !pos.is_check() {
            if pos.side_to_move == Side::WHITE {
                if (pos.castling_rights & position::CR_WHITE_KING_SIDE) != 0
                    && (all_pieces & F1G1_MASK) == 0
                    && pos.attackers_by_side(Side::BLACK, Square::F1) == 0
                {
                    self.add(Move::make(Square::E1, Square::G1, Piece::KING, Piece::NONE));
                }
                if (pos.castling_rights & position::CR_WHITE_QUEEN_SIDE) != 0
                    && (all_pieces & B1D1_MASK) == 0
                    && pos.attackers_by_side(Side::BLACK, Square::D1) == 0
                {
                    self.add(Move::make(Square::E1, Square::C1, Piece::KING, Piece::NONE));
                }
            } else {
                if (pos.castling_rights & position::CR_BLACK_KING_SIDE) != 0
                    && (all_pieces & F8G8_MASK) == 0
                    && pos.attackers_by_side(Side::WHITE, Square::F8) == 0
                {
                    self.add(Move::make(Square::E8, Square::G8, Piece::KING, Piece::NONE));
                }
                if (pos.castling_rights & position::CR_BLACK_QUEEN_SIDE) != 0
                    && (all_pieces & B8D8_MASK) == 0
                    && pos.attackers_by_side(Side::WHITE, Square::D8) == 0
                {
                    self.add(Move::make(Square::E8, Square::C8, Piece::KING, Piece::NONE));
                }
            }
        }
    }

    pub fn gen_captures(&mut self, pos: &Position) {
        self.size = 0;

        let own_pieces = pos.colours(pos.side_to_move);
        let opp_pieces = pos.colours(pos.side_to_move.opp());
        let noisy_target = opp_pieces;
        let all_pieces = pos.all_pieces();
        let own_pawns = pos.pawns & own_pieces;

        if let Some(ep_sq) = pos.ep_square {
            let mut from_bb = bitboard::pawn_attacks(pos.side_to_move.opp(), ep_sq) & own_pawns;
            while from_bb != 0 {
                let from = bitboard::first_one(from_bb);
                self.add(Move::make(from, ep_sq, Piece::PAWN, Piece::PAWN));
                from_bb &= from_bb - 1;
            }
        }

        if pos.side_to_move == Side::WHITE {
            self.gen_pawn_moves(
                pos,
                bitboard::up(own_pawns & RANK7_MASK) & !all_pieces,
                8,
                false,
            );
            self.gen_pawn_moves(pos, bitboard::up_left(own_pawns) & noisy_target, 7, false);
            self.gen_pawn_moves(pos, bitboard::up_right(own_pawns) & noisy_target, 9, false);
        } else {
            self.gen_pawn_moves(
                pos,
                bitboard::down(own_pawns & RANK2_MASK) & !all_pieces,
                -8,
                false,
            );
            self.gen_pawn_moves(
                pos,
                bitboard::down_left(own_pawns) & noisy_target,
                -9,
                false,
            );
            self.gen_pawn_moves(
                pos,
                bitboard::down_right(own_pawns) & noisy_target,
                -7,
                false,
            );
        }

        self.gen_piece_moves(pos, own_pieces, opp_pieces);
        self.gen_king_moves(pos, opp_pieces);
    }

    pub fn gen_legal_moves(&mut self, pos: &Position) {
        self.gen_moves(pos);
        let mut child: Position = unsafe { std::mem::zeroed() };
        let mut size = 0;
        for i in 0..self.size {
            let item = self.moves[i];
            if !pos.make_move(item.mv, &mut child) {
                continue;
            }
            self.moves[size] = item;
            size += 1;
        }
        self.size = size;
    }

    fn add(self: &mut MoveList, m: Move) {
        self.moves[self.size] = OrderedMove { mv: m, key: 0 };
        self.size += 1;
    }

    fn gen_piece_moves(&mut self, pos: &Position, source: u64, target: u64) {
        let all_pieces = pos.all_pieces();

        let mut from_bb: u64;
        let mut to_bb: u64;

        from_bb = pos.knights & source;
        while from_bb != 0 {
            let from = bitboard::first_one(from_bb);
            to_bb = bitboard::knight_attacks(from) & target;
            while to_bb != 0 {
                let to = bitboard::first_one(to_bb);
                self.add(Move::make(from, to, Piece::KNIGHT, pos.piece_on_square(to)));
                to_bb &= to_bb - 1;
            }
            from_bb &= from_bb - 1;
        }

        from_bb = pos.bishops & source;
        while from_bb != 0 {
            let from = bitboard::first_one(from_bb);
            to_bb = bitboard::bishop_attacks(from, all_pieces) & target;
            while to_bb != 0 {
                let to = bitboard::first_one(to_bb);
                self.add(Move::make(from, to, Piece::BISHOP, pos.piece_on_square(to)));
                to_bb &= to_bb - 1;
            }
            from_bb &= from_bb - 1;
        }

        from_bb = pos.rooks & source;
        while from_bb != 0 {
            let from = bitboard::first_one(from_bb);
            to_bb = bitboard::rook_attacks(from, all_pieces) & target;
            while to_bb != 0 {
                let to = bitboard::first_one(to_bb);
                self.add(Move::make(from, to, Piece::ROOK, pos.piece_on_square(to)));
                to_bb &= to_bb - 1;
            }
            from_bb &= from_bb - 1;
        }

        from_bb = pos.queens & source;
        while from_bb != 0 {
            let from = bitboard::first_one(from_bb);
            to_bb = bitboard::queen_attacks(from, all_pieces) & target;
            while to_bb != 0 {
                let to = bitboard::first_one(to_bb);
                self.add(Move::make(from, to, Piece::QUEEN, pos.piece_on_square(to)));
                to_bb &= to_bb - 1;
            }
            from_bb &= from_bb - 1;
        }
    }

    fn gen_king_moves(&mut self, pos: &Position, target: u64) {
        let from = pos.king_sq(pos.side_to_move);
        let mut to_bb = bitboard::king_attacks(from) & target;
        while to_bb != 0 {
            let to = bitboard::first_one(to_bb);
            self.add(Move::make(from, to, Piece::KING, pos.piece_on_square(to)));
            to_bb &= to_bb - 1;
        }
    }

    fn gen_pawn_moves(
        &mut self,
        pos: &Position,
        target: u64,
        direction: isize,
        all_promotions: bool,
    ) {
        let mut to_bb = target;
        while to_bb != 0 {
            let to = bitboard::first_one(to_bb);
            if to.rank() == Square::RANK_8 || to.rank() == Square::RANK_1 {
                let mv = Move::make(
                    to.with_step(-direction),
                    to,
                    Piece::PAWN,
                    pos.piece_on_square(to),
                );
                self.add(mv.with_promotion(Piece::QUEEN));
                if all_promotions {
                    self.add(mv.with_promotion(Piece::KNIGHT));
                    self.add(mv.with_promotion(Piece::BISHOP));
                    self.add(mv.with_promotion(Piece::ROOK));
                }
            } else {
                self.add(Move::make(
                    to.with_step(-direction),
                    to,
                    Piece::PAWN,
                    pos.piece_on_square(to),
                ));
            }
            to_bb &= to_bb - 1;
        }
    }
}

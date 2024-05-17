use super::bitboard::*;
use super::square::*;
use super::*;

const F1G1_MASK: u64 = (1_u64 << SQUARE_F1) | (1_u64 << SQUARE_G1);
const B1D1_MASK: u64 = (1_u64 << SQUARE_B1) | (1_u64 << SQUARE_C1) | (1_u64 << SQUARE_D1);
const F8G8_MASK: u64 = (1_u64 << SQUARE_F8) | (1_u64 << SQUARE_G8);
const B8D8_MASK: u64 = (1_u64 << SQUARE_B8) | (1_u64 << SQUARE_C8) | (1_u64 << SQUARE_D8);

fn add_promotions(ml: &mut MoveList, m: Move) {
    ml.add(m.with_promotion(PIECE_QUEEN));
    ml.add(m.with_promotion(PIECE_ROOK));
    ml.add(m.with_promotion(PIECE_BISHOP));
    ml.add(m.with_promotion(PIECE_KNIGHT));
}

pub fn generate_moves(pos: &Position, ml: &mut MoveList) {
    ml.clear();
    let own_pieces = pos.colours(pos.side_to_move);
    let opp_pieces = pos.colours(pos.side_to_move ^ 1);
    let target = !own_pieces;
    let all_pieces = pos.white | pos.black;

    let mut from_bb: u64;
    let mut to_bb: u64;

    if pos.ep_square != SQUARE_NONE {
        from_bb = pawn_attacks(pos.side_to_move ^ 1, pos.ep_square) & pos.pawns & own_pieces;
        while from_bb != 0 {
            let from = first_one(from_bb);
            ml.add(Move::make(
                from,
                pos.ep_square,
                PIECE_PAWN,
                PIECE_PAWN,
            ));
            from_bb &= from_bb - 1;
        }
    }

    if pos.side_to_move==SIDE_WHITE {
        from_bb=pos.pawns& own_pieces& !RANK7_MASK;
        while from_bb !=0 {
            let from = first_one(from_bb);
            if square_mask(from+8)&all_pieces==0 {
                ml.add(Move::make(from, from + 8, PIECE_PAWN, PIECE_EMPTY));
                if rank(from)==RANK_2 && square_mask(from+16)&all_pieces==0{
                    ml.add(Move::make(from, from + 16, PIECE_PAWN, PIECE_EMPTY));
                }
            }
            if file(from)>FILE_A && square_mask(from+7)&opp_pieces!=0{
                ml.add(Move::make(from, from + 7, PIECE_PAWN, pos.piece_on_square(from+7)));
            }
            if file(from)<FILE_H && square_mask(from+9)&opp_pieces!=0{
                ml.add(Move::make(from, from + 9, PIECE_PAWN, pos.piece_on_square(from+9)));
            }
            from_bb&=from_bb-1;
        }

        from_bb=pos.pawns& own_pieces& RANK7_MASK;
        while from_bb !=0 {
            let from = first_one(from_bb);
            if square_mask(from+8)&all_pieces==0 {
                add_promotions(ml, Move::make(from, from + 8, PIECE_PAWN, PIECE_EMPTY));
            }
            if file(from)>FILE_A && square_mask(from+7)&opp_pieces!=0{
                add_promotions(ml, Move::make(from, from + 7, PIECE_PAWN, pos.piece_on_square(from+7)));
            }
            if file(from)<FILE_H && square_mask(from+9)&opp_pieces!=0{
                add_promotions(ml, Move::make(from, from + 9, PIECE_PAWN, pos.piece_on_square(from+9)));
            }
            from_bb&=from_bb-1;
        }
    } else {
        from_bb=pos.pawns& own_pieces& !RANK2_MASK;
        while from_bb !=0 {
            let from = first_one(from_bb);
            if square_mask(from-8)&all_pieces==0 {
                ml.add(Move::make(from, from - 8, PIECE_PAWN, PIECE_EMPTY));
                if rank(from)==RANK_7 && square_mask(from-16)&all_pieces==0{
                    ml.add(Move::make(from, from - 16, PIECE_PAWN, PIECE_EMPTY));
                }
            }
            if file(from)>FILE_A && square_mask(from-9)&opp_pieces!=0{
                ml.add(Move::make(from, from -9, PIECE_PAWN, pos.piece_on_square(from-9)));
            }
            if file(from)<FILE_H && square_mask(from-7)&opp_pieces!=0{
                ml.add(Move::make(from, from -7, PIECE_PAWN, pos.piece_on_square(from-7)));
            }
            from_bb&=from_bb-1;
        }

        from_bb=pos.pawns& own_pieces& RANK2_MASK;
        while from_bb !=0 {
            let from = first_one(from_bb);
            if square_mask(from-8)&all_pieces==0 {
                add_promotions(ml, Move::make(from, from - 8, PIECE_PAWN, PIECE_EMPTY));
            }
            if file(from)>FILE_A && square_mask(from-9)&opp_pieces!=0{
                add_promotions(ml, Move::make(from, from -9, PIECE_PAWN, pos.piece_on_square(from-9)));
            }
            if file(from)<FILE_H && square_mask(from-7)&opp_pieces!=0{
                add_promotions(ml, Move::make(from, from -7, PIECE_PAWN, pos.piece_on_square(from-7)));
            }
            from_bb&=from_bb-1;
        }
    }

    /*if pos.side_to_move == SIDE_WHITE {
        // pawn non captures
        to_bb = up(pos.pawns & own_pieces) & !all_pieces;
        while to_bb != 0 {
            let to = first_one(to_bb);
            let from = to - 8;
            let m = Move::make(from, to, PIECE_PAWN, PIECE_EMPTY);
            if rank(to) == RANK_8 {
                ml.add(m.with_promotion(PIECE_QUEEN));
                ml.add(m.with_promotion(PIECE_KNIGHT));
                ml.add(m.with_promotion(PIECE_BISHOP));
                ml.add(m.with_promotion(PIECE_ROOK));
            } else {
                ml.add(m);
                // double push
                if rank(from) == RANK_2 && (square_mask(from + 16) & all_pieces) == 0 {
                    ml.add(Move::make(from, from + 16, PIECE_PAWN, PIECE_EMPTY));
                }
            }
            to_bb &= to_bb - 1;
        }

        // pawn captures
        to_bb = up_left(pos.pawns & own_pieces) & opp_pieces;
        while to_bb != 0 {
            let to = first_one(to_bb);
            let from = to - 7;
            let m = Move::make(from, to, PIECE_PAWN, pos.piece_on_square(to));
            if rank(to) == RANK_8 {
                ml.add(m.with_promotion(PIECE_QUEEN));
                ml.add(m.with_promotion(PIECE_KNIGHT));
                ml.add(m.with_promotion(PIECE_BISHOP));
                ml.add(m.with_promotion(PIECE_ROOK));
            } else {
                ml.add(m);
            }
            to_bb &= to_bb - 1;
        }

        to_bb = up_right(pos.pawns & own_pieces) & opp_pieces;
        while to_bb != 0 {
            let to = first_one(to_bb);
            let from = to - 9;
            let m = Move::make(from, to, PIECE_PAWN, pos.piece_on_square(to));
            if rank(to) == RANK_8 {
                ml.add(m.with_promotion(PIECE_QUEEN));
                ml.add(m.with_promotion(PIECE_KNIGHT));
                ml.add(m.with_promotion(PIECE_BISHOP));
                ml.add(m.with_promotion(PIECE_ROOK));
            } else {
                ml.add(m);
            }
            to_bb &= to_bb - 1;
        }
    } else {
        // pawn non captures
        to_bb = down(pos.pawns & own_pieces) & !all_pieces;
        while to_bb != 0 {
            let to = first_one(to_bb);
            let from = to + 8;
            let m = Move::make(from, to, PIECE_PAWN, PIECE_EMPTY);
            if rank(to) == RANK_1 {
                ml.add(m.with_promotion(PIECE_QUEEN));
                ml.add(m.with_promotion(PIECE_KNIGHT));
                ml.add(m.with_promotion(PIECE_BISHOP));
                ml.add(m.with_promotion(PIECE_ROOK));
            } else {
                ml.add(m);
                // double push
                if rank(from) == RANK_7 && (square_mask(from - 16) & all_pieces) == 0 {
                    ml.add(Move::make(from, from - 16, PIECE_PAWN, PIECE_EMPTY));
                }
            }
            to_bb &= to_bb - 1;
        }

        // pawn captures
        to_bb = down_left(pos.pawns & own_pieces) & opp_pieces;
        while to_bb != 0 {
            let to = first_one(to_bb);
            let from = to + 9;
            let m = Move::make(from, to, PIECE_PAWN, pos.piece_on_square(to));
            if rank(to) == RANK_1 {
                ml.add(m.with_promotion(PIECE_QUEEN));
                ml.add(m.with_promotion(PIECE_KNIGHT));
                ml.add(m.with_promotion(PIECE_BISHOP));
                ml.add(m.with_promotion(PIECE_ROOK));
            } else {
                ml.add(m);
            }
            to_bb &= to_bb - 1;
        }

        to_bb = down_right(pos.pawns & own_pieces) & opp_pieces;
        while to_bb != 0 {
            let to = first_one(to_bb);
            let from = to + 7;
            let m = Move::make(from, to, PIECE_PAWN, pos.piece_on_square(to));
            if rank(to) == RANK_1 {
                ml.add(m.with_promotion(PIECE_QUEEN));
                ml.add(m.with_promotion(PIECE_KNIGHT));
                ml.add(m.with_promotion(PIECE_BISHOP));
                ml.add(m.with_promotion(PIECE_ROOK));
            } else {
                ml.add(m);
            }
            to_bb &= to_bb - 1;
        }
    }*/

    from_bb = pos.knights & own_pieces;
    while from_bb != 0 {
        let from = first_one(from_bb);
        to_bb = knight_attacks(from) & target;
        while to_bb != 0 {
            let to = first_one(to_bb);
            ml.add(Move::make(
                from,
                to,
                PIECE_KNIGHT,
                pos.piece_on_square(to),
            ));
            to_bb &= to_bb - 1;
        }
        from_bb &= from_bb - 1;
    }

    from_bb = pos.bishops & own_pieces;
    while from_bb != 0 {
        let from = first_one(from_bb);
        to_bb = bishop_attacks(from, all_pieces) & target;
        while to_bb != 0 {
            let to = first_one(to_bb);
            ml.add(Move::make(
                from,
                to,
                PIECE_BISHOP,
                pos.piece_on_square(to),
            ));
            to_bb &= to_bb - 1;
        }
        from_bb &= from_bb - 1;
    }

    from_bb = pos.rooks & own_pieces;
    while from_bb != 0 {
        let from = first_one(from_bb);
        to_bb = rook_attacks(from, all_pieces) & target;
        while to_bb != 0 {
            let to = first_one(to_bb);
            ml.add(Move::make(
                from,
                to,
                PIECE_ROOK,
                pos.piece_on_square(to),
            ));
            to_bb &= to_bb - 1;
        }
        from_bb &= from_bb - 1;
    }

    from_bb = pos.queens & own_pieces;
    while from_bb != 0 {
        let from = first_one(from_bb);
        to_bb = queen_attacks(from, all_pieces) & target;
        while to_bb != 0 {
            let to = first_one(to_bb);
            ml.add(Move::make(
                from,
                to,
                PIECE_QUEEN,
                pos.piece_on_square(to),
            ));
            to_bb &= to_bb - 1;
        }
        from_bb &= from_bb - 1;
    }

    let from = pos.king_sq(pos.side_to_move);
    to_bb = king_attacks(from) & !own_pieces;
    while to_bb != 0 {
        let to = first_one(to_bb);
        ml.add(Move::make(
            from,
            to,
            PIECE_KING,
            pos.piece_on_square(to),
        ));
        to_bb &= to_bb - 1;
    }

    if !pos.is_check() {
        if pos.side_to_move == SIDE_WHITE {
            if (pos.castling_rights & WHITE_KING_SIDE) != 0
                && (all_pieces & F1G1_MASK) == 0
                && pos.attackers_by_side(SIDE_BLACK, SQUARE_F1) == 0
            {
                ml.add(Move::make(
                    SQUARE_E1,
                    SQUARE_G1,
                    PIECE_KING,
                    PIECE_EMPTY,
                ));
            }
            if (pos.castling_rights & WHITE_QUEEN_SIDE) != 0
                && (all_pieces & B1D1_MASK) == 0
                && pos.attackers_by_side(SIDE_BLACK, SQUARE_D1) == 0
            {
                ml.add(Move::make(
                    SQUARE_E1,
                    SQUARE_C1,
                    PIECE_KING,
                    PIECE_EMPTY,
                ));
            }
        } else {
            if (pos.castling_rights & BLACK_KING_SIDE) != 0
                && (all_pieces & F8G8_MASK) == 0
                && pos.attackers_by_side(SIDE_WHITE, SQUARE_F8) == 0
            {
                ml.add(Move::make(
                    SQUARE_E8,
                    SQUARE_G8,
                    PIECE_KING,
                    PIECE_EMPTY,
                ));
            }
            if (pos.castling_rights & BLACK_QUEEN_SIDE) != 0
                && (all_pieces & B8D8_MASK) == 0
                && pos.attackers_by_side(SIDE_WHITE, SQUARE_D8) == 0
            {
                ml.add(Move::make(
                    SQUARE_E8,
                    SQUARE_C8,
                    PIECE_KING,
                    PIECE_EMPTY,
                ));
            }
        }
    }    
}

pub fn generate_noisy_moves(pos: &Position, ml: &mut MoveList) {
    ml.clear();
    let own_pieces = pos.colours(pos.side_to_move);
    let opp_pieces = pos.colours(pos.side_to_move ^ 1);
    let target = opp_pieces;
    let all_pieces = pos.white | pos.black;

    let mut from_bb: u64;
    let mut to_bb: u64;

    if pos.ep_square != SQUARE_NONE {
        from_bb = pawn_attacks(pos.side_to_move ^ 1, pos.ep_square) & pos.pawns & own_pieces;
        while from_bb != 0 {
            let from = first_one(from_bb);
            ml.add(Move::make(
                from,
                pos.ep_square,
                PIECE_PAWN,
                PIECE_PAWN,
            ));
            from_bb &= from_bb - 1;
        }
    }

    if pos.side_to_move == SIDE_WHITE {
        from_bb=(bitboard::all_pawn_attacks(SIDE_BLACK, opp_pieces)|RANK7_MASK)&pos.pawns&own_pieces;
        while from_bb != 0 {
            let from = first_one(from_bb);
            let promotion = if rank(from) == RANK_7 {
                PIECE_QUEEN
            } else {
                PIECE_EMPTY
            };
            if rank(from) == RANK_7 && square_mask(from+8) & all_pieces == 0 {
                ml.add(Move::make(from, from+8, PIECE_PAWN, PIECE_EMPTY)
                    .with_promotion(promotion));
            }
            if file(from)>FILE_A && square_mask(from+7)&opp_pieces!=0{
                ml.add(Move::make(from, from+7, PIECE_PAWN, pos.piece_on_square(from+7))
                    .with_promotion(promotion));
            }
            if file(from)<FILE_H && square_mask(from+9)&opp_pieces!=0{
                ml.add(Move::make(from, from+9, PIECE_PAWN, pos.piece_on_square(from+9))
                    .with_promotion(promotion));
            }
            from_bb &= from_bb - 1;
        }
    } else {
        from_bb=(bitboard::all_pawn_attacks(SIDE_WHITE, opp_pieces)|RANK2_MASK)&pos.pawns&own_pieces;
        while from_bb != 0 {
            let from = first_one(from_bb);
            let promotion = if rank(from) == RANK_2 {
                PIECE_QUEEN
            } else {
                PIECE_EMPTY
            };
            if rank(from) == RANK_2 && square_mask(from-8) & all_pieces == 0 {
                ml.add(Move::make(from, from-8, PIECE_PAWN, PIECE_EMPTY)
                    .with_promotion(promotion));
            }
            if file(from)>FILE_A && square_mask(from-9)&opp_pieces!=0{
                ml.add(Move::make(from, from-9, PIECE_PAWN, pos.piece_on_square(from-9))
                    .with_promotion(promotion));
            }
            if file(from)<FILE_H && square_mask(from-7)&opp_pieces!=0{
                ml.add(Move::make(from, from-7, PIECE_PAWN, pos.piece_on_square(from-7))
                    .with_promotion(promotion));
            }
            from_bb &= from_bb - 1;
        }
    }

    /*if pos.side_to_move == SIDE_WHITE {
        // pawn non captures
        to_bb = up(pos.pawns & own_pieces) & !all_pieces & RANK8_MASK;
        while to_bb != 0 {
            let to = first_one(to_bb);
            let from = to - 8;
            let m = Move::make(from, to, PIECE_PAWN, PIECE_EMPTY);
            ml.add(m.with_promotion(PIECE_QUEEN));
            to_bb &= to_bb - 1;
        }

        // pawn captures
        to_bb = up_left(pos.pawns & own_pieces) & opp_pieces;
        while to_bb != 0 {
            let to = first_one(to_bb);
            let from = to - 7;
            let m = Move::make(from, to, PIECE_PAWN, pos.piece_on_square(to));
            if rank(to) == RANK_8 {
                ml.add(m.with_promotion(PIECE_QUEEN));
            } else {
                ml.add(m);
            }
            to_bb &= to_bb - 1;
        }

        to_bb = up_right(pos.pawns & own_pieces) & opp_pieces;
        while to_bb != 0 {
            let to = first_one(to_bb);
            let from = to - 9;
            let m = Move::make(from, to, PIECE_PAWN, pos.piece_on_square(to));
            if rank(to) == RANK_8 {
                ml.add(m.with_promotion(PIECE_QUEEN));                    
            } else {
                ml.add(m);
            }
            to_bb &= to_bb - 1;
        }
    } else {
        // pawn non captures
        to_bb = down(pos.pawns & own_pieces) & !all_pieces & RANK1_MASK;
        while to_bb != 0 {
            let to = first_one(to_bb);
            let from = to + 8;
            let m = Move::make(from, to, PIECE_PAWN, PIECE_EMPTY);
            ml.add(m.with_promotion(PIECE_QUEEN));
            to_bb &= to_bb - 1;
        }

        // pawn captures
        to_bb = down_left(pos.pawns & own_pieces) & opp_pieces;
        while to_bb != 0 {
            let to = first_one(to_bb);
            let from = to + 9;
            let m = Move::make(from, to, PIECE_PAWN, pos.piece_on_square(to));
            if rank(to) == RANK_1 {
                ml.add(m.with_promotion(PIECE_QUEEN));                    
            } else {
                ml.add(m);
            }
            to_bb &= to_bb - 1;
        }

        to_bb = down_right(pos.pawns & own_pieces) & opp_pieces;
        while to_bb != 0 {
            let to = first_one(to_bb);
            let from = to + 7;
            let m = Move::make(from, to, PIECE_PAWN, pos.piece_on_square(to));
            if rank(to) == RANK_1 {
                ml.add(m.with_promotion(PIECE_QUEEN));                    
            } else {
                ml.add(m);
            }
            to_bb &= to_bb - 1;
        }
    }*/

    from_bb = pos.knights & own_pieces;
    while from_bb != 0 {
        let from = first_one(from_bb);
        to_bb = knight_attacks(from) & target;
        while to_bb != 0 {
            let to = first_one(to_bb);
            ml.add(Move::make(
                from,
                to,
                PIECE_KNIGHT,
                pos.piece_on_square(to),
            ));
            to_bb &= to_bb - 1;
        }
        from_bb &= from_bb - 1;
    }

    from_bb = pos.bishops & own_pieces;
    while from_bb != 0 {
        let from = first_one(from_bb);
        to_bb = bishop_attacks(from, all_pieces) & target;
        while to_bb != 0 {
            let to = first_one(to_bb);
            ml.add(Move::make(
                from,
                to,
                PIECE_BISHOP,
                pos.piece_on_square(to),
            ));
            to_bb &= to_bb - 1;
        }
        from_bb &= from_bb - 1;
    }

    from_bb = pos.rooks & own_pieces;
    while from_bb != 0 {
        let from = first_one(from_bb);
        to_bb = rook_attacks(from, all_pieces) & target;
        while to_bb != 0 {
            let to = first_one(to_bb);
            ml.add(Move::make(
                from,
                to,
                PIECE_ROOK,
                pos.piece_on_square(to),
            ));
            to_bb &= to_bb - 1;
        }
        from_bb &= from_bb - 1;
    }

    from_bb = pos.queens & own_pieces;
    while from_bb != 0 {
        let from = first_one(from_bb);
        to_bb = queen_attacks(from, all_pieces) & target;
        while to_bb != 0 {
            let to = first_one(to_bb);
            ml.add(Move::make(
                from,
                to,
                PIECE_QUEEN,
                pos.piece_on_square(to),
            ));
            to_bb &= to_bb - 1;
        }
        from_bb &= from_bb - 1;
    }

    let from = pos.king_sq(pos.side_to_move);
    to_bb = king_attacks(from) & opp_pieces;
    while to_bb != 0 {
        let to = first_one(to_bb);
        ml.add(Move::make(
            from,
            to,
            PIECE_KING,
            pos.piece_on_square(to),
        ));
        to_bb &= to_bb - 1;
    }
}

pub fn generate_legal_moves(pos: &Position, ml: &mut MoveList) {    
    let mut buffer = MoveList::new();    
    generate_moves(pos, &mut buffer);
    let mut history = History::new();
    ml.clear();
    for item in buffer.moves[..buffer.size].iter() {
        let mut child = pos.clone();
        if !child.make_move(item.mv, &mut history) {
            continue;
        }
        ml.add(item.mv);
    }
}

use crate::chess::{Move, Piece, Position, Side, Square, bitboard};

#[derive(Debug)]
struct SeeTest {
    fen: &'static str,
    mv: &'static str,
    val: isize,
}

#[test]
fn test_see() {
    unsafe { crate::chess::init() };
    let tests = [
        SeeTest {
            fen: "3r2qk/3rnbpp/8/3p4/8/4NBP1/3R2QP/3R2K1 w - - 0 1",
            mv: "e3d5",
            val: piece_value(Piece::PAWN) - piece_value(Piece::KNIGHT),
        },
        SeeTest {
            fen: "3r1q1k/3rnbpp/8/3p4/8/4NBP1/3R2QP/3R2K1 w - - 0 1",
            mv: "e3d5",
            val: piece_value(Piece::PAWN),
        },
        SeeTest {
            fen: "8/4k3/8/3n4/8/8/3R4/3K4 w - - 0 1",
            mv: "d2d5",
            val: piece_value(Piece::KNIGHT),
        },
        SeeTest {
            fen: "8/4k3/1n6/3n4/8/8/3R4/3K4 w - - 0 1",
            mv: "d2d5",
            val: piece_value(Piece::KNIGHT) - piece_value(Piece::ROOK),
        },
        SeeTest {
            fen: "4r1k1/5pp1/nbp4p/1p2p2q/1P2P1b1/1BP2N1P/1B2QPPK/3R4 b - -",
            mv: "g4f3",
            val: piece_value(Piece::KNIGHT) - piece_value(Piece::BISHOP),
        },
        SeeTest {
            fen: "1k1r3q/1ppn3p/p4b2/4p3/8/P2N2P1/1PP1R1BP/2K1Q3 w - - ",
            mv: "d3e5",
            val: piece_value(Piece::PAWN) - piece_value(Piece::KNIGHT),
        },
        SeeTest {
            fen: "8/p3q1kp/1p2Pnp1/3pQ3/2pP4/1nP3NP/1B4P1/6K1 b - - 0 1",
            mv: "e7e6",
            val: piece_value(Piece::PAWN) - piece_value(Piece::QUEEN),
        },
    ];
    for test in tests.iter() {
        let pos = Position::from_fen(&test.fen);
        assert!(pos.is_some());
        let pos = &pos.unwrap();
        let mv = Move::parse_lan(pos, &test.mv);
        assert!(mv.is_some());
        let mv = mv.unwrap();
        if !see_ge(&pos, mv, test.val) {
            panic!("see#1 {:?}", test);
        }
        if see_ge(&pos, mv, test.val + 1) {
            panic!("see#2 {:?}", test);
        }
    }
}

// based on Ethereal
pub fn see_ge(pos: &Position, mv: Move, threshold: isize) -> bool {
    let from = mv.from();
    let to = mv.to();
    let moving_piece = mv.moving_piece();
    let captured_piece = mv.captured_piece();
    let promotion_piece = mv.promotion();

    let mut next_victim = moving_piece;
    if promotion_piece != Piece::NONE {
        next_victim = promotion_piece;
    }

    let mut balance = piece_value(captured_piece);
    if promotion_piece != Piece::NONE {
        balance += piece_value(promotion_piece) - piece_value(Piece::PAWN);
    }
    balance -= threshold;

    if balance < 0 {
        return false;
    }

    balance -= piece_value(next_victim);
    if balance >= 0 {
        return true;
    }

    let mut occupied = pos.all_pieces() & !from.to_bitboard() | to.to_bitboard();
    if moving_piece == Piece::PAWN && Some(to) == pos.ep_square {
        let cap_sq = if pos.side_to_move == Side::WHITE {
            to.with_step(-8)
        } else {
            to.with_step(8)
        };
        occupied &= !cap_sq.to_bitboard();
    }

    let mut attackers = compute_attackers(pos, to, occupied) & occupied;

    let bishops = pos.bishops | pos.queens;
    let rooks = pos.rooks | pos.queens;

    let mut side = pos.side_to_move.opp();

    loop {
        let my_attackers = attackers & pos.colours(side);
        if my_attackers == 0 {
            break;
        }

        let (attacker_type, attacker_from) = get_least_valuable_attacker(pos, my_attackers);

        occupied &= !attacker_from.to_bitboard();

        if attacker_type == Piece::PAWN
            || attacker_type == Piece::BISHOP
            || attacker_type == Piece::QUEEN
        {
            attackers |= bitboard::bishop_attacks(to, occupied) & bishops
        }
        if attacker_type == Piece::ROOK || attacker_type == Piece::QUEEN {
            attackers |= bitboard::rook_attacks(to, occupied) & rooks
        }

        attackers &= occupied;

        side = side.opp();

        balance = -balance - 1 - piece_value(attacker_type);
        if balance >= 0 {
            if attacker_type == Piece::KING && (attackers & pos.colours(side)) != 0 {
                side = side.opp();
            }
            break;
        }
    }

    return side != pos.side_to_move;
}

fn compute_attackers(pos: &Position, sq: Square, occ: u64) -> u64 {
    return (bitboard::pawn_attacks(Side::WHITE, sq) & pos.pawns & pos.black)
        | (bitboard::pawn_attacks(Side::BLACK, sq) & pos.pawns & pos.white)
        | (bitboard::knight_attacks(sq) & pos.knights)
        | (bitboard::king_attacks(sq) & pos.kings)
        | (bitboard::bishop_attacks(sq, occ) & (pos.bishops | pos.queens))
        | (bitboard::rook_attacks(sq, occ) & (pos.rooks | pos.queens));
}

fn get_least_valuable_attacker(p: &Position, attackers: u64) -> (Piece, Square) {
    if let Some(from) = first_one(p.pawns & attackers) {
        return (Piece::PAWN, from);
    }
    if let Some(from) = first_one(p.knights & attackers) {
        return (Piece::KNIGHT, from);
    }
    if let Some(from) = first_one(p.bishops & attackers) {
        return (Piece::BISHOP, from);
    }
    if let Some(from) = first_one(p.rooks & attackers) {
        return (Piece::ROOK, from);
    }
    if let Some(from) = first_one(p.queens & attackers) {
        return (Piece::QUEEN, from);
    }
    if let Some(from) = first_one(p.kings & attackers) {
        return (Piece::KING, from);
    }
    unreachable!("get_least_valuable_attacker");
}

fn piece_value(piece: Piece) -> isize {
    match piece {
        Piece::PAWN => 1,
        Piece::KNIGHT => 4,
        Piece::BISHOP => 4,
        Piece::ROOK => 6,
        Piece::QUEEN => 12,
        Piece::KING => 120,
        _ => 0,
    }
}

fn first_one(b: u64) -> Option<Square> {
    if b == 0 {
        return None;
    }
    return Some(bitboard::first_one(b));
}

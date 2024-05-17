use crate::chess::{self, Square, PIECE_BISHOP};

#[derive(Debug)]
struct SeeTest {
    fen: &'static str,
    mv: &'static str,
    val: isize,
}

#[test]
fn test_see() {
    chess::init();
    let tests = [
        SeeTest {
            fen: "8/4k3/8/3n4/8/8/3R4/3K4 w - - 0 1",
            mv: "d2d5",
            val: PIECE_VALUES_SEE[chess::PIECE_KNIGHT],
        },
        SeeTest {
            fen: "8/4k3/1n6/3n4/8/8/3R4/3K4 w - - 0 1",
            mv: "d2d5",
            val: PIECE_VALUES_SEE[chess::PIECE_KNIGHT] - PIECE_VALUES_SEE[chess::PIECE_ROOK],
        },
        SeeTest {
            fen: "4r1k1/5pp1/nbp4p/1p2p2q/1P2P1b1/1BP2N1P/1B2QPPK/3R4 b - -",
            mv: "g4f3",
            val: PIECE_VALUES_SEE[chess::PIECE_KNIGHT] - PIECE_VALUES_SEE[chess::PIECE_BISHOP],
        },
        SeeTest {
            fen: "1k1r3q/1ppn3p/p4b2/4p3/8/P2N2P1/1PP1R1BP/2K1Q3 w - - ",
            mv: "d3e5",
            val: PIECE_VALUES_SEE[chess::PIECE_PAWN] - PIECE_VALUES_SEE[chess::PIECE_KNIGHT],
        },
    ];
    for test in tests.iter() {
        let pos = chess::Position::from_fen(&test.fen);
        assert!(pos.is_some());
        let pos = &pos.unwrap();
        let mv = chess::chessmove::Move::parse_lan(pos, &test.mv);
        assert!(mv.is_some());
        let mv = mv.unwrap();
        let val = see(pos, mv);
        if val != test.val {
            panic!("{} {:?}", val, test);
        }
        assert!(see(pos, mv) == test.val);
    }
}

fn see(pos: &chess::Position, mv: chess::Move) -> isize {
    let from = mv.from();
    let to = mv.to();
    let mut pc = mv.moving_piece();
    let sd = pos.side_to_move;
    let mut sc = 0_isize;
    if mv.captured_piece() != chess::PIECE_EMPTY {
        sc += PIECE_VALUES_SEE[mv.captured_piece()];
    }
    if mv.promotion() != chess::PIECE_EMPTY {
        pc = mv.promotion();
        sc += PIECE_VALUES_SEE[pc] - PIECE_VALUES_SEE[chess::PIECE_PAWN];
    }
    let pieces =
        pos.all_pieces() & !chess::bitboard::square_mask(from) | chess::bitboard::square_mask(to);
    sc -= see_rec(pos, sd ^ 1, to, pieces, pc);
    return sc;
}

fn see_rec(
    pos: &chess::Position,
    side: usize,
    to: chess::Square,
    pieces: u64,
    cp: chess::Piece,
) -> isize {
    let mut bs = 0_isize;
    let (pc, from) = least_valuable_attacker(pos, to, side, pieces);
    if from != chess::square::SQUARE_NONE {
        let mut sc = PIECE_VALUES_SEE[cp];
        if cp != chess::PIECE_KING {
            sc -= see_rec(
                pos,
                side ^ 1,
                to,
                pieces & !chess::bitboard::square_mask(from),
                pc,
            );
        }
        if sc > bs {
            bs = sc;
        }
    }
    return bs;
}

fn least_valuable_attacker(
    p: &chess::Position,
    to: chess::Square,
    side: usize,
    occ: u64,
) -> (chess::Piece, chess::Square) {
    let own = p.colours(side) & occ;
    let attacks = chess::bitboard::pawn_attacks(side ^ 1, to) & own & p.pawns;
    if attacks != 0 {
        return (chess::PIECE_PAWN, chess::bitboard::first_one(attacks));
    }
    let attacks = chess::bitboard::knight_attacks(to) & own & p.knights;
    if attacks != 0 {
        return (chess::PIECE_KNIGHT, chess::bitboard::first_one(attacks));
    }
    let bishop_attacks = chess::bitboard::bishop_attacks(to, occ);
    let atatcks = bishop_attacks & own & p.bishops;
    if atatcks != 0 {
        return (chess::PIECE_BISHOP, chess::bitboard::first_one(attacks));
    }
    let rook_attacks = chess::bitboard::rook_attacks(to, occ);
    let atatcks = rook_attacks & own & p.rooks;
    if atatcks != 0 {
        return (chess::PIECE_ROOK, chess::bitboard::first_one(attacks));
    }
    let atatcks = (bishop_attacks | rook_attacks) & own & p.queens;
    if atatcks != 0 {
        return (chess::PIECE_QUEEN, chess::bitboard::first_one(attacks));
    }
    let atatcks = chess::bitboard::king_attacks(to) & own & p.kings;
    if atatcks != 0 {
        return (chess::PIECE_KING, chess::bitboard::first_one(attacks));
    }

    return (chess::PIECE_EMPTY, chess::square::SQUARE_NONE);
}

static PIECE_VALUES_SEE: [isize; chess::PIECE_NB] = [0, 1, 4, 4, 6, 12, 120, 0];

//------------

fn compute_attackers(pos: &chess::Position, sq: Square, occ: u64) -> u64 {
    return (chess::bitboard::pawn_attacks(chess::SIDE_WHITE, sq) & pos.pawns & pos.black)
        | (chess::bitboard::pawn_attacks(chess::SIDE_BLACK, sq) & pos.pawns & pos.white)
        | (chess::bitboard::knight_attacks(sq) & pos.knights)
        | (chess::bitboard::king_attacks(sq) & pos.kings)
        | (chess::bitboard::bishop_attacks(sq, occ) & (pos.bishops | pos.queens))
        | (chess::bitboard::rook_attacks(sq, occ) & (pos.rooks | pos.queens));
}

fn get_least_valuable_attacker(
    p: &chess::Position,
    attackers: u64,
) -> (chess::Piece, chess::Square) {
    if p.pawns & attackers != 0 {
        return (
            chess::PIECE_PAWN,
            chess::bitboard::first_one(p.pawns & attackers),
        );
    }
    if p.knights & attackers != 0 {
        return (
            chess::PIECE_KNIGHT,
            chess::bitboard::first_one(p.knights & attackers),
        );
    }
    if p.bishops & attackers != 0 {
        return (
            chess::PIECE_BISHOP,
            chess::bitboard::first_one(p.bishops & attackers),
        );
    }
    if p.rooks & attackers != 0 {
        return (
            chess::PIECE_ROOK,
            chess::bitboard::first_one(p.rooks & attackers),
        );
    }
    if p.queens & attackers != 0 {
        return (
            chess::PIECE_QUEEN,
            chess::bitboard::first_one(p.queens & attackers),
        );
    }
    if p.kings & attackers != 0 {
        return (
            chess::PIECE_KING,
            chess::bitboard::first_one(p.kings & attackers),
        );
    }
    return (chess::PIECE_EMPTY, chess::square::SQUARE_NONE);
}

// based on Ethereal
pub fn see_ge(pos: &chess::Position, mv: chess::Move, threshold: isize) -> bool {
    let from = mv.from();
    let to = mv.to();
    let moving_piece = mv.moving_piece();
    let captured_piece = mv.captured_piece();
    let promotion_piece = mv.promotion();

    let mut next_victim = moving_piece;
    if promotion_piece != chess::PIECE_EMPTY {
        next_victim = promotion_piece;
    }

    let mut balance = PIECE_VALUES_SEE[captured_piece];
    if promotion_piece != chess::PIECE_EMPTY {
        balance += PIECE_VALUES_SEE[promotion_piece] - PIECE_VALUES_SEE[chess::PIECE_PAWN];
    }
    balance -= threshold;

    if balance < 0 {
        return false;
    }

    balance -= PIECE_VALUES_SEE[next_victim];
    if balance >= 0 {
        return true;
    }

    let mut occupied =
        pos.all_pieces() & !chess::bitboard::square_mask(from) | chess::bitboard::square_mask(to);
    /*if movingPiece == Pawn && to == pos.EpSquare {
        var capSq int
        if pos.WhiteMove {
            capSq = to - 8
        } else {
            capSq = to + 8
        }
        occupied &^= SquareMask[capSq]
    }*/

    let mut attackers = compute_attackers(pos, to, occupied) & occupied;

    let bishops = pos.bishops | pos.queens;
    let rooks = pos.rooks | pos.queens;

    let mut side = pos.side_to_move ^ 1;

    loop {
        let my_attackers = attackers & pos.colours(side);
        if my_attackers == 0 {
            break;
        }

        let (attacker_type, attacker_from) = get_least_valuable_attacker(pos, my_attackers);

        occupied &= !chess::bitboard::square_mask(attacker_from);

        if attacker_type == chess::PIECE_PAWN
            || attacker_type == chess::PIECE_BISHOP
            || attacker_type == chess::PIECE_QUEEN
        {
            attackers |= chess::bitboard::bishop_attacks(to, occupied) & bishops
        }
        if attacker_type == chess::PIECE_ROOK || attacker_type == chess::PIECE_QUEEN {
            attackers |= chess::bitboard::rook_attacks(to, occupied) & rooks
        }

        attackers &= occupied;

        side = side ^ 1;

        balance = -balance - 1 - PIECE_VALUES_SEE[attacker_type];
        if balance >= 0 {
            if attacker_type == chess::PIECE_KING && (attackers & pos.colours(side)) != 0 {
                side = side ^ 1;
            }
            break;
        }
    }

    return side != pos.side_to_move;
}

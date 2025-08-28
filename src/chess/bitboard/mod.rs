mod magic;

use super::{Side, Square};
pub use magic::{bishop_attacks, rook_attacks};

pub const FILEA_MASK: u64 = FILEMASK[Square::FILE_A];
pub const FILEH_MASK: u64 = FILEMASK[Square::FILE_H];

pub static FILEMASK: [u64; 8] = [
    0x0101010101010101,
    0x0101010101010101 << 1,
    0x0101010101010101 << 2,
    0x0101010101010101 << 3,
    0x0101010101010101 << 4,
    0x0101010101010101 << 5,
    0x0101010101010101 << 6,
    0x0101010101010101 << 7,
];

pub static RANKMASK: [u64; 8] = [
    0xFF,
    0xFF << (8 * 1),
    0xFF << (8 * 2),
    0xFF << (8 * 3),
    0xFF << (8 * 4),
    0xFF << (8 * 5),
    0xFF << (8 * 6),
    0xFF << (8 * 7),
];

pub fn pop_count(b: u64) -> isize {
    return b.count_ones() as isize;
}

pub fn multiple(value: u64) -> bool {
    return value != 0 && ((value - 1) & value) != 0;
}

pub fn first_one(b: u64) -> Square {
    debug_assert!(b != 0);
    return Square(b.trailing_zeros() as u8);
}

pub const fn up(b: u64) -> u64 {
    return b << 8;
}

pub const fn down(b: u64) -> u64 {
    return b >> 8;
}

pub const fn right(b: u64) -> u64 {
    return (b & !FILEH_MASK) << 1;
}

pub const fn left(b: u64) -> u64 {
    return (b & !FILEA_MASK) >> 1;
}

pub const fn up_right(b: u64) -> u64 {
    return up(right(b));
}

pub const fn up_left(b: u64) -> u64 {
    return up(left(b));
}

pub const fn down_right(b: u64) -> u64 {
    return down(right(b));
}

pub const fn down_left(b: u64) -> u64 {
    return down(left(b));
}

pub fn pawn_attacks(side: Side, from: Square) -> u64 {
    match side {
        Side::WHITE => ATTACK_TABLES.pawn_attacks[0][from.index()],
        Side::BLACK => ATTACK_TABLES.pawn_attacks[1][from.index()],
    }
}

pub fn knight_attacks(from: Square) -> u64 {
    return ATTACK_TABLES.knight_attacks[from.index()];
}

pub fn king_attacks(from: Square) -> u64 {
    return ATTACK_TABLES.king_attacks[from.index()];
}

pub fn queen_attacks(from: Square, occ: u64) -> u64 {
    return bishop_attacks(from, occ) | rook_attacks(from, occ);
}

struct AttackTables {
    pawn_attacks: [[u64; 64]; 2],
    knight_attacks: [u64; 64],
    king_attacks: [u64; 64],
}

static ATTACK_TABLES: AttackTables = init_attacks();

const fn init_attacks() -> AttackTables {
    let mut res = AttackTables {
        pawn_attacks: [[0_u64; 64]; 2],
        knight_attacks: [0_u64; 64],
        king_attacks: [0_u64; 64],
    };
    let mut sq = 0;
    while sq < 64 {
        let b = Square(sq as u8).to_bitboard();

        res.pawn_attacks[Side::WHITE.index()][sq] = up(left(b) | right(b));
        res.pawn_attacks[Side::BLACK.index()][sq] = down(left(b) | right(b));

        res.knight_attacks[sq] = right(up(right(b)))
            | up(up(right(b)))
            | up(up(left(b)))
            | left(up(left(b)))
            | left(down(left(b)))
            | down(down(left(b)))
            | down(down(right(b)))
            | right(down(right(b)));

        res.king_attacks[sq] = up(right(b))
            | up(b)
            | up(left(b))
            | left(b)
            | down(left(b))
            | down(b)
            | down(right(b))
            | right(b);

        sq += 1;
    }
    return res;
}

pub(super) unsafe fn init() {
    unsafe { magic::init() }
}

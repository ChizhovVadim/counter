pub const SQUARE_NB: usize = 64;
pub const RANK_NB: usize = 8;
pub const FILE_NB: usize = 8;

pub const FILE_A: usize = 0;
pub const FILE_H: usize = 7;

pub const RANK_1: usize = 0;
pub const RANK_2: usize = 1;
pub const RANK_3: usize = 2;
pub const RANK_4: usize = 3;
pub const RANK_5: usize = 4;
pub const RANK_6: usize = 5;
pub const RANK_7: usize = 6;
pub const RANK_8: usize = 7;

pub type Square = usize;

pub const SQUARE_A1: Square = 0;
pub const SQUARE_B1: Square = 1;
pub const SQUARE_C1: Square = 2;
pub const SQUARE_D1: Square = 3;
pub const SQUARE_E1: Square = 4;
pub const SQUARE_F1: Square = 5;
pub const SQUARE_G1: Square = 6;
pub const SQUARE_H1: Square = 7;

pub const SQUARE_A8: Square = 56;
pub const SQUARE_B8: Square = 57;
pub const SQUARE_C8: Square = 58;
pub const SQUARE_D8: Square = 59;
pub const SQUARE_E8: Square = 60;
pub const SQUARE_F8: Square = 61;
pub const SQUARE_G8: Square = 62;
pub const SQUARE_H8: Square = 63;

pub const SQUARE_NONE: Square = 64;

pub fn flip_square(sq: Square) -> Square {
    return sq ^ 56;
}

pub fn file(sq: Square) -> usize {
    return sq & 7;
}

pub fn rank(sq: Square) -> usize {
    return sq >> 3;
}

pub fn make_square(file: usize, rank: usize) -> Square {
    return ((rank << 3) | file) as Square;
}

const FILE_NAMES:&str = "abcdefgh";//b prefix?
const RANK_NAMES:&str = "12345678";

static FILE_NAMES2:[char;8] = ['a','b','c','d','e','f','g','h'];
static RANK_NAMES2:[char;8] = ['1','2','3','4','5','6','7','8'];

pub fn file_name(sq: Square) ->char {
    return FILE_NAMES2[file(sq)];
}

pub fn rank_name(sq: Square) -> char {
    return RANK_NAMES2[rank(sq)];
}

pub fn square_name(sq: Square) -> String {
    let mut s = String::new();
    s.push(FILE_NAMES2[file(sq)]);
    s.push(RANK_NAMES2[rank(sq)]);
    return s;
}

pub fn parse_square(s: &str) -> Square {
    if s == "-" {
        return SQUARE_NONE;
    }
    let file = FILE_NAMES.find(&s[0..1]);
    let rank = RANK_NAMES.find(&s[1..2]);
    return make_square(file.unwrap(), rank.unwrap());
}

use crate::chess::Side;
use std::fmt;

#[derive(Clone, Copy, PartialEq)]
pub struct Square(pub(super) u8);

impl fmt::Debug for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            FILE_NAMES2[self.file() as usize],
            RANK_NAMES2[self.rank() as usize]
        )
    }
}

impl Square {
    pub const SQUARE_NB: usize = 64;

    pub const FILE_A: usize = 0;
    pub const FILE_B: usize = 1;
    pub const FILE_C: usize = 2;
    pub const FILE_D: usize = 3;
    pub const FILE_E: usize = 4;
    pub const FILE_F: usize = 5;
    pub const FILE_G: usize = 6;
    pub const FILE_H: usize = 7;

    pub const RANK_1: u8 = 0;
    pub const RANK_2: u8 = 1;
    pub const RANK_3: u8 = 2;
    pub const RANK_4: u8 = 3;
    pub const RANK_5: u8 = 4;
    pub const RANK_6: u8 = 5;
    pub const RANK_7: u8 = 6;
    pub const RANK_8: u8 = 7;

    pub const A1: Square = Square(0);
    pub const B1: Square = Square(1);
    pub const C1: Square = Square(2);
    pub const D1: Square = Square(3);
    pub const E1: Square = Square(4);
    pub const F1: Square = Square(5);
    pub const G1: Square = Square(6);
    pub const H1: Square = Square(7);

    pub const A8: Square = Square(56);
    pub const B8: Square = Square(57);
    pub const C8: Square = Square(58);
    pub const D8: Square = Square(59);
    pub const E8: Square = Square(60);
    pub const F8: Square = Square(61);
    pub const G8: Square = Square(62);
    pub const H8: Square = Square(63);

    pub fn make(file: usize, rank: usize) -> Square {
        return Square(((rank << 3) | file) as u8);
    }

    pub fn parse(s: &str) -> Option<Square> {
        if s == "-" {
            return None;
        }
        let file = FILE_NAMES.find(&s[0..1]);
        let rank = RANK_NAMES.find(&s[1..2]);
        return Some(Self::make(file.unwrap(), rank.unwrap()));
    }

    pub const fn index(self) -> usize {
        return self.0 as usize;
    }

    pub const fn to_bitboard(self) -> u64 {
        return 1_u64 << self.0;
    }

    pub const fn flip(self) -> Square {
        return Square(self.0 ^ 56);
    }

    pub const fn file(self) -> u8 {
        return self.0 & 7;
    }

    pub const fn rank(self) -> u8 {
        return self.0 >> 3;
    }

    pub fn forward(self, side: Side) -> Square {
        if side == Side::WHITE {
            Square(self.0 + 8)
        } else {
            Square(self.0 - 8)
        }
    }

    pub fn with_step(self, step: isize) -> Square {
        Square((self.0 as isize + step) as u8)
    }
}

const FILE_NAMES: &str = "abcdefgh"; //b prefix?
const RANK_NAMES: &str = "12345678";

pub static FILE_NAMES2: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
pub static RANK_NAMES2: [char; 8] = ['1', '2', '3', '4', '5', '6', '7', '8'];

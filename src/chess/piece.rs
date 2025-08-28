#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Side {
    WHITE = 0,
    BLACK = 1,
}

impl Side {
    pub const SIDE_NB: usize = 2;

    pub fn opp(self) -> Side {
        match self {
            Self::WHITE => Self::BLACK,
            Self::BLACK => Self::WHITE,
        }
    }

    pub const fn index(self) -> usize {
        return self as usize;
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Piece {
    NONE = 0,
    PAWN = 1,
    KNIGHT = 2,
    BISHOP = 3,
    ROOK = 4,
    QUEEN = 5,
    KING = 6,
}

impl Piece {
    pub const PIECE_NB: usize = 8;
}

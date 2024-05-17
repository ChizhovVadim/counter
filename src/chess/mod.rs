use std::sync;
use std::time::Duration;

pub use chessmove::*;
pub use movegen::*;
pub use movelist::*;
pub use position::*;
pub use square::Square;

pub mod bitboard;
pub mod chessmove;
pub mod movegen;
pub mod movelist;
pub mod position;
mod rand;
pub mod square;

pub const SIDE_NB: usize = 2;
pub const SIDE_WHITE: usize = 0;
pub const SIDE_BLACK: usize = 1;

pub type Piece = usize;
pub const PIECE_NB: usize = 8;
pub const PIECE_EMPTY: Piece = 0;
pub const PIECE_PAWN: Piece = 1;
pub const PIECE_KNIGHT: Piece = 2;
pub const PIECE_BISHOP: Piece = 3;
pub const PIECE_ROOK: Piece = 4;
pub const PIECE_QUEEN: Piece = 5;
pub const PIECE_KING: Piece = 6;

pub mod piece {
    use super::Piece;

    pub const NB: usize = 8;
    pub const EMPTY: Piece = 0;
    pub const PAWN: Piece = 1;
    pub const KNIGHT: Piece = 2;
    pub const BISHOP: Piece = 3;
    pub const ROOK: Piece = 4;
    pub const QUEEN: Piece = 5;
    pub const KING: Piece = 6;
}

pub fn init() {
    bitboard::init_bitborads();
    position::init_position();
}

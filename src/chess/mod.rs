pub mod bitboard;
mod movegen;
mod moves;
mod piece;
mod position;
mod rand;
mod square;

pub use movegen::MoveList;
pub use moves::Move;
pub use piece::{Piece, Side};
pub use position::Position;
pub use square::Square;

pub unsafe fn init() {
    unsafe {
        bitboard::init();
    }
}

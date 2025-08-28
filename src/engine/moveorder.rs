use super::history::{self, HistoryTable};
use super::{see, utils};
use crate::chess::{Move, MoveList, Piece, Position, Side};

pub struct MoveOrderContext {
    pub side: Side,
    pub trans_move: Move,
    pub killer1: Move,
    pub killer2: Move,
    pub counter_index: Option<usize>,
    pub follow_index: Option<usize>,
}

impl MoveOrderContext {
    pub fn new(
        side: Side,
        trans_move: Move,
        killer1: Move,
        killer2: Move,
        counter_move: Move,
        follow_move: Move,
    ) -> Self {
        return MoveOrderContext {
            side,
            trans_move,
            killer1,
            killer2,
            counter_index: history::try_piece_square_index(side.opp(), counter_move),
            follow_index: history::try_piece_square_index(side, follow_move),
        };
    }

    pub fn evaluate_moves_counter55(
        &self,
        ml: &mut MoveList,
        pos: &Position,
        history: &HistoryTable,
    ) {
        for item in &mut ml.moves[..ml.size] {
            let mv = item.mv;
            let key = if mv == self.trans_move {
                102_000
            } else if utils::is_capture_or_promotion(mv) {
                if see::see_ge(pos, mv, 0) {
                    101_000 + mvvlva(mv)
                } else {
                    0 + mvvlva(mv)
                }
            } else if mv == self.killer1 {
                100_001
            } else if mv == self.killer2 {
                100_000
            } else {
                history.read_total(self, mv)
            };
            item.key = key as i32;
        }
    }
}

pub fn evaluate_captures(ml: &mut MoveList) {
    for item in &mut ml.moves[..ml.size] {
        let mv = item.mv;
        let key = if utils::is_capture_or_promotion(mv) {
            mvvlva(mv)
        } else {
            -100_000
        };
        item.key = key as i32;
    }
}

fn piece_value(piece: Piece) -> isize {
    match piece {
        Piece::PAWN => 1,
        Piece::KNIGHT => 2,
        Piece::BISHOP => 3,
        Piece::ROOK => 4,
        Piece::QUEEN => 5,
        Piece::KING => 6,
        _ => 0,
    }
}

fn mvvlva(mv: Move) -> isize {
    return 8 * (piece_value(mv.captured_piece()) + piece_value(mv.promotion()))
        - piece_value(mv.moving_piece());
}

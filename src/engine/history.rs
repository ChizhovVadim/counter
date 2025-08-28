use super::moveorder::MoveOrderContext;
use crate::chess::{Move, Side, Square};

const MAIN_HISTORY_SIZE: usize = Side::SIDE_NB * Square::SQUARE_NB * Square::SQUARE_NB;

pub struct HistoryTable {
    main_history: [i16; MAIN_HISTORY_SIZE],
    continuation_history: Vec<[i16; 1_024]>,
}

impl HistoryTable {
    pub fn new() -> Self {
        return HistoryTable {
            main_history: [0_i16; MAIN_HISTORY_SIZE],
            continuation_history: (0..1_024).map(|_| [0_i16; 1_024]).collect(),
        };
    }

    pub fn clear(&mut self) {
        self.main_history.fill(0);
        for x in self.continuation_history.iter_mut() {
            for y in x.iter_mut() {
                *y = 0;
            }
        }
    }

    pub fn read_total(&self, context: &MoveOrderContext, mv: Move) -> isize {
        let mut result = 0_isize;
        result += self.main_history[side_from_to_index(context.side, mv)] as isize;
        let piece_to_index = piece_square_index(context.side, mv);
        if let Some(counter_index) = context.counter_index {
            result += self.continuation_history[counter_index][piece_to_index] as isize;
        }
        if let Some(follow_index) = context.follow_index {
            result += self.continuation_history[follow_index][piece_to_index] as isize;
        }
        return result;
    }

    pub fn update(
        &mut self,
        context: &MoveOrderContext,
        quiets_searched: &[Move],
        best_move: Move,
        depth: isize,
    ) {
        let bonus = (depth * depth).min(400);

        for &m in quiets_searched {
            if m == best_move {
                break;
            }
            let main_index = side_from_to_index(context.side, m);
            update_history_entry(&mut self.main_history[main_index], bonus, false);

            let piece_to_index = piece_square_index(context.side, m);
            if let Some(counter_index) = context.counter_index {
                update_history_entry(
                    &mut self.continuation_history[counter_index][piece_to_index],
                    bonus,
                    false,
                );
            }
            if let Some(follow_index) = context.follow_index {
                update_history_entry(
                    &mut self.continuation_history[follow_index][piece_to_index],
                    bonus,
                    false,
                );
            }
        }

        let main_index = side_from_to_index(context.side, best_move);
        update_history_entry(&mut self.main_history[main_index], bonus, true);

        let piece_to_index = piece_square_index(context.side, best_move);
        if let Some(counter_index) = context.counter_index {
            update_history_entry(
                &mut self.continuation_history[counter_index][piece_to_index],
                bonus,
                true,
            );
        }
        if let Some(follow_index) = context.follow_index {
            update_history_entry(
                &mut self.continuation_history[follow_index][piece_to_index],
                bonus,
                true,
            );
        }
    }
}

fn side_from_to_index(side: Side, mv: Move) -> usize {
    return (side.index() << 12) ^ (mv.from().index() << 6) ^ mv.to().index();
}

fn piece_square_index(side: Side, mv: Move) -> usize {
    return (side.index() << 9) ^ ((mv.moving_piece() as usize) << 6) ^ mv.to().index();
}

pub fn try_piece_square_index(side: Side, mv: Move) -> Option<usize> {
    match mv {
        Move::NONE => None,
        Move::NULL => None, // TODO Можно учитывать в истории NULL MOVE
        _ => Some(piece_square_index(side, mv)),
    }
}

// Exponential moving average
fn update_history_entry(v: &mut i16, bonus: isize, good: bool) {
    const HISTORY_MAX: isize = 1 << 14;
    let new_val = if good { HISTORY_MAX } else { -HISTORY_MAX };
    *v += ((new_val - (*v as isize)) * bonus / 512) as i16;
}

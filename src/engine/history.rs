use crate::chess::Move;
use crate::chess::{SIDE_NB, square::SQUARE_NB};

pub struct HistoryTable {
    main_history: [i16; SIDE_NB * SQUARE_NB * SQUARE_NB],
    continuation_history: Vec<[i16; 1_024]>, //[[i16; 1_024]; 1_024]
}

impl HistoryTable {
    pub fn new() -> Self{
        let mut result = HistoryTable{
            main_history: [0_i16; SIDE_NB * SQUARE_NB * SQUARE_NB],
            continuation_history: Vec::with_capacity(1_024),
        };
        for _ in 0..1_024 {
            result.continuation_history.push([0_i16; 1_024]);
        }
        return result;
    }

    pub fn clear(&mut self) {
        for item in self.main_history.iter_mut() {
            *item = 0;
        }
        for x in self.continuation_history.iter_mut() {
            for y in x.iter_mut() {
                *y = 0;
            }
        }
    }

    pub fn read(
        &self,
        side: usize,
        mv: Move,
        counter: Option<Move>,
        follow: Option<Move>,
    ) -> isize {
        let mut result = 0_isize;

        result += self.main_history[side_from_to_index(side, mv)] as isize;

        let piece_to_index = piece_square_index(side, mv);
        if let Some(counter) = counter {
            result += self.continuation_history[piece_square_index(side ^ 1, counter)]
                [piece_to_index] as isize;
        }
        if let Some(follow) = follow {
            result += self.continuation_history[piece_square_index(side, follow)][piece_to_index]
                as isize;
        }

        return result;
    }

    pub fn update(
        &mut self,
        side: usize,
        quiets_searched: &[Move],
        best_move: Move,
        depth: isize,
        counter: Option<Move>,
        follow: Option<Move>,
    ) {
        if quiets_searched.len() <= 1 && depth <= 3 {
            return;
        }
        let bonus = (depth * depth).min(400);
        let counter_index = counter.map(|mv| piece_square_index(side ^ 1, mv));
        let follow_index = follow.map(|mv| piece_square_index(side, mv));

        for &m in quiets_searched {
            let good = m == best_move;
            let main_index = side_from_to_index(side, m);
            update_history_entry(&mut self.main_history[main_index], bonus, good);

            let piece_to_index = piece_square_index(side, m);
            if let Some(counter_index) = counter_index {
                update_history_entry(
                    &mut self.continuation_history[counter_index][piece_to_index],
                    bonus,
                    good,
                );
            }
            if let Some(follow_index) = follow_index {
                update_history_entry(
                    &mut self.continuation_history[follow_index][piece_to_index],
                    bonus,
                    good,
                );
            }

            if good {
                break;
            }
        }
    }
}

// Exponential moving average
fn update_history_entry(v: &mut i16, bonus: isize, good: bool) {
    const HISTORY_MAX: isize = 1 << 14;
    let new_val = if good { HISTORY_MAX } else { -HISTORY_MAX };
    *v += ((new_val - (*v as isize)) * bonus / 512) as i16;
}

fn side_from_to_index(side: usize, mv: Move) -> usize {
    return (side << 12) ^ (mv.from() << 6) ^ mv.to();
}

fn piece_square_index(side: usize, mv: Move) -> usize {
    match mv {
        Move::EMPTY => 0,
        mv => (side << 9) ^ (mv.moving_piece() << 6) ^ mv.to(),
    }
}

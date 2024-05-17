use crate::chess;
use crate::engine::{history, see};

use super::is_cap_or_prom;

const SORT_TABLE_KEY_IMPORTANT: isize = 100_000;

pub struct MovePicker<'a> {
    pub ml: &'a mut [chess::OrderedMove],
    pub next: usize,
}

impl<'a> MovePicker<'a> {
    pub fn new(ml: &'a mut [chess::OrderedMove]) -> Self {
        return MovePicker { ml: ml, next: 0 };
    }

    pub fn next(&mut self) -> Option<chess::Move> {
        if self.next >= self.ml.len() {
            return None;
        }
        move_to_top(&mut self.ml[self.next..]);
        let m = self.ml[self.next].mv;
        self.next += 1;
        return Some(m);
    }

    pub fn skip_queits(&mut self) {
        let mut i = self.next;
        for j in self.next..self.ml.len() {
            if !is_cap_or_prom(self.ml[j].mv) {
                swap(&mut self.ml, i, j);
                i += 1;
            }
        }
        self.next = i;
    }
}

fn move_to_top(ml: &mut [chess::OrderedMove]) {
    let mut best_index = 0;
    for i in 1..ml.len() {
        if ml[i].key > ml[best_index].key {
            best_index = i;
        }
    }
    swap(ml, 0, best_index);
}

fn swap(ml: &mut [chess::OrderedMove], i: usize, j: usize) {
    if i != j {
        let temp = ml[i];
        ml[i] = ml[j];
        ml[j] = temp;
    }
}

pub fn eval_noisy(moves: &mut [chess::OrderedMove], pos: &chess::Position,trans_move: chess::Move,) {
    for item in moves.iter_mut() {
        let m = item.mv;
        let score = if m == trans_move {
            SORT_TABLE_KEY_IMPORTANT + 2_000
        } else {
            SORT_TABLE_KEY_IMPORTANT + 1000 + mvvlva(m)
        };
        item.key = score as i32;
    }
}

pub fn eval_moves(
    moves: &mut [chess::OrderedMove],
    pos: &chess::Position,
    trans_move: chess::Move,
    killer1: chess::Move,
    killer2: chess::Move,
    history: &history::HistoryTable,
    counter: Option<chess::Move>,
    follow: Option<chess::Move>,
) {
    let side = pos.side_to_move;
    for item in moves.iter_mut() {
        let m = item.mv;
        let score = if m == trans_move {
            SORT_TABLE_KEY_IMPORTANT + 2_000
        } else if is_cap_or_prom(m) {
            if see::see_ge(pos, m, 0) {
                SORT_TABLE_KEY_IMPORTANT + 1000 + mvvlva(m)
            } else {
                mvvlva(m)
            }
        } else if m == killer1 || m == killer2 {
            SORT_TABLE_KEY_IMPORTANT
        } else {
            history.read(side, m, counter, follow)
        };
        item.key = score as i32;
    }
}

fn mvvlva(chess_move: chess::Move) -> isize {
    return 8 * (chess_move.captured_piece() as isize + chess_move.promotion() as isize)
        - chess_move.moving_piece() as isize;
}

static PIECE_VALUES_SORT: [isize; chess::PIECE_NB] = [0, 1, 2, 3, 4, 5, 6, 0];

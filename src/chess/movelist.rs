use super::Move;

pub const MAX_MOVES: usize = 256;

#[derive(Copy, Clone, Debug)]
pub struct OrderedMove {
    pub mv: Move,
    pub key: i32,
}

pub struct MoveList {
    pub moves: [OrderedMove; MAX_MOVES],
    pub size: usize,
}

impl MoveList {
    pub fn new() -> MoveList {
        return MoveList {
            moves: unsafe { std::mem::MaybeUninit::uninit().assume_init() },
            size: 0,
        };
    }

    pub fn clear(self: &mut MoveList) {
        self.size = 0;
    }

    pub fn add(self: &mut MoveList, m: Move) {
        //self.moves[self.size].chessmove=m;
        self.moves[self.size] = OrderedMove {
            mv: m,
            key: 0,
        };
        self.size += 1;
    }
}

use crate::chess::Move;

pub const BOUND_LOWER: usize = 1;
pub const BOUND_UPPER: usize = 2;
pub const BOUND_EXACT: usize = BOUND_LOWER | BOUND_UPPER;

pub struct TransTable {
    megabytes: usize,
    entries: Vec<TransEntry>,
    date: u16,
}

struct TransEntry {
    key: u32,
    mv: Move,
    date: u16,
    score: i16,
    depth: u8,
    bound: u8,
}

impl Default for TransEntry {
    fn default() -> Self {
        TransEntry {
            key: 0,
            mv: Move::EMPTY,
            date: 0,
            score: 0,
            depth: 0,
            bound: 0,
        }
    }
}

impl TransTable {
    pub fn new(megabytes: usize) -> Self {
        eprintln!("init trans table {}", megabytes);
        let size = 1_024 * 1_024 * megabytes / std::mem::size_of::<TransEntry>();
        let mut table = Vec::with_capacity(size);
        for _ in 0..size {
            table.push(TransEntry::default());
        }
        TransTable {
            megabytes: megabytes,
            entries: table,
            date: 0,
        }
    }

    pub fn size(&self) -> usize {
        return self.megabytes;
    }

    pub fn inc_date(&mut self) {
        self.date += 1;
    }

    pub fn clear(&mut self) {
        self.date = 0;
        for entry in self.entries.iter_mut() {
            *entry = TransEntry::default();
        }
    }

    pub fn update(&mut self, key: u64, depth: isize, score: isize, bound: usize, mv: Move) {
        let index = key % (self.entries.len() as u64);
        let entry = &mut self.entries[index as usize];
        let replace = if entry.key == (key >> 32) as u32 {
            depth >= (entry.depth as isize) - 3 || bound == BOUND_EXACT
        } else {
            entry.date != self.date || depth >= entry.depth as isize
        };
        if replace {
            entry.date = self.date;
            entry.key = (key >> 32) as u32;
            entry.depth = depth as u8;
            entry.score = score as i16;
            entry.bound = bound as u8;
            entry.mv = mv;
        }
    }

    //(depth, score, bound, move, ok)
    pub fn read(&mut self, key: u64) -> (isize, isize, usize, Move, bool) {
        let index = key % (self.entries.len() as u64);
        let entry = &mut self.entries[index as usize];
        if entry.key == (key >> 32) as u32 {
            entry.date = self.date;
            return (
                entry.depth as isize,
                entry.score as isize,
                entry.bound as usize,
                entry.mv,
                true,
            );
        } else {
            return (0, 0, 0, Move::EMPTY, false);
        }
    }
}

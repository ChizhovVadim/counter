use crate::chess::Move;

pub const BOUND_LOWER: usize = 1;
pub const BOUND_UPPER: usize = 2;
pub const BOUND_EXACT: usize = BOUND_LOWER | BOUND_UPPER;

pub struct TransTable {
    megabytes: usize,
    entries: Vec<TransEntry>,
    date: u16,
}

#[derive(Default)]
struct TransEntry {
    key: u32,
    mv: Move,
    date: u16,
    score: i16,
    depth: i8,
    bound: u8,
}

impl TransTable {
    pub fn new(megabytes: usize) -> Self {
        let mut tt = TransTable {
            megabytes: 0,
            entries: Vec::new(),
            date: 0,
        };
        tt.resize(megabytes);
        return tt;
    }

    pub fn size(&self) -> usize {
        return self.megabytes;
    }

    pub fn resize(&mut self, megabytes: usize) {
        let size = (1_usize << 20) * megabytes / std::mem::size_of::<TransEntry>();
        if size == self.entries.len() {
            return;
        }
        self.entries = Vec::new(); // clear large object in heap
        eprintln!("init trans table {megabytes}");
        let mut table = Vec::with_capacity(size);
        for _ in 0..size {
            table.push(TransEntry::default());
        }
        self.megabytes = megabytes;
        self.entries = table;
    }

    pub fn inc_date(&mut self) {
        self.date += 1;
    }

    pub fn clear(&mut self) {
        self.date = 0;
        self.entries.fill_with(Default::default);
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
            return (0, 0, 0, Move::default(), false);
        }
    }

    pub fn update(&mut self, key: u64, depth: isize, score: isize, bound: usize, mv: Move) {
        let index = key % (self.entries.len() as u64);
        let entry = &mut self.entries[index as usize];
        let existing = entry.key == (key >> 32) as u32;
        if existing {
            if !(depth >= (entry.depth as isize) - 3 || bound == BOUND_EXACT) {
                if mv != Move::NONE
                    && !(entry.mv != Move::NONE && (entry.bound as usize & BOUND_LOWER) != 0)
                {
                    entry.mv = mv;
                }
                return;
            }
            if mv != Move::NONE {
                entry.mv = mv;
            }
        } else {
            if !(entry.date != self.date || depth >= entry.depth as isize) {
                return;
            }
            entry.key = (key >> 32) as u32;
            entry.mv = mv;
        }
        entry.date = self.date;
        entry.depth = depth as i8;
        entry.score = score as i16;
        entry.bound = bound as u8;
    }

    pub fn update_old_policy(
        &mut self,
        key: u64,
        depth: isize,
        score: isize,
        bound: usize,
        mv: Move,
    ) {
        let index = key % (self.entries.len() as u64);
        let entry = &mut self.entries[index as usize];
        let existing = entry.key == (key >> 32) as u32;
        if existing {
            if !(depth >= (entry.depth as isize) - 3 || bound == BOUND_EXACT) {
                return;
            }
        } else {
            if !(entry.date != self.date || depth >= entry.depth as isize) {
                return;
            }
        }
        entry.date = self.date;
        entry.key = (key >> 32) as u32;
        entry.depth = depth as i8;
        entry.score = score as i16;
        entry.bound = bound as u8;
        entry.mv = mv;
    }
}

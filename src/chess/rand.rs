pub struct XorshiftRng {
    seed: u64,
}

//https://en.wikipedia.org/wiki/Xorshift
//http://vigna.di.unimi.it/ftp/papers/xorshift.pdf
impl XorshiftRng {
    pub const fn new() -> Self {
        return XorshiftRng { seed: 1070372_u64 };
    }

    pub const fn next(&mut self) -> u64 {
        self.seed ^= self.seed >> 12;
        self.seed ^= self.seed << 25;
        self.seed ^= self.seed >> 27;
        return self.seed.wrapping_mul(2685821657736338717_u64);
    }
}


pub struct PseudoRandom {
    pub seed: u64
}

impl PseudoRandom {
    pub fn new() -> PseudoRandom {
        PseudoRandom { seed: 0 }
    }

    pub fn next(&mut self) -> u64 {
        self.seed = (self.seed * 53 + 79) % 7499;
        self.seed
    }
}


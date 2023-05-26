use super::*;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
/// A struct to keep track of the space needed to store a stream of integers
/// using different codes, this can be used to determine which code is the
/// most efficient for a given stream.
pub struct CodesStats {
    pub unary: usize,
    pub gamma: usize,
    pub delta: usize,
    pub zeta2: usize,
    pub zeta3: usize,
    pub zeta4: usize,
}

impl CodesStats {
    /// Create a new `CodesStats` struct
    pub fn new() -> Self {
        Default::default()
    }

    /// Update the stats with the length of the code for `value`
    pub fn update(&mut self, value: u64) {
        self.unary = self.unary.saturating_add(len_unary(value));
        self.gamma = self.gamma.saturating_add(len_gamma(value));
        self.delta = self.delta.saturating_add(len_delta(value));
        self.zeta2 = self.zeta2.saturating_add(len_zeta(2, value));
        self.zeta3 = self.zeta3.saturating_add(len_zeta(3, value));
        self.zeta4 = self.zeta4.saturating_add(len_zeta(4, value));
    }
    /// Return the best code for the stream, as in the one that needed the
    /// least space
    pub fn get_best_code(&self) -> Code {
        let mut best = self.unary;
        let mut best_code = Code::Unary;

        if self.gamma < best {
            best = self.gamma;
            best_code = Code::Gamma;
        }

        if self.delta < best {
            best = self.delta;
            best_code = Code::Delta;
        }

        if self.zeta2 < best {
            best = self.zeta2;
            best_code = Code::Zeta { k: 2 };
        }

        if self.zeta3 < best {
            best = self.zeta3;
            best_code = Code::Zeta { k: 3 };
        }

        if self.zeta4 < best {
            best_code = Code::Zeta { k: 4 };
        }

        best_code
    }
}

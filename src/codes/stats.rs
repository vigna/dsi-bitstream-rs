use super::*;

pub fn len_golomb(value: u64, b: u64) -> usize {
    len_unary(value / b) + len_minimal_binary(value % b, b)
}

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
    pub zeta5: usize,
    pub zeta6: usize,
    pub golomb2: usize,
    pub golomb3: usize,
    pub golomb4: usize,
    pub golomb5: usize,
    pub golomb6: usize,
}

impl CodesStats {
    /// Create a new `CodesStats` struct
    pub fn new() -> Self {
        Default::default()
    }

    /// Update the stats with the length of the code for `value` and return back
    /// `value` for convienience
    pub fn update(&mut self, value: u64) -> u64 {
        self.unary = self.unary.saturating_add(len_unary(value));
        self.gamma = self.gamma.saturating_add(len_gamma(value));
        self.delta = self.delta.saturating_add(len_delta(value));

        self.zeta2 = self.zeta2.saturating_add(len_zeta(value, 2));
        self.zeta3 = self.zeta3.saturating_add(len_zeta(value, 3));
        self.zeta4 = self.zeta4.saturating_add(len_zeta(value, 4));
        self.zeta5 = self.zeta5.saturating_add(len_zeta(value, 5));
        self.zeta6 = self.zeta6.saturating_add(len_zeta(value, 6));

        self.golomb2 = self.golomb2.saturating_add(len_golomb(value, 2));
        self.golomb3 = self.golomb3.saturating_add(len_golomb(value, 3));
        self.golomb4 = self.golomb4.saturating_add(len_golomb(value, 4));
        self.golomb5 = self.golomb5.saturating_add(len_golomb(value, 5));
        self.golomb6 = self.golomb6.saturating_add(len_golomb(value, 6));
        value
    }
    /// Return the best code for the stream, as in the one that needed the
    /// least space, and the space needed by that code
    pub fn get_best_code(&self) -> (Code, usize) {
        // TODO!: make cleaner
        let mut best = self.unary;
        let mut best_code = Code::Unary;

        macro_rules! check {
            ($code:expr, $len:expr) => {
                if $len < best {
                    best = $len;
                    best_code = $code;
                }
            };
        }

        check!(Code::Gamma, self.gamma);
        check!(Code::Delta, self.delta);

        check!(Code::Zeta { k: 2 }, self.zeta2);
        check!(Code::Zeta { k: 3 }, self.zeta3);
        check!(Code::Zeta { k: 4 }, self.zeta4);
        check!(Code::Zeta { k: 5 }, self.zeta5);
        check!(Code::Zeta { k: 6 }, self.zeta6);

        check!(Code::Golomb { b: 2 }, self.golomb2);
        check!(Code::Golomb { b: 3 }, self.golomb3);
        check!(Code::Golomb { b: 4 }, self.golomb4);
        check!(Code::Golomb { b: 5 }, self.golomb5);
        check!(Code::Golomb { b: 6 }, self.golomb6);

        (best_code, best)
    }
}

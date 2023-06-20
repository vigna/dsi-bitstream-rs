use super::*;
use core::sync::atomic::{AtomicUsize, Ordering};

pub fn len_golomb(value: u64, b: u64) -> usize {
    len_unary(value / b) + len_minimal_binary(value % b, b)
}

#[derive(Default, Debug)]
/// A struct to keep track of the space needed to store a stream of integers
/// using different codes, this can be used to determine which code is the
/// most efficient for a given stream.
pub struct CodesStats {
    pub unary: AtomicUsize,
    pub gamma: AtomicUsize,
    pub delta: AtomicUsize,
    pub zeta: [AtomicUsize; 10],
    pub golomb: [AtomicUsize; 20],
}

impl CodesStats {
    /// Create a new `CodesStats` struct
    pub fn new() -> Self {
        Default::default()
    }

    /// Update the stats with the length of the code for `value` and return back
    /// `value` for convienience
    pub fn update(&self, value: u64) -> u64 {
        self.unary.fetch_add(len_unary(value), Ordering::Relaxed);
        self.gamma.fetch_add(len_gamma(value), Ordering::Relaxed);
        self.delta.fetch_add(len_delta(value), Ordering::Relaxed);

        for (k, val) in self.zeta.iter().enumerate() {
            val.fetch_add(len_zeta(value, (k + 1) as _), Ordering::Relaxed);
        }
        for (b, val) in self.golomb.iter().enumerate() {
            val.fetch_add(len_golomb(value, (b + 1) as _), Ordering::Relaxed);
        }
        value
    }
    /// Return the best code for the stream, as in the one that needed the
    /// least space, and the space needed by that code
    pub fn get_best_code(&self) -> (Code, usize) {
        // TODO!: make cleaner
        let mut best = self.unary.load(Ordering::Relaxed);
        let mut best_code = Code::Unary;

        macro_rules! check {
            ($code:expr, $len:expr) => {
                let len = $len.load(Ordering::Relaxed);
                if len < best {
                    best = len;
                    best_code = $code;
                }
            };
        }

        check!(Code::Gamma, self.gamma);
        check!(Code::Delta, self.delta);

        for (k, val) in self.zeta.iter().enumerate() {
            check!(Code::Zeta { k: (k + 1) as _ }, *val);
        }
        for (b, val) in self.golomb.iter().enumerate() {
            check!(Code::Golomb { b: (b + 1) as _ }, *val);
        }

        (best_code, best)
    }
}

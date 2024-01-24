/*
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use crate::prelude::{len_delta, len_gamma, len_minimal_binary, len_zeta, Code};

// To be replaced when Golomb codes will be implemented.
fn len_golomb(value: u64, b: u64) -> usize {
    (value / b) as usize + 1 + len_minimal_binary(value % b, b)
}

const ZETA: usize = 10;
const GOLOMB: usize = 20;

/// Keeps track of the space needed to store a stream of integers using
/// different codes.
///
/// This structure can be used to determine empirically which code provides the
/// best compression for a given stream. You have to [update the
/// structure](Self::update) with the integers in the stream; at any time, you
/// can examine the statistics or call [`best_code`](Self::best_code) to get the
/// best code.
#[derive(Default, Debug)]
pub struct CodesStats {
    pub unary: u64,
    pub gamma: u64,
    pub delta: u64,
    pub zeta: [u64; ZETA],
    pub golomb: [u64; GOLOMB],
}

impl CodesStats {
    /// Update the stats with the lengths of the codes for `n` and return
    /// `n` for convenience.
    pub fn update(&mut self, n: u64) -> u64 {
        self.unary += n + 1;
        self.gamma += len_gamma(n) as u64;
        self.delta += len_delta(n) as u64;

        for (k, val) in self.zeta.iter_mut().enumerate() {
            *val += len_zeta(n, (k + 1) as _) as u64;
        }
        for (b, val) in self.golomb.iter_mut().enumerate() {
            *val += len_golomb(n, (b + 1) as _) as u64;
        }
        n
    }

    // Combines additively this stats with another one.
    pub fn add(&mut self, rhs: &Self) {
        self.unary += rhs.unary;
        self.gamma += rhs.gamma;
        self.delta += rhs.delta;
        for (a, b) in self.zeta.iter_mut().zip(rhs.zeta.iter()) {
            *a += *b;
        }
        for (a, b) in self.golomb.iter_mut().zip(rhs.golomb.iter()) {
            *a += *b;
        }
    }

    /// Return the best code for the stream and its space usage.
    pub fn best_code(&self) -> (Code, u64) {
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

        for (k, val) in self.zeta.iter().enumerate() {
            check!(Code::Zeta { k: (k + 1) as _ }, *val);
        }
        for (b, val) in self.golomb.iter().enumerate() {
            check!(Code::Golomb { b: (b + 1) as _ }, *val);
        }

        (best_code, best)
    }
}

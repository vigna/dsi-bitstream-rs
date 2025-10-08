/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

#![allow(dead_code)]

//! Generation functions for data used in benchmarks.
//!
//! For each code, we generate samples either using its implied distribution
//! p(w) = 2<sup>−|w|</sup> or a universal distribution (a Zipf distribution of
//! exponent one). Moreover, the functions return the hit ratio, that is, the
//! ratio of values that is decodable using tables.
use super::*;
use rand::{rngs::SmallRng, SeedableRng};

// Given data to benchmark a code, tables for that code, and a length
// function for the code, this macro computes the hit ratio, that is,
// the ratio of values that is decodable using the given tables.
macro_rules! compute_hit_ratio {
    ($data:expr, $table:ident, $len_func:ident) => {{
        let mut total = 0.0;
        for value in &$data {
            #[cfg(feature = "reads")]
            if $len_func(*value) <= $table::READ_BITS as usize {
                total += 1.0;
            }
            #[cfg(not(feature = "reads"))]
            if *value <= $table::WRITE_MAX {
                total += 1.0;
            }
        }
        total / $data.len() as f64
    }};
}

/// Generates N samples from the implied distribution of a code using the given
/// length function (if the `univ` feature is not enabled) or from a universal
/// distribution ≈1/x on the first billion integers (if the `univ` feature is
/// enabled).
pub fn gen_data(_len: fn(u64) -> usize) -> Vec<u64> {
    let mut rng = SmallRng::seed_from_u64(42);

    #[cfg(not(feature = "univ"))]
    let samples = sample_implied_distribution(&_len, &mut rng);
    #[cfg(feature = "univ")]
    let samples = {
        use rand::Rng;
        let distr = rand_distr::Zipf::new(1E9 as f64, 1.0).unwrap();
        (&mut rng).sample_iter(distr).map(|x| x as u64 - 1)
    };

    return samples.take(N).collect();
}

/// Generate data to benchmark γ code.
pub fn gen_gamma_data() -> (f64, Vec<u64>) {
    let gamma_data = gen_data(len_gamma);
    let ratio = compute_hit_ratio!(gamma_data, gamma_tables, len_gamma);
    (ratio, gamma_data)
}

/// Generate data to benchmark δ code.
pub fn gen_delta_data() -> (f64, Vec<u64>) {
    let delta_data = gen_data(len_delta);
    let ratio = compute_hit_ratio!(delta_data, delta_tables, len_delta);
    (ratio, delta_data)
}

/// Generate data to benchmark ζ₃ code.
pub fn gen_zeta3_data() -> (f64, Vec<u64>) {
    let zeta3_data = gen_data(|x| len_zeta(x, 3));

    let ratio = zeta3_data
        .iter()
        .map(|value| {
            #[cfg(feature = "reads")]
            if len_zeta(*value, 3) <= zeta_tables::READ_BITS {
                1
            } else {
                0
            }
            #[cfg(not(feature = "reads"))]
            if *value <= zeta_tables::WRITE_MAX {
                1
            } else {
                0
            }
        })
        .sum::<usize>() as f64
        / N as f64;

    (ratio, zeta3_data)
}

/// Generate data to benchmark ω code.
pub fn gen_omega_data() -> (f64, Vec<u64>) {
    let omega_data = gen_data(len_omega);
    let ratio = compute_hit_ratio!(omega_data, omega_tables, len_omega);
    (ratio, omega_data)
}

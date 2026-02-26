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
//! p(w) = 2<sup>-|w|</sup> or a universal Zipf distribution ≈1/_x_
//! Moreover, the functions return the hit ratio, that is, the ratio of values
//! that is decodable using tables.

use super::N;
use dsi_bitstream::prelude::*;
use dsi_bitstream::utils::sample_implied_distribution;
use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng};

/// Generates N samples from the implied distribution of a code using the given
/// length function or from a universal Zipf distribution ≈1/_x_ on the first
/// billion integers (if `univ` is true).
pub fn gen_data(len: impl Fn(u64) -> usize, univ: bool) -> Vec<u64> {
    let mut rng = SmallRng::seed_from_u64(42);

    if univ {
        let distr = rand_distr::Zipf::new(1E9_f64, 1.0).unwrap();
        rng.sample_iter(distr)
            .map(|x| x as u64 - 1)
            .take(N)
            .collect()
    } else {
        sample_implied_distribution(&len, &mut rng)
            .take(N)
            .collect()
    }
}

/// Computes the read hit ratio: fraction of values decodable via read tables.
pub fn read_hit_ratio(data: &[u64], len_fn: impl Fn(u64) -> usize, read_bits: usize) -> f64 {
    let hits = data.iter().filter(|&&v| len_fn(v) <= read_bits).count();
    hits as f64 / data.len() as f64
}

/// Computes the write hit ratio: fraction of values encodable via write tables.
pub fn write_hit_ratio(data: &[u64], write_max: u64) -> f64 {
    let hits = data.iter().filter(|&&v| v <= write_max).count();
    hits as f64 / data.len() as f64
}

/// Generates data to benchmark gamma code.
pub fn gen_gamma_data(univ: bool) -> (f64, Vec<u64>) {
    let data = gen_data(len_gamma, univ);
    #[cfg(feature = "bench-reads")]
    let ratio = read_hit_ratio(&data, len_gamma, gamma_tables::READ_BITS);
    #[cfg(not(feature = "bench-reads"))]
    let ratio = write_hit_ratio(&data, gamma_tables::WRITE_MAX);
    (ratio, data)
}

/// Generates data to benchmark delta code.
pub fn gen_delta_data(univ: bool) -> (f64, Vec<u64>) {
    let data = gen_data(len_delta, univ);
    #[cfg(feature = "bench-reads")]
    let ratio = read_hit_ratio(&data, len_delta, delta_tables::READ_BITS);
    #[cfg(not(feature = "bench-reads"))]
    let ratio = write_hit_ratio(&data, delta_tables::WRITE_MAX);
    (ratio, data)
}

/// Generates data to benchmark zeta3 code.
pub fn gen_zeta3_data(univ: bool) -> (f64, Vec<u64>) {
    let data = gen_data(|x| len_zeta(x, 3), univ);
    #[cfg(feature = "bench-reads")]
    let ratio = read_hit_ratio(&data, |x| len_zeta(x, 3), zeta_tables::READ_BITS);
    #[cfg(not(feature = "bench-reads"))]
    let ratio = write_hit_ratio(&data, zeta_tables::WRITE_MAX);
    (ratio, data)
}

/// Generates data to benchmark pi2 code.
pub fn gen_pi2_data(univ: bool) -> (f64, Vec<u64>) {
    let data = gen_data(|x| len_pi(x, 2), univ);
    #[cfg(feature = "bench-reads")]
    let ratio = read_hit_ratio(&data, |x| len_pi(x, 2), pi_tables::READ_BITS);
    #[cfg(not(feature = "bench-reads"))]
    let ratio = write_hit_ratio(&data, pi_tables::WRITE_MAX);
    (ratio, data)
}

/// Generates data to benchmark omega code.
pub fn gen_omega_data(univ: bool) -> (f64, Vec<u64>) {
    let data = gen_data(len_omega, univ);
    #[cfg(feature = "bench-reads")]
    let ratio = read_hit_ratio(&data, len_omega, omega_tables::READ_BITS);
    #[cfg(not(feature = "bench-reads"))]
    let ratio = write_hit_ratio(&data, omega_tables::WRITE_MAX);
    (ratio, data)
}

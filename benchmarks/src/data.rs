/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

#![allow(dead_code)]

/*!

Generation functions for data used in benchmarks.

For each code, this module provides a function to generate data with
a distribution similar to the intended distribution of the code, that is,
p(w) = 2<sup>–|w|</sup>. Moreover, the function returns the
hit ratio, that is, the ratio of values that is decodable using tables.

*/
use super::*;
use once_cell::sync::Lazy;

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

/// Generate data to benchmark γ code.
pub fn gen_gamma_data() -> (f64, Vec<u64>) {
    let mut rng = rand::thread_rng();

    let distr = rand_distr::Zeta::new(2.0).unwrap();
    let gamma_data = (0..N)
        .map(|_| rng.sample(distr) as u64 - 1)
        .collect::<Vec<_>>();

    let ratio = compute_hit_ratio!(gamma_data, gamma_tables, len_gamma);

    (ratio, gamma_data)
}

/// The size of [`DELTA_CUM_DISTR`].
pub const DELTA_DISTR_SIZE: usize = 1_000_000;

/// A slightly tweaked finite cumulative distribution similar to the intended
/// cumulative distribution of δ code.
pub static DELTA_CUM_DISTR: Lazy<Vec<f64>> = Lazy::new(|| {
    let mut delta_distr = vec![0.];
    let mut s = 0.;
    for n in 1..DELTA_DISTR_SIZE {
        let x = n as f64;
        s += 1. / (2. * (x + 3.) * (x.log2() + 2.) * (x.log2() + 2.));
        delta_distr.push(s)
    }
    let last = *delta_distr.last().unwrap();

    for x in &mut delta_distr {
        *x /= last;
    }

    delta_distr
});

/// Generate data to benchmark δ code.
pub fn gen_delta_data() -> (f64, Vec<u64>) {
    let mut rng = rand::thread_rng();

    let distr = rand_distr::Uniform::new(0.0, 1.0);
    let delta_data = (0..N)
        .map(|_| {
            let p = rng.sample(distr);
            let s = DELTA_CUM_DISTR.binary_search_by(|v| v.partial_cmp(&p).unwrap());
            match s {
                Ok(x) => x as u64,
                Err(x) => x as u64 - 1,
            }
        })
        .collect::<Vec<_>>();

    let ratio = compute_hit_ratio!(delta_data, delta_tables, len_delta);

    (ratio, delta_data)
}

/// Generate data to benchmark ζ₃ code.
pub fn gen_zeta3_data() -> (f64, Vec<u64>) {
    let mut rng = rand::thread_rng();

    let distr = rand_distr::Zeta::new(1.0 + 1.0 / 3.0).unwrap();
    let zeta3_data = (0..N)
        .map(|_| rng.sample(distr) as u64 - 1)
        .collect::<Vec<_>>();

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

/// Generate data to benchmark ζ₃ code.
pub fn gen_zeta_data(k: u64) -> Vec<u64> {
    let mut rng = rand::thread_rng();

    let distr = rand_distr::Zeta::new(1.0 + 1.0 / k as f64).unwrap();
    (0..N)
        .map(|_| rng.sample(distr) as u64 - 1)
        .collect::<Vec<_>>()
}

/// Generate data to benchmark unary code.
pub fn gen_unary_data() -> Vec<u64> {
    let mut rng = rand::thread_rng();

    let distr = rand_distr::Geometric::new(0.5).unwrap();
    (0..N).map(|_| rng.sample(distr) as u64).collect::<Vec<_>>()
}

/// Generate data to benchmark golomb codes.
pub fn gen_golomb_data(b: u64) -> Vec<u64> {
    let mut rng = rand::thread_rng();

    let distr = rand_distr::Geometric::new(dsi_bitstream::codes::golomb::p(b)).unwrap();
    (0..N).map(|_| rng.sample(distr) as u64).collect::<Vec<_>>()
}

/// Generate data to benchmark golomb codes.
pub fn gen_rice_data(log2_b: usize) -> Vec<u64> {
    let mut rng = rand::thread_rng();

    let distr = rand_distr::Geometric::new(dsi_bitstream::codes::rice::p(log2_b as u64)).unwrap();
    (0..N).map(|_| rng.sample(distr) as u64).collect::<Vec<_>>()
}

/// Generate data to benchmark δ code.
pub fn gen_exp_golomb_data(k: usize) -> Vec<u64> {
    let mut rng = rand::thread_rng();
    let mut data = gen_gamma_data().1;
    data.iter_mut().for_each(|x| {
        *x = *x * k as u64 + rng.gen_range(0..k as u64);
    });
    data
}

pub fn gen_pi_data(k: u64) -> Vec<u64> {
    let mut rng = rand::thread_rng();

    let distr = rand_distr::Zeta::new(1.0 + 1.0 / (1 << k) as f64).unwrap();
    (0..N)
        .map(|_| rng.sample(distr) as u64 - 1)
        .collect::<Vec<_>>()
}

pub fn gen_pi_web_data(k: u64) -> Vec<u64> {
    let mut rng = rand::thread_rng();

    let distr = rand_distr::Zeta::new(1.0 + 1.0 / (1 << k) as f64).unwrap();
    (0..N)
        .map(|_| {
            if rng.gen_bool(0.5) {
                0
            } else {
                rng.sample(distr) as u64
            }
        })
        .collect::<Vec<_>>()
}

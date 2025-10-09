/*
 * SPDX-FileCopyrightText: 2025 Tommaso Fontana
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

#[cfg(feature = "alloc")]
use crate::utils::FindChangePoints;
#[cfg(feature = "alloc")]
use rand::distr::weighted::WeightedIndex;
#[cfg(feature = "alloc")]
use rand::prelude::*;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Given the len function of a code, generates data that allows to sample
/// its implied distribution, i.e. a code-word with length l has
/// probability 2^(-l).
///
/// This code works only with monotonic non decreasing len functions.
///
/// Returns two vectors, the first one contains the input values where the
/// function changes value and the code length at that point. The second
/// vector contains the probability of each code length.
///
/// Since we cannot write more than 64 bits at once, the codes are limited to
/// 128 bits.
#[cfg(feature = "alloc")]
pub fn get_implied_distribution(f: impl Fn(u64) -> usize) -> (Vec<(u64, usize)>, Vec<f64>) {
    let change_points = FindChangePoints::new(f)
        .take_while(|(_input, len)| *len <= 128)
        .collect::<Vec<_>>();

    // convert to len probabilities
    let probabilities = change_points
        .windows(2)
        .map(|window| {
            let (input, len) = window[0];
            let (next_input, _next_len) = window[1];
            let prob = 2.0_f64.powi(-(len as i32));
            prob * (next_input - input) as f64
        })
        .collect::<Vec<_>>();
    // TODO!: this ignores the last change point

    (change_points, probabilities)
}

#[derive(Clone, Copy, Debug)]
/// An infinite iterator that always returns ().
pub struct InfiniteIterator;

impl Iterator for InfiniteIterator {
    type Item = ();

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        Some(())
    }
}

/// Returns an **infinite iterator** of samples from the implied distribution of
/// the given code length function.
/// The function f should be the len function of the code.
///
/// This code works only with monotonic non decreasing len functions and
/// the codes are limited to 128 bits as we cannot write more than 64 bits at once.
///
/// # Example
///
/// ```rust
/// use dsi_bitstream::utils::sample_implied_distribution;
/// use dsi_bitstream::codes::len_gamma;
/// use rand::SeedableRng;
/// use rand::rngs::SmallRng;
///
/// let mut rng = SmallRng::seed_from_u64(42);
/// let vals: Vec<u64> = sample_implied_distribution(len_gamma, &mut rng)
///     .take(1000).collect::<Vec<_>>();
///
/// assert_eq!(vals.len(), 1000);
/// ```
#[cfg(feature = "alloc")]
pub fn sample_implied_distribution(
    f: impl Fn(u64) -> usize,
    rng: &mut impl Rng,
) -> impl Iterator<Item = u64> + '_ {
    let (change_points, probabilities) = get_implied_distribution(f);
    let dist = WeightedIndex::new(probabilities).unwrap();

    InfiniteIterator.map(move |_| {
        // sample a len with the correct probability
        let idx = dist.sample(rng);
        // now we sample a random value with the sampled len
        let (start_input, _len) = change_points[idx];
        let (end_input, _len) = change_points[idx + 1];
        rng.random_range(start_input..end_input)
    })
}

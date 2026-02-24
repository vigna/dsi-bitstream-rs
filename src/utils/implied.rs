/*
 * SPDX-FileCopyrightText: 2025 Tommaso Fontana
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use core::iter::FusedIterator;

use crate::utils::FindChangePoints;
use alloc::vec::Vec;
use rand::distr::weighted::WeightedIndex;
use rand::prelude::*;

/// Given the len function of a code, generates data that allows to sample
/// its implied distribution, i.e. a code-word with length l has
/// probability 2^(-l).
///
/// This code works only with monotonic non-decreasing len functions.
///
/// Returns two vectors. The first contains one entry per distinct code length
/// up to 128, plus one sentinel entry (the first change point with length >
/// 128) that serves as the upper bound for the last valid group. The second
/// vector contains the probability of each valid length group; its length is
/// always one less than that of the first vector.
///
/// Since we cannot write more than 64 bits at once, the codes are limited to
/// 128 bits.
pub fn get_implied_distribution(f: impl Fn(u64) -> usize) -> (Vec<(u64, usize)>, Vec<f64>) {
    // Collect change points with code length up to 128, plus the first
    // change point with length > 128 as a sentinel upper bound for the
    // last valid length group.
    let mut change_points = Vec::new();
    for item in FindChangePoints::new(f) {
        let len = item.1;
        change_points.push(item);
        if len > 128 {
            break;
        }
    }

    // Convert to length probabilities. The sentinel serves as the upper
    // bound (next_input) of the last valid group.
    let probabilities = change_points
        .windows(2)
        .map(|window| {
            let (input, len) = window[0];
            let (next_input, _next_len) = window[1];
            let prob = 2.0_f64.powi(-(len as i32));
            prob * (next_input - input) as f64
        })
        .collect::<Vec<_>>();

    (change_points, probabilities)
}

/// An infinite iterator that always returns ().
#[derive(Clone, Copy, Debug)]
pub struct InfiniteIterator;

impl Iterator for InfiniteIterator {
    type Item = ();

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        Some(())
    }
}

impl FusedIterator for InfiniteIterator {}

/// Returns an **infinite iterator** of samples from the implied distribution of
/// the given code length function.
/// The function f should be the len function of the code.
///
/// This code works only with monotonic non-decreasing len functions and
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
pub fn sample_implied_distribution(
    f: impl Fn(u64) -> usize,
    rng: &mut impl Rng,
) -> impl Iterator<Item = u64> + '_ {
    let (change_points, probabilities) = get_implied_distribution(f);
    let dist = WeightedIndex::new(probabilities)
        .expect("get_implied_distribution returns non-empty, positive weights");

    InfiniteIterator.map(move |_| {
        // sample a len with the correct probability
        let idx = dist.sample(rng);
        // now we sample a random value with the sampled len
        let (start_input, _len) = change_points[idx];
        let (end_input, _len) = change_points[idx + 1];
        rng.random_range(start_input..end_input)
    })
}

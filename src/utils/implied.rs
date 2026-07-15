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

/// Given the length function of a code, generates data that allows to sample
/// its implied distribution, in which a codeword with length *n* has
/// probability 2⁻*ⁿ*.
///
/// This function works only with monotonic non-decreasing length functions. It
/// returns two vectors: the first contains one entry per distinct code length,
/// followed by a sentinel entry that upper-bounds the last group -- either the
/// first change point with length > 128, or, if the function never exceeds 128,
/// the end of the domain (`u64::MAX`, exclusive). The second vector contains the
/// probability of each length group; its length is always one less than that of
/// the first.
pub fn get_implied_distribution(f: impl Fn(u64) -> usize) -> (Vec<(u64, usize)>, Vec<f64>) {
    // Collect change points with code length up to 128, plus the first
    // change point with length > 128 as a sentinel upper bound for the
    // last valid length group.
    let mut change_points = Vec::new();
    let mut hit_sentinel = false;
    for item in FindChangePoints::new(f) {
        let len = item.1;
        change_points.push(item);
        if len > 128 {
            hit_sentinel = true;
            break;
        }
    }
    // Real code length functions stay well below 128, so FindChangePoints
    // exhausts without ever emitting the > 128 sentinel. Append an upper bound
    // at the end of the domain (u64::MAX, exclusive) so the windows(2) below
    // does not drop the final (highest-value) length group.
    if !hit_sentinel {
        if let Some(&(last_input, last_len)) = change_points.last() {
            if last_input != u64::MAX {
                change_points.push((u64::MAX, last_len));
            }
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
struct InfiniteIterator;

impl Iterator for InfiniteIterator {
    type Item = ();

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        Some(())
    }
}

impl FusedIterator for InfiniteIterator {}

/// Returns an infinite iterator of samples from the implied distribution of
/// the given code length function.
///
/// The function `len` must be the length function of the code.
///
/// This function works only with monotonic non-decreasing length functions.
///
/// # Examples
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
    len: impl Fn(u64) -> usize,
    rng: &mut impl Rng,
) -> impl Iterator<Item = u64> + '_ {
    let (change_points, probabilities) = get_implied_distribution(len);
    // A length function that is constant over the whole u64 domain yields a
    // single change point and no probability groups; its implied distribution
    // is then uniform (every value has the same codeword length). Handle that
    // case without panicking, and use the weighted distribution otherwise.
    let dist = if probabilities.is_empty() {
        None
    } else {
        Some(
            WeightedIndex::new(probabilities)
                .expect("get_implied_distribution returns positive weights"),
        )
    };

    InfiniteIterator.map(move |_| match &dist {
        None => rng.random::<u64>(),
        Some(dist) => {
            // sample a length with the correct probability, then a value with it
            let idx = dist.sample(rng);
            let (start_input, _len) = change_points[idx];
            let (end_input, _len) = change_points[idx + 1];
            rng.random_range(start_input..end_input)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::sample_implied_distribution;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    #[test]
    fn constant_length_samples_without_panic() {
        let mut rng = SmallRng::seed_from_u64(0);
        // A constant length function yields a single length group spanning the
        // whole domain; sampling must not panic.
        let vals: Vec<u64> = sample_implied_distribution(|_| 7, &mut rng)
            .take(100)
            .collect();
        assert_eq!(vals.len(), 100);
    }

    #[test]
    fn distribution_includes_final_group() {
        use super::get_implied_distribution;
        use crate::codes::len_gamma;
        let (change_points, probabilities) = get_implied_distribution(len_gamma);
        // Real code lengths never exceed 128, so the last group used to be
        // dropped; it must now be present, bounded by the domain-end sentinel.
        assert!(!probabilities.is_empty());
        assert_eq!(probabilities.len(), change_points.len() - 1);
        assert_eq!(change_points.last().unwrap().0, u64::MAX);
    }
}

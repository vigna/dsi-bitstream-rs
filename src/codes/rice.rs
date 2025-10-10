/*
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Rice codes.
//!
//! Rice codes (AKA Golomb−Rice codes) are a form of approximated [Golomb
//! codes](crate::codes::golomb) in which the parameter *b* is a power of
//! two. This restriction makes the code less precise in modeling data with a
//! geometric distribution, but encoding and decoding can be performed without
//! any integer arithmetic, and thus much more quickly.
//!
//! The implied distribution of a Rice code is [the same as that of a Golomb
//! code with the same parameter](crate::codes::golomb).
//!
//! For natural numbers distributed with a geometric distribution with base *p*,
//! the base-2 logarithm of the optimal *b* is [⌈log₂(ln((√5 + 1)/2) / ln(1 -
//! *p*))⌉](log2_b).
//!
//! The supported range is [0 . . 2⁶⁴) for log₂(*b*) in [0 . . 64), but writing
//! 2⁶⁴ – 1 when *b* = 1 requires writing the unary code for 2⁶⁴ – 1, which
//! might not be possible depending on the [`BitWrite`](crate::traits::BitWrite)
//! implementation (and would require writing 2⁶⁴ bits anyway).
//!
//! # References
//!
//! Robert F. Rice, “[Some practical universal noiseless coding
//! techniques](https://ntrs.nasa.gov/api/citations/19790014634/downloads/19790014634.pdf)”.
//! Jet Propulsion Laboratory, Pasadena, CA, Tech. Rep. JPL-79-22, JPL-83-17,
//! and JPL-91-3, March 1979.
//!
//! Aaron Kiely. “[Selecting the Golomb parameter in Rice
//! coding](https://tda.jpl.nasa.gov/progress_report/42-159/159E.pdf)”.
//! Interplanetary Network Progress report 42-159, Jet Propulsion Laboratory,
//! 2004.

use crate::traits::*;

/// Returns the length of the Rice code for `n` with parameter `log2_b`.
#[must_use]
#[inline(always)]
pub fn len_rice(n: u64, log2_b: usize) -> usize {
    debug_assert!(log2_b < 64);
    (n >> log2_b) as usize + 1 + log2_b
}

/// Returns the optimal value of log₂*b* for a geometric distribution of base
/// *p*, that is, ⌈log₂(ln((√5 + 1)/2) / ln(1 - *p*))⌉
pub fn log2_b(p: f64) -> usize {
    ((-((5f64.sqrt() + 1.0) / 2.0).ln() / (-p).ln_1p()).log2()).ceil() as usize
}

/// Trait for reading Rice codes.
pub trait RiceRead<E: Endianness>: BitRead<E> {
    #[inline(always)]
    fn read_rice(&mut self, log2_b: usize) -> Result<u64, Self::Error> {
        debug_assert!(log2_b < 64);
        Ok((self.read_unary()? << log2_b) + self.read_bits(log2_b)?)
    }
}

/// Trait for writing Rice codes.
pub trait RiceWrite<E: Endianness>: BitWrite<E> {
    #[inline(always)]
    fn write_rice(&mut self, n: u64, log2_b: usize) -> Result<usize, Self::Error> {
        debug_assert!(log2_b < 64);

        let mut written_bits = self.write_unary(n >> log2_b)?;
        #[cfg(feature = "checks")]
        {
            // Clean up n in case checks are enabled
            let n = n & (1_u128 << log2_b).wrapping_sub(1) as u64;
            written_bits += self.write_bits(n, log2_b)?;
        }
        #[cfg(not(feature = "checks"))]
        {
            written_bits += self.write_bits(n, log2_b)?;
        }
        Ok(written_bits)
    }
}

impl<E: Endianness, B: BitRead<E>> RiceRead<E> for B {}
impl<E: Endianness, B: BitWrite<E>> RiceWrite<E> for B {}

#[cfg(test)]
#[test]
fn test_log2_b() {
    use crate::prelude::golomb::b;

    let mut p = 1.0;
    for _ in 0..100 {
        p *= 0.9;
        let golomb = b(p);
        if golomb & -(golomb as i64) as u64 == golomb {
            assert_eq!(golomb, 1 << log2_b(p));
        }
    }
}

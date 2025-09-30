/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Minimal binary codes.
//!
//! A minimal binary code with upper bound *u* > 0 (AKA [truncated binary
//! encoding](https://en.wikipedia.org/wiki/Truncated_binary_encoding)) is an
//! optimal prefix-free code for the first *u* natural numbers with uniform
//! distribution.
//!
//! There are several such codes, and the one implemented here is defined as
//! follows: let *s* = ⌈log₂*u*⌉; then, given *x* < *u*, if *x* <
//! 2*ˢ* − *u* then *x* is coded as the binary representation of *x*
//! in *s* − 1 bits; otherwise, *x* is coded as the binary representation of *x*
//! − *u* + 2*ˢ* in *s* bits.
//!
//! The supported range for *u* is [0 . . 2⁶⁴).
//!
//! See the [codes module documentation](crate::codes) for some elaboration on
//! the difference between the big-endian and little-endian versions of the
//! codes.

use crate::traits::*;

/// Returns the length of the minimal binary code for `n` with upper bound `u`.
#[must_use]
#[inline(always)]
pub fn len_minimal_binary(n: u64, u: u64) -> usize {
    debug_assert!(n < u);
    if u == 0 {
        return 0;
    }
    let l = u.ilog2();
    let limit = ((1_u64 << l) << 1).wrapping_sub(u);
    let mut result = l as usize;
    if n >= limit {
        result += 1;
    }
    result
}

/// Trait for reading minimal binary codes.
pub trait MinimalBinaryRead<E: Endianness>: BitRead<E> {
    #[inline(always)]
    fn read_minimal_binary(&mut self, u: u64) -> Result<u64, Self::Error> {
        let l = u.ilog2();
        let mut prefix = self.read_bits(l as _)?;
        let limit = ((1_u64 << l) << 1).wrapping_sub(u);

        Ok(if prefix < limit {
            prefix
        } else {
            prefix <<= 1;
            prefix |= self.read_bits(1)?;
            prefix - limit
        })
    }
}

/// Trait for writing minimal binary codes.
pub trait MinimalBinaryWrite<E: Endianness>: BitWrite<E> {
    #[inline(always)]
    fn write_minimal_binary(&mut self, n: u64, u: u64) -> Result<usize, Self::Error> {
        debug_assert!(n < u);
        let l = u.ilog2();
        let limit = ((1_u64 << l) << 1).wrapping_sub(u);

        if n < limit {
            self.write_bits(n, l as _)?;
            Ok(l as usize)
        } else {
            let to_write = n + limit;
            self.write_bits(to_write >> 1, l as _)?;
            self.write_bits(to_write & 1, 1)?;
            Ok((l + 1) as usize)
        }
    }
}

impl<E: Endianness, B: BitRead<E>> MinimalBinaryRead<E> for B {}
impl<E: Endianness, B: BitWrite<E>> MinimalBinaryWrite<E> for B {}

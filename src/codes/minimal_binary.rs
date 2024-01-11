/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! # Minimal Binary
//!
//! Also called [Truncated binary encoding](https://en.wikipedia.org/wiki/Truncated_binary_encoding)
//! is optimal for uniform distributions.
//! When the size of the alphabet is a power of two, this is equivalent to
//! the classical binary encoding.

use crate::traits::*;

/// Returns how long the minimal binary code for `value` will be for a given
/// `max`
#[must_use]
#[inline]
pub fn len_minimal_binary(value: u64, max: u64) -> usize {
    if max == 0 {
        return 0;
    }
    let l = max.ilog2();
    let limit = (1 << (l + 1)) - max;
    let mut result = l as usize;
    if value >= limit {
        result += 1;
    }
    result
}

#[inline(always)]
fn ensure_max(max: u64) {
    assert!(max > 0, "max = {}", max);
}

/// Trait for objects that can read Minimal Binary codes
pub trait MinimalBinaryRead<BO: Endianness>: BitRead<BO> {
    /// Read a minimal binary code from the stream.
    ///
    /// # Errors
    /// This function fails only if the BitRead backend has problems reading
    /// bits, as when the stream ends unexpectedly
    #[inline(always)]
    fn read_minimal_binary(&mut self, max: u64) -> Result<u64, Self::Error> {
        ensure_max(max);
        let l = max.ilog2();
        let mut value = self.read_bits(l as _)?;
        let limit = (1 << (l + 1)) - max;

        Ok(if value < limit {
            value
        } else {
            value <<= 1;
            value |= self.read_bits(1)?;
            value - limit
        })
    }

    /// Read a minimal binary code from the stream.
    ///
    /// # Errors
    /// This function fails only if the BitRead backend has problems reading
    /// bits, as when the stream ends unexpectedly
    #[inline(always)]
    fn skip_minimal_binary(&mut self, max: u64) -> Result<(), Self::Error> {
        ensure_max(max);
        let l = max.ilog2();
        let limit = (1 << (l + 1)) - max;

        let value = self.read_bits(l as _)?;

        if value >= limit {
            self.skip_bits(1)?;
        }
        Ok(())
    }
}

/// Trait for objects that can write Minimal Binary codes
pub trait MinimalBinaryWrite<BO: Endianness>: BitWrite<BO> {
    /// Write a value on the stream and return the number of bits written.
    ///
    /// # Errors
    /// This function fails only if the BitRead backend has problems writing
    /// bits, as when the stream ends unexpectedly
    #[inline]
    fn write_minimal_binary(&mut self, value: u64, max: u64) -> Result<usize, Self::Error> {
        ensure_max(max);
        let l = max.ilog2();
        let limit = (1 << (l + 1)) - max;

        if value < limit {
            self.write_bits(value, l as _)?;
            Ok(l as usize)
        } else {
            let to_write = value + limit;
            self.write_bits(to_write >> 1, l as _)?;
            self.write_bits(to_write & 1, 1)?;
            Ok((l + 1) as usize)
        }
    }
}

impl<BO: Endianness, B: BitRead<BO>> MinimalBinaryRead<BO> for B {}
impl<BO: Endianness, B: BitWrite<BO>> MinimalBinaryWrite<BO> for B {}

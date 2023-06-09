/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! # Elias Gamma
//! Optimal for Zipf of exponent 2
//! Elias’ γ universal coding of x ∈ N+ is obtained by representing x in binary
//! preceded by a unary representation of its length (minus one).
//! More precisely, to represent x we write in unary floor(log(x)) and then in
//! binary x - 2^ceil(log(x)) (on floor(log(x)) bits)
//!

use super::gamma_tables;
use crate::traits::*;
use anyhow::Result;

/// Returns how long the gamma code for `value` will be
///
/// `USE_TABLE` enables or disables the use of pre-computed tables
/// for decoding
#[must_use]
#[inline]
pub fn len_gamma_param<const USE_TABLE: bool>(mut value: u64) -> usize {
    if USE_TABLE {
        if let Some(idx) = gamma_tables::LEN.get(value as usize) {
            return *idx as usize;
        }
    }
    value += 1;
    let number_of_bits_to_write = value.ilog2();
    2 * number_of_bits_to_write as usize + 1
}

pub fn len_gamma(value: u64) -> usize {
    #[cfg(target_arch = "arm")]
    return len_gamma_param::<false>(value);
    #[cfg(not(target_arch = "arm"))]
    return len_gamma_param::<true>(value);
}

pub trait GammaRead<E: Endianness>: BitRead<E> {
    fn read_gamma(&mut self) -> Result<u64>;
    fn skip_gamma(&mut self) -> Result<()>;
}

/// Trait for objects that can read Gamma codes
pub trait GammaReadParam<E: Endianness>: BitRead<E> {
    /// Read a gamma code from the stream.
    ///
    /// `USE_TABLE` enables or disables the use of pre-computed tables
    /// for decoding
    ///
    /// # Errors
    /// This function fails only if the BitRead backend has problems reading
    /// bits, as when the stream ends unexpectedly
    fn read_gamma_param<const USE_TABLE: bool>(&mut self) -> Result<u64>;

    /// Skip a number of gamma codes from the stream.
    ///
    /// `USE_TABLE` enables or disables the use of pre-computed tables
    /// for decoding
    ///
    /// # Errors
    /// This function fails only if the BitRead backend has problems reading
    /// bits, as when the stream ends unexpectedly
    fn skip_gamma_param<const USE_TABLE: bool>(&mut self) -> Result<()>;
}

/// Common part of the BE and LE impl
///
/// # Errors
/// Forward `read_unary` and `read_bits` errors.
#[inline(always)]
fn default_read_gamma<E: Endianness, B: BitRead<E>>(backend: &mut B) -> Result<u64> {
    let len = backend.read_unary_param::<false>()?;
    debug_assert!(len <= 64);
    Ok(backend.read_bits(len as usize)? + (1 << len) - 1)
}

#[inline(always)]
fn default_skip_gamma<E: Endianness, B: BitRead<E>>(backend: &mut B) -> Result<()> {
    let len = backend.read_unary_param::<false>()?;
    debug_assert!(len <= 64);
    backend.skip_bits(len as usize)
}

impl<B: BitRead<BE>> GammaReadParam<BE> for B {
    #[inline]
    fn read_gamma_param<const USE_TABLE: bool>(&mut self) -> Result<u64> {
        if USE_TABLE {
            if let Some((res, _)) = gamma_tables::read_table_be(self)? {
                return Ok(res);
            }
        }
        default_read_gamma(self)
    }

    #[inline]
    fn skip_gamma_param<const USE_TABLE: bool>(&mut self) -> Result<()> {
        if USE_TABLE {
            if let Some((_, _)) = gamma_tables::read_table_be(self)? {
                return Ok(());
            }
        }
        default_skip_gamma(self)
    }
}
impl<B: BitRead<LE>> GammaReadParam<LE> for B {
    #[inline]
    fn read_gamma_param<const USE_TABLE: bool>(&mut self) -> Result<u64> {
        if USE_TABLE {
            if let Some((res, _)) = gamma_tables::read_table_le(self)? {
                return Ok(res);
            }
        }
        default_read_gamma(self)
    }

    #[inline]
    fn skip_gamma_param<const USE_TABLE: bool>(&mut self) -> Result<()> {
        if USE_TABLE {
            if let Some((_, _)) = gamma_tables::read_table_le(self)? {
                return Ok(());
            }
        }
        default_skip_gamma(self)
    }
}

pub trait GammaWrite<E: Endianness>: BitWrite<E> {
    fn write_gamma(&mut self, value: u64) -> Result<usize>;
}

/// Trait for objects that can write Gamma codes
pub trait GammaWriteParam<E: Endianness>: BitWrite<E> {
    /// Write a value on the stream
    ///
    /// `USE_TABLE` enables or disables the use of pre-computed tables
    /// for decoding
    ///
    /// # Errors
    /// This function fails only if the BitWrite backend has problems writing
    /// bits, as when the stream ends unexpectedly
    fn write_gamma_param<const USE_TABLE: bool>(&mut self, value: u64) -> Result<usize>;
}

impl<B: BitWrite<BE>> GammaWriteParam<BE> for B {
    #[inline]
    #[allow(clippy::collapsible_if)]
    fn write_gamma_param<const USE_TABLE: bool>(&mut self, value: u64) -> Result<usize> {
        if USE_TABLE {
            if let Some(len) = gamma_tables::write_table_be(self, value)? {
                return Ok(len);
            }
        }
        default_write_gamma(self, value)
    }
}

impl<B: BitWrite<LE>> GammaWriteParam<LE> for B {
    #[inline]
    #[allow(clippy::collapsible_if)]
    fn write_gamma_param<const USE_TABLE: bool>(&mut self, value: u64) -> Result<usize> {
        if USE_TABLE {
            if let Some(len) = gamma_tables::write_table_le(self, value)? {
                return Ok(len);
            }
        }
        default_write_gamma(self, value)
    }
}

/// Common part of the BE and LE impl
///
/// # Errors
/// Forward `read_unary` and `read_bits` errors.
#[inline(always)]
fn default_write_gamma<E: Endianness, B: BitWrite<E>>(
    backend: &mut B,
    mut value: u64,
) -> Result<usize> {
    value += 1;
    let number_of_bits_to_write = value.ilog2();
    debug_assert!(number_of_bits_to_write <= u8::MAX as _);
    // remove the most significant 1
    let short_value = value - (1 << number_of_bits_to_write);
    // Write the code
    Ok(
        backend.write_unary_param::<false>(number_of_bits_to_write as _)?
            + backend.write_bits(short_value, number_of_bits_to_write as usize)?,
    )
}

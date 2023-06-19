/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! # Elias’ δ
//! universal coding of x ∈ N+ is obtained by representing x in binary
//! preceded by a representation of its length in γ.

use super::{
    delta_tables, fast_floor_log2, gamma_tables, len_gamma_param, GammaReadParam, GammaWriteParam,
};
use crate::traits::*;
use anyhow::Result;

#[must_use]
#[inline]
/// Returns how long the Delta code for `value` will be
///
/// `USE_DELTA_TABLE` enables or disables the use of pre-computed tables
/// for decoding
pub fn len_delta_param<const USE_DELTA_TABLE: bool, const USE_GAMMA_TABLE: bool>(
    value: u64,
) -> usize {
    if USE_DELTA_TABLE {
        if let Some(idx) = delta_tables::LEN.get(value as usize) {
            return *idx as usize;
        }
    }
    let l = fast_floor_log2(value + 1);
    l as usize + len_gamma_param::<USE_GAMMA_TABLE>(l as _)
}

/// Returns how long the Delta code for `value` will be
///
/// `USE_DELTA_TABLE` enables or disables the use of pre-computed tables
/// for decoding
#[inline(always)]
pub fn len_delta(value: u64) -> usize {
    #[cfg(target_arch = "arm")]
    return len_delta_param::<false, false>(value);
    #[cfg(not(target_arch = "arm"))]
    return len_delta_param::<false, true>(value);
}

pub trait DeltaRead<E: Endianness>: BitRead<E> {
    fn read_delta(&mut self) -> Result<u64>;
    fn skip_delta(&mut self, n: usize) -> Result<usize>;
}

/// Trait for objects that can read Delta codes
pub trait DeltaReadParam<E: Endianness>: GammaReadParam<E> {
    /// Read a delta code from the stream.
    ///
    /// `USE_DELTA_TABLE` enables or disables the use of pre-computed tables
    /// for decoding delta codes. `USE_GAMMA_TABLE` enables or disables the use
    /// of pre-computed tables for decoding the gamma-coded length of the
    /// delta code.
    ///
    /// # Errors
    /// This function fails only if the BitRead backend has problems reading
    /// bits, as when the stream ends unexpectedly
    fn read_delta_param<const USE_DELTA_TABLE: bool, const USE_GAMMA_TABLE: bool>(
        &mut self,
    ) -> Result<u64>;

    /// Skip a number of dleta codes from the stream.
    ///
    /// `USE_DELTA_TABLE` enables or disables the use of pre-computed tables
    /// for decoding delta codes. `USE_GAMMA_TABLE` enables or disables the use
    /// of pre-computed tables for decoding the gamma-coded length of the
    /// delta code.
    ///
    /// # Errors
    /// This function fails only if the BitRead backend has problems reading
    /// bits, as when the stream ends unexpectedly
    fn skip_delta_param<const USE_DELTA_TABLE: bool, const USE_GAMMA_TABLE: bool>(
        &mut self,
        n: usize,
    ) -> Result<usize>;
}

/// Common part of the BE and LE impl
///
/// # Errors
/// Forward `read_unary` and `read_bits` errors.
#[inline(always)]
fn default_read_delta<E: Endianness, B: GammaReadParam<E>, const USE_GAMMA_TABLE: bool>(
    backend: &mut B,
) -> Result<u64> {
    let len = backend.read_gamma_param::<USE_GAMMA_TABLE>()?;
    debug_assert!(len <= 64);
    Ok(backend.read_bits(len as usize)? + (1 << len) - 1)
}

macro_rules! default_skip_delta_impl {
    ($endianness:ty, $default_skip_delta: ident, $read_table: ident) => {
        #[inline(always)]
        fn $default_skip_delta<B: GammaReadParam<$endianness>, const USE_GAMMA_TABLE: bool>(
            backend: &mut B,
        ) -> Result<usize> {
            /*
            // TODO: debug this faster implementation
            let (value, len) = 'gamma: {
                eprintln!("skip delta");
                if USE_GAMMA_TABLE {
                    if let Some((value, len)) = gamma_tables::$read_table(backend)? {
                        break 'gamma (value, len);
                    }
                };
                let len = backend.read_unary()?;
                dbg!(len);
                debug_assert!(len <= 64);
                (
                    backend.read_bits(len as usize)? + (1 << len) - 1,
                    2 * len as usize - 1,
                )
            };
            dbg!(value, len);
            debug_assert!(len <= 64);
            backend.skip_bits(value as usize)?;
            Ok(value as usize + len)
            */
            let value = backend.read_gamma_param::<USE_GAMMA_TABLE>()?;
            backend.skip_bits(value as usize)?;
            Ok(len_gamma_param::<USE_GAMMA_TABLE>(value as _) + value as usize)
        }
    };
}

default_skip_delta_impl!(LE, default_skip_delta_le, read_table_le);
default_skip_delta_impl!(BE, default_skip_delta_be, read_table_be);

impl<B: GammaReadParam<BE>> DeltaReadParam<BE> for B {
    #[inline]
    fn read_delta_param<const USE_DELTA_TABLE: bool, const USE_GAMMA_TABLE: bool>(
        &mut self,
    ) -> Result<u64> {
        if USE_DELTA_TABLE {
            if let Some((res, _)) = delta_tables::read_table_be(self)? {
                return Ok(res);
            }
        }
        default_read_delta::<BE, _, USE_GAMMA_TABLE>(self)
    }

    #[inline]
    fn skip_delta_param<const USE_DELTA_TABLE: bool, const USE_GAMMA_TABLE: bool>(
        &mut self,
        n: usize,
    ) -> Result<usize> {
        let mut skipped_bits = 0;
        for _ in 0..n {
            if USE_DELTA_TABLE {
                if let Some((_, len)) = delta_tables::read_table_be(self)? {
                    skipped_bits += len;
                    continue;
                }
            }
            skipped_bits += default_skip_delta_be::<_, USE_GAMMA_TABLE>(self)?;
        }

        Ok(skipped_bits)
    }
}
impl<B: GammaReadParam<LE>> DeltaReadParam<LE> for B {
    #[inline]
    fn read_delta_param<const USE_DELTA_TABLE: bool, const USE_GAMMA_TABLE: bool>(
        &mut self,
    ) -> Result<u64> {
        if USE_DELTA_TABLE {
            if let Some((res, _)) = delta_tables::read_table_le(self)? {
                return Ok(res);
            }
        }
        default_read_delta::<LE, _, USE_GAMMA_TABLE>(self)
    }

    #[inline]
    fn skip_delta_param<const USE_DELTA_TABLE: bool, const USE_GAMMA_TABLE: bool>(
        &mut self,
        n: usize,
    ) -> Result<usize> {
        let mut skipped_bits = 0;
        for _ in 0..n {
            if USE_DELTA_TABLE {
                if let Some((_, len)) = delta_tables::read_table_le(self)? {
                    skipped_bits += len;
                    continue;
                }
            }
            skipped_bits += default_skip_delta_le::<_, USE_GAMMA_TABLE>(self)?;
        }

        Ok(skipped_bits)
    }
}

pub trait DeltaWrite<E: Endianness>: BitWrite<E> {
    fn write_delta(&mut self, value: u64) -> Result<usize>;
}

/// Trait for objects that can write Delta codes
pub trait DeltaWriteParam<E: Endianness>: GammaWriteParam<E> {
    /// Write a value on the stream
    ///
    /// `USE_DELTA_TABLE` enables or disables the use of pre-computed tables
    /// for decoding
    ///
    /// # Errors
    /// This function fails only if the BitWrite backend has problems writing
    /// bits, as when the stream ends unexpectedly
    fn write_delta_param<const USE_DELTA_TABLE: bool, const USE_GAMMA_TABLE: bool>(
        &mut self,
        value: u64,
    ) -> Result<usize>;
}

impl<B: GammaWriteParam<BE>> DeltaWriteParam<BE> for B {
    #[inline]
    #[allow(clippy::collapsible_if)]
    fn write_delta_param<const USE_DELTA_TABLE: bool, const USE_GAMMA_TABLE: bool>(
        &mut self,
        value: u64,
    ) -> Result<usize> {
        if USE_DELTA_TABLE {
            if let Some(len) = delta_tables::write_table_be(self, value)? {
                return Ok(len);
            }
        }
        default_write_delta::<BE, _, USE_GAMMA_TABLE>(self, value)
    }
}

impl<B: GammaWriteParam<LE>> DeltaWriteParam<LE> for B {
    #[inline]
    #[allow(clippy::collapsible_if)]
    fn write_delta_param<const USE_DELTA_TABLE: bool, const USE_GAMMA_TABLE: bool>(
        &mut self,
        value: u64,
    ) -> Result<usize> {
        if USE_DELTA_TABLE {
            if let Some(len) = delta_tables::write_table_le(self, value)? {
                return Ok(len);
            }
        }
        default_write_delta::<LE, _, USE_GAMMA_TABLE>(self, value)
    }
}

/// Default impl, so specialized impls can call it
///
/// # Errors
/// Forward `write_unary` and `write_bits` errors.
#[inline(always)]
fn default_write_delta<E: Endianness, B: GammaWriteParam<E>, const USE_GAMMA_TABLE: bool>(
    backend: &mut B,
    mut value: u64,
) -> Result<usize> {
    value += 1;
    let number_of_bits_to_write = fast_floor_log2(value);
    debug_assert!(number_of_bits_to_write <= u8::MAX as _);
    // remove the most significant 1
    let short_value = value - (1 << number_of_bits_to_write);
    // Write the code
    Ok(
        backend.write_gamma_param::<USE_GAMMA_TABLE>(number_of_bits_to_write as _)?
            + backend.write_bits(short_value, number_of_bits_to_write as usize)?,
    )
}

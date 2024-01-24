/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Elias δ code.
//!
//! The δ code of a natural number `n` is the concatenation of the [γ](crate::codes::gamma)
//! code of `⌊log₂(n + 1)⌋` and the binary representation of `n + 1` with the
//! most significant bit removed.
//!
//! The `USE_DELTA_TABLE` parameter enables or disables the use of
//! pre-computed tables for decoding δ codes, and the `USE_GAMMA_TABLE` parameter
//! enables or disables the use of pre-computed tables for decoding the
//! the initial γ code in case the whole δ code could not be decoded
//! by tables.

use super::{delta_tables, len_gamma_param, GammaReadParam, GammaWriteParam};
use crate::traits::*;

/// Return the length of the δ code for `n`.
#[must_use]
#[inline]
pub fn len_delta_param<const USE_DELTA_TABLE: bool, const USE_GAMMA_TABLE: bool>(n: u64) -> usize {
    if USE_DELTA_TABLE {
        if let Some(idx) = delta_tables::LEN.get(n as usize) {
            return *idx as usize;
        }
    }
    let l = (n + 1).ilog2();
    l as usize + len_gamma_param::<USE_GAMMA_TABLE>(l as _)
}

/// Return the length of the δ code for `n` using
/// a default value for `USE_DELTA_TABLE` and `USE_GAMMA_TABLE`.
#[inline(always)]
pub fn len_delta(n: u64) -> usize {
    #[cfg(target_arch = "arm")]
    return len_delta_param::<false, false>(n);
    #[cfg(not(target_arch = "arm"))]
    return len_delta_param::<false, true>(n);
}

/// Trait for reading δ codes.
///
/// This is the trait you should usually pull in scope to read δ codes.
pub trait DeltaRead<E: Endianness>: BitRead<E> {
    fn read_delta(&mut self) -> Result<u64, Self::Error>;
}

/// Parametric trait for reading δ codes.
///
/// This trait is is more general than [`DeltaRead`], as it makes it possible
/// to specify how to use tables using const parameters.
///
/// We provide an implementation of this trait for [`BitRead`]. An implementation
/// of [`DeltaRead`] using default values is usually provided exploiting the
/// [`crate::codes::params::ReadParams`] mechanism.
pub trait DeltaReadParam<E: Endianness>: GammaReadParam<E> {
    fn read_delta_param<const USE_DELTA_TABLE: bool, const USE_GAMMA_TABLE: bool>(
        &mut self,
    ) -> Result<u64, Self::Error>;
}

/// Default, internal non-table based implementation that works
/// for any endianness.
#[inline(always)]
fn default_read_delta<E: Endianness, B: GammaReadParam<E>, const USE_GAMMA_TABLE: bool>(
    backend: &mut B,
) -> Result<u64, B::Error> {
    let len = backend.read_gamma_param::<USE_GAMMA_TABLE>()?;
    debug_assert!(len <= 64);
    Ok(backend.read_bits(len as usize)? + (1 << len) - 1)
}

impl<B: GammaReadParam<BE>> DeltaReadParam<BE> for B {
    #[inline]
    fn read_delta_param<const USE_DELTA_TABLE: bool, const USE_GAMMA_TABLE: bool>(
        &mut self,
    ) -> Result<u64, B::Error> {
        if USE_DELTA_TABLE {
            if let Some((res, _)) = delta_tables::read_table_be(self) {
                return Ok(res);
            }
        }
        default_read_delta::<BE, _, USE_GAMMA_TABLE>(self)
    }
}

impl<B: GammaReadParam<LE>> DeltaReadParam<LE> for B {
    #[inline]
    fn read_delta_param<const USE_DELTA_TABLE: bool, const USE_GAMMA_TABLE: bool>(
        &mut self,
    ) -> Result<u64, B::Error> {
        if USE_DELTA_TABLE {
            if let Some((res, _)) = delta_tables::read_table_le(self) {
                return Ok(res);
            }
        }
        default_read_delta::<LE, _, USE_GAMMA_TABLE>(self)
    }
}

/// Trait for writing δ codes.
///
/// This is the trait you should usually pull in scope to write δ codes.
pub trait DeltaWrite<E: Endianness>: BitWrite<E> {
    fn write_delta(&mut self, n: u64) -> Result<usize, Self::Error>;
}

/// Parametric trait for writing δ codes.
///
/// This trait is is more general than [`DeltaWrite`], as it makes it possible
/// to specify how to use tables using const parameters.
///
/// We provide an implementation of this trait for [`BitWrite`]. An implementation
/// of [`DeltaWrite`] using default values is usually provided exploiting the
/// [`crate::codes::params::WriteParams`] mechanism.
pub trait DeltaWriteParam<E: Endianness>: GammaWriteParam<E> {
    fn write_delta_param<const USE_DELTA_TABLE: bool, const USE_GAMMA_TABLE: bool>(
        &mut self,
        n: u64,
    ) -> Result<usize, Self::Error>;
}

impl<B: GammaWriteParam<BE>> DeltaWriteParam<BE> for B {
    #[inline]
    #[allow(clippy::collapsible_if)]
    fn write_delta_param<const USE_DELTA_TABLE: bool, const USE_GAMMA_TABLE: bool>(
        &mut self,
        n: u64,
    ) -> Result<usize, Self::Error> {
        if USE_DELTA_TABLE {
            if let Some(len) = delta_tables::write_table_be(self, n)? {
                return Ok(len);
            }
        }
        default_write_delta::<BE, _, USE_GAMMA_TABLE>(self, n)
    }
}

impl<B: GammaWriteParam<LE>> DeltaWriteParam<LE> for B {
    #[inline]
    #[allow(clippy::collapsible_if)]
    fn write_delta_param<const USE_DELTA_TABLE: bool, const USE_GAMMA_TABLE: bool>(
        &mut self,
        n: u64,
    ) -> Result<usize, Self::Error> {
        if USE_DELTA_TABLE {
            if let Some(len) = delta_tables::write_table_le(self, n)? {
                return Ok(len);
            }
        }
        default_write_delta::<LE, _, USE_GAMMA_TABLE>(self, n)
    }
}

/// Default, internal non-table based implementation that works
/// for any endianness.
#[inline(always)]
fn default_write_delta<E: Endianness, B: GammaWriteParam<E>, const USE_GAMMA_TABLE: bool>(
    backend: &mut B,
    mut n: u64,
) -> Result<usize, B::Error> {
    n += 1;
    let number_of_bits_to_write = n.ilog2();

    #[cfg(feature = "checks")]
    {
        // Clean up n in case checks are enabled
        n ^= 1 << number_of_bits_to_write;
    }

    Ok(
        backend.write_gamma_param::<USE_GAMMA_TABLE>(number_of_bits_to_write as _)?
            + backend.write_bits(n, number_of_bits_to_write as _)?,
    )
}

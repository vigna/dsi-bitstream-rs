/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Elias δ code.
//!
//! The δ code of a natural number *n* is the concatenation of the
//! [γ](crate::codes::gamma) code of ⌊log₂(*n* + 1)⌋ and the binary
//! representation of *n* + 1 with the most significant bit removed.
//!
//! The implied distribution of the δ code is ≈ 1/2*x*(log *x*)².
//!
//! The `USE_DELTA_TABLE` parameter enables or disables the use of pre-computed
//! tables for decoding δ codes, and the `USE_GAMMA_TABLE` parameter enables or
//! disables the use of pre-computed tables for decoding the the initial γ code
//! in case the whole δ code could not be decoded by tables.
//!
//! The supported range is [0 . . 2⁶⁴ – 1).
//!
//! # Table-Based Optimization
//!
//! Like [ω](super::omega) codes, δ codes use a special optimization for partial
//! decoding. Due to the structure of δ codes (a γ code followed by fixed bits),
//! when a complete codeword cannot be read from the table, the table may still
//! provide partial information about the γ prefix that was successfully decoded.
//! This partial state is used to directly read the remaining fixed bits,
//! avoiding re-reading the γ prefix.
//!
//!
//! # References
//!
//! Peter Elias, “[Universal codeword sets and representations of the
//! integers](https://doi.org/10.1109/TIT.1975.1055349)”. IEEE Transactions on
//! Information Theory, 21(2):194−203, March 1975.

use super::{GammaReadParam, GammaWriteParam, delta_tables, len_gamma_param};
use crate::traits::*;

/// Returns the length of the δ code for `n`.
#[must_use]
#[inline(always)]
pub fn len_delta_param<const USE_DELTA_TABLE: bool, const USE_GAMMA_TABLE: bool>(n: u64) -> usize {
    debug_assert!(n < u64::MAX);
    if USE_DELTA_TABLE {
        if let Some(idx) = delta_tables::LEN.get(n as usize) {
            return *idx as usize;
        }
    }
    let λ = (n + 1).ilog2();
    λ as usize + len_gamma_param::<USE_GAMMA_TABLE>(λ as _)
}

/// Returns the length of the δ code for `n` using
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
    debug_assert!(len < 64);
    Ok(backend.read_bits(len as usize)? + (1 << len) - 1)
}

impl<B: GammaReadParam<BE>> DeltaReadParam<BE> for B {
    #[inline(always)]
    fn read_delta_param<const USE_DELTA_TABLE: bool, const USE_GAMMA_TABLE: bool>(
        &mut self,
    ) -> Result<u64, B::Error> {
        if USE_DELTA_TABLE {
            let (len_with_flag, value_or_gamma) = delta_tables::read_table_be(self);
            if (len_with_flag & 0x80) != 0 {
                // Partial code: gamma decoded, need to read fixed part
                // Bits already skipped in read_table
                let gamma_len = value_or_gamma;
                debug_assert!(gamma_len < 64);
                return Ok(self.read_bits(gamma_len as usize)? + (1 << gamma_len) - 1);
            } else if len_with_flag != 0 {
                // Complete code - bits already skipped in read_table
                return Ok(value_or_gamma);
            }
            // len_with_flag == 0: no valid decoding (gamma not decoded), fall through
        }
        default_read_delta::<BE, _, USE_GAMMA_TABLE>(self)
    }
}

impl<B: GammaReadParam<LE>> DeltaReadParam<LE> for B {
    #[inline(always)]
    fn read_delta_param<const USE_DELTA_TABLE: bool, const USE_GAMMA_TABLE: bool>(
        &mut self,
    ) -> Result<u64, B::Error> {
        if USE_DELTA_TABLE {
            let (len_with_flag, value_or_gamma) = delta_tables::read_table_le(self);
            if (len_with_flag & 0x80) != 0 {
                // Partial code: gamma decoded, need to read fixed part
                // Bits already skipped in read_table
                let gamma_len = value_or_gamma;
                debug_assert!(gamma_len < 64);
                return Ok(self.read_bits(gamma_len as usize)? + (1 << gamma_len) - 1);
            } else if len_with_flag != 0 {
                // Complete code - bits already skipped in read_table
                return Ok(value_or_gamma);
            }
            // len_with_flag == 0: no valid decoding (gamma not decoded), fall through
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
    #[inline(always)]
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
    #[inline(always)]
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
    debug_assert!(n < u64::MAX);
    n += 1;
    let λ = n.ilog2();

    #[cfg(feature = "checks")]
    {
        // Clean up n in case checks are enabled
        n ^= 1 << λ;
    }

    Ok(backend.write_gamma_param::<USE_GAMMA_TABLE>(λ as _)? + backend.write_bits(n, λ as _)?)
}

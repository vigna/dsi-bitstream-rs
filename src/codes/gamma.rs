/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Elias γ code.
//!
//! The γ code of a natural number *n* is the concatenation of the unary code of
//! ⌊log₂(*n* + 1)⌋ and of the binary representation of *n* + 1 with the most
//! significant bit removed.
//!
//! The implied distribution of the γ code is ∝ 1/2*x*².
//!
//! The `USE_TABLE` parameter enables or disables the use of pre-computed tables
//! for decoding.
//!
//! # References
//!
//! Peter Elias, “[Universal codeword sets and representations of the
//! integers](https://doi.org/10.1109/TIT.1975.1055349)”. IEEE Transactions on
//! Information Theory, 21(2):194–203, March 1975.

use super::gamma_tables;
use crate::traits::*;

/// Returns the length of the γ code for `n`.
#[must_use]
#[inline]
pub fn len_gamma_param<const USE_TABLE: bool>(mut n: u64) -> usize {
    if USE_TABLE {
        if let Some(idx) = gamma_tables::LEN.get(n as usize) {
            return *idx as usize;
        }
    }
    n += 1;
    let λ = n.ilog2();
    2 * λ as usize + 1
}

/// Returns the length of the γ code for `n` using
/// a default value for `USE_TABLE`.
pub fn len_gamma(n: u64) -> usize {
    #[cfg(target_arch = "arm")]
    return len_gamma_param::<false>(n);
    #[cfg(not(target_arch = "arm"))]
    return len_gamma_param::<true>(n);
}

/// Trait for reading γ codes.
///
/// This is the trait you should usually pull in scope to read γ codes.
pub trait GammaRead<E: Endianness>: BitRead<E> {
    fn read_gamma(&mut self) -> Result<u64, Self::Error>;
}

/// Parametric trait for reading γ codes.
///
/// This trait is is more general than [`GammaRead`], as it makes it possible
/// to specify how to use tables using const parameters.
///
/// We provide an implementation of this trait for [`BitRead`]. An implementation
/// of [`GammaRead`] using default values is usually provided exploiting the
/// [`crate::codes::params::ReadParams`] mechanism.
pub trait GammaReadParam<E: Endianness>: BitRead<E> {
    fn read_gamma_param<const USE_TABLE: bool>(&mut self) -> Result<u64, Self::Error>;
}

/// Default, internal non-table based implementation that works
/// for any endianness.
#[inline(always)]
fn default_read_gamma<E: Endianness, B: BitRead<E>>(backend: &mut B) -> Result<u64, B::Error> {
    let len = backend.read_unary()?;
    debug_assert!(len <= 64);
    Ok(backend.read_bits(len as usize)? + (1 << len) - 1)
}

impl<B: BitRead<BE>> GammaReadParam<BE> for B {
    #[inline(always)]
    fn read_gamma_param<const USE_TABLE: bool>(&mut self) -> Result<u64, Self::Error> {
        if USE_TABLE {
            if let Some((res, _)) = gamma_tables::read_table_be(self) {
                return Ok(res);
            }
        }
        default_read_gamma(self)
    }
}

impl<B: BitRead<LE>> GammaReadParam<LE> for B {
    #[inline(always)]
    fn read_gamma_param<const USE_TABLE: bool>(&mut self) -> Result<u64, Self::Error> {
        if USE_TABLE {
            if let Some((res, _)) = gamma_tables::read_table_le(self) {
                return Ok(res);
            }
        }
        default_read_gamma(self)
    }
}

/// Trait for writing γ codes.
///
/// This is the trait you should usually pull in scope to write γ codes.
pub trait GammaWrite<E: Endianness>: BitWrite<E> {
    fn write_gamma(&mut self, n: u64) -> Result<usize, Self::Error>;
}

/// Parametric trait for writing γ codes.
///
/// This trait is is more general than [`GammaWrite`], as it makes it possible
/// to specify how to use tables using const parameters.
///
/// We provide an implementation of this trait for [`BitWrite`]. An implementation
/// of [`GammaWrite`] using default values is usually provided exploiting the
/// [`crate::codes::params::WriteParams`] mechanism.
pub trait GammaWriteParam<E: Endianness>: BitWrite<E> {
    fn write_gamma_param<const USE_TABLE: bool>(&mut self, n: u64) -> Result<usize, Self::Error>;
}

impl<B: BitWrite<BE>> GammaWriteParam<BE> for B {
    #[inline(always)]
    #[allow(clippy::collapsible_if)]
    fn write_gamma_param<const USE_TABLE: bool>(&mut self, n: u64) -> Result<usize, Self::Error> {
        if USE_TABLE {
            if let Some(len) = gamma_tables::write_table_be(self, n)? {
                return Ok(len);
            }
        }
        default_write_gamma(self, n)
    }
}

impl<B: BitWrite<LE>> GammaWriteParam<LE> for B {
    #[inline(always)]
    #[allow(clippy::collapsible_if)]
    fn write_gamma_param<const USE_TABLE: bool>(&mut self, n: u64) -> Result<usize, Self::Error> {
        if USE_TABLE {
            if let Some(len) = gamma_tables::write_table_le(self, n)? {
                return Ok(len);
            }
        }
        default_write_gamma(self, n)
    }
}

/// Default, internal non-table based implementation that works
/// for any endianness.
#[inline(always)]
fn default_write_gamma<E: Endianness, B: BitWrite<E>>(
    backend: &mut B,
    mut n: u64,
) -> Result<usize, B::Error> {
    n += 1;
    let λ = n.ilog2();

    #[cfg(feature = "checks")]
    {
        // Clean up n in case checks are enabled
        n ^= 1 << λ;
    }

    Ok(backend.write_unary(λ as _)? + backend.write_bits(n, λ as _)?)
}

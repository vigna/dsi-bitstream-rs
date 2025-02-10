/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Boldi–Vigna ζ codes.
//!
//! The ζ code with parameter *k* ≥ 1 of a natural number *n* is the
//! concatenation of of the unary code of *h* = ⌊⌊log₂(*n* + 1)⌋ / *k*⌋ and of
//! the [minimal binary code](crate::codes::minimal_binary) of *n* + 1 – 2*ʰᵏ*
//! with 2⁽*ʰ* ⁺ ¹⁾*ᵏ* – 2*ʰᵏ* as upper bound.
//!
//! The implied distribution of a ζ code with parameter *k* is ≈ 1/*x*<sup>1 +
//! 1/*k*</sup>.
//!
//! Note that ζ₁ = [π₀](crate::codes::pi) = [γ](crate::codes::gamma) and ζ₂ =
//! [π₁](crate::codes::pi).
//!
//! This module provides a generic implementation of ζ codes, and a specialized
//! implementation for ζ₃ that may use tables.
//!
//! # References
//!
//! Paolo Boldi and Sebastiano Vigna. “[Codes for the World−Wide
//! Web](https://doi.org/10.1080/15427951.2005.10129113)”. Internet Math.,
//! 2(4):405−427, 2005.

use super::{len_minimal_binary, zeta_tables, MinimalBinaryRead, MinimalBinaryWrite};
use crate::traits::*;

/// Returns the length of the ζ code with parameter `k` for `n`.
#[must_use]
#[inline]
#[allow(clippy::collapsible_if)]
pub fn len_zeta_param<const USE_TABLE: bool>(mut n: u64, k: usize) -> usize {
    if USE_TABLE {
        if k == zeta_tables::K {
            if let Some(idx) = zeta_tables::LEN.get(n as usize) {
                return *idx as usize;
            }
        }
    }
    n += 1;
    let h = n.ilog2() as usize / k;
    let u = 1 << ((h + 1) * k);
    let l = 1 << (h * k);
    h + 1 + len_minimal_binary(n - l, u - l)
}

/// Returns the length of the ζ code with parameter `k` for `n` using
/// a default value for `USE_TABLE`.
#[inline(always)]
pub fn len_zeta(n: u64, k: usize) -> usize {
    len_zeta_param::<true>(n, k)
}

/// Trait for reading ζ codes.
///
/// This is the trait you should usually pull in scope to read ζ codes.
pub trait ZetaRead<E: Endianness>: BitRead<E> {
    fn read_zeta(&mut self, k: usize) -> Result<u64, Self::Error>;
    fn read_zeta3(&mut self) -> Result<u64, Self::Error>;
}

/// Parametric trait for reading ζ codes.
///
/// This trait is is more general than [`ZetaRead`], as it makes it possible
/// to specify how to use tables using const parameters.
///
/// We provide an implementation of this trait for [`BitRead`]. An implementation
/// of [`ZetaRead`] using default values is usually provided exploiting the
/// [`crate::codes::params::ReadParams`] mechanism.
pub trait ZetaReadParam<E: Endianness>: MinimalBinaryRead<E> {
    fn read_zeta_param(&mut self, k: usize) -> Result<u64, Self::Error>;
    fn read_zeta3_param<const USE_TABLE: bool>(&mut self) -> Result<u64, Self::Error>;
}

impl<B: BitRead<BE>> ZetaReadParam<BE> for B {
    #[inline(always)]
    fn read_zeta_param(&mut self, k: usize) -> Result<u64, B::Error> {
        default_read_zeta(self, k)
    }

    #[inline(always)]
    fn read_zeta3_param<const USE_TABLE: bool>(&mut self) -> Result<u64, B::Error> {
        if USE_TABLE {
            if let Some((res, _)) = zeta_tables::read_table_be(self) {
                return Ok(res);
            }
        }
        default_read_zeta(self, 3)
    }
}

impl<B: BitRead<LE>> ZetaReadParam<LE> for B {
    #[inline(always)]
    fn read_zeta_param(&mut self, k: usize) -> Result<u64, B::Error> {
        default_read_zeta(self, k)
    }

    #[inline(always)]
    fn read_zeta3_param<const USE_TABLE: bool>(&mut self) -> Result<u64, B::Error> {
        if USE_TABLE {
            if let Some((res, _)) = zeta_tables::read_table_le(self) {
                return Ok(res);
            }
        }
        default_read_zeta(self, 3)
    }
}

/// Default, internal non-table based implementation that works
/// for any endianness.
#[inline(always)]
fn default_read_zeta<BO: Endianness, B: BitRead<BO>>(
    backend: &mut B,
    k: usize,
) -> Result<u64, B::Error> {
    let h = backend.read_unary()? as usize;
    let u = 1 << ((h + 1) * k);
    let l = 1 << (h * k);
    let res = backend.read_minimal_binary(u - l)?;
    Ok(l + res - 1)
}

/// Trait for writing ζ codes.
///
/// This is the trait you should usually pull in scope to write ζ codes.
pub trait ZetaWrite<E: Endianness>: BitWrite<E> {
    fn write_zeta(&mut self, n: u64, k: usize) -> Result<usize, Self::Error>;
    fn write_zeta3(&mut self, n: u64) -> Result<usize, Self::Error>;
}

/// Parametric trait for writing ζ codes.
///
/// This trait is is more general than [`ZetaWrite`], as it makes it possible
/// to specify how to use tables using const parameters.
///
/// We provide an implementation of this trait for [`BitWrite`]. An implementation
/// of [`ZetaWrite`] using default values is usually provided exploiting the
/// [`crate::codes::params::WriteParams`] mechanism.
pub trait ZetaWriteParam<E: Endianness>: MinimalBinaryWrite<E> {
    fn write_zeta_param<const USE_TABLE: bool>(
        &mut self,
        n: u64,
        k: usize,
    ) -> Result<usize, Self::Error>;
    fn write_zeta3_param<const USE_TABLE: bool>(&mut self, n: u64) -> Result<usize, Self::Error>;
}

impl<B: BitWrite<BE>> ZetaWriteParam<BE> for B {
    #[inline(always)]
    fn write_zeta_param<const USE_TABLE: bool>(
        &mut self,
        n: u64,
        k: usize,
    ) -> Result<usize, Self::Error> {
        default_write_zeta(self, n, k)
    }

    #[inline(always)]
    #[allow(clippy::collapsible_if)]
    fn write_zeta3_param<const USE_TABLE: bool>(&mut self, n: u64) -> Result<usize, Self::Error> {
        if USE_TABLE {
            if let Some(len) = zeta_tables::write_table_be(self, n)? {
                return Ok(len);
            }
        }
        default_write_zeta(self, n, 3)
    }
}

impl<B: BitWrite<LE>> ZetaWriteParam<LE> for B {
    #[inline(always)]
    fn write_zeta_param<const USE_TABLE: bool>(
        &mut self,
        n: u64,
        k: usize,
    ) -> Result<usize, Self::Error> {
        default_write_zeta(self, n, k)
    }

    #[inline(always)]
    #[allow(clippy::collapsible_if)]
    fn write_zeta3_param<const USE_TABLE: bool>(&mut self, n: u64) -> Result<usize, Self::Error> {
        if USE_TABLE {
            if let Some(len) = zeta_tables::write_table_le(self, n)? {
                return Ok(len);
            }
        }
        default_write_zeta(self, n, 3)
    }
}

/// Default, internal non-table based implementation that works
/// for any endianness.
#[inline(always)]
fn default_write_zeta<E: Endianness, B: BitWrite<E>>(
    backend: &mut B,
    mut n: u64,
    k: usize,
) -> Result<usize, B::Error> {
    n += 1;
    let h = n.ilog2() as usize / k;
    let u = 1 << ((h + 1) * k);
    let l = 1 << (h * k);

    debug_assert!(l <= n, "{} <= {}", l, n);
    debug_assert!(n < u, "{} < {}", n, u);

    // Write the code
    Ok(backend.write_unary(h as u64)? + backend.write_minimal_binary(n - l, u - l)?)
}

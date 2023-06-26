/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! # Zeta
//!

use anyhow::Result;

use super::*;
use super::{len_minimal_binary, len_unary, zeta_tables, MinimalBinaryRead, MinimalBinaryWrite};
use crate::traits::*;

/// Returns how long the zeta code for `value` will be
///
/// `USE_TABLE` enables or disables the use of pre-computed tables
/// for decoding
#[must_use]
#[inline]
#[allow(clippy::collapsible_if)]
pub fn len_zeta_param<const USE_TABLE: bool>(mut value: u64, k: u64) -> usize {
    if USE_TABLE {
        if k == zeta_tables::K {
            if let Some(idx) = zeta_tables::LEN.get(value as usize) {
                return *idx as usize;
            }
        }
    }
    value += 1;
    let h = (fast_floor_log2(value) as u64) / k;
    let u = 1 << ((h + 1) * k);
    let l = 1 << (h * k);
    len_unary(h) + len_minimal_binary(value - l, u - l)
}

#[inline(always)]
pub fn len_zeta(value: u64, k: u64) -> usize {
    len_zeta_param::<true>(value, k)
}

pub trait ZetaRead<E: Endianness>: BitRead<E> {
    fn read_zeta(&mut self, k: u64) -> Result<u64>;
    fn read_zeta3(&mut self) -> Result<u64>;
    fn skip_zeta(&mut self, k: u64, n: usize) -> Result<()>;
    fn skip_zeta3(&mut self, n: usize) -> Result<()>;
}

/// Trait for objects that can read Zeta codes
pub trait ZetaReadParam<E: Endianness>: MinimalBinaryRead<E> {
    /// Generic ζ code reader
    ///
    /// # Errors
    /// This function fails only if the BitRead backend has problems reading
    /// bits, as when the stream ends unexpectedly
    fn read_zeta_param<const USE_TABLE: bool>(&mut self, k: u64) -> Result<u64>;

    /// Specialized ζ code reader for k = 3
    ///
    /// # Errors
    /// This function fails only if the BitRead backend has problems reading
    /// bits, as when the stream ends unexpectedly
    fn read_zeta3_param<const USE_TABLE: bool>(&mut self) -> Result<u64>;

    /// Generic ζ code reader
    ///
    /// # Errors
    /// This function fails only if the BitRead backend has problems reading
    /// bits, as when the stream ends unexpectedly
    fn skip_zeta_param<const USE_TABLE: bool>(&mut self, k: u64, n: usize) -> Result<()>;
    /// Specialized ζ code reader for k = 3
    ///
    /// # Errors
    /// This function fails only if the BitRead backend has problems reading
    /// bits, as when the stream ends unexpectedly
    fn skip_zeta3_param<const USE_TABLE: bool>(&mut self, n: usize) -> Result<()>;
}

impl<B: BitRead<BE>> ZetaReadParam<BE> for B {
    #[inline]
    fn read_zeta_param<const USE_TABLE: bool>(&mut self, k: u64) -> Result<u64> {
        default_read_zeta(self, k)
    }

    fn skip_zeta_param<const USE_TABLE: bool>(&mut self, k: u64, n: usize) -> Result<()> {
        for _ in 0..n {
            default_skip_zeta(self, k)?;
        }
        Ok(())
    }

    #[inline]
    fn read_zeta3_param<const USE_TABLE: bool>(&mut self) -> Result<u64> {
        if USE_TABLE {
            if let Some((res, _)) = zeta_tables::read_table_be(self)? {
                return Ok(res);
            }
        }
        default_read_zeta(self, 3)
    }

    #[inline]
    fn skip_zeta3_param<const USE_TABLE: bool>(&mut self, n: usize) -> Result<()> {
        for _ in 0..n {
            if USE_TABLE {
                if let Some((_, _)) = zeta_tables::read_table_be(self)? {
                    continue;
                }
            }
            default_skip_zeta(self, 3)?;
        }
        Ok(())
    }
}

impl<B: BitRead<LE>> ZetaReadParam<LE> for B {
    #[inline]
    fn read_zeta_param<const USE_TABLE: bool>(&mut self, k: u64) -> Result<u64> {
        default_read_zeta(self, k)
    }

    fn skip_zeta_param<const USE_TABLE: bool>(&mut self, k: u64, n: usize) -> Result<()> {
        for _ in 0..n {
            default_skip_zeta(self, k)?;
        }
        Ok(())
    }

    #[inline]
    fn read_zeta3_param<const USE_TABLE: bool>(&mut self) -> Result<u64> {
        if USE_TABLE {
            if let Some((res, _)) = zeta_tables::read_table_le(self)? {
                return Ok(res);
            }
        }
        default_read_zeta(self, 3)
    }

    #[inline(always)]
    fn skip_zeta3_param<const USE_TABLE: bool>(&mut self, n: usize) -> Result<()> {
        for _ in 0..n {
            if USE_TABLE {
                if let Some((_, _)) = zeta_tables::read_table_le(self)? {
                    continue;
                }
            }
            default_skip_zeta(self, 3)?;
        }
        Ok(())
    }
}

#[inline(always)]
fn default_read_zeta<BO: Endianness, B: BitRead<BO>>(backend: &mut B, k: u64) -> Result<u64> {
    let h = backend.read_unary_param::<false>()?;
    let u = 1 << ((h + 1) * k);
    let l = 1 << (h * k);
    let res = backend.read_minimal_binary(u - l)?;
    Ok(l + res - 1)
}

#[inline(always)]
fn default_skip_zeta<BO: Endianness, B: BitRead<BO>>(backend: &mut B, k: u64) -> Result<()> {
    let h = backend.read_unary_param::<false>()?;
    let u = 1 << ((h + 1) * k);
    let l = 1 << (h * k);
    backend.skip_minimal_binary(u - l, 1)
}

pub trait ZetaWrite<E: Endianness>: BitWrite<E> {
    fn write_zeta(&mut self, value: u64, k: u64) -> Result<usize>;
    fn write_zeta3(&mut self, value: u64) -> Result<usize>;
}

/// Trait for objects that can write Zeta codes
pub trait ZetaWriteParam<E: Endianness>: MinimalBinaryWrite<E> {
    /// Generic ζ code writer
    ///
    /// # Errors
    /// This function fails only if the BitWrite backend has problems writing
    /// bits, as when the stream ends unexpectedly
    fn write_zeta_param<const USE_TABLE: bool>(&mut self, value: u64, k: u64) -> Result<usize>;
    /// Specialized ζ code writer for k = 3 and return the number of bits written.
    ///
    /// # Errors
    /// This function fails only if the BitWrite backend has problems writing
    /// bits, as when the stream ends unexpectedly
    fn write_zeta3_param<const USE_TABLE: bool>(&mut self, value: u64) -> Result<usize>;
}

impl<B: BitWrite<BE>> ZetaWriteParam<BE> for B {
    #[inline]
    fn write_zeta_param<const USE_TABLE: bool>(&mut self, value: u64, k: u64) -> Result<usize> {
        default_write_zeta(self, value, k)
    }

    #[inline]
    #[allow(clippy::collapsible_if)]
    fn write_zeta3_param<const USE_TABLE: bool>(&mut self, value: u64) -> Result<usize> {
        if USE_TABLE {
            if let Some(len) = zeta_tables::write_table_be(self, value)? {
                return Ok(len);
            }
        }
        default_write_zeta(self, value, 3)
    }
}

impl<B: BitWrite<LE>> ZetaWriteParam<LE> for B {
    #[inline]
    fn write_zeta_param<const USE_TABLE: bool>(&mut self, value: u64, k: u64) -> Result<usize> {
        default_write_zeta(self, value, k)
    }

    #[inline]
    #[allow(clippy::collapsible_if)]
    fn write_zeta3_param<const USE_TABLE: bool>(&mut self, value: u64) -> Result<usize> {
        if USE_TABLE {
            if let Some(len) = zeta_tables::write_table_le(self, value)? {
                return Ok(len);
            }
        }
        default_write_zeta(self, value, 3)
    }
}

/// Common part of the BE and LE impl
///
/// # Errors
/// Forward `read_unary` and `read_bits` errors.
#[inline(always)]
fn default_write_zeta<E: Endianness, B: BitWrite<E>>(
    backend: &mut B,
    mut value: u64,
    k: u64,
) -> Result<usize> {
    value += 1;
    let h = fast_floor_log2(value) as u64 / k;
    let u = 1 << ((h + 1) * k);
    let l = 1 << (h * k);

    debug_assert!(l <= value, "{} <= {}", l, value);
    debug_assert!(value < u, "{} < {}", value, u);

    // Write the code
    Ok(backend.write_unary_param::<false>(h)? + backend.write_minimal_binary(value - l, u - l)?)
}

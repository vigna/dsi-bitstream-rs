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

use super::{delta_tables, fast_floor_log2, len_gamma, GammaRead, GammaWrite};
use crate::traits::*;
use anyhow::Result;

#[must_use]
#[inline]
/// Returns how long the Delta code for `value` will be
///
/// `USE_TABLE` enables or disables the use of pre-computed tables
/// for decoding
pub fn len_delta<const USE_TABLE: bool>(value: u64) -> usize {
    if USE_TABLE {
        if let Some(idx) = delta_tables::LEN.get(value as usize) {
            return *idx as usize;
        }
    }
    let l = fast_floor_log2(value + 1);
    l as usize + len_gamma::<USE_TABLE>(l as _)
}

/// Trait for objects that can read Delta codes
pub trait DeltaRead<
    BO: BitOrder,
    const USE_DELTA_TABLE: bool = false,
    const USE_GAMMA_TABLE: bool = true,
>: GammaRead<BO>
{
    /// Read a delta code from the stream.
    ///
    /// `USE_TABLE` enables or disables the use of pre-computed tables
    /// for decoding
    ///
    /// # Errors
    /// This function fails only if the BitRead backend has problems reading
    /// bits, as when the stream ended unexpectedly
    fn read_delta(&mut self) -> Result<u64>;
}

impl<B: GammaRead<BE>, const USE_DELTA_TABLE: bool, const USE_GAMMA_TABLE: bool>
    DeltaRead<BE, USE_DELTA_TABLE, USE_GAMMA_TABLE> for B
{
    #[inline]
    fn read_delta(&mut self) -> Result<u64> {
        if USE_DELTA_TABLE {
            if let Some(res) = delta_tables::read_table_be(self)? {
                return Ok(res);
            }
        }
        default_read_delta::<BE, _, USE_GAMMA_TABLE>(self)
    }
}
impl<B: GammaRead<LE>, const USE_DELTA_TABLE: bool, const USE_GAMMA_TABLE: bool>
    DeltaRead<LE, USE_DELTA_TABLE, USE_GAMMA_TABLE> for B
{
    #[inline]
    fn read_delta(&mut self) -> Result<u64> {
        if USE_DELTA_TABLE {
            if let Some(res) = delta_tables::read_table_le(self)? {
                return Ok(res);
            }
        }
        default_read_delta::<LE, _, USE_GAMMA_TABLE>(self)
    }
}

#[inline(always)]
/// Default impl, so specialized impls can call it
///
/// # Errors
/// Forward `read_unary` and `read_bits` errors.
fn default_read_delta<BO: BitOrder, B: GammaRead<BO>, const USE_GAMMA_TABLE: bool>(
    backend: &mut B,
) -> Result<u64> {
    let n_bits = backend.read_gamma::<USE_GAMMA_TABLE>()?;
    debug_assert!(n_bits <= 64);
    Ok(backend.read_bits(n_bits as usize)? + (1 << n_bits) - 1)
}

/// Trait for objects that can write Delta codes
pub trait DeltaWrite<BO: BitOrder>: GammaWrite<BO> {
    /// Write a value on the stream
    ///
    /// `USE_TABLE` enables or disables the use of pre-computed tables
    /// for decoding
    ///
    /// # Errors
    /// This function fails only if the BitWrite backend has problems writing
    /// bits, as when the stream ended unexpectedly
    fn write_delta<const USE_TABLE: bool, const USE_GAMMA_TABLE: bool>(
        &mut self,
        value: u64,
    ) -> Result<()>;
}

impl<B: GammaWrite<BE>> DeltaWrite<BE> for B {
    #[inline]
    #[allow(clippy::collapsible_if)]
    fn write_delta<const USE_TABLE: bool, const USE_GAMMA_TABLE: bool>(
        &mut self,
        value: u64,
    ) -> Result<()> {
        if USE_TABLE {
            if delta_tables::write_table_be(self, value)? {
                return Ok(());
            }
        }
        default_write_delta::<BE, _, USE_GAMMA_TABLE>(self, value)
    }
}

impl<B: GammaWrite<LE>> DeltaWrite<LE> for B {
    #[inline]
    #[allow(clippy::collapsible_if)]
    fn write_delta<const USE_TABLE: bool, const USE_GAMMA_TABLE: bool>(
        &mut self,
        value: u64,
    ) -> Result<()> {
        if USE_TABLE {
            if delta_tables::write_table_le(self, value)? {
                return Ok(());
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
fn default_write_delta<BO: BitOrder, B: GammaWrite<BO>, const USE_GAMMA_TABLE: bool>(
    backend: &mut B,
    mut value: u64,
) -> Result<()> {
    value += 1;
    let number_of_bits_to_write = fast_floor_log2(value);
    debug_assert!(number_of_bits_to_write <= u8::MAX as _);
    // remove the most significant 1
    let short_value = value - (1 << number_of_bits_to_write);
    // Write the code
    backend.write_gamma::<USE_GAMMA_TABLE>(number_of_bits_to_write as _)?;
    backend.write_bits(short_value, number_of_bits_to_write as usize)?;
    Ok(())
}

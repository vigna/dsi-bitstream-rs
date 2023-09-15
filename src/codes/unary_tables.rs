// THIS FILE HAS BEEN GENERATED WITH THE SCRIPT gen_code_tables.py
// ~~~~~~~~~~~~~~~~~~~ DO NOT MODIFY ~~~~~~~~~~~~~~~~~~~~~~
// Pre-computed constants used to speedup the reading and writing of unary codes
use crate::traits::{BitRead, BitWrite, BE, LE};
use anyhow::Result;
use common_traits::*;
/// How many bits are needed to read the tables in this
pub const READ_BITS: usize = 0;
/// The len we assign to a code that cannot be decoded through the table
pub const MISSING_VALUE_LEN: u8 = 255;
/// Maximum value writable using the table(s)
pub const WRITE_MAX: u64 = 0;

#[inline(always)]
/// Autogenerated function to lookup a read table, if the result is `Some` the
/// value was found, otherwise we were not able to decode the value and you
/// should fallback to the default implementation
///
/// # Errors
/// This function errors if it wasn't able to skip_bits
pub fn read_table_le<B: BitRead<LE>>(backend: &mut B) -> Result<Option<(u64, usize)>> {
    if let Ok(idx) = backend.peek_bits(READ_BITS) {
        let idx: u64 = idx.upcast();
        let (value, len) = READ_LE[idx as usize];
        if len != MISSING_VALUE_LEN {
            backend.skip_bits_after_table_lookup(len as usize)?;
            return Ok(Some((value as u64, len as usize)));
        }
    }
    Ok(None)
}

#[inline(always)]
#[allow(clippy::unnecessary_cast)] // rationale: "*bits as u64" is flaky redundant
/// Autogenerated function to lookup a write table, if the result is `Some` the
/// value was found, otherwise we were not able to decode the value and you
/// should fallback to the default implementation
///
/// # Errors
/// This function errors if it wasn't able to skip_bits
pub fn write_table_le<B: BitWrite<LE>>(backend: &mut B, value: u64) -> Result<Option<usize>> {
    Ok(if let Some((bits, len)) = WRITE_LE.get(value as usize) {
        backend.write_bits(*bits as u64, *len as usize)?;
        Some(*len as usize)
    } else {
        None
    })
}

#[inline(always)]
/// Autogenerated function to lookup a read table, if the result is `Some` the
/// value was found, otherwise we were not able to decode the value and you
/// should fallback to the default implementation
///
/// # Errors
/// This function errors if it wasn't able to skip_bits
pub fn read_table_be<B: BitRead<BE>>(backend: &mut B) -> Result<Option<(u64, usize)>> {
    if let Ok(idx) = backend.peek_bits(READ_BITS) {
        let idx: u64 = idx.upcast();
        let (value, len) = READ_BE[idx as usize];
        if len != MISSING_VALUE_LEN {
            backend.skip_bits_after_table_lookup(len as usize)?;
            return Ok(Some((value as u64, len as usize)));
        }
    }
    Ok(None)
}

#[inline(always)]
#[allow(clippy::unnecessary_cast)] // rationale: "*bits as u64" is flaky redundant
/// Autogenerated function to lookup a write table, if the result is `Some` the
/// value was found, otherwise we were not able to decode the value and you
/// should fallback to the default implementation
///
/// # Errors
/// This function errors if it wasn't able to skip_bits
pub fn write_table_be<B: BitWrite<BE>>(backend: &mut B, value: u64) -> Result<Option<usize>> {
    Ok(if let Some((bits, len)) = WRITE_BE.get(value as usize) {
        backend.write_bits(*bits as u64, *len as usize)?;
        Some(*len as usize)
    } else {
        None
    })
}
///Table used to speed up the reading of unary codes
pub const READ_BE: &[(u8, u8)] = &[(0, 255)];
///Table used to speed up the reading of unary codes
pub const READ_LE: &[(u8, u8)] = &[(0, 255)];
///Table used to speed up the writing of unary codes
pub const WRITE_BE: &[(u8, u8)] = &[(1, 1)];
///Table used to speed up the writing of unary codes
pub const WRITE_LE: &[(u8, u8)] = &[(1, 1)];
///Table used to speed up the skipping of unary codes
pub const LEN: &[u8] = &[1];

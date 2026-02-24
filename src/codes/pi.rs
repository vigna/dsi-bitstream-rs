/*
 * SPDX-FileCopyrightText: 2024 Tommaso Fontana
 * SPDX-FileCopyrightText: 2025 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Streamlined Apostolico−Drovandi π codes
//!
//! The streamlined π code with parameter *k* ≥ 0 of a natural number *n* is the
//! concatenation of the [Rice code](super::rice) with parameter 2*ᵏ* of
//! ⌊log₂(*n* + 1)⌋ and of the binary representation of *n* + 1 with the most
//! significant bit removed.
//!
//! The implied distribution of a π code with parameter *k* is ≈
//! 1/*x*<sup>1 + 1/2*ᵏ*</sup>.
//!
//! Note that π₀ = [ζ₁](super::zeta) = [γ](super::gamma) and π₁ =
//! [ζ₂](super::zeta). However, due to [subtle problems with
//! endianness](crate::codes), in the little-endian case π₁ and ζ₂ have the same
//! codeword lengths but slightly permuted bits.
//!
//! This module provides a generic implementation of π codes, and a specialized
//! implementation for π₂ that may use tables.
//!
//! The supported range is [0 . . 2⁶⁴ – 1) for *k* in [0 . . 64).
//!
//! In the original paper the definition of the code is very convoluted, as the
//! authors appear to have missed the connection with [Rice codes](super::rice).
//! The codewords implemented by this module are equivalent to the ones in the
//! paper, in the sense that corresponding codewords have the same length, but
//! the codewords for *k* ≥ 2 are different, and encoding/decoding is
//! faster—hence the name "streamlined π codes".
//!
//! # Table-Based Optimization
//!
//! Like [δ](super::delta) codes, π codes use a special optimization for partial
//! decoding. Due to the structure of π codes (a Rice code followed by fixed bits),
//! when a complete codeword cannot be read from the table, the table may still
//! provide partial information about the Rice prefix (λ) that was
//! successfully decoded.
//! This partial state is used to directly read the remaining λ fixed bits,
//! avoiding re-reading the Rice prefix.
//!
//! # References
//!
//! Alberto Apostolico and Guido Drovandi. "[Graph Compression by
//! BFS](https://doi.org/10.3390/a2031031)", Algorithms, 2:1031-1044, 2009.

use crate::traits::*;

use super::{RiceRead, RiceWrite, len_rice, pi_tables};

/// Returns the length of the π code with parameter `k` for `n`.
#[must_use]
#[inline(always)]
#[allow(clippy::collapsible_if)]
pub fn len_pi_param<const USE_TABLE: bool>(mut n: u64, k: usize) -> usize {
    debug_assert!(k < 64);
    if USE_TABLE {
        if k == pi_tables::K {
            if let Some(idx) = pi_tables::LEN.get(n as usize) {
                return *idx as usize;
            }
        }
    }
    debug_assert!(n < u64::MAX);
    n += 1;
    let λ = n.ilog2() as usize;
    len_rice(λ as u64, k) + λ
}

/// Returns the length of the π code for `n` with parameter `k` using
/// a default value for `USE_TABLE`.
#[must_use]
#[inline(always)]
pub fn len_pi(n: u64, k: usize) -> usize {
    len_pi_param::<true>(n, k)
}

/// Trait for reading π codes.
///
/// This is the trait you should usually pull in scope to read π codes.
pub trait PiRead<E: Endianness>: BitRead<E> {
    fn read_pi(&mut self, k: usize) -> Result<u64, Self::Error>;
    fn read_pi2(&mut self) -> Result<u64, Self::Error>;
}

/// Parametric trait for reading π codes.
///
/// This trait is more general than [`PiRead`], as it makes it possible
/// to specify how to use tables using const parameters.
///
/// We provide an implementation of this trait for [`BitRead`]. An implementation
/// of [`PiRead`] using default values is usually provided exploiting the
/// [`crate::codes::params::ReadParams`] mechanism.
pub trait PiReadParam<E: Endianness>: BitRead<E> {
    fn read_pi_param(&mut self, k: usize) -> Result<u64, Self::Error>;
    fn read_pi2_param<const USE_TABLE: bool>(&mut self) -> Result<u64, Self::Error>;
}

impl<B: BitRead<BE>> PiReadParam<BE> for B {
    #[inline(always)]
    fn read_pi_param(&mut self, k: usize) -> Result<u64, B::Error> {
        default_read_pi(self, k)
    }

    #[inline(always)]
    fn read_pi2_param<const USE_TABLE: bool>(&mut self) -> Result<u64, B::Error> {
        const {
            if USE_TABLE {
                pi_tables::check_read_table(B::PEEK_BITS)
            }
        }
        if USE_TABLE {
            let (len_with_flag, value_or_lambda) = pi_tables::read_table_be(self);
            if len_with_flag > 0 {
                // Complete code - bits already skipped in read_table
                return Ok(value_or_lambda);
            } else if len_with_flag < 0 {
                // Partial code: rice decoded, need to read fixed part
                // Bits already skipped in read_table
                let λ = value_or_lambda;
                debug_assert!(λ < 64);
                return Ok((1 << λ) + self.read_bits(λ as usize)? - 1);
            }
            // len_with_flag == 0: no valid decoding, fall through
        }
        default_read_pi(self, 2)
    }
}

impl<B: BitRead<LE>> PiReadParam<LE> for B {
    #[inline(always)]
    fn read_pi_param(&mut self, k: usize) -> Result<u64, B::Error> {
        default_read_pi(self, k)
    }

    #[inline(always)]
    fn read_pi2_param<const USE_TABLE: bool>(&mut self) -> Result<u64, B::Error> {
        const {
            if USE_TABLE {
                pi_tables::check_read_table(B::PEEK_BITS)
            }
        }
        if USE_TABLE {
            let (len_with_flag, value_or_lambda) = pi_tables::read_table_le(self);
            if len_with_flag > 0 {
                // Complete code - bits already skipped in read_table
                return Ok(value_or_lambda);
            } else if len_with_flag < 0 {
                // Partial code: rice decoded, need to read fixed part
                // Bits already skipped in read_table
                let λ = value_or_lambda;
                debug_assert!(λ < 64);
                return Ok((1 << λ) + self.read_bits(λ as usize)? - 1);
            }
            // len_with_flag == 0: no valid decoding, fall through
        }
        default_read_pi(self, 2)
    }
}

/// Default, internal non-table based implementation that works
/// for any endianness.
#[inline(always)]
fn default_read_pi<E: Endianness, B: BitRead<E>>(
    backend: &mut B,
    k: usize,
) -> Result<u64, B::Error> {
    debug_assert!(k < 64);
    let λ = backend.read_rice(k)?;
    debug_assert!(λ < 64);
    Ok((1 << λ) + backend.read_bits(λ as usize)? - 1)
}

/// Trait for writing π codes.
///
/// This is the trait you should usually pull in scope to write π codes.
pub trait PiWrite<E: Endianness>: BitWrite<E> {
    fn write_pi(&mut self, n: u64, k: usize) -> Result<usize, Self::Error>;
    fn write_pi2(&mut self, n: u64) -> Result<usize, Self::Error>;
}

/// Parametric trait for writing π codes.
///
/// This trait is more general than [`PiWrite`], as it makes it possible
/// to specify how to use tables using const parameters.
///
/// We provide an implementation of this trait for [`BitWrite`]. An implementation
/// of [`PiWrite`] using default values is usually provided exploiting the
/// [`crate::codes::params::WriteParams`] mechanism.
pub trait PiWriteParam<E: Endianness>: BitWrite<E> {
    fn write_pi_param(&mut self, n: u64, k: usize) -> Result<usize, Self::Error>;
    fn write_pi2_param<const USE_TABLE: bool>(&mut self, n: u64) -> Result<usize, Self::Error>;
}

impl<B: BitWrite<BE>> PiWriteParam<BE> for B {
    #[inline(always)]
    fn write_pi_param(&mut self, n: u64, k: usize) -> Result<usize, Self::Error> {
        default_write_pi(self, n, k)
    }

    #[inline(always)]
    #[allow(clippy::collapsible_if)]
    fn write_pi2_param<const USE_TABLE: bool>(&mut self, n: u64) -> Result<usize, Self::Error> {
        if USE_TABLE {
            if let Some(len) = pi_tables::write_table_be(self, n)? {
                return Ok(len);
            }
        }
        default_write_pi(self, n, 2)
    }
}

impl<B: BitWrite<LE>> PiWriteParam<LE> for B {
    #[inline(always)]
    fn write_pi_param(&mut self, n: u64, k: usize) -> Result<usize, Self::Error> {
        default_write_pi(self, n, k)
    }

    #[inline(always)]
    #[allow(clippy::collapsible_if)]
    fn write_pi2_param<const USE_TABLE: bool>(&mut self, n: u64) -> Result<usize, Self::Error> {
        if USE_TABLE {
            if let Some(len) = pi_tables::write_table_le(self, n)? {
                return Ok(len);
            }
        }
        default_write_pi(self, n, 2)
    }
}

/// Default, internal non-table based implementation that works
/// for any endianness.
#[inline(always)]
fn default_write_pi<E: Endianness, B: BitWrite<E>>(
    backend: &mut B,
    mut n: u64,
    k: usize,
) -> Result<usize, B::Error> {
    debug_assert!(k < 64);
    debug_assert!(n < u64::MAX);

    n += 1;
    let λ = n.ilog2() as usize;

    #[cfg(feature = "checks")]
    {
        // Clean up n in case checks are enabled
        n ^= 1 << λ;
    }

    Ok(backend.write_rice(λ as u64, k)? + backend.write_bits(n, λ)?)
}

#[cfg(test)]
mod tests {
    use crate::{
        codes::pi::{PiReadParam, PiWriteParam},
        prelude::*,
    };

    #[test]
    fn test_roundtrip() {
        let k = 3;
        for value in (0..64).map(|i| 1 << i).chain(0..1024).chain([u64::MAX - 1]) {
            let mut data = [0_u64; 10];
            let mut writer = <BufBitWriter<BE, _>>::new(MemWordWriterSlice::new(&mut data));
            let code_len = writer.write_pi(value, k).unwrap();
            assert_eq!(code_len, len_pi(value, k));
            drop(writer);
            let mut reader = <BufBitReader<BE, _>>::new(MemWordReader::new(&data));
            assert_eq!(
                reader.read_pi(k).unwrap(),
                value,
                "for value: {} with k {}",
                value,
                k
            );
        }
    }

    #[test]
    fn test_roundtrip_pi2() {
        // Test the specialized pi2 methods
        for value in (0..64).map(|i| 1 << i).chain(0..1024).chain([u64::MAX - 1]) {
            // Test BE
            let mut data = [0_u64; 10];
            let mut writer = <BufBitWriter<BE, _>>::new(MemWordWriterSlice::new(&mut data));
            let code_len = writer.write_pi2(value).unwrap();
            assert_eq!(code_len, len_pi(value, 2));
            drop(writer);
            let mut reader = <BufBitReader<BE, _>>::new(MemWordReader::new(&data));
            assert_eq!(reader.read_pi2().unwrap(), value, "BE for value: {}", value,);

            // Test LE
            let mut data = [0_u64; 10];
            let mut writer = <BufBitWriter<LE, _>>::new(MemWordWriterSlice::new(&mut data));
            let code_len = writer.write_pi2(value).unwrap();
            assert_eq!(code_len, len_pi(value, 2));
            drop(writer);
            let mut reader = <BufBitReader<LE, _>>::new(MemWordReader::new(&data));
            assert_eq!(reader.read_pi2().unwrap(), value, "LE for value: {}", value,);
        }
    }

    #[test]
    fn test_roundtrip_pi2_param() {
        // Test the parametric pi2 methods with tables
        for value in (0..64).map(|i| 1 << i).chain(0..1024).chain([u64::MAX - 1]) {
            // Test BE with tables
            let mut data = [0_u64; 10];
            let mut writer = <BufBitWriter<BE, _>>::new(MemWordWriterSlice::new(&mut data));
            let code_len = writer.write_pi2_param::<true>(value).unwrap();
            assert_eq!(code_len, len_pi(value, 2));
            drop(writer);
            let mut reader = <BufBitReader<BE, _>>::new(MemWordReader::new(&data));
            assert_eq!(
                reader.read_pi2_param::<true>().unwrap(),
                value,
                "BE table for value: {}",
                value,
            );

            // Test BE without tables
            let mut data = [0_u64; 10];
            let mut writer = <BufBitWriter<BE, _>>::new(MemWordWriterSlice::new(&mut data));
            let code_len = writer.write_pi2_param::<false>(value).unwrap();
            assert_eq!(code_len, len_pi(value, 2));
            drop(writer);
            let mut reader = <BufBitReader<BE, _>>::new(MemWordReader::new(&data));
            assert_eq!(
                reader.read_pi2_param::<false>().unwrap(),
                value,
                "BE no table for value: {}",
                value,
            );

            // Test LE with tables
            let mut data = [0_u64; 10];
            let mut writer = <BufBitWriter<LE, _>>::new(MemWordWriterSlice::new(&mut data));
            let code_len = writer.write_pi2_param::<true>(value).unwrap();
            assert_eq!(code_len, len_pi(value, 2));
            drop(writer);
            let mut reader = <BufBitReader<LE, _>>::new(MemWordReader::new(&data));
            assert_eq!(
                reader.read_pi2_param::<true>().unwrap(),
                value,
                "LE table for value: {}",
                value,
            );

            // Test LE without tables
            let mut data = [0_u64; 10];
            let mut writer = <BufBitWriter<LE, _>>::new(MemWordWriterSlice::new(&mut data));
            let code_len = writer.write_pi2_param::<false>(value).unwrap();
            assert_eq!(code_len, len_pi(value, 2));
            drop(writer);
            let mut reader = <BufBitReader<LE, _>>::new(MemWordReader::new(&data));
            assert_eq!(
                reader.read_pi2_param::<false>().unwrap(),
                value,
                "LE no table for value: {}",
                value,
            );
        }
    }

    #[test]
    #[allow(clippy::unusual_byte_groupings)]
    fn test_bits() {
        for (k, value, expected) in [
            (2, 20, 0b01_00_0101 << (64 - 8)),
            (2, 0, 0b100 << (64 - 3)),
            (2, 1, 0b1010 << (64 - 4)),
            (2, 2, 0b1011 << (64 - 4)),
            (2, 3, 0b1_1000 << (64 - 5)),
            (2, 4, 0b1_1001 << (64 - 5)),
            (2, 5, 0b1_1010 << (64 - 5)),
            (2, 6, 0b1_1011 << (64 - 5)),
            (2, 7, 0b11_1000 << (64 - 6)),
            (3, 0, 0b1000 << (64 - 4)),
            (3, 1, 0b1_0010 << (64 - 5)),
            (3, 2, 0b1_0011 << (64 - 5)),
            (3, 3, 0b1_01000 << (64 - 6)),
            (3, 4, 0b1_01001 << (64 - 6)),
            (3, 5, 0b1_01010 << (64 - 6)),
            (3, 6, 0b1_01011 << (64 - 6)),
            (3, 7, 0b101_1000 << (64 - 7)),
        ] {
            let mut data = [0_u64; 10];
            let mut writer = <BufBitWriter<BE, _>>::new(MemWordWriterSlice::new(&mut data));
            let code_len = writer.write_pi(value, k).unwrap();
            assert_eq!(code_len, len_pi(value, k));
            drop(writer);
            assert_eq!(
                data[0].to_be(),
                expected,
                "\nfor value: {} with k {}\ngot: {:064b}\nexp: {:064b}\ngot_len: {} exp_len: {}\n",
                value,
                k,
                data[0].to_be(),
                expected,
                code_len,
                len_pi(value, k),
            );
        }
    }

    #[test]
    fn test_against_zeta() {
        // BE: π₀ = ζ₁ and π₁ = ζ₂
        for k in 0..2 {
            for value in 0..100 {
                let mut data_pi = [0_u64; 10];
                let mut data_zeta = [0_u64; 10];

                let mut writer = <BufBitWriter<BE, _>>::new(MemWordWriterSlice::new(&mut data_pi));
                let code_len = writer.write_pi(value, k).unwrap();
                assert_eq!(code_len, len_pi(value, k));
                drop(writer);

                let mut writer =
                    <BufBitWriter<BE, _>>::new(MemWordWriterSlice::new(&mut data_zeta));
                let code_len = writer.write_zeta(value, 1 << k).unwrap();
                assert_eq!(code_len, len_zeta(value, 1 << k));
                drop(writer);

                assert_eq!(data_pi[0], data_zeta[0]);
            }
        }

        // LE: π₀ = ζ₁; π₁ and ζ₂ have the same lengths but permuted bits
        for value in 0..100 {
            let mut data_pi = [0_u64; 10];
            let mut data_zeta = [0_u64; 10];

            let mut writer = <BufBitWriter<LE, _>>::new(MemWordWriterSlice::new(&mut data_pi));
            let code_len = writer.write_pi(value, 0).unwrap();
            assert_eq!(code_len, len_pi(value, 0));
            drop(writer);

            let mut writer = <BufBitWriter<LE, _>>::new(MemWordWriterSlice::new(&mut data_zeta));
            let code_len = writer.write_zeta(value, 1).unwrap();
            assert_eq!(code_len, len_zeta(value, 1));
            drop(writer);

            assert_eq!(data_pi[0], data_zeta[0]);
        }

        for value in 0..100 {
            assert_eq!(len_pi(value, 1), len_zeta(value, 2));
        }
    }
}

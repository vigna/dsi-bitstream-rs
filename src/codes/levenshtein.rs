/*
 * SPDX-FileCopyrightText: 2024 Tommaso Fontana
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Levenshtein's code.
//!
//! # References
//!

use crate::traits::*;

/// Returns the length of the Levenshtein code for `n`.
#[inline(always)]
pub fn len_levenshtein(n: u64) -> usize {
    if n == 0 {
        return 1;
    }
    recursive_len(1, n)
}

fn recursive_len(blocks: usize, n: u64) -> usize {
    if n == 1 {
        return blocks + 1;
    }
    let λ = n.ilog2();
    recursive_len(blocks + 1, λ as u64) + λ as usize
}

/// Trait for reading ω codes.
///
/// This is the trait you should pull in scope to read ω codes.
pub trait LevenshteinRead<E: Endianness>: BitRead<E> {
    // Levenshtein codes are indexed from 1
    fn read_levenshtein(&mut self) -> Result<u64, Self::Error> {
        let λ = self.read_unary()?;
        if λ == 0 {
            return Ok(0);
        }
        let mut block_len = 0_u64;
        for _ in 0..λ {
            let block = self.read_bits(block_len as usize)?;
            block_len = (1 << block_len) | block;
        }

        Ok(block_len)
    }
}

/// Trait for writing ω codes.
///
/// This is the trait you should pull in scope to write ω codes.
pub trait LevenshteinWrite<E: Endianness>: BitWrite<E> {
    fn write_levenshtein(&mut self, n: u64) -> Result<usize, Self::Error> {
        if n == 0 {
            return self.write_bits(1, 1);
        }
        recursive_write::<E, Self>(self, 1, n)
    }
}

fn recursive_write<E: Endianness, B: BitWrite<E> + ?Sized>(
    writer: &mut B,
    blocks: usize,
    n: u64,
) -> Result<usize, B::Error> {
    if n == 1 {
        return writer.write_unary(blocks as u64);
    }
    let λ = n.ilog2() as usize;
    Ok(recursive_write(writer, blocks + 1, λ as u64)? + writer.write_bits(n, λ)?)
}

impl<E: Endianness, B: BitRead<E>> LevenshteinRead<E> for B {}
impl<E: Endianness, B: BitWrite<E>> LevenshteinWrite<E> for B {}

#[cfg(test)]
mod test {
    use crate::prelude::*;

    #[test]
    fn test_roundtrip() {
        for value in (0..64).map(|i| 1 << i).chain(0..1024).chain([u64::MAX]) {
            let mut data = vec![0_u64];
            let mut writer = <BufBitWriter<BE, _>>::new(MemWordWriterVec::new(&mut data));
            let code_len = writer.write_levenshtein(value).unwrap();
            assert_eq!(code_len, len_levenshtein(value));
            drop(writer);
            let mut reader = <BufBitReader<BE, _>>::new(MemWordReader::new(&data));
            assert_eq!(reader.read_levenshtein().unwrap(), value);

            let mut writer = <BufBitWriter<LE, _>>::new(MemWordWriterVec::new(&mut data));
            let code_len = writer.write_levenshtein(value).unwrap();
            assert_eq!(code_len, len_levenshtein(value));
            drop(writer);
            let mut reader = <BufBitReader<LE, _>>::new(MemWordReader::new(&data));
            assert_eq!(reader.read_levenshtein().unwrap(), value,);
        }
    }

    #[test]
    fn test_bits() {
        for (value, len, expected_be, expected_le) in [
            (0, 1, 1 << 63, 1),
            (1, 2, 0b01 << (64 - 2), 0b10),
            (2, 4, 0b001_0 << (64 - 4), 0b0_100),
            (3, 4, 0b001_1 << (64 - 4), 0b1_100),
            (4, 7, 0b0001_0_00 << (64 - 7), 0b_00_0_1000),
            (5, 7, 0b0001_0_01 << (64 - 7), 0b_01_0_1000),
            (6, 7, 0b0001_0_10 << (64 - 7), 0b_10_0_1000),
            (7, 7, 0b0001_0_11 << (64 - 7), 0b_11_0_1000),
            (15, 8, 0b0001_1_111 << (64 - 8), 0b111_1_1000),
            (
                99,
                14,
                0b00001_0_10_100011 << (64 - 14),
                0b100011_10_0_10000,
            ),
            (
                999,
                18,
                0b00001_1_001_111100111 << (64 - 18),
                0b111100111_001_1_10000,
            ),
            (
                999_999,
                32,
                0b000001_0_00_0011_1110100001000111111 << (64 - 32),
                0b1110100001000111111_0011_00_0_100000,
            ),
        ] {
            assert_eq!(len_levenshtein(value), len);

            let mut data = vec![0_u64];
            let mut writer = <BufBitWriter<BE, _>>::new(MemWordWriterVec::new(&mut data));
            assert_eq!(writer.write_levenshtein(value).unwrap(), len);

            drop(writer);
            assert_eq!(
                data[0].to_be(),
                expected_be,
                "\nfor value: {}\ngot: {:064b}\nexp: {:064b}\n",
                value,
                data[0].to_be(),
                expected_be,
            );

            let mut data = vec![0_u64];
            let mut writer = <BufBitWriter<LE, _>>::new(MemWordWriterVec::new(&mut data));
            assert_eq!(writer.write_levenshtein(value).unwrap(), len);
            drop(writer);
            assert_eq!(
                data[0].to_le(),
                expected_le,
                "\nfor value: {}\ngot: {:064b}\nexp: {:064b}\n",
                value,
                data[0].to_le(),
                expected_le,
            );
        }
    }
}

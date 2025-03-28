/*
 * SPDX-FileCopyrightText: 2024 Tommaso Fontana
 * SPDX-FileCopyrightText: 2025 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Elias ω code.
//!
//! Elias [γ](super::gamma) and [δ](super::delta) codes encode a number *n* by
//! storing the binary representation of *n* + 1, with the most significant bit
//! removed, prefixed by its length in unary or [γ](super::gamma) code,
//! respectively. Thus, [δ](super::delta) can be seen as adding one level of
//! recursion in the length representation with respect to [γ](super::gamma).
//! The ω code encodes the length of the binary representation of *n* + 1
//! recursively.
//!
//! The implied distribution for the ω code is difficult to write analytically,
//! but essentially it is as close as possible to ≈ 1/*x* (as there is no code
//! for that distribution).
//!
//! The ω code is easier to describe the format of a code, rather than the
//! encoding algorithm.
//!
//! A codeword is given by the concatenation of blocks *b*₀ *b*₁ …  *b*ₙ `0`,
//! where each block *b*ᵢ is a binary string starting with `1` and *b*₀ = `10`
//! or `11`. One can interpret the highest bit of each block as a continuation
//! bit, and the last `0` as a terminator of the code.
//!
//! The condition for a valid codeword is that the value represented by each
//! block, incremented by one, is the length of the following block, except for
//! the last block.
//!
//! The value associated with a codeword is 0 if the code is `0`, and otherwise
//! the value of the last block, decremented by one.
//!
//! For example, `1110010`, which is formed by the blocks `11`, `1011`, and `0`,
//! represents 10.
//!
//! As discussed in the [codes module documentation](crate::codes), to make the
//! code readable in the little-endian case, rather than reversing the bits of
//! the blocks, which would be expensive, we simply rotate by one on the left
//! each block, with the result that the most significant bit of the block is
//! now the first bit in the stream, making it possible to check for the
//! presence of a continuation bit. For example, in the little-endian case, the
//! code for 10 is `0011111`, which is formed by the blocks `11`, `0111`, and
//! `0`.
//!
//! # References
//!
//! Peter Elias. “[Universal codeword sets and representations of the
//! integers](https://doi.org/10.1109/TIT.1975.1055349)”. IEEE Transactions on
//! Information Theory, 21(2):194−203, March 1975.

use crate::traits::*;
use common_traits::CastableInto;

/// Returns the length of the ω code for `n`.
#[inline(always)]
pub fn len_omega(n: u64) -> usize {
    recursive_len(n + 1)
}

fn recursive_len(n: u64) -> usize {
    if n <= 1 {
        return 1;
    }
    let λ = n.ilog2() as u64;
    recursive_len(λ) + λ as usize + 1
}

/// Trait for reading ω codes.
///
/// This is the trait you should pull in scope to read ω codes.
pub trait OmegaRead<E: Endianness>: BitRead<E> {
    #[inline(always)]
    fn read_omega(&mut self) -> Result<u64, Self::Error> {
        let mut n = 1;
        loop {
            let bit = self.peek_bits(1)?.cast();
            if bit == 0 {
                self.skip_bits_after_peek(1);
                return Ok(n - 1);
            }

            let λ = n;
            n = self.read_bits(λ as usize + 1)?;

            if core::any::TypeId::of::<E>() == core::any::TypeId::of::<LE>() {
                // Little-endian case: rotate right the lower λ + 1 bits (the
                // lowest bit is a one) to reverse the rotation performed when
                // writing
                n = (n >> 1) | (1 << λ);
            }
        }
    }
}

/// Trait for writing ω codes.
///
/// This is the trait you should pull in scope to write ω codes.
pub trait OmegaWrite<E: Endianness>: BitWrite<E> {
    #[inline(always)]
    fn write_omega(&mut self, n: u64) -> Result<usize, Self::Error> {
        // omega codes are indexed from 1
        Ok(recursive_write::<E, Self>(n + 1, self)? + self.write_bits(0, 1)?)
    }
}

fn recursive_write<E: Endianness, B: BitWrite<E> + ?Sized>(
    mut n: u64,
    writer: &mut B,
) -> Result<usize, B::Error> {
    if n <= 1 {
        return Ok(0);
    }
    let λ = n.ilog2();

    if core::any::TypeId::of::<E>() == core::any::TypeId::of::<LE>() {
        // Little-endian case: rotate left the lower λ + 1 bits (the bit in
        // position λ is a one) so that the lowest bit can be peeked to find the
        // block.
        n = (n << 1) | 1;
        #[cfg(feature = "checks")]
        {
            // Clean up n in case checks are enabled
            n &= u64::MAX >> (u64::BITS - 1 - λ);
        }
    }
    Ok(recursive_write(λ as u64, writer)? + writer.write_bits(n, λ as usize + 1)?)
}

impl<E: Endianness, B: BitRead<E>> OmegaRead<E> for B {}
impl<E: Endianness, B: BitWrite<E>> OmegaWrite<E> for B {}

#[cfg(test)]
mod test {
    use crate::prelude::*;

    #[test]
    fn test_roundtrip() {
        for value in (0..64).map(|i| 1 << i).chain(0..1024).chain([u64::MAX - 1]) {
            let mut data = vec![0_u64];
            let mut writer = <BufBitWriter<BE, _>>::new(MemWordWriterVec::new(&mut data));
            let code_len = writer.write_omega(value).unwrap();
            assert_eq!(code_len, len_omega(value));
            drop(writer);
            let mut reader = <BufBitReader<BE, _>>::new(MemWordReader::new(&data));
            assert_eq!(reader.read_omega().unwrap(), value);

            let mut writer = <BufBitWriter<LE, _>>::new(MemWordWriterVec::new(&mut data));
            let code_len = writer.write_omega(value).unwrap();
            assert_eq!(code_len, len_omega(value));
            drop(writer);
            let mut reader = <BufBitReader<LE, _>>::new(MemWordReader::new(&data));
            assert_eq!(reader.read_omega().unwrap(), value,);
        }
    }

    #[test]
    #[allow(clippy::unusual_byte_groupings)]
    fn test_omega() {
        for (value, expected_be, expected_le) in [
            (0, 0, 0),
            (1, 0b10_0 << (64 - 3), 0b0_01),
            (2, 0b11_0 << (64 - 3), 0b0_11),
            (3, 0b10_100_0 << (64 - 6), 0b0_001_01),
            (4, 0b10_101_0 << (64 - 6), 0b0_011_01),
            (5, 0b10_110_0 << (64 - 6), 0b0_101_01),
            (6, 0b10_111_0 << (64 - 6), 0b0_111_01),
            (7, 0b11_1000_0 << (64 - 7), 0b0_0001_11),
            (15, 0b10_100_10000_0 << (64 - 11), 0b0_00001_001_01),
            (99, 0b10_110_1100100_0 << (64 - 13), 0b0_1001001_101_01),
            (
                999,
                0b11_1001_1111101000_0 << (64 - 17),
                0b0_1111010001_0011_11,
            ),
            (
                999_999,
                0b10_100_10011_11110100001001000000_0 << (64 - 31),
                0b0_11101000010010000001_00111_001_01,
            ),
        ] {
            let mut data = vec![0_u64];
            let mut writer = <BufBitWriter<BE, _>>::new(MemWordWriterVec::new(&mut data));
            writer.write_omega(value).unwrap();
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
            writer.write_omega(value).unwrap();
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

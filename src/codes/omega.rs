/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Elias ω code.
//!
//! Like Elias gamma coding and Elias delta coding, it works by prefixing the
//! positive integer with a representation of its order of magnitude in a
//! universal code. Unlike those other two codes, however, Elias omega
//! recursively encodes that prefix; thus, they are sometimes known as
//! recursive Elias codes.
//!
//! The ω code is decodable in BigEndian order only. This is because the read
//! algorithm requires to peek at the MSB bit of the next value, which is not
//! possible in LittleEndian order. To write in LittleEndian order, we rotate by
//! one the bits to the left, so that the MSB bit is the LSB bit, and we can peek
//! at it.

use crate::traits::*;
use common_traits::CastableInto;

fn ceil_log(n: u64) -> u64 {
    n.ilog2() as u64 + (!n.is_power_of_two()) as u64
}

/// Return the length of the ω code for `n`.
#[inline(always)]
pub fn len_omega(n: u64) -> usize {
    // omega codes are indexed from 1
    recursive_len(n + 1)
}

fn recursive_len(n: u64) -> usize {
    if n <= 1 {
        return 1;
    }
    let l = ceil_log(n.saturating_add(1));
    recursive_len(l - 1) + l as usize
}

/// Trait for reading ω codes.
///
/// This is the trait you should usually pull in scope to read ω codes.
pub trait OmegaRead<E: Endianness>: BitRead<E> {
    // omega codes are indexed from 1
    fn read_omega(&mut self) -> Result<u64, Self::Error> {
        let mut n = 1;
        loop {
            let bit = self.peek_bits(1)?.cast();
            if bit == 0 {
                self.skip_bits(1)?;
                return Ok(n - 1);
            }

            let old_n = n;
            n = self.read_bits(1 + n as usize)?;

            if core::any::TypeId::of::<E>() == core::any::TypeId::of::<LE>() {
                n = (n >> 1) | (1 << old_n);
            }
        }
    }
}

/// Trait for writing ω codes.
///
/// This is the trait you should usually pull in scope to write ω codes.
pub trait OmegaWrite<E: Endianness>: BitWrite<E> {
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
    let l = ceil_log(n.saturating_add(1));
    if core::any::TypeId::of::<E>() == core::any::TypeId::of::<LE>() {
        // move the front 1 to the back so we can peek it
        n = (n << 1) | 1;
        // clean the highest 1
        n &= u64::MAX >> (64 - l);
    }
    Ok(recursive_write(l - 1, writer)? + writer.write_bits(n, l as usize)?)
}

impl<E: Endianness, B: BitRead<E>> OmegaRead<E> for B {}
impl<E: Endianness, B: BitWrite<E>> OmegaWrite<E> for B {}

#[cfg(test)]
mod test {
    use crate::prelude::*;

    #[test]
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
            println!("");

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
            println!("");
        }
    }
}

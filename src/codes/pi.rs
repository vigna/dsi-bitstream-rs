/*
 * SPDX-FileCopyrightText: 2024 Tommaso Fontana
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! π codes
//!
//! π codes allow efficient encoding of Zipf distributions that have an
//! exponent even closer to 1 compared to [ζ codes](crate::codes:zeta}
//!
//! The intended distribution is:
//! Θ( 1 / N^(1 + 2^-k) )
//! so it's optimal for Zipf distributions of exponent α ≈ 1 + 2^−k
//!
//! π web codes are a modified version of π-codes in which 0 is encoded
//! by 1 and any other positive integer n is encoded with a 0 followed by
//! the π-code of n.
//!
//! ## Reference
//! Alberto Apostolico and Guido Drovandi.
//! "Graph Compression by BFS,"
//! Algorithms 2009, 2, 1031-1044; <https://doi.org/10.3390/a2031031>.

use crate::traits::*;

/// Returns the length of the π code for `n`.
///
/// ```rust
/// use dsi_bitstream::codes::len_pi;
///
/// // k = 0
/// assert_eq!(len_pi(0, 0), 1, "π_0(0)");
/// assert_eq!(len_pi(1, 0), 3, "π_0(1)");
/// assert_eq!(len_pi(2, 0), 3, "π_0(2)");
/// assert_eq!(len_pi(3, 0), 5, "π_0(3)");
/// assert_eq!(len_pi(4, 0), 5, "π_0(4)");
/// assert_eq!(len_pi(5, 0), 5, "π_0(5)");
/// assert_eq!(len_pi(6, 0), 5, "π_0(6)");
/// assert_eq!(len_pi(7, 0), 7, "π_0(7)");
///
/// // k = 1
/// assert_eq!(len_pi(0, 1), 2, "π_1(0)");
/// assert_eq!(len_pi(1, 1), 3, "π_1(1)");
/// assert_eq!(len_pi(2, 1), 3, "π_1(2)");
/// assert_eq!(len_pi(3, 1), 5, "π_1(3)");
/// assert_eq!(len_pi(4, 1), 5, "π_1(4)");
/// assert_eq!(len_pi(5, 1), 5, "π_1(5)");
/// assert_eq!(len_pi(6, 1), 5, "π_1(6)");
/// assert_eq!(len_pi(7, 1), 6, "π_1(7)");
///
/// // k = 2
/// assert_eq!(len_pi(0, 2), 3, "π_2(0)");
/// assert_eq!(len_pi(1, 2), 4, "π_2(1)");
/// assert_eq!(len_pi(2, 2), 4, "π_2(2)");
/// assert_eq!(len_pi(3, 2), 5, "π_2(3)");
/// assert_eq!(len_pi(4, 2), 5, "π_2(4)");
/// assert_eq!(len_pi(5, 2), 5, "π_2(5)");
/// assert_eq!(len_pi(6, 2), 5, "π_2(6)");
/// assert_eq!(len_pi(7, 2), 6, "π_2(7)");
///
/// // k = 3
/// assert_eq!(len_pi(0, 3), 4, "π_3(0)");
/// assert_eq!(len_pi(1, 3), 5, "π_3(1)");
/// assert_eq!(len_pi(2, 3), 5, "π_3(2)");
/// assert_eq!(len_pi(3, 3), 6, "π_3(3)");
/// assert_eq!(len_pi(4, 3), 6, "π_3(4)");
/// assert_eq!(len_pi(5, 3), 6, "π_3(5)");
/// assert_eq!(len_pi(6, 3), 6, "π_3(6)");
/// assert_eq!(len_pi(7, 3), 7, "π_3(7)");
/// ```
#[must_use]
#[inline]
pub fn len_pi(mut n: u64, k: u64) -> usize {
    n += 1; // π codes are indexed from 1
    let rem = n.ilog2() as usize;
    let h = 1 + rem;
    let l = h.div_ceil(1 << k);
    k as usize + l + rem
}

/// Trait for reading π codes.
///
/// This is the trait you should usually pull in scope to read π codes.
pub trait PiRead<E: Endianness>: BitRead<E> {
    #[inline]
    fn read_pi(&mut self, k: u64) -> Result<u64, Self::Error> {
        let l = self.read_unary()? + 1;
        let v = self.read_bits(k as usize)?;
        let h = l * (1 << k) - v;
        let r = h - 1;
        let rem = self.read_bits(r as usize)?;
        Ok((1 << r) + rem - 1)
    }
}

/// Trait for writing π codes.
///
/// This is the trait you should usually pull in scope to write π codes.
pub trait PiWrite<E: Endianness>: BitWrite<E> {
    #[inline]
    fn write_pi(&mut self, mut n: u64, k: u64) -> Result<usize, Self::Error> {
        n += 1; // π codes are indexed from 1
        let r = n.ilog2() as usize;
        let h = 1 + r;
        let l = h.div_ceil(1 << k);
        let v = (l * (1 << k) - h) as u64;
        let rem = n & !(u64::MAX << r);

        let mut written_bits = 0;
        written_bits += self.write_unary((l - 1) as u64)?;
        written_bits += self.write_bits(v, k as usize)?;
        written_bits += self.write_bits(rem, r)?;

        Ok(written_bits)
    }
}

impl<E: Endianness, B: BitRead<E> + ?Sized> PiRead<E> for B {}
impl<E: Endianness, B: BitWrite<E> + ?Sized> PiWrite<E> for B {}

#[cfg(test)]
mod test {
    use crate::codes::PiWrite;
    use crate::prelude::*;
    #[test]
    fn test_pi_roundtrip() {
        let k = 3;
        for value in 0..1_000_000 {
            let mut data = vec![0_u64];
            let mut writer = <BufBitWriter<BE, _>>::new(MemWordWriterVec::new(&mut data));
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
    fn test_pi() {
        for (k, value, expected) in [
            (2, 20, 0b01_11_0101 << (64 - 8)),
            (2, 0, 0b111 << (64 - 3)),
            (2, 1, 0b1100 << (64 - 4)),
            (2, 2, 0b1101 << (64 - 4)),
            (2, 3, 0b1_0100 << (64 - 5)),
            (2, 4, 0b1_0101 << (64 - 5)),
            (2, 5, 0b1_0110 << (64 - 5)),
            (2, 6, 0b1_0111 << (64 - 5)),
            (2, 7, 0b10_0000 << (64 - 6)),
            (3, 0, 0b1111 << (64 - 4)),
            (3, 1, 0b1_1100 << (64 - 5)),
            (3, 2, 0b1_1101 << (64 - 5)),
            (3, 3, 0b11_0100 << (64 - 6)),
            (3, 4, 0b11_0101 << (64 - 6)),
            (3, 5, 0b11_0110 << (64 - 6)),
            (3, 6, 0b11_0111 << (64 - 6)),
            (3, 7, 0b110_0000 << (64 - 7)),
        ] {
            let mut data = vec![0_u64];
            let mut writer = <BufBitWriter<BE, _>>::new(MemWordWriterVec::new(&mut data));
            let code_len = writer.write_pi(value, k).unwrap();
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
}

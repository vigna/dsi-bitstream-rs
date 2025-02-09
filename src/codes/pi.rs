/*
 * SPDX-FileCopyrightText: 2024 Tommaso Fontana
 * SPDX-FileCopyrightText: 2025 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Streamlined Apostolico–Drovandi π codes
//!
//! The streamlined π code with parameter *k* of a natural number *n* is the
//! concatenation of the [Rice code](super::rice) with parameter *k* of
//! ⌊log₂(*n* + 1)⌋ and of the binary representation of *n* + 1 with the most
//! significant bit removed.
//!
//! The implied distribution of a π code with parameter *k* code is ≈ 1/*x*^(1 +
//! 1/2*ᵏ*).
//!
//! Note that π₀ = [ζ₁](super::zeta) = [γ](super::gamma) and π₁ =
//! [ζ₂](super::zeta).
//!
//! In the original paper the definition of the code is very convoluted, as the
//! authors appear to have missed the connection with [Rice codes](super::rice).
//! The codewords implemented by this module are equivalent to the ones in the
//! paper, in the sense that corresponding codewords have the same length, but
//! the codewords for *k* ≥ 2 are different, and encoding/decoding is
//! faster—hence the name “streamlined π codes”.
//!
//! # References
//!
//! Alberto Apostolico and Guido Drovandi. “[Graph Compression by
//! BFS](https://doi.org/10.3390/a2031031)”, Algorithms, 2:1031-1044, 2009.

use crate::traits::*;

use super::{len_rice, RiceRead, RiceWrite};

/// Return the length of the π code for `n` with parameter `k`.
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
pub fn len_pi(mut n: u64, k: usize) -> usize {
    n += 1;
    let λ = n.ilog2() as usize;
    len_rice(λ as u64, k) + λ
}

/// Trait for reading π codes.
///
/// This is the trait you should pull in scope to read π codes.
pub trait PiRead<E: Endianness>: BitRead<E> + RiceRead<E> {
    #[inline(always)]
    fn read_pi(&mut self, k: usize) -> Result<u64, Self::Error> {
        let λ = self.read_rice(k)?;
        Ok((1 << λ) + self.read_bits(λ as usize)? - 1)
    }
}

/// Trait for writing π codes.
///
/// This is the trait you should pull in scope to write π codes.
pub trait PiWrite<E: Endianness>: BitWrite<E> + RiceWrite<E> {
    #[inline(always)]
    fn write_pi(&mut self, mut n: u64, k: usize) -> Result<usize, Self::Error> {
        n += 1;
        let λ = n.ilog2() as usize;

        #[cfg(feature = "checks")]
        {
            // Clean up n in case checks are enabled
            n ^= 1 << λ;
        }

        Ok(self.write_rice(λ as u64, k)? + self.write_bits(n, λ)?)
    }
}

impl<E: Endianness, B: BitRead<E> + RiceRead<E> + ?Sized> PiRead<E> for B {}
impl<E: Endianness, B: BitWrite<E> + RiceWrite<E> + ?Sized> PiWrite<E> for B {}

#[cfg(test)]
mod test {
    use crate::prelude::*;

    #[test]
    fn test_roundtrip() {
        let k = 3;
        for value in (0..64).map(|i| 1 << i).chain(0..1024).chain([u64::MAX - 1]) {
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
            let mut data = vec![0_u64];
            let mut writer = <BufBitWriter<BE, _>>::new(MemWordWriterVec::new(&mut data));
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
}

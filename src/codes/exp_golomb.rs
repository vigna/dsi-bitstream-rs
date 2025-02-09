/*
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Exponential Golomb codes.
//!
//! Exponential Golomb codes are a variant of Golomb codes with power-of-two
//! modulus (i.e., [Rice codes](super::rice)) in which the prefix is written
//! using [Elias γ code](super::gamma) instead of unary code. More precisely,
//! the exponential Golomb code with parameter *k* of a natural number *n* is
//! given ⌊*x* / 2*ᵏ*⌋ in [γ code](super::gamma) followed by *x* mod 2*ᵏ* in
//! binary *k*-bit representations.
//!
//! The implied distribution of an exponential Golomb code with parameter *k* is
//! ∝ 1/2(*x*/2*ᵏ*)².
//!
//! Note that the exponential Golomb code for *k* = 0 is exactly the [γ
//! code](super::gamma).
//!
//! Exponential Golomb codes are used in [H.264
//! (MPEG-4)](https://en.wikipedia.org/wiki/Advanced_Video_Coding) and
//! [H.265](https://en.wikipedia.org/wiki/High_Efficiency_Video_Coding).

use super::gamma::{len_gamma, GammaRead, GammaWrite};
use crate::traits::*;

/// Returns the length of the exponential Golomb code for `n` with parameter `k`.
#[must_use]
#[inline]
pub fn len_exp_golomb(n: u64, k: usize) -> usize {
    len_gamma(n >> k) + k
}

/// Trait for reading exponential Golomb codes.
pub trait ExpGolombRead<E: Endianness>: BitRead<E> + GammaRead<E> {
    #[inline]
    fn read_exp_golomb(&mut self, k: usize) -> Result<u64, Self::Error> {
        Ok((self.read_gamma()? << k) + self.read_bits(k)?)
    }
}

/// Trait for writing exponential Golomb codes.
pub trait ExpGolombWrite<E: Endianness>: BitWrite<E> + GammaWrite<E> {
    #[inline]
    fn write_exp_golomb(&mut self, n: u64, k: usize) -> Result<usize, Self::Error> {
        let mut written_bits = self.write_gamma(n >> k)?;
        #[cfg(feature = "checks")]
        {
            // Clean up n in case checks are enabled
            let n = n & (1_u128 << k).wrapping_sub(1) as u64;
            written_bits += self.write_bits(n, k)?;
        }
        #[cfg(not(feature = "checks"))]
        {
            written_bits += self.write_bits(n, k)?;
        }
        Ok(written_bits)
    }
}

impl<E: Endianness, B: BitRead<E> + GammaRead<E>> ExpGolombRead<E> for B {}
impl<E: Endianness, B: BitWrite<E> + GammaWrite<E>> ExpGolombWrite<E> for B {}

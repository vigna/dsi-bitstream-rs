/*
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Golomb codes.
//!
//! Given a modulo *b*, the Golomb code of a natural number *x* is given by ⌊*x*
//! / *b*⌋ in [unary code](BitRead::read_unary) followed by the [minimal binary
//! code](super::minimal_binary) of *x* mod *b*.
//!
//! For natural numbers distributed with a geometric distribution with base *p*,
//! the optimal code is a Golomb code with [*b* = ⌈–log(2 – *p*) / log(1 –
//! *p*)⌉](b).
//!
//! For a faster, less precise alternative, see [Rice codes](super::rice).
//!
//! # References
//!
//! S. Golomb, “Run-length encodings (Corresp.)”, IEEE Transactions on
//! Information Theory, vol. 12, no. 3, pp. 399-401, July 1966, doi:
//! <https://doi.org/10.1109/TIT.1966.1053907>.
//!
//! R. Gallager and D. van Voorhis, “Optimal source codes for geometrically
//! distributed integer alphabets (Corresp.)”, IEEE Transactions on Information
//! Theory, vol. 21, no. 2, pp. 228-230, March 1975, doi:
//! <https://doi.org/10.1109/TIT.1975.1055357>.

use super::minimal_binary::{len_minimal_binary, MinimalBinaryRead, MinimalBinaryWrite};
use crate::traits::*;

/// Returns the length of the Golomb code for `n` with modulo `b`.
#[must_use]
#[inline]
pub fn len_golomb(n: u64, b: u64) -> usize {
    (n / b) as usize + 1 + len_minimal_binary(n % b, b)
}

/// Returns the optimal value of *b* for a geometric distribution of base *p*,
/// that is, ⌈–log(2 – *p*) / log(1 – *p*)⌉.
pub fn b(p: f64) -> u64 {
    (-(2.0 - p).ln() / (1.0 - p).ln()).ceil() as u64
}

/// Trait for reading Golomb codes.
pub trait GolombRead<E: Endianness>: BitRead<E> + MinimalBinaryRead<E> {
    #[inline]
    fn read_golomb(&mut self, b: u64) -> Result<u64, Self::Error> {
        Ok(self.read_unary()? * b + self.read_minimal_binary(b)?)
    }
}

/// Trait for writing Golomb codes.
pub trait GolombWrite<E: Endianness>: BitWrite<E> + MinimalBinaryWrite<E> {
    #[inline]
    fn write_golomb(&mut self, n: u64, b: u64) -> Result<usize, Self::Error> {
        Ok(self.write_unary(n / b)? + self.write_minimal_binary(n % b, b)?)
    }
}

impl<E: Endianness, B: BitRead<E>> GolombRead<E> for B {}
impl<E: Endianness, B: BitWrite<E>> GolombWrite<E> for B {}

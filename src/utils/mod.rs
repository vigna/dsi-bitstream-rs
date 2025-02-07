/*
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

/*!

Debug helpers and statistics.

[`CountBitReader`] and [`CountBitWriter`] keep track of the number
of bits read or written to a [`BitRead`](crate::traits::BitRead)
and [`BitWrite`](crate::traits::BitWrite), respectively,
optionally printing on standard error the operations performed on the stream.

[`DbgBitReader`] and [`DbgBitWriter`] print on standard error all
operation beformed by a [`BitRead`](crate::traits::BitRead) or
[`BitWrite`](crate::traits::BitWrite).

[`CodesStats`] keeps track of the space needed to store a stream of
integers using different codes.

*/

mod count;
use common_traits::{AsBytes, SignedInt, UnsignedInt};
pub use count::*;

mod dbg_codes;
pub use dbg_codes::*;

pub mod stats;
pub use stats::{CodesStats, CodesStatsWrapper};

/// Extension trait mapping natural numbers bijectively to integers.
///
/// The method [`to_int`](#tymethod.to_int) will map a natural number `x` to `x
/// / 2` if `x` is even, and to `–(x + 1) / 2` if `x` is odd. The inverse
/// transformation is provided by the [`ToNat`] trait.
///
/// This pair of bijections makes it possible to use instantaneous codes for
/// signed integers by mapping them to natural numbers and back.
///
/// This bijection is best known as the “ZigZag” transformation in Google's
/// [Protocol Buffers](https://protobuf.dev/), albeit it has been used by
/// [WebGraph](http://webgraph.di.unimi.it/) since 2003, and much likely in
/// other software, for the same purpose. Note that the compression standards
/// H.264/H.265 uses a different transformation for exponential Golomb codes,
/// mapping a positive integer `x` to `2x – 1` and a zero or negative integer
/// `x` to `–2x`.
///
/// The implementation is just based on the traits [`UnsignedInt`] and
/// [`AsBytes`]. We provide blanket implementations for all primitive unsigned
/// integer types, but it can be used with any type implementing those traits.
pub trait ToInt: UnsignedInt + AsBytes {
    #[inline]
    fn to_int(self) -> Self::SignedInt {
        (self >> Self::ONE).to_signed() ^ (-(self & Self::ONE).to_signed())
    }
}

impl ToInt for u128 {}
impl ToInt for u64 {}
impl ToInt for u32 {}
impl ToInt for u16 {}
impl ToInt for u8 {}
impl ToInt for usize {}

/// Extension trait mapping signed integers bijectively to natural numbers.
///
/// The method [`to_nat`](#tymethod.to_nat) will map an nonnegative integer `x`
/// to `2x` and a negative integer `x` to `–2x – 1`. The inverse transformation
/// is provided by the [`ToInt`] trait.
///
/// This pair of bijections makes it possible to use instantaneous codes
/// for signed integers by mapping them to natural numbers and back.
///
/// This bijection is best known as the “ZigZag” transformation in Google's
/// [Protocol Buffers](https://protobuf.dev/), albeit it has been used by
/// [WebGraph](http://webgraph.di.unimi.it/) since 2003, and much likely in
/// other software, for the same purpose. Note that the compression standards
/// H.264/H.265 uses a different transformation for exponential Golomb codes,
/// mapping a positive integer `x` to `2x – 1` and a zero or negative integer
/// `x` to `–2x`.
///
/// The implementation is just based on the traits [`SignedInt`] and
/// [`AsBytes`]. We provide blanket implementations for all primitive signed
/// integer types, but it can be used with any type implementing those traits.
pub trait ToNat: SignedInt + AsBytes {
    #[inline]
    fn to_nat(self) -> Self::UnsignedInt {
        (self << Self::ONE).to_unsigned() ^ (self >> (Self::BITS - 1)).to_unsigned()
    }
}

impl ToNat for i128 {}
impl ToNat for i64 {}
impl ToNat for i32 {}
impl ToNat for i16 {}
impl ToNat for i8 {}
impl ToNat for isize {}

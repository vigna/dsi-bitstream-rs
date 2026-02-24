/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Traits for reading and writing instantaneous codes.
//!
//! This module contains code for reading and writing instantaneous codes.
//! Codewords are uniformly indexed from 0 for all codes. For example, the first
//! few words of [unary](crate::traits::BitRead::read_unary), [γ](gamma), and
//! [δ](delta) codes are:
//!
//! | Arg |  unary   |    γ    |     δ    |
//! |-----|---------:|--------:|---------:|
//! | 0   |        1 |       1 |        1 |
//! | 1   |       01 |     010 |     0100 |
//! | 2   |      001 |     011 |     0101 |
//! | 3   |     0001 |   00100 |    01100 |
//! | 4   |    00001 |   00101 |    01101 |
//! | 5   |   000001 |   00110 |    01110 |
//! | 6   |  0000001 |   00111 |    01111 |
//! | 7   | 00000001 | 0001000 | 00100000 |
//!
//! If you need to encode signed integers, please use the [`ToInt`] and
//! [`ToNat`] traits, which provide a bijection between signed integers and
//! natural numbers.
//!
//! Each code is implemented as a pair of traits for reading and writing (e.g.,
//! [`GammaReadParam`] and [`GammaWriteParam`]). The traits for reading depend
//! on [`BitRead`](crate::traits::BitRead), whereas the traits for writing
//! depend on [`BitWrite`](crate::traits::BitWrite). Note that most codes cannot
//! write the number [`u64::MAX`] because of overflow issues, which could be
//! avoided with tests, but at the price of a significant performance drop.
//!
//! The traits ending with `Param` make it possible to specify parameters—for
//! example, whether to use decoding tables. Usually, one would instead pull in
//! scope non-parametric traits such as [`GammaRead`] and [`GammaWrite`], for
//! which defaults are provided using the mechanism described in the [`params`]
//! module.
//!
//! # Big-endian vs. little-endian
//!
//! As discussed in the [traits module](crate::traits), in general reversing the
//! bits of a big-endian bit stream will not yield a little-endian bit stream
//! containing the same sequence of fixed-width integers. The same is true for
//! codes, albeit the situation is more complex.
//!
//! The only code that can be safely reversed is the unary code. All other codes
//! contain some value, and that value is written without reversing its bits.
//! Thus, reversing the bits of a big-endian bit stream containing a sequence of
//! instantaneous codes will not yield a little-endian bit stream containing the
//! same sequence of codes (again, with the exception of unary codes).
//! Technically, the codes written for the little-endian case are different than
//! those written for the big-endian case.
//!
//! For example, the [γ code](gamma) of 4 is `00101` in big-endian order, but it
//! is `01100` in little-endian order, so that upon reading the unary code for 2
//! we can read the `01` part without a bit reversal.
//!
//! The case of [minimal binary codes](minimal_binary) is even more convoluted:
//! for example, the code with upper bound 7 has codewords `00`, `010`, `011`,
//! `100`, `101`, `110`, and `111`. To decode such a code without peeking at
//! more bits than necessary, one first reads two bits, and then decides, based
//! on their value, whether to read a further bit and add it on the right. But
//! this means that we have to encode 2 as `011` in the big-endian case, and as
//! `101` in the little-endian case, because we need to read the first two bits
//! to decide whether to read the third one.
//!
//! In some cases, we resort to completely *ad hoc* solutions: for example, in
//! the case of the [ω code](omega), for the little-endian case instead of
//! reversing the bits written at each recursive call (which in principle would
//! be necessary), we simply rotate them to the left by one position, exposing
//! the most significant bit as first bit. This is sufficient to make the
//! decoding possible, and the rotation is a much faster operation than bit
//! reversal.
//!
//! # Dispatch
//!
//! The basic method for accessing codes is through traits like
//! [`GammaRead`] and [`GammaWrite`]. This approach, however, forces a choice of code in the
//! source. To pass a choice of code dynamically, please have a look at the
//! [`dispatch`](crate::dispatch) module.

use num_primitive::{PrimitiveSigned, PrimitiveUnsigned};
use num_traits::{AsPrimitive, ConstOne};

pub mod params;

pub mod gamma;
pub use gamma::{GammaRead, GammaWrite, len_gamma};

pub mod delta;
pub use delta::{DeltaRead, DeltaWrite, len_delta};

pub mod omega;
pub use omega::{OmegaRead, OmegaWrite, len_omega};

pub mod minimal_binary;
pub use minimal_binary::{MinimalBinaryRead, MinimalBinaryWrite, len_minimal_binary};

pub mod zeta;
pub use zeta::{ZetaRead, ZetaWrite, len_zeta};

pub mod pi;
pub use pi::{PiRead, PiWrite, len_pi};

pub mod golomb;
pub use golomb::{GolombRead, GolombWrite, len_golomb};

pub mod rice;
pub use rice::{RiceRead, RiceWrite, len_rice};

pub mod exp_golomb;
pub use exp_golomb::{ExpGolombRead, ExpGolombWrite, len_exp_golomb};

pub mod vbyte;
pub use vbyte::{
    VByteBeRead, VByteBeWrite, VByteLeRead, VByteLeWrite, bit_len_vbyte, byte_len_vbyte,
};
#[cfg(feature = "std")]
pub use vbyte::{
    vbyte_read, vbyte_read_be, vbyte_read_le, vbyte_write, vbyte_write_be, vbyte_write_le,
};

pub mod delta_tables;
pub mod gamma_tables;
pub mod omega_tables;
pub mod pi_tables;
pub mod zeta_tables;

/// Extension trait mapping natural numbers bijectively to integers.
///
/// The method [`to_int`](#method.to_int) will map a natural number `x` to `x
/// / 2` if `x` is even, and to `−(x + 1) / 2` if `x` is odd. The inverse
/// transformation is provided by the [`ToNat`] trait.
///
/// This pair of bijections makes it possible to use instantaneous codes for
/// signed integers by mapping them to natural numbers and back.
///
/// This bijection is best known as the “ZigZag” transformation in Google's
/// [Protocol Buffers](https://protobuf.dev/), albeit it has been used by
/// [WebGraph](http://webgraph.di.unimi.it/) since 2003, and most likely in
/// other software, for the same purpose. Note that the compression standards
/// H.264/H.265 use a different transformation for exponential Golomb codes,
/// mapping a positive integer `x` to `2x − 1` and a zero or negative integer
/// `x` to `−2x`.
///
/// The implementation uses a blanket implementation for all primitive
/// unsigned integer types.
pub trait ToInt {
    type Signed;
    #[must_use]
    fn to_int(self) -> Self::Signed;
}

impl<U: PrimitiveUnsigned + ConstOne + AsPrimitive<U::Signed>> ToInt for U
where
    U::Signed: PrimitiveSigned + Copy + 'static,
{
    type Signed = U::Signed;
    #[inline]
    fn to_int(self) -> U::Signed {
        (self >> 1u32).as_() ^ -((self & U::ONE).as_())
    }
}

/// Extension trait mapping signed integers bijectively to natural numbers.
///
/// The method [`to_nat`](#method.to_nat) will map a nonnegative integer `x`
/// to `2x` and a negative integer `x` to `−2x − 1`. The inverse transformation
/// is provided by the [`ToInt`] trait.
///
/// This pair of bijections makes it possible to use instantaneous codes
/// for signed integers by mapping them to natural numbers and back.
///
/// This bijection is best known as the “ZigZag” transformation in Google's
/// [Protocol Buffers](https://protobuf.dev/), albeit it has been used by
/// [WebGraph](http://webgraph.di.unimi.it/) since 2003, and most likely in
/// other software, for the same purpose. Note that the compression standards
/// H.264/H.265 use a different transformation for exponential Golomb codes,
/// mapping a positive integer `x` to `2x − 1` and a zero or negative integer
/// `x` to `−2x`.
///
/// The implementation uses a blanket implementation for all primitive
/// signed integer types.
pub trait ToNat {
    type Unsigned;
    #[must_use]
    fn to_nat(self) -> Self::Unsigned;
}

impl<S: PrimitiveSigned + AsPrimitive<S::Unsigned>> ToNat for S
where
    S::Unsigned: PrimitiveUnsigned + Copy + 'static,
{
    type Unsigned = S::Unsigned;
    #[inline]
    fn to_nat(self) -> S::Unsigned {
        (self << 1u32).as_() ^ (self >> (S::BITS - 1)).as_()
    }
}

/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Traits for reading and writing instantaneous codes.
//!
//! This modules contains code for reading and writing instantaneous codes.
//! Codewords are uniformely indexed from 0 for all codes. For example, the
//! first few words of [unary](crate::traits::BitRead::read_unary), [γ](gamma),
//! and [δ](delta) codes are:
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
//! If you need to encode signed integers, please use the
//! [`ToInt`](crate::utils::ToInt) and [`ToNat`](crate::utils::ToNat) traits,
//! which provide a bijection between signed integers and natural numbers.
//!
//! Each code is implemented as a pair of traits for reading and writing (e.g.,
//! [`GammaReadParam`] and [`GammaWriteParam`]). The traits for reading depend
//! on [`BitRead`], whereas the traits for writing depend on [`BitWrite`]. Note
//! that most codes cannot write the number [`u64::MAX`] because of overflow
//! issues, which could be avoided with tests, but at the price of a significant
//! performance drop.
//!
//! The traits ending with `Param` make it possible to specify parameters—for
//! example, whether to use decoding tables. Usually, one whould instead pull in
//! scope non-parametric traits such as [`GammaRead`] and [`GammaWrite`], for
//! which defaults are provided using the mechanism described in the [`params`]
//! module.
//!
//! Note that if you are using decoding tables, you must ensure that the
//! [`peek_bits`](crate::traits::BitRead::peek_bits) method of your [`BitRead`]
//! implementation returns a sufficient number of bits: if it does not, an
//! assertion will be triggered in test mode, but behavior will be unpredictable
//! otherwise. This is unfortunately difficult to check statically. To stay on
//! the safe side, we recommend to use a an implementation that is able to peek
//! at least at 16 bits.
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

use anyhow::Result;
#[cfg(feature = "mem_dbg")]
use mem_dbg::{MemDbg, MemSize};

use crate::prelude::{BitRead, BitWrite};
pub mod params;

pub mod dispatch;
pub use dispatch::{code_consts, Codes, ConstCode, FuncCodeReader, FuncCodeWriter};
pub use dispatch::{
    CodeLen, GenericCodeRead, GenericCodeWrite, SpecificCodeRead, SpecificCodeWrite,
};

pub mod gamma;
pub use gamma::{
    len_gamma, len_gamma_param, GammaRead, GammaReadParam, GammaWrite, GammaWriteParam,
};

pub mod delta;
pub use delta::{
    len_delta, len_delta_param, DeltaRead, DeltaReadParam, DeltaWrite, DeltaWriteParam,
};

pub mod omega;
pub use omega::{len_omega, OmegaRead, OmegaWrite};

pub mod minimal_binary;
pub use minimal_binary::{len_minimal_binary, MinimalBinaryRead, MinimalBinaryWrite};

pub mod zeta;
pub use zeta::{len_zeta, len_zeta_param, ZetaRead, ZetaReadParam, ZetaWrite, ZetaWriteParam};

pub mod pi;
pub use pi::{len_pi, PiRead, PiWrite};

pub mod golomb;
pub use golomb::{len_golomb, GolombRead, GolombWrite};

pub mod rice;
pub use rice::{len_rice, RiceRead, RiceWrite};

pub mod exp_golomb;
pub use exp_golomb::{len_exp_golomb, ExpGolombRead, ExpGolombWrite};

pub mod vbyte;
pub use vbyte::{bit_len_vbyte, VByteRead, VByteWrite};

use crate::prelude::Endianness;

pub mod delta_tables;
pub mod gamma_tables;
pub mod zeta_tables;

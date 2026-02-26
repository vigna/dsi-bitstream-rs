/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Mechanisms for selecting parameters.
//!
//! Traits and structures in this file are of no interest for the standard
//! user. Their purpose is to provide a systematic way, and in particular
//! a default way, to select parameters for parameterized traits
//! such as [`GammaReadParam`] and [`GammaWriteParam`].
//!
//! The traits and structure in this module work closely with the
//! bitstream readers and writers in [`impls`](crate::impls), which have an
//! additional type parameter `RP`/`WP` that must
//! implement marker traits [`ReadParams`] or [`WriteParams`], respectively.
//! The type is then used as a selector type to provide blanket implementations
//! of parameterless traits in [`codes`](crate::codes) such as [`GammaRead`],
//! [`GammaWrite`], [`DeltaRead`], [`DeltaWrite`], and so on.
//!
//! This module provides default selector types
//! [`DefaultReadParams`] and [`DefaultWriteParams`] which are also
//! the default value for the parameter `RP`/`WP` in the bitstream
//! readers and writers in [`crate::impls`]. Type-selected blanket
//! implementations of all parameterless traits in [`crate::codes`]
//! are provided for the bitstream readers and writers in
//! [`impls`](crate::impls). Thus, if you do not specify a value for
//! the parameter `RP`/`WP`, you will obtain automatically the
//! blanket implementations for parameterless traits contained in
//! this module.
//!
//! However, you can also create new selector types implementing
//! [`ReadParams`]/[`WriteParams`] and write blanket implementations
//! for the bitstream readers and writers in [`crate::impls`] where
//! `RP`/`WP` is set to your selector types. Then, by specifying
//! your type as value of the parameter `RP`/`WP` when creating such
//! readers and writers you will use automatically your blanket
//! implementations instead of the ones provided by this module.
//!
//! Note that the default implementations provided by this module are targeted at
//! `u32` read words and `u64` write words. If you use different word sizes,
//! you may want to write your own selector types.
//!
//! # Table peek-bits checks
//!
//! The `read_*_param` methods in each code module (e.g.,
//! [`GammaReadParam::read_gamma_param`]) verify at compile time, via `const {
//! }` blocks using [`BitRead::PEEK_BITS`], that the reader's peek word is large
//! enough for the table when the corresponding `USE_TABLE` const parameter is
//! `true`. These checks are short-circuited when the table is not used, so they
//! are only triggered for the tables actually selected.

use crate::codes::{delta::*, gamma::*, omega::*, pi::*, zeta::*};
use crate::impls::*;
use crate::traits::*;
#[cfg(feature = "mem_dbg")]
use mem_dbg::{MemDbg, MemSize};
use num_traits::AsPrimitive;

/// Marker trait for read-parameters selector types.
///
/// Note that in principle marker traits are not necessary to use
/// selector types, but they are useful to avoid that the user specifies
/// a nonsensical type, and to document the meaning of type parameters.
pub trait ReadParams {}

/// A selector type for read parameters providing reasonable defaults.
///
/// If you want to optimize these choices for your architecture, we suggest to
/// run the benchmarks in the `benches` directory and write your
/// own implementation.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
#[cfg_attr(feature = "mem_dbg", mem_size_flat)]
pub struct DefaultReadParams;
impl ReadParams for DefaultReadParams {}

macro_rules! impl_default_read_codes {
    ($($endianness:ident),*) => {$(
        impl<WR: WordRead<Word: DoubleType>> GammaRead<$endianness>
            for BufBitReader<$endianness, WR, DefaultReadParams>
        {
            #[inline(always)]
            fn read_gamma(&mut self) -> Result<u64, Self::Error> {
                // From our tests on all architectures ɣ codes are faster
                // without tables
                self.read_gamma_param::<false>()
            }
        }

        impl<WR: WordRead<Word: DoubleType>> DeltaRead<$endianness>
            for BufBitReader<$endianness, WR, DefaultReadParams>
        {
            #[inline(always)]
            fn read_delta(&mut self) -> Result<u64, Self::Error> {
                self.read_delta_param::<false, true>()
            }
        }

        impl<WR: WordRead<Word: DoubleType>> OmegaRead<$endianness>
            for BufBitReader<$endianness, WR, DefaultReadParams>
        {
            #[inline(always)]
            fn read_omega(&mut self) -> Result<u64, Self::Error> {
                self.read_omega_param::<true>()
            }
        }

        impl<WR: WordRead<Word: DoubleType>> ZetaRead<$endianness>
            for BufBitReader<$endianness, WR, DefaultReadParams>
        {
            #[inline(always)]
            fn read_zeta(&mut self, k: usize) -> Result<u64, Self::Error> {
                self.read_zeta_param(k)
            }

            #[inline(always)]
            fn read_zeta3(&mut self) -> Result<u64, Self::Error> {
                self.read_zeta3_param::<true>()
            }
        }

        impl<WR: WordRead<Word: DoubleType>> PiRead<$endianness>
            for BufBitReader<$endianness, WR, DefaultReadParams>
        {
            #[inline(always)]
            fn read_pi(&mut self, k: usize) -> Result<u64, Self::Error> {
                self.read_pi_param(k)
            }

            #[inline(always)]
            fn read_pi2(&mut self) -> Result<u64, Self::Error> {
                self.read_pi2_param::<false>()
            }
        }

        impl<WR: WordRead<Word = u64> + WordSeek<Error = <WR as WordRead>::Error>> GammaRead<$endianness>
            for BitReader<$endianness, WR, DefaultReadParams>
        {
            #[inline(always)]
            fn read_gamma(&mut self) -> Result<u64, Self::Error> {
                // From our tests, the ARM architecture is faster
                // without tables for ɣ codes.
                self.read_gamma_param::<false>()
            }
        }

        impl<WR: WordRead<Word = u64> + WordSeek<Error = <WR as WordRead>::Error>> DeltaRead<$endianness>
            for BitReader<$endianness, WR, DefaultReadParams>
        {
            #[inline(always)]
            fn read_delta(&mut self) -> Result<u64, Self::Error> {
                self.read_delta_param::<false, true>()
            }
        }

        impl<WR: WordRead<Word = u64> + WordSeek<Error = <WR as WordRead>::Error>> OmegaRead<$endianness>
            for BitReader<$endianness, WR, DefaultReadParams>
        {
            #[inline(always)]
            fn read_omega(&mut self) -> Result<u64, Self::Error> {
                self.read_omega_param::<true>()
            }
        }

        impl<WR: WordRead<Word = u64> + WordSeek<Error = <WR as WordRead>::Error>> ZetaRead<$endianness>
            for BitReader<$endianness, WR, DefaultReadParams>
        {
            #[inline(always)]
            fn read_zeta(&mut self, k: usize) -> Result<u64, Self::Error> {
                self.read_zeta_param(k)
            }

            #[inline(always)]
            fn read_zeta3(&mut self) -> Result<u64, Self::Error> {
                self.read_zeta3_param::<true>()
            }
        }

        impl<WR: WordRead<Word = u64> + WordSeek<Error = <WR as WordRead>::Error>> PiRead<$endianness>
            for BitReader<$endianness, WR, DefaultReadParams>
        {
            #[inline(always)]
            fn read_pi(&mut self, k: usize) -> Result<u64, Self::Error> {
                self.read_pi_param(k)
            }

            #[inline(always)]
            fn read_pi2(&mut self) -> Result<u64, Self::Error> {
                self.read_pi2_param::<false>()
            }
        }
    )*};
}

impl_default_read_codes! {LittleEndian, BigEndian}

/// Marker trait for write-parameters selector types.
///
/// Note that in principle marker traits are not necessary to use
/// selector types, but they are useful to avoid that the user specifies
/// a nonsensical type, and to document the meaning of type parameters.
pub trait WriteParams {}

/// A selector type for write parameters providing reasonable defaults.
///
/// If you want to optimize these choices for your architecture, we suggest to
/// run the benchmarks in the `benches` directory and write your
/// own implementation.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
#[cfg_attr(feature = "mem_dbg", mem_size_flat)]
pub struct DefaultWriteParams;
impl WriteParams for DefaultWriteParams {}

macro_rules! impl_default_write_codes {
    ($($endianness:ident),*) => {$(
        impl<WR: WordWrite, WP: WriteParams> GammaWrite<$endianness>
            for BufBitWriter<$endianness, WR, WP>
            where u64: AsPrimitive<WR::Word>,
        {
            #[inline(always)]
            fn write_gamma(&mut self, n: u64) -> Result<usize, Self::Error> {
                self.write_gamma_param::<true>(n)
            }
        }

        impl<WR: WordWrite, WP: WriteParams> DeltaWrite<$endianness>
            for BufBitWriter<$endianness, WR, WP>
            where u64: AsPrimitive<WR::Word>,
        {
            #[inline(always)]
            fn write_delta(&mut self, n: u64) -> Result<usize, Self::Error> {
                self.write_delta_param::<true, true>(n)
            }
        }

        impl<WR: WordWrite, WP: WriteParams> OmegaWrite<$endianness>
            for BufBitWriter<$endianness, WR, WP>
            where u64: AsPrimitive<WR::Word>,
        {
            #[inline(always)]
            fn write_omega(&mut self, n: u64) -> Result<usize, Self::Error> {
                self.write_omega_param::<true>(n)
            }
        }

        impl<WR: WordWrite, WP: WriteParams> ZetaWrite<$endianness>
            for BufBitWriter<$endianness, WR, WP>
            where u64: AsPrimitive<WR::Word>,
        {
            #[inline(always)]
            fn write_zeta(&mut self, n: u64, k: usize) -> Result<usize, Self::Error> {
                self.write_zeta_param(n, k)
            }

            #[inline(always)]
            fn write_zeta3(&mut self, n: u64) -> Result<usize, Self::Error> {
                self.write_zeta3_param::<true>(n)
            }
        }

        impl<WR: WordWrite, WP: WriteParams> PiWrite<$endianness>
            for BufBitWriter<$endianness, WR, WP>
            where u64: AsPrimitive<WR::Word>,
        {
            #[inline(always)]
            fn write_pi(&mut self, n: u64, k: usize) -> Result<usize, Self::Error> {
                self.write_pi_param(n, k)
            }

            #[inline(always)]
            fn write_pi2(&mut self, n: u64) -> Result<usize, Self::Error> {
                self.write_pi2_param::<true>(n)
            }
        }

    )*};
}

impl_default_write_codes! {LittleEndian, BigEndian}

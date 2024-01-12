/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

/*!

Mechanisms for selecting parameters.

Traits and structures in this file are of no interest for the standard
user. Their purpose is to provide a systematic way, and in particular
a default way, to select parameters for parameterized traits
such as [`GammaReadParam`] and [`GammaWriteParam`].

The traits and structure in this module work closely with the
bitstream readers and writers in [`crate::impls`], which have an
additional type parameter `RP`/`WP` that must be a type implementing [`ReadParams`] or
[`WriteParams`], respectively. These traits have no methods, but the
type assigned to the parameter can be used as a selector for a blanket implementation
of all parameterless traits in [`crate::codes`] such as [`GammaRead`],
[`GammaWrite`], [`DeltaRead`], [`DeltaWrite`], and so on.

The default implementation of [`ReadParams`] and [`WriteParams`] are
[`DefaultReadParams`] and [`DefaultWriteParams`], respectively. These
are also the default value for the parameter `RP`/`WP` in the bitstream
readers and writers in [`crate::impls`]. Thus, if you not specify
a value for the parameter `RP`/`WP`, you will obtain automatically
the blanket implementations for parameterless traits contained in this module.

However, you can also create a new type implementing [`ReadParams`]/[`WriteParams`] and
write blanket implementations for the bitstream readers and writers in [`crate::impls`],
fixing `RP`/`WP` to your type. Then, by specifying your type as value of the
parameter `RP`/`WP` when creating such readers and writers you will use
automatically your blanket implementations instead of the ones provided by this module.

*/

use crate::codes::*;
use crate::impls::*;
use crate::traits::*;
use common_traits::*;
use std::error::Error;

/// Selection trait for parameters of code-reading methods.
pub trait ReadParams {}

/// An implementation of [`ReadParams`] providing reasonable defaults.
///
/// If you want to optimize these choices for your architecture, we suggest to
/// run the benchmarks in the `benchmarks` directory and write your
/// own implementation.
#[derive(Debug, Clone)]
pub struct DefaultReadParams;
impl ReadParams for DefaultReadParams {}

macro_rules! impl_default_read_codes {
    ($($endianess:ident),*) => {$(
        impl<WR: WordRead> GammaRead<$endianess>
            for BufBitReader<$endianess, WR, DefaultReadParams>
        where
            WR:: Word: DoubleType + UpcastableInto<u64>,
            <WR::Word as DoubleType>::DoubleType: CastableInto<u64>,
        {
            #[inline(always)]
            fn read_gamma(&mut self) -> Result<u64, Self::Error> {
                // From our tests, the ARM architecture is faster
                // without tables for ɣ codes.
                return self.read_gamma_param::<false>();
            }

            #[inline(always)]
            fn skip_gamma(&mut self) -> Result<(), Self::Error> {
                // From our tests, the ARM architecture is faster
                // without tables for ɣ codes.
                return self.skip_gamma_param::<false>();
            }
        }

        impl<WR: WordRead> DeltaRead<$endianess>
            for BufBitReader<$endianess, WR, DefaultReadParams>
        where
            WR:: Word: DoubleType + UpcastableInto<u64>,
            <WR::Word as DoubleType>::DoubleType: CastableInto<u64>,
        {
            #[inline(always)]
            fn read_delta(&mut self) -> Result<u64, Self::Error> {
                return self.read_delta_param::<false, true>();
            }

            #[inline(always)]
            fn skip_delta(&mut self) -> Result<(), Self::Error> {
                return self.skip_delta_param::<false, true>();
            }
        }

        impl<WR: WordRead> ZetaRead<$endianess>
            for BufBitReader<$endianess, WR, DefaultReadParams>
        where
            WR:: Word: DoubleType + UpcastableInto<u64>,
            <WR::Word as DoubleType>::DoubleType: CastableInto<u64>,
        {
            #[inline(always)]
            fn read_zeta(&mut self, k: u64) -> Result<u64, Self::Error> {
                self.read_zeta_param(k)
            }

            #[inline(always)]
            fn skip_zeta(&mut self, k: u64) -> Result<(), Self::Error> {
                self.skip_zeta_param(k)
            }

            #[inline(always)]
            fn read_zeta3(&mut self) -> Result<u64, Self::Error> {
                self.read_zeta3_param::<true>()
            }

            #[inline(always)]
            fn skip_zeta3(&mut self) -> Result<(), Self::Error> {
                self.skip_zeta3_param::<true>()
            }
        }

        impl<E: Error, WR: WordRead<Error = E, Word = u64> + WordSeek<Error = E>> GammaRead<$endianess>
            for BitReader<$endianess, WR, DefaultReadParams>
        where
            WR:: Word: DoubleType + UpcastableInto<u64>,
            <WR::Word as DoubleType>::DoubleType: CastableInto<u64>,
        {
            #[inline(always)]
            fn read_gamma(&mut self) -> Result<u64, Self::Error> {
                // From our tests, the ARM architecture is faster
                // without tables for ɣ codes.
                return self.read_gamma_param::<false>();
            }

            #[inline(always)]
            fn skip_gamma(&mut self) -> Result<(), Self::Error> {
                // From our tests, the ARM architecture is faster
                // without tables for ɣ codes.
                return self.skip_gamma_param::<false>();
            }
        }

        impl<E: Error, WR: WordRead<Error = E, Word = u64> + WordSeek<Error = E>> DeltaRead<$endianess>
            for BitReader<$endianess, WR, DefaultReadParams>
        where
            WR:: Word: DoubleType + UpcastableInto<u64>,
            <WR::Word as DoubleType>::DoubleType: CastableInto<u64>,
        {
            #[inline(always)]
            fn read_delta(&mut self) -> Result<u64, Self::Error> {
                return self.read_delta_param::<false, true>();
            }

            #[inline(always)]
            fn skip_delta(&mut self) -> Result<(), Self::Error> {
                return self.skip_delta_param::<false, true>();
            }
        }

        impl<E: Error, WR: WordRead<Error = E, Word = u64> + WordSeek<Error = E>> ZetaRead<$endianess>
            for BitReader<$endianess, WR, DefaultReadParams>
        where
            WR:: Word: DoubleType + UpcastableInto<u64>,
            <WR::Word as DoubleType>::DoubleType: CastableInto<u64>,
        {
            #[inline(always)]
            fn read_zeta(&mut self, k: u64) -> Result<u64, Self::Error> {
                self.read_zeta_param(k)
            }

            #[inline(always)]
            fn skip_zeta(&mut self, k: u64) -> Result<(), Self::Error> {
                self.skip_zeta_param(k)
            }

            #[inline(always)]
            fn read_zeta3(&mut self) -> Result<u64, Self::Error> {
                self.read_zeta3_param::<true>()
            }

            #[inline(always)]
            fn skip_zeta3(&mut self) -> Result<(), Self::Error> {
                self.skip_zeta3_param::<true>()
            }
        }
    )*};
}

impl_default_read_codes! {LittleEndian, BigEndian}

/// Selection trait for parameters of code-writing methods.
pub trait WriteParams {}

#[derive(Debug, Clone)]
/// An implementation of [`WriteParams`] providing reasonable defaults.
///
/// If you want to optimize these choices for your architecture, we suggest to
/// run the benchmarks in the `benchmarks` directory and write your
/// own implementation.
pub struct DefaultWriteParams;
impl WriteParams for DefaultWriteParams {}

macro_rules! impl_default_write_codes {
    ($($endianess:ident),*) => {$(
        impl<WR: WordWrite> GammaWrite<$endianess>
            for BufBitWriter<$endianess, WR, DefaultWriteParams>
            where u64: CastableInto<WR::Word>,
        {
            #[inline(always)]
            fn write_gamma(&mut self, value: u64) -> Result<usize, Self::Error> {
                self.write_gamma_param::<true>(value)
            }
        }

        impl<WR: WordWrite, DC: WriteParams> DeltaWrite<$endianess>
            for BufBitWriter<$endianess, WR, DC>
            where u64: CastableInto<WR::Word>,
        {
            #[inline(always)]
            fn write_delta(&mut self, value: u64) -> Result<usize, Self::Error> {
                self.write_delta_param::<true, true>(value)
            }
        }

        impl<WR: WordWrite, DC: WriteParams> ZetaWrite<$endianess>
            for BufBitWriter<$endianess, WR, DC>
            where u64: CastableInto<WR::Word>,
        {
            #[inline(always)]
            fn write_zeta(&mut self, value: u64, k: u64) -> Result<usize, Self::Error> {
                self.write_zeta_param::<true>(value, k)
            }

            #[inline(always)]
            fn write_zeta3(&mut self, value: u64) -> Result<usize, Self::Error> {
                self.write_zeta3_param::<true>(value)
            }
        }

    )*};
}

impl_default_write_codes! {LittleEndian, BigEndian}

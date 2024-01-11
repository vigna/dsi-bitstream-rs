/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

/*!

Traits and structures to manage the usage of decoding tables.

Traits and structures in this file are of no interest for the standard
user. They can be use to choose whether to use decoding tables for reading and writing
instantaneous codes. They make available easy-to-use parameterless
functions like [`GammaRead::read_gamma`] and [`GammaWrite::write_gamma`].,
as opposed to the more general [`GammaReadParam::read_gamma_param`] and
[`GammaWriteParam::write_gamma_param`].

These choices work well across several architectures. If you
would like to perform further tuning,  the `benchmark` directory contains
scripts to test the speed of reading from and writing to
bit streams under a variety of parameters.

*/

use crate::codes::*;
use crate::impls::*;
use crate::traits::*;
use common_traits::*;
use std::error::Error;

pub trait ReadParams {}

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

        impl<E: Error + Send + Sync, WR: WordRead<Error = E, Word = u64> + WordSeek<Error = E>> GammaRead<$endianess>
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

        impl<E: Error + Send + Sync, WR: WordRead<Error = E, Word = u64> + WordSeek<Error = E>> DeltaRead<$endianess>
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

        impl<E: Error + Send + Sync, WR: WordRead<Error = E, Word = u64> + WordSeek<Error = E>> ZetaRead<$endianess>
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

pub trait WriteParams {}

#[derive(Debug, Clone)]
pub struct DefaultWriteParams;
impl WriteParams for DefaultWriteParams {}

macro_rules! impl_default_write_codes {
    ($($endianess:ident),*) => {$(
        impl<WR: WordWrite, DC: WriteParams> GammaWrite<$endianess>
            for BufBitWriter<$endianess, WR, DC>
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

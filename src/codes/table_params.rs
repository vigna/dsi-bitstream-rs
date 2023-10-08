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

These choices work well across several architectures:
if you thing they are not good for yours, we suggest to run
``
./python/bench_code_tables_read.py | ./python/plot_code_tables_read.py
./python/bench_code_tables_write.py | ./python/plot_code_tables_write.py
``
These scripts will generate graph displaying the speed of reads and
write under different table sizes (or absence of tables) and layout
of tables (two separated arrays, or merged in a single array).

By writing another implementation similar to the one in this file
you can only choose whether to use tables or not,
and in particular for δ codes you can choose also whether to use
table to decode the initial γ code. To change the other choices
you need to run `./python/gen_code_tables.py` after changing
the values in the function `generate_default_tables()`.

*/

use crate::codes::*;
use crate::impls::*;
use crate::traits::*;
use anyhow::Result;
use common_traits::*;

pub trait ReadParams {}

#[derive(Debug, Clone)]
pub struct DefaultReadParams;
impl ReadParams for DefaultReadParams {}

macro_rules! impl_default_read_codes {
    ($($endianess:ident),*) => {$(
        impl<BB: Word, WR: WordRead, DC: ReadParams> GammaRead<$endianess>
            for BufBitReader<$endianess, BB, WR, DC>
        where
            BB: DowncastableInto<WR::Word> + CastableInto<u64>,
            WR::Word: UpcastableInto<BB> + UpcastableInto<u64>,
        {
            #[inline(always)]
            fn read_gamma(&mut self) -> Result<u64> {
                // From our tests, the ARM architecture is faster
                // without tables ɣ codes.
                return self.read_gamma_param::<false>();
            }

            #[inline(always)]
            fn skip_gamma(&mut self) -> Result<()> {
                // From our tests, the ARM architecture is faster
                // without tables ɣ codes.
                return self.skip_gamma_param::<false>();
            }
        }

        impl<BB: Word, WR: WordRead, DC: ReadParams> DeltaRead<$endianess>
            for BufBitReader<$endianess, BB, WR, DC>
        where
            BB: DowncastableInto<WR::Word> + CastableInto<u64>,
            WR::Word: UpcastableInto<BB> + UpcastableInto<u64>,
        {
            #[inline(always)]
            fn read_delta(&mut self) -> Result<u64> {
                return self.read_delta_param::<false, true>();
            }

            #[inline(always)]
            fn skip_delta(&mut self) -> Result<()> {
                return self.skip_delta_param::<false, true>();
            }
        }

        impl<BB: Word, WR: WordRead, DC: ReadParams> ZetaRead<$endianess>
            for BufBitReader<$endianess, BB, WR, DC>
        where
            BB: DowncastableInto<WR::Word> + CastableInto<u64>,
            WR::Word: UpcastableInto<BB> + UpcastableInto<u64>,
        {
            #[inline(always)]
            fn read_zeta(&mut self, k: u64) -> Result<u64> {
                self.read_zeta_param(k)
            }

            #[inline(always)]
            fn skip_zeta(&mut self, k: u64) -> Result<()> {
                self.skip_zeta_param(k)
            }

            #[inline(always)]
            fn read_zeta3(&mut self) -> Result<u64> {
                self.read_zeta3_param::<true>()
            }

            #[inline(always)]
            fn skip_zeta3(&mut self) -> Result<()> {
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
        impl<BB: Word, WR: WordWrite, DC: WriteParams> GammaWrite<$endianess>
            for BufBitWriter<$endianess, BB, WR, DC>
            where BB: DowncastableInto<WR::Word>,
            u64: CastableInto<BB>,
            u64: DowncastableInto<WR::Word>,
        {
            #[inline(always)]
            fn write_gamma(&mut self, value: u64) -> Result<usize> {
                self.write_gamma_param::<true>(value)
            }
        }

        impl<BB: Word, WR: WordWrite, DC: WriteParams> DeltaWrite<$endianess>
            for BufBitWriter<$endianess, BB, WR, DC>
            where BB: DowncastableInto<WR::Word>,
            u64: CastableInto<BB>,
            u64: DowncastableInto<WR::Word>,
        {
            #[inline(always)]
            fn write_delta(&mut self, value: u64) -> Result<usize> {
                self.write_delta_param::<true, true>(value)
            }
        }

        impl<BB: Word, WR: WordWrite, DC: WriteParams> ZetaWrite<$endianess>
            for BufBitWriter<$endianess, BB, WR, DC>
            where BB: DowncastableInto<WR::Word>,
            u64: CastableInto<BB>,
            u64: DowncastableInto<WR::Word>,
        {
            #[inline(always)]
            fn write_zeta(&mut self, value: u64, k: u64) -> Result<usize> {
                self.write_zeta_param::<true>(value, k)
            }

            #[inline(always)]
            fn write_zeta3(&mut self, value: u64) -> Result<usize> {
                self.write_zeta3_param::<true>(value)
            }
        }

    )*};
}
impl_default_write_codes! {LittleEndian, BigEndian}

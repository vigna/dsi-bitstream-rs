/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use crate::codes::table_params::{DefaultWriteParams, WriteParams};
use crate::codes::unary_tables;
use crate::traits::*;
use anyhow::{bail, Result};
use common_traits::{CastableInto, DowncastableInto, Integer, Number, UpcastableInto, Word};

/// An implementation of [`BitWrite`] for a
/// [`WordWrite`] with word `u64` and of [`BitSeek`] for a [`WordSeek`].
///
/// Endianness can be selected using the parameter `E`.
///
/// This implementation uses a
/// bit buffer to store bits that are not yet written. The type of the bit buffer
/// is `u128`.

#[derive(Debug)]
pub struct BufBitWriter<
    E: BBSWDrop<BB, WR, WCP>,
    BB: Word,
    WR: WordWrite,
    WCP: WriteParams = DefaultWriteParams,
> {
    /// The [`WordWrite`] to which we will write words.
    backend: WR,
    /// The buffer where we store code writes until we have a word worth of bits.
    buffer: BB,
    /// Counter of how many bits in buffer are to consider valid and should be
    /// written to be backend
    bits_in_buffer: usize,
    _marker_endianness: core::marker::PhantomData<(E, WCP)>,
}

impl<E: BBSWDrop<BB, WR, WCP>, BB: Word, WR: WordWrite, WCP: WriteParams>
    BufBitWriter<E, BB, WR, WCP>
{
    /// Create a new [`BufBitWriter`] from a backend word writer
    pub fn new(backend: WR) -> Self {
        Self {
            backend,
            buffer: BB::ZERO,
            bits_in_buffer: 0,
            _marker_endianness: core::marker::PhantomData,
        }
    }

    #[inline(always)]
    #[must_use]
    fn space_left_in_buffer(&self) -> usize {
        BB::BITS - self.bits_in_buffer
    }
}

impl<E: BBSWDrop<BB, WR, WCP>, BB: Word, WR: WordWrite, WCP: WriteParams> core::ops::Drop
    for BufBitWriter<E, BB, WR, WCP>
{
    fn drop(&mut self) {
        // During a drop we can't save anything if it goes bad :/
        let _ = E::flush(self);
    }
}

/// Ignore. Inner trait needed for dispatching of drop logic based on endianess
/// of a [`BufBitWriter`]. This is public to avoid the leak of
/// private traits in public defs, an user should never need to implement this.
///
/// I discussed this [here](https://users.rust-lang.org/t/on-generic-associated-enum-and-type-comparisons/92072).
pub trait BBSWDrop<BB: Word, WR: WordWrite, WCP: WriteParams>: Sized + Endianness {
    /// handle the drop
    fn flush(data: &mut BufBitWriter<Self, BB, WR, WCP>) -> Result<()>;
}

impl<BB: Word, WR: WordWrite, WCP: WriteParams> BBSWDrop<BB, WR, WCP> for BE
where
    BB: DowncastableInto<WR::Word>,
{
    #[inline]
    fn flush(data: &mut BufBitWriter<Self, BB, WR, WCP>) -> Result<()> {
        data.partial_flush()?;
        if data.bits_in_buffer > 0 {
            let mut word = data.buffer.downcast();
            let shamt = WR::Word::BITS as usize - data.bits_in_buffer;
            word <<= shamt;
            data.backend.write_word(word.to_be())?;

            data.bits_in_buffer = 0;
        }
        Ok(())
    }
}

impl<BB: Word, WR: WordWrite, WCP: WriteParams> BufBitWriter<BE, BB, WR, WCP>
where
    BB: DowncastableInto<WR::Word>,
{
    #[inline]
    fn partial_flush(&mut self) -> Result<()> {
        if self.bits_in_buffer < WR::Word::BITS as usize {
            return Ok(());
        }
        self.bits_in_buffer -= WR::Word::BITS as usize;
        let word = (self.buffer >> self.bits_in_buffer).downcast();
        self.backend.write_word(word.to_be())?;
        Ok(())
    }
}

impl<BB: Word, WR: WordWrite, WCP: WriteParams> BitWrite<BE> for BufBitWriter<BE, BB, WR, WCP>
where
    BB: DowncastableInto<WR::Word>,
    u64: CastableInto<BB>,
{
    fn flush(mut self) -> Result<()> {
        BE::flush(&mut self)
    }

    #[inline]
    fn write_bits(&mut self, mut value: u64, n_bits: usize) -> Result<usize> {
        if n_bits == 0 {
            return Ok(0);
        }

        if n_bits > 64 {
            bail!(
                "The n of bits to read has to be in [0, 64] and {} is not.",
                n_bits
            );
        }

        #[cfg(test)]
        if (value & (1_u128 << n_bits).wrapping_sub(1) as u64) != value {
            bail!("Error value {} does not fit in {} bits", value, n_bits);
        }

        let mut to_write = n_bits;
        loop {
            self.partial_flush()?;
            if to_write <= self.space_left_in_buffer() {
                self.buffer <<= to_write;
                self.buffer |= value.cast();
                self.bits_in_buffer += to_write;
                return Ok(n_bits);
            }
            self.buffer <<= WR::Word::BITS;
            to_write -= WR::Word::BITS as usize;
            self.buffer |= value.cast() >> to_write;
            value &= (1 << to_write) - 1;
        }
    }

    #[inline]
    #[allow(clippy::collapsible_if)]
    fn write_unary_param<const USE_TABLE: bool>(&mut self, mut value: u64) -> Result<usize> {
        if USE_TABLE {
            if let Some(len) = unary_tables::write_table_be(self, value)? {
                return Ok(len);
            }
        }

        let code_length = value + 1;
        let space_left = self.space_left_in_buffer() as u64;

        if code_length <= space_left {
            self.bits_in_buffer += code_length as usize;
            self.buffer = self.buffer << value << 1; // Might be code_length == BB::BITS
            self.buffer |= BB::ONE;
            return Ok(code_length as usize);
        }

        self.buffer <<= space_left;
        let high_word = (self.buffer >> WR::Word::BITS).downcast();
        let low_word = self.buffer.downcast();
        self.backend.write_word(high_word.to_be())?;
        self.backend.write_word(low_word.to_be())?;

        value -= space_left;

        for _ in 0..value / WR::Word::BITS as u64 {
            self.backend.write_word(WR::Word::ZERO)?;
        }

        value %= WR::Word::BITS as u64;

        self.buffer = BB::ONE;
        self.bits_in_buffer = value as usize + 1;
        Ok(code_length as usize)
    }

    fn write_unary(&mut self, value: u64) -> Result<usize> {
        self.write_unary_param::<true>(value)
    }
}

impl<BB: Word, WR: WordWrite, WCP: WriteParams> BBSWDrop<BB, WR, WCP> for LE
where
    BB: DowncastableInto<WR::Word>,
{
    #[inline]
    fn flush(data: &mut BufBitWriter<Self, BB, WR, WCP>) -> Result<()> {
        data.partial_flush()?;
        if data.bits_in_buffer > 0 {
            dbg!(std::any::type_name::<BB>());
            dbg!(std::any::type_name::<WR::Word>());
            let mut word = (data.buffer >> WR::Word::BITS).downcast();
            let shamt = WR::Word::BITS as usize - data.bits_in_buffer;
            word >>= shamt;
            data.backend.write_word(word.to_le())?;
            data.bits_in_buffer = 0;
        }
        Ok(())
    }
}

impl<BB: Word, WR: WordWrite, WCP: WriteParams> BufBitWriter<LE, BB, WR, WCP>
where
    BB: DowncastableInto<WR::Word>,
{
    #[inline]
    fn partial_flush(&mut self) -> Result<()> {
        if self.bits_in_buffer < WR::Word::BITS as usize {
            return Ok(());
        }
        let word = (self.buffer >> (BB::BITS - self.bits_in_buffer)).downcast();
        self.bits_in_buffer -= WR::Word::BITS as usize;
        self.backend.write_word(word.to_le())?;
        Ok(())
    }
}

impl<BB: Word, WR: WordWrite, WCP: WriteParams> BitWrite<LE> for BufBitWriter<LE, BB, WR, WCP>
where
    BB: DowncastableInto<WR::Word>,
    u64: CastableInto<BB>,
{
    fn flush(mut self) -> Result<()> {
        LE::flush(&mut self)
    }

    #[inline]
    fn write_bits(&mut self, mut value: u64, n_bits: usize) -> Result<usize> {
        if n_bits == 0 {
            return Ok(0);
        }

        if n_bits > 64 {
            bail!(
                "The n of bits to read has to be in [0, 64] and {} is not.",
                n_bits
            );
        }

        #[cfg(test)]
        if (value & (1_u128 << n_bits).wrapping_sub(1) as u64) != value {
            bail!("Error value {} does not fit in {} bits", value, n_bits);
        }

        let mut to_write = n_bits;
        loop {
            self.partial_flush()?;
            if to_write <= self.space_left_in_buffer() {
                self.buffer >>= to_write;
                self.buffer |= value.cast() << (BB::BITS - to_write);
                self.bits_in_buffer += to_write;
                return Ok(n_bits);
            }
            self.buffer >>= WR::Word::BITS;
            to_write -= WR::Word::BITS as usize;
            self.buffer |= (value & u64::MAX >> (64 - WR::Word::BITS)).cast();
            value = value >> (WR::Word::BITS as usize - 1) >> 1;
        }
    }

    #[inline]
    #[allow(clippy::collapsible_if)]
    fn write_unary_param<const USE_TABLE: bool>(&mut self, mut value: u64) -> Result<usize> {
        debug_assert_ne!(value, u64::MAX);
        if USE_TABLE {
            if let Some(len) = unary_tables::write_table_le(self, value)? {
                return Ok(len);
            }
        }

        let code_length = value + 1;
        let space_left = self.space_left_in_buffer() as u64;

        if code_length <= space_left {
            self.bits_in_buffer += code_length as usize;
            self.buffer = self.buffer >> value >> 1;
            self.buffer |= BB::ONE << BB::BITS - 1;
            return Ok(code_length as usize);
        }

        self.buffer >>= space_left;
        let high_word = (self.buffer >> WR::Word::BITS).downcast();
        let low_word = self.buffer.downcast();
        self.backend.write_word(low_word.to_le())?;
        self.backend.write_word(high_word.to_le())?;

        value -= space_left;

        for _ in 0..value / WR::Word::BITS as u64 {
            self.backend.write_word(WR::Word::ZERO)?;
        }

        value %= WR::Word::BITS as u64;

        self.buffer = BB::ONE << BB::BITS - 1;
        self.bits_in_buffer = value as usize + 1;
        Ok(code_length as usize)
    }

    fn write_unary(&mut self, value: u64) -> Result<usize> {
        self.write_unary_param::<true>(value)
    }
}

macro_rules! test_buf_bit_writer {
    ($f: ident, $bb:ty, $word:ty) => {
        #[test]
        fn $f() -> Result<(), anyhow::Error> {
            use super::MemWordWriterVec;
            use crate::{
                codes::{GammaRead, GammaWrite},
                prelude::{
                    len_delta, len_gamma, len_unary, BufBitReader, DeltaRead, DeltaWrite,
                    MemWordReader,
                },
            };
            use rand::Rng;
            use rand::{rngs::SmallRng, SeedableRng};

            let mut buffer_be: Vec<$word> = vec![];
            let mut buffer_le: Vec<$word> = vec![];
            let mut big = BufBitWriter::<BE, $bb, _>::new(MemWordWriterVec::new(&mut buffer_be));
            let mut little = BufBitWriter::<LE, $bb, _>::new(MemWordWriterVec::new(&mut buffer_le));

            let mut r = SmallRng::seed_from_u64(0);
            const ITER: u64 = 128;

            for i in 0..ITER {
                /*                 assert_eq!(big.write_gamma(i)?, len_gamma(i));
                assert_eq!(little.write_gamma(i)?, len_gamma(i));
                assert_eq!(big.write_gamma(i)?, len_gamma(i));
                assert_eq!(little.write_gamma(i)?, len_gamma(i));
                assert_eq!(big.write_delta(i)?, len_delta(i));
                assert_eq!(little.write_delta(i)?, len_delta(i));
                assert_eq!(big.write_delta(i)?, len_delta(i));
                assert_eq!(little.write_delta(i)?, len_delta(i));
                big.write_bits(1, r.gen_range(1..=64))?;
                little.write_bits(1, r.gen_range(1..=64))?;
                assert_eq!(big.write_unary_param::<true>(i)?, len_unary(i));
                assert_eq!(little.write_unary_param::<true>(i)?, len_unary(i));*/
                //assert_eq!(big.write_unary(i)?, len_unary(i));
                assert_eq!(little.write_unary(i)?, len_unary(i));
            }

            drop(big);
            drop(little);

            type ReadWord = u32;
            type ReadBuffer = $word;
            let be_trans: &[ReadWord] = unsafe {
                core::slice::from_raw_parts(
                    buffer_be.as_ptr() as *const ReadWord,
                    buffer_be.len()
                        * (core::mem::size_of::<$word>() / core::mem::size_of::<ReadWord>()),
                )
            };
            let le_trans: &[ReadWord] = unsafe {
                core::slice::from_raw_parts(
                    buffer_le.as_ptr() as *const ReadWord,
                    buffer_le.len()
                        * (core::mem::size_of::<$word>() / core::mem::size_of::<ReadWord>()),
                )
            };

            let mut big_buff = BufBitReader::<BE, ReadBuffer, _>::new(MemWordReader::new(be_trans));
            let mut little_buff =
                BufBitReader::<LE, ReadBuffer, _>::new(MemWordReader::new(le_trans));

            let mut r = SmallRng::seed_from_u64(0);

            for i in 0..ITER {
                /*                 assert_eq!(big_buff.read_gamma()?, i);
                assert_eq!(little_buff.read_gamma()?, i);
                assert_eq!(big_buff.read_gamma()?, i);
                assert_eq!(little_buff.read_gamma()?, i);
                assert_eq!(big_buff.read_delta()?, i);
                assert_eq!(little_buff.read_delta()?, i);
                assert_eq!(big_buff.read_delta()?, i);
                assert_eq!(little_buff.read_delta()?, i);
                assert_eq!(big_buff.read_bits(r.gen_range(1..=64))?, 1);
                assert_eq!(little_buff.read_bits(r.gen_range(1..=64))?, 1);
                assert_eq!(big_buff.read_unary()?, i);
                assert_eq!(little_buff.read_unary()?, i);*/
                //assert_eq!(big_buff.read_unary()?, i);
                assert_eq!(little_buff.read_unary()?, i);
            }

            Ok(())
        }
    };
}

test_buf_bit_writer!(test_u128_u64, u128, u64);
test_buf_bit_writer!(test_u64_u32, u64, u32);
//test_buf_bit_writer!(test_u32_u16, u32, u16);

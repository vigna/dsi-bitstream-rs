/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use crate::codes::unary_tables;
use crate::traits::*;
use anyhow::{bail, Result};

/// An implementation of [`BitWrite`] on a generic [`WordWrite`]
#[derive(Debug)]
pub struct BufferedBitStreamWrite<BO: BBSWDrop<WR>, WR: WordWrite> {
    /// The backend used to write words to
    backend: WR,
    /// The buffer where we store code writes until we have a word worth of bits
    buffer: u128,
    /// Counter of how many bits in buffer are to consider valid and should be
    /// written to be backend
    bits_in_buffer: usize,
    /// make the compiler happy :)
    _marker: core::marker::PhantomData<BO>,
}

impl<BO: BBSWDrop<WR>, WR: WordWrite> BufferedBitStreamWrite<BO, WR> {
    /// Create a new [`BufferedBitStreamWrite`] from a backend word writer
    pub fn new(backend: WR) -> Self {
        Self {
            backend,
            buffer: 0,
            bits_in_buffer: 0,
            _marker: core::marker::PhantomData::default(),
        }
    }

    #[inline(always)]
    #[must_use]
    fn space_left_in_buffer(&self) -> usize {
        128 - self.bits_in_buffer
    }
}

impl<BO: BBSWDrop<WR>, WR: WordWrite> core::ops::Drop for BufferedBitStreamWrite<BO, WR> {
    fn drop(&mut self) {
        // During a drop we can't save anything if it goes bad :/
        let _ = BO::drop(self);
    }
}

/// Ignore. Inner trait needed for dispatching of drop logic based on endianess
/// of a [`BufferedBitStreamWrite`]. This is public to avoid the leak of
/// private traits in public defs, an user should never need to implement this.
///
/// I discussed this [here](https://users.rust-lang.org/t/on-generic-associated-enum-and-type-comparisons/92072).
pub trait BBSWDrop<WR: WordWrite>: Sized + BitOrder {
    /// handle the drop
    fn drop(data: &mut BufferedBitStreamWrite<Self, WR>) -> Result<()>;
}

impl<WR: WordWrite<Word = u64>> BBSWDrop<WR> for BE {
    #[inline]
    fn drop(data: &mut BufferedBitStreamWrite<Self, WR>) -> Result<()> {
        data.partial_flush()?;
        if data.bits_in_buffer > 0 {
            let mut word = data.buffer as u64;
            let shamt = 64 - data.bits_in_buffer;
            word <<= shamt;
            data.backend.write_word(word.to_be())?;
        }
        Ok(())
    }
}

impl<WR: WordWrite<Word = u64>> BitWriteBuffered<BE> for BufferedBitStreamWrite<BE, WR> {
    #[inline]
    fn partial_flush(&mut self) -> Result<()> {
        if self.bits_in_buffer < 64 {
            return Ok(());
        }
        self.bits_in_buffer -= 64;
        let word = (self.buffer >> self.bits_in_buffer) as u64;
        self.backend.write_word(word.to_be())?;
        Ok(())
    }
}

impl<WR: WordWrite<Word = u64>> BitWrite<BE> for BufferedBitStreamWrite<BE, WR> {
    #[inline]
    fn write_bits(&mut self, value: u64, n_bits: usize) -> Result<usize> {
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

        if n_bits > self.space_left_in_buffer() {
            self.partial_flush()?;
        }
        self.buffer <<= n_bits;
        self.buffer |= value as u128;
        self.bits_in_buffer += n_bits;
        Ok(n_bits)
    }

    #[inline]
    #[allow(clippy::collapsible_if)]
    fn write_unary<const USE_TABLE: bool>(&mut self, value: u64) -> Result<usize> {
        debug_assert_ne!(value, u64::MAX);
        if USE_TABLE {
            if let Some(len) = unary_tables::write_table_be(self, value)? {
                return Ok(len);
            }
        }

        let mut code_length = value + 1;

        loop {
            let space_left = self.space_left_in_buffer() as u64;
            if code_length <= space_left {
                break;
            }
            if space_left == 128 {
                self.buffer = 0;
                self.backend.write_word(0)?;
                self.backend.write_word(0)?;
            } else {
                self.buffer <<= space_left;
                let high_word = (self.buffer >> 64) as u64;
                let low_word = self.buffer as u64;
                self.backend.write_word(high_word.to_be())?;
                self.backend.write_word(low_word.to_be())?;
                self.buffer = 0;
            }
            code_length -= space_left;
            self.bits_in_buffer = 0;
        }
        self.bits_in_buffer += code_length as usize;
        self.buffer = self.buffer << (code_length - 1) << 1;
        self.buffer |= 1_u128;

        Ok((value + 1) as usize)
    }
}

impl<WR: WordWrite<Word = u64>> BBSWDrop<WR> for LE {
    #[inline]
    fn drop(data: &mut BufferedBitStreamWrite<Self, WR>) -> Result<()> {
        data.partial_flush()?;
        if data.bits_in_buffer > 0 {
            let mut word = (data.buffer >> 64) as u64;
            let shamt = 64 - data.bits_in_buffer;
            word >>= shamt;
            data.backend.write_word(word.to_le())?;
        }
        Ok(())
    }
}

impl<WR: WordWrite<Word = u64>> BitWriteBuffered<LE> for BufferedBitStreamWrite<LE, WR> {
    #[inline]
    fn partial_flush(&mut self) -> Result<()> {
        if self.bits_in_buffer < 64 {
            return Ok(());
        }
        let word = (self.buffer >> (128 - self.bits_in_buffer)) as u64;
        self.bits_in_buffer -= 64;
        self.backend.write_word(word.to_le())?;
        Ok(())
    }
}

impl<WR: WordWrite<Word = u64>> BitWrite<LE> for BufferedBitStreamWrite<LE, WR> {
    #[inline]
    fn write_bits(&mut self, value: u64, n_bits: usize) -> Result<usize> {
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

        if n_bits > self.space_left_in_buffer() {
            self.partial_flush()?;
        }

        self.buffer >>= n_bits;
        self.buffer |= (value as u128) << (128 - n_bits);
        self.bits_in_buffer += n_bits;

        Ok(n_bits)
    }

    #[inline]
    #[allow(clippy::collapsible_if)]
    fn write_unary<const USE_TABLE: bool>(&mut self, value: u64) -> Result<usize> {
        debug_assert_ne!(value, u64::MAX);
        if USE_TABLE {
            if let Some(len) = unary_tables::write_table_le(self, value)? {
                return Ok(len);
            }
        }
        let mut code_length = value + 1;

        loop {
            let space_left = self.space_left_in_buffer() as u64;
            if code_length <= space_left {
                break;
            }
            if space_left == 128 {
                self.buffer = 0;
                self.backend.write_word(0)?;
                self.backend.write_word(0)?;
            } else {
                self.buffer >>= space_left;
                let high_word = (self.buffer >> 64) as u64;
                let low_word = self.buffer as u64;
                self.backend.write_word(low_word.to_le())?;
                self.backend.write_word(high_word.to_le())?;
                self.buffer = 0;
            }
            code_length -= space_left;
            self.bits_in_buffer = 0;
        }
        self.bits_in_buffer += code_length as usize;
        self.buffer = self.buffer >> (code_length - 1) >> 1;
        self.buffer |= 1_u128 << 127;

        Ok((value + 1) as usize)
    }
}

#[cfg(test)]
#[test]
fn test_buffered_bit_stream_writer() {
    use super::MemWordWriteVec;
    use crate::{
        codes::{GammaRead, GammaWrite},
        prelude::{BufferedBitStreamRead, MemWordRead},
    };
    use rand::Rng;
    use rand::{rngs::SmallRng, SeedableRng};

    let mut buffer_be: Vec<u64> = vec![];
    let mut buffer_le: Vec<u64> = vec![];
    let mut big = BufferedBitStreamWrite::<BE, _>::new(MemWordWriteVec::new(&mut buffer_be));
    let mut little = BufferedBitStreamWrite::<LE, _>::new(MemWordWriteVec::new(&mut buffer_le));

    let mut r = SmallRng::seed_from_u64(0);
    const ITER: u64 = 128;

    for i in 0..ITER {
        big.write_gamma::<false>(i).unwrap();
        little.write_gamma::<false>(i).unwrap();
        big.write_gamma::<true>(i).unwrap();
        little.write_gamma::<true>(i).unwrap();
        big.write_bits(1, r.gen_range(1..=64));
        little.write_bits(1, r.gen_range(1..=64));
        big.write_unary::<false>(r.gen_range(0..=1024));
        little.write_unary::<false>(r.gen_range(0..=1024));
        big.write_unary::<true>(r.gen_range(0..=1024));
        little.write_unary::<true>(r.gen_range(0..=1024));
    }

    drop(big);
    drop(little);

    type ReadWord = u32;
    type ReadBuffer = u64;
    let be_trans: &[ReadWord] = unsafe {
        core::slice::from_raw_parts(
            buffer_be.as_ptr() as *const ReadWord,
            buffer_be.len() * (core::mem::size_of::<u64>() / core::mem::size_of::<ReadWord>()),
        )
    };
    let le_trans: &[ReadWord] = unsafe {
        core::slice::from_raw_parts(
            buffer_le.as_ptr() as *const ReadWord,
            buffer_le.len() * (core::mem::size_of::<u64>() / core::mem::size_of::<ReadWord>()),
        )
    };

    let mut big_buff = BufferedBitStreamRead::<BE, ReadBuffer, _>::new(MemWordRead::new(be_trans));
    let mut little_buff =
        BufferedBitStreamRead::<LE, ReadBuffer, _>::new(MemWordRead::new(le_trans));

    let mut r = SmallRng::seed_from_u64(0);

    for i in 0..ITER {
        assert_eq!(big_buff.read_gamma::<false>().unwrap(), i);
        assert_eq!(little_buff.read_gamma::<false>().unwrap(), i);
        assert_eq!(big_buff.read_gamma::<false>().unwrap(), i);
        assert_eq!(little_buff.read_gamma::<false>().unwrap(), i);
        assert_eq!(big_buff.read_bits(r.gen_range(1..=64)).unwrap(), 1);
        assert_eq!(little_buff.read_bits(r.gen_range(1..=64)).unwrap(), 1);
        assert_eq!(
            big_buff.read_unary::<false>().unwrap(),
            r.gen_range(0..=1024)
        );
        assert_eq!(
            little_buff.read_unary::<false>().unwrap(),
            r.gen_range(0..=1024)
        );
        assert_eq!(
            big_buff.read_unary::<false>().unwrap(),
            r.gen_range(0..=1024)
        );
        assert_eq!(
            little_buff.read_unary::<false>().unwrap(),
            r.gen_range(0..=1024)
        );
    }
}

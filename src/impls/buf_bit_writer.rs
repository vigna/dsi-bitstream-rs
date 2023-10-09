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
use common_traits::{CastableInto, Integer, Number};

/// An implementation of [`BitWrite`] for a
/// [`WordWrite`] and of [`BitSeek`] for a [`WordSeek`].
///
/// Endianness can be selected using the parameter `E`.
///
/// This implementation uses a bit buffer to store bits that are not yet written.
/// The size of the bit buffer is the size of the word used by the [`WordWrite`].
///
/// The additional type parameter `WCP` is used to select the parameters for the
/// instantanous codes, but the casual user should be happy with the default value.
/// See [`WriteParams`] for more details.

#[derive(Debug)]
pub struct BufBitWriter<E: BBSWDrop<WR, WCP>, WR: WordWrite, WCP: WriteParams = DefaultWriteParams>
{
    /// The [`WordWrite`] to which we will write words.
    backend: WR,
    /// The buffer where we store bits until we have a word worth of them.
    /// Note that only `bits_in_buffer` bits are valid: the rest have undefined value.
    buffer: WR::Word,
    /// How many bits in buffer are valid, from zero
    /// to `WR::Word::BITS`, both included.
    bits_in_buffer: usize,
    _marker_endianness: core::marker::PhantomData<(E, WCP)>,
}

impl<E: BBSWDrop<WR, WCP>, WR: WordWrite, WCP: WriteParams> BufBitWriter<E, WR, WCP> {
    /// Create a new [`BufBitWriter`] around a [`WordWrite`].
    pub fn new(backend: WR) -> Self {
        Self {
            backend,
            buffer: WR::Word::ZERO,
            bits_in_buffer: 0,
            _marker_endianness: core::marker::PhantomData,
        }
    }

    #[inline(always)]
    #[must_use]
    fn space_left_in_buffer(&self) -> usize {
        WR::Word::BITS - self.bits_in_buffer
    }
}

impl<E: BBSWDrop<WR, WCP>, WR: WordWrite, WCP: WriteParams> core::ops::Drop
    for BufBitWriter<E, WR, WCP>
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
pub trait BBSWDrop<WR: WordWrite, WCP: WriteParams>: Sized + Endianness {
    /// handle the drop
    fn flush(data: &mut BufBitWriter<Self, WR, WCP>) -> Result<()>;
}

impl<WR: WordWrite, WCP: WriteParams> BBSWDrop<WR, WCP> for BE {
    #[inline]
    fn flush(data: &mut BufBitWriter<Self, WR, WCP>) -> Result<()> {
        if data.bits_in_buffer > 0 {
            data.buffer <<= WR::Word::BITS as usize - data.bits_in_buffer;
            data.backend.write_word(data.buffer.to_be())?;
            data.bits_in_buffer = 0;
        }
        Ok(())
    }
}

impl<WR: WordWrite, WCP: WriteParams> BitWrite<BE> for BufBitWriter<BE, WR, WCP>
where
    u64: CastableInto<WR::Word>,
{
    fn flush(mut self) -> Result<()> {
        BE::flush(&mut self)
    }

    #[inline]
    fn write_bits(&mut self, value: u64, n_bits: usize) -> Result<usize> {
        if n_bits > 64 {
            bail!("Too many bits: {} > 64", n_bits);
        }

        #[cfg(test)]
        if (value & (1_u128 << n_bits).wrapping_sub(1) as u64) != value {
            bail!("Error value {} does not fit in {} bits", value, n_bits);
        }

        let space_left_in_buffer = self.space_left_in_buffer();
        // Easy way out: we fit the buffer
        if n_bits <= space_left_in_buffer {
            if n_bits == 0 {
                // Handling this case algorithmically is a pain
                return Ok(0);
            }
            // n_bits might be 64 and the buffer might be empty
            self.buffer = self.buffer << n_bits - 1 << 1;
            self.buffer |= value.cast();
            self.bits_in_buffer += n_bits;
            return Ok(n_bits);
        }

        // Load the bottom of the buffer, if necessary, and dump the whole buffer
        if space_left_in_buffer != 0 {
            self.buffer <<= space_left_in_buffer;
            self.buffer |= (value >> n_bits - space_left_in_buffer).cast();
        }
        self.backend.write_word(self.buffer.to_be())?;

        let mut to_write = n_bits - space_left_in_buffer;

        for _ in 0..to_write / WR::Word::BITS as usize {
            to_write -= WR::Word::BITS;
            self.backend
                .write_word((value >> to_write).cast().to_be())?;
        }

        self.bits_in_buffer = to_write;
        self.buffer = value.cast() & (WR::Word::ONE << to_write) - WR::Word::ONE;
        Ok(n_bits)
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
        let space_left_in_buffer = self.space_left_in_buffer() as u64;

        // Easy way out: we fit the buffer
        if code_length <= space_left_in_buffer {
            self.bits_in_buffer += code_length as usize;
            self.buffer = self.buffer << value << 1; // Might be code_length == WR::Word::BITS
            self.buffer |= WR::Word::ONE;
            return Ok(code_length as usize);
        }

        if space_left_in_buffer == WR::Word::BITS as _ {
            self.backend.write_word(WR::Word::ZERO)?;
        } else {
            self.buffer = self.buffer << space_left_in_buffer;
            self.backend.write_word(self.buffer.to_be())?;
        }

        value -= space_left_in_buffer;

        for _ in 0..value / WR::Word::BITS as u64 {
            self.backend.write_word(WR::Word::ZERO)?;
        }

        self.buffer = WR::Word::ONE;
        self.bits_in_buffer = (value % WR::Word::BITS as u64) as usize + 1;
        Ok(code_length as usize)
    }

    fn write_unary(&mut self, value: u64) -> Result<usize> {
        self.write_unary_param::<true>(value)
    }
}

impl<WR: WordWrite, WCP: WriteParams> BBSWDrop<WR, WCP> for LE {
    #[inline]
    fn flush(data: &mut BufBitWriter<Self, WR, WCP>) -> Result<()> {
        if data.bits_in_buffer > 0 {
            data.buffer >>= WR::Word::BITS as usize - data.bits_in_buffer;
            data.backend.write_word(data.buffer.to_le())?;
            data.bits_in_buffer = 0;
        }
        Ok(())
    }
}

impl<WR: WordWrite, WCP: WriteParams> BitWrite<LE> for BufBitWriter<LE, WR, WCP>
where
    u64: CastableInto<WR::Word>,
{
    fn flush(mut self) -> Result<()> {
        LE::flush(&mut self)
    }

    #[inline]
    fn write_bits(&mut self, mut value: u64, n_bits: usize) -> Result<usize> {
        if n_bits > 64 {
            bail!("Too many bits: {} > 64", n_bits);
        }

        #[cfg(test)]
        if (value & (1_u128 << n_bits).wrapping_sub(1) as u64) != value {
            bail!("Error value {} does not fit in {} bits", value, n_bits);
        }

        let space_left_in_buffer = self.space_left_in_buffer();
        // Easy way out: we fit the buffer
        if n_bits <= space_left_in_buffer {
            if n_bits == 0 {
                // Handling this case algorithmically is a pain
                return Ok(0);
            }
            // to_write might be 64 and the buffer might be empty
            self.buffer = self.buffer >> n_bits - 1 >> 1;
            self.buffer |= value.cast() << (WR::Word::BITS - n_bits);
            self.bits_in_buffer += n_bits;
            return Ok(n_bits);
        }

        // Load the top of the buffer, if necessary, and dump the whole buffer
        if space_left_in_buffer != 0 {
            self.buffer >>= space_left_in_buffer;
            self.buffer |= value.cast() << self.bits_in_buffer;
        }
        self.backend.write_word(self.buffer.to_le())?;

        let to_write = n_bits - space_left_in_buffer;
        value >>= space_left_in_buffer;

        for _ in 0..to_write / WR::Word::BITS as usize {
            self.backend.write_word(value.cast().to_le())?;
            // This might be executed with WR::Word == u64,
            // but it cannot be executed with WR::Word == u128
            value = value >> WR::Word::BITS - 1 >> 1;
        }

        self.bits_in_buffer = to_write % WR::Word::BITS;
        self.buffer = value.cast().rotate_right(self.bits_in_buffer as u32);
        Ok(n_bits)
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
        let space_left_in_buffer = self.space_left_in_buffer() as u64;

        // Easy way out: we fit the buffer
        if code_length <= space_left_in_buffer {
            self.bits_in_buffer += code_length as usize;
            self.buffer = self.buffer >> value >> 1; // Might be code_length == WR::Word::BITS
            self.buffer |= WR::Word::ONE << WR::Word::BITS - 1;
            return Ok(code_length as usize);
        }

        if space_left_in_buffer == WR::Word::BITS as _ {
            self.backend.write_word(WR::Word::ZERO)?;
        } else {
            self.buffer = self.buffer >> space_left_in_buffer;
            self.backend.write_word(self.buffer.to_le())?;
        }

        value -= space_left_in_buffer;

        for _ in 0..value / WR::Word::BITS as u64 {
            self.backend.write_word(WR::Word::ZERO)?;
        }

        value %= WR::Word::BITS as u64;

        self.buffer = WR::Word::ONE << WR::Word::BITS - 1;
        self.bits_in_buffer = value as usize + 1;
        Ok(code_length as usize)
    }

    fn write_unary(&mut self, value: u64) -> Result<usize> {
        self.write_unary_param::<true>(value)
    }
}

macro_rules! test_buf_bit_writer {
    ($f: ident, $word:ty) => {
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
            let mut big = BufBitWriter::<BE, _>::new(MemWordWriterVec::new(&mut buffer_be));
            let mut little = BufBitWriter::<LE, _>::new(MemWordWriterVec::new(&mut buffer_le));

            let mut r = SmallRng::seed_from_u64(0);
            const ITER: usize = 1_000_000;

            for _ in 0..ITER {
                let value = r.gen_range(0..128);
                assert_eq!(big.write_gamma(value)?, len_gamma(value));
                let value = r.gen_range(0..128);
                assert_eq!(little.write_gamma(value)?, len_gamma(value));
                let value = r.gen_range(0..128);
                assert_eq!(big.write_gamma(value)?, len_gamma(value));
                let value = r.gen_range(0..128);
                assert_eq!(little.write_gamma(value)?, len_gamma(value));
                let value = r.gen_range(0..128);
                assert_eq!(big.write_delta(value)?, len_delta(value));
                let value = r.gen_range(0..128);
                assert_eq!(little.write_delta(value)?, len_delta(value));
                let value = r.gen_range(0..128);
                assert_eq!(big.write_delta(value)?, len_delta(value));
                let value = r.gen_range(0..128);
                assert_eq!(little.write_delta(value)?, len_delta(value));
                let n_bits = r.gen_range(0..=64);
                if n_bits == 0 {
                    big.write_bits(0, 0)?;
                } else {
                    big.write_bits(1, n_bits)?;
                }
                let n_bits = r.gen_range(0..=64);
                if n_bits == 0 {
                    little.write_bits(0, 0)?;
                } else {
                    little.write_bits(1, n_bits)?;
                }
                let value = r.gen_range(0..128);
                assert_eq!(big.write_unary_param::<false>(value)?, len_unary(value));
                let value = r.gen_range(0..128);
                assert_eq!(little.write_unary_param::<false>(value)?, len_unary(value));
                let value = r.gen_range(0..128);
                assert_eq!(big.write_unary_param::<true>(value)?, len_unary(value));
                let value = r.gen_range(0..128);
                assert_eq!(little.write_unary_param::<true>(value)?, len_unary(value));
            }

            drop(big);
            drop(little);

            type ReadWord = u16;
            type ReadBuffer = u32;
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

            for _ in 0..ITER {
                assert_eq!(big_buff.read_gamma()?, r.gen_range(0..128));
                assert_eq!(little_buff.read_gamma()?, r.gen_range(0..128));
                assert_eq!(big_buff.read_gamma()?, r.gen_range(0..128));
                assert_eq!(little_buff.read_gamma()?, r.gen_range(0..128));
                assert_eq!(big_buff.read_delta()?, r.gen_range(0..128));
                assert_eq!(little_buff.read_delta()?, r.gen_range(0..128));
                assert_eq!(big_buff.read_delta()?, r.gen_range(0..128));
                assert_eq!(little_buff.read_delta()?, r.gen_range(0..128));
                let n_bits = r.gen_range(0..=64);
                if n_bits == 0 {
                    assert_eq!(big_buff.read_bits(0)?, 0);
                } else {
                    assert_eq!(big_buff.read_bits(n_bits)?, 1);
                }
                let n_bits = r.gen_range(0..=64);
                if n_bits == 0 {
                    assert_eq!(little_buff.read_bits(0)?, 0);
                } else {
                    assert_eq!(little_buff.read_bits(n_bits)?, 1);
                }

                assert_eq!(big_buff.read_unary()?, r.gen_range(0..128));
                assert_eq!(little_buff.read_unary()?, r.gen_range(0..128));
                assert_eq!(big_buff.read_unary()?, r.gen_range(0..128));
                assert_eq!(little_buff.read_unary()?, r.gen_range(0..128));
            }

            Ok(())
        }
    };
}

test_buf_bit_writer!(test_u128, u128);
test_buf_bit_writer!(test_u64, u64);
test_buf_bit_writer!(test_u32, u32);

test_buf_bit_writer!(test_u16, u16);
test_buf_bit_writer!(test_usize, usize);

/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use core::any::TypeId;
use core::{mem, ptr};

use crate::codes::params::{DefaultWriteParams, WriteParams};
use crate::traits::*;
use common_traits::{AsBytes, CastableInto, FiniteRangeNumber, Integer, Number};

/// An implementation of [`BitWrite`] for a [`WordWrite`].
///
/// This implementation uses a bit buffer to store bits that are not yet written.
/// The size of the bit buffer is the size of the word used by the [`WordWrite`],
/// which on most platform should be `usize`.
///
/// The additional type parameter `WP` is used to select the parameters for the
/// instantanous codes, but the casual user should be happy with the default value.
/// See [`WriteParams`] for more details.

#[derive(Debug)]
pub struct BufBitWriter<E: Endianness, WW: WordWrite, WP: WriteParams = DefaultWriteParams> {
    /// The [`WordWrite`] to which we will write words.
    backend: WW,
    /// The buffer where we store bits until we have a word worth of them.
    /// It might be empty, but it is never full.
    /// Only the lower (BE) or upper (LE) `WW::Word::BITS - space_left_in_buffer`
    /// bits are valid: the rest have undefined value.
    buffer: WW::Word,
    /// Number of upper (BE) or lower (LE) free bits in the buffer. It is always greater than zero.
    space_left_in_buffer: usize,
    _marker_endianness: core::marker::PhantomData<(E, WP)>,
}

impl<E: Endianness, WW: WordWrite, WP: WriteParams> BufBitWriter<E, WW, WP>
where
    BufBitWriter<E, WW, WP>: BitWrite<E>,
{
    /// Create a new [`BufBitWriter`] around a [`WordWrite`].
    ///
    /// ### Example
    /// ```
    /// use dsi_bitstream::prelude::*;
    /// let buffer = Vec::<usize>::new();
    /// let word_writer = MemWordWriterVec::new(buffer);
    /// let mut buf_bit_writer = <BufBitWriter<BE, _>>::new(word_writer);
    pub fn new(backend: WW) -> Self {
        Self {
            backend,
            buffer: WW::Word::ZERO,
            space_left_in_buffer: WW::Word::BITS,
            _marker_endianness: core::marker::PhantomData,
        }
    }

    ///  Return the backend, consuming this writer after
    /// [flushing it](BufBitWriter::flush).
    pub fn into_inner(mut self) -> Result<WW, <Self as BitWrite<E>>::Error> {
        self.flush()?;
        // SAFETY: forget(self) prevents double dropping backend
        let backend = unsafe { ptr::read(&self.backend) };
        mem::forget(self);
        Ok(backend)
    }
}

impl<E: Endianness, WW: WordWrite, WP: WriteParams> core::ops::Drop for BufBitWriter<E, WW, WP> {
    fn drop(&mut self) {
        if TypeId::of::<E>() == TypeId::of::<LE>() {
            flush_le(self).unwrap()
        } else {
            // TypeId::of::<E>() = TypeId::of::<BE>()
            flush_be(self).unwrap()
        }
    }
}

/// Helper function flushing a [`BufBitWriter`] in big-endian fashion.
///
/// The endianness is hardwired because the function is called
/// from [`BufBitWriter::drop`] using a check on the
/// [`TypeId`] of the endianness.
fn flush_be<E: Endianness, WW: WordWrite, WP: WriteParams>(
    buf_bit_writer: &mut BufBitWriter<E, WW, WP>,
) -> Result<(), WW::Error> {
    if buf_bit_writer.space_left_in_buffer != WW::Word::BITS {
        buf_bit_writer.buffer <<= buf_bit_writer.space_left_in_buffer;
        buf_bit_writer
            .backend
            .write_word(buf_bit_writer.buffer.to_be())?;
        buf_bit_writer.space_left_in_buffer = WW::Word::BITS;
    }
    buf_bit_writer.backend.flush()?;
    Ok(())
}

impl<WW: WordWrite, WP: WriteParams> BitWrite<BE> for BufBitWriter<BE, WW, WP>
where
    u64: CastableInto<WW::Word>,
{
    type Error = <WW as WordWrite>::Error;

    fn flush(&mut self) -> Result<(), Self::Error> {
        flush_be(self)
    }

    #[allow(unused_mut)]
    #[inline]
    fn write_bits(&mut self, mut value: u64, n_bits: usize) -> Result<usize, Self::Error> {
        debug_assert!(n_bits <= 64);
        #[cfg(feature = "checks")]
        assert!(
            value & (1_u128 << n_bits).wrapping_sub(1) as u64 == value,
            "Error: value {} does not fit in {} bits",
            value,
            n_bits
        );
        debug_assert!(self.space_left_in_buffer > 0);

        #[cfg(test)]
        if n_bits < 64 {
            // We put garbage in the higher bits for testing
            value |= u64::MAX << n_bits;
        }

        // Easy way out: we fit the buffer
        if n_bits < self.space_left_in_buffer {
            self.buffer <<= n_bits;
            // Clean up bits higher than n_bits
            self.buffer |= value.cast() & !(WW::Word::MAX << n_bits as u32);
            self.space_left_in_buffer -= n_bits;
            return Ok(n_bits);
        }

        // Load the bottom of the buffer, if necessary, and dump the whole buffer
        self.buffer = self.buffer << (self.space_left_in_buffer - 1) << 1;
        // The first shift discards bits higher than n_bits
        self.buffer |= (value << (64 - n_bits) >> (64 - self.space_left_in_buffer)).cast();
        self.backend.write_word(self.buffer.to_be())?;

        let mut to_write = n_bits - self.space_left_in_buffer;

        for _ in 0..to_write / WW::Word::BITS {
            to_write -= WW::Word::BITS;
            self.backend
                .write_word((value >> to_write).cast().to_be())?;
        }

        self.space_left_in_buffer = WW::Word::BITS - to_write;
        self.buffer = value.cast();
        Ok(n_bits)
    }

    #[inline]
    #[allow(clippy::collapsible_if)]
    fn write_unary(&mut self, mut value: u64) -> Result<usize, Self::Error> {
        debug_assert_ne!(value, u64::MAX);
        debug_assert!(self.space_left_in_buffer > 0);

        let code_length = value + 1;

        // Easy way out: we fit the buffer
        if code_length <= self.space_left_in_buffer as u64 {
            self.space_left_in_buffer -= code_length as usize;
            self.buffer = self.buffer << value << 1;
            self.buffer |= WW::Word::ONE;
            if self.space_left_in_buffer == 0 {
                self.backend.write_word(self.buffer.to_be())?;
                self.space_left_in_buffer = WW::Word::BITS;
            }
            return Ok(code_length as usize);
        }

        self.buffer = self.buffer << (self.space_left_in_buffer - 1) << 1;
        self.backend.write_word(self.buffer.to_be())?;

        value -= self.space_left_in_buffer as u64;

        for _ in 0..value / WW::Word::BITS as u64 {
            self.backend.write_word(WW::Word::ZERO)?;
        }

        value %= WW::Word::BITS as u64;

        if value == WW::Word::BITS as u64 - 1 {
            self.backend.write_word(WW::Word::ONE.to_be())?;
            self.space_left_in_buffer = WW::Word::BITS;
        } else {
            self.buffer = WW::Word::ONE;
            self.space_left_in_buffer = WW::Word::BITS - (value as usize + 1);
        }

        Ok(code_length as usize)
    }

    #[cfg(not(feature = "no_copy_impls"))]
    fn copy_from<F: Endianness>(
        &mut self,
        bit_read: &mut impl BitRead<F>,
        mut n: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if n < self.space_left_in_buffer as u64 {
            self.buffer = self.buffer << n | bit_read.read_bits(n as usize)?.cast();
            self.space_left_in_buffer -= n as usize;
            return Ok(());
        }

        self.buffer = self.buffer << (self.space_left_in_buffer - 1) << 1
            | bit_read.read_bits(self.space_left_in_buffer)?.cast();
        n -= self.space_left_in_buffer as u64;

        self.backend.write_word(self.buffer.to_be())?;

        for _ in 0..n / WW::Word::BITS as u64 {
            self.backend
                .write_word(bit_read.read_bits(WW::Word::BITS)?.cast().to_be())?;
        }

        n %= WW::Word::BITS as u64;
        self.buffer = bit_read.read_bits(n as usize)?.cast();
        self.space_left_in_buffer = WW::Word::BITS - n as usize;

        Ok(())
    }
}

/// Helper function flushing a [`BufBitWriter`] in big-endian fashion.
///
/// The endianness is hardwired because the function is called
/// from [`BufBitWriter::drop`] using a check on the
/// [`TypeId`] of the endianness.
fn flush_le<E: Endianness, WW: WordWrite, WP: WriteParams>(
    buf_bit_writer: &mut BufBitWriter<E, WW, WP>,
) -> Result<(), WW::Error> {
    if buf_bit_writer.space_left_in_buffer != WW::Word::BITS {
        buf_bit_writer.buffer >>= buf_bit_writer.space_left_in_buffer;
        buf_bit_writer
            .backend
            .write_word(buf_bit_writer.buffer.to_le())?;
        buf_bit_writer.space_left_in_buffer = WW::Word::BITS;
    }
    buf_bit_writer.backend.flush()?;
    Ok(())
}

impl<WW: WordWrite, WP: WriteParams> BitWrite<LE> for BufBitWriter<LE, WW, WP>
where
    u64: CastableInto<WW::Word>,
{
    type Error = <WW as WordWrite>::Error;

    fn flush(&mut self) -> Result<(), Self::Error> {
        flush_le(self)
    }

    #[inline]
    fn write_bits(&mut self, mut value: u64, n_bits: usize) -> Result<usize, Self::Error> {
        debug_assert!(n_bits <= 64);
        #[cfg(feature = "checks")]
        assert!(
            value & (1_u128 << n_bits).wrapping_sub(1) as u64 == value,
            "Error: value {} does not fit in {} bits",
            value,
            n_bits
        );
        debug_assert!(self.space_left_in_buffer > 0);

        #[cfg(test)]
        if n_bits < 64 {
            // We put garbage in the higher bits for testing
            value |= u64::MAX << n_bits;
        }

        // Easy way out: we fit the buffer
        if n_bits < self.space_left_in_buffer {
            self.buffer >>= n_bits;
            // Clean up bits higher than n_bits
            self.buffer |=
                (value.cast() & !(WW::Word::MAX << n_bits as u32)).rotate_right(n_bits as u32);
            self.space_left_in_buffer -= n_bits;
            return Ok(n_bits);
        }

        // Load the top of the buffer, if necessary, and dump the whole buffer
        self.buffer = self.buffer >> (self.space_left_in_buffer - 1) >> 1;
        self.buffer |= value.cast() << (WW::Word::BITS - self.space_left_in_buffer);
        self.backend.write_word(self.buffer.to_le())?;

        let to_write = n_bits - self.space_left_in_buffer;
        value = value >> (self.space_left_in_buffer - 1) >> 1;

        for _ in 0..to_write / WW::Word::BITS {
            self.backend.write_word(value.cast().to_le())?;
            // This cannot be executed with WW::Word::BITS >= 64
            value >>= WW::Word::BITS;
        }

        self.space_left_in_buffer = WW::Word::BITS - to_write % WW::Word::BITS;
        self.buffer = value.cast().rotate_right(to_write as u32);
        Ok(n_bits)
    }

    #[inline]
    #[allow(clippy::collapsible_if)]
    fn write_unary(&mut self, mut value: u64) -> Result<usize, Self::Error> {
        debug_assert_ne!(value, u64::MAX);
        debug_assert!(self.space_left_in_buffer > 0);

        let code_length = value + 1;

        // Easy way out: we fit the buffer
        if code_length <= self.space_left_in_buffer as u64 {
            self.space_left_in_buffer -= code_length as usize;
            self.buffer = self.buffer >> value >> 1;
            self.buffer |= WW::Word::ONE << (WW::Word::BITS - 1);
            if self.space_left_in_buffer == 0 {
                self.backend.write_word(self.buffer.to_le())?;
                self.space_left_in_buffer = WW::Word::BITS;
            }
            return Ok(code_length as usize);
        }

        self.buffer = self.buffer >> (self.space_left_in_buffer - 1) >> 1;
        self.backend.write_word(self.buffer.to_le())?;

        value -= self.space_left_in_buffer as u64;

        for _ in 0..value / WW::Word::BITS as u64 {
            self.backend.write_word(WW::Word::ZERO)?;
        }

        value %= WW::Word::BITS as u64;

        if value == WW::Word::BITS as u64 - 1 {
            self.backend
                .write_word((WW::Word::ONE << (WW::Word::BITS - 1)).to_le())?;
            self.space_left_in_buffer = WW::Word::BITS;
        } else {
            self.buffer = WW::Word::ONE << (WW::Word::BITS - 1);
            self.space_left_in_buffer = WW::Word::BITS - (value as usize + 1);
        }

        Ok(code_length as usize)
    }

    #[cfg(not(feature = "no_copy_impls"))]

    fn copy_from<F: Endianness>(
        &mut self,
        bit_read: &mut impl BitRead<F>,
        mut n: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if n < self.space_left_in_buffer as u64 {
            self.buffer =
                self.buffer >> n | (bit_read.read_bits(n as usize)?.cast()).rotate_right(n as u32);
            self.space_left_in_buffer -= n as usize;
            return Ok(());
        }

        self.buffer = self.buffer >> (self.space_left_in_buffer - 1) >> 1
            | (bit_read.read_bits(self.space_left_in_buffer)?.cast())
                .rotate_right(self.space_left_in_buffer as u32);
        n -= self.space_left_in_buffer as u64;

        self.backend.write_word(self.buffer.to_le())?;

        for _ in 0..n / WW::Word::BITS as u64 {
            self.backend
                .write_word(bit_read.read_bits(WW::Word::BITS)?.cast().to_le())?;
        }

        n %= WW::Word::BITS as u64;
        self.buffer = bit_read
            .read_bits(n as usize)?
            .cast()
            .rotate_right(n as u32);
        self.space_left_in_buffer = WW::Word::BITS - n as usize;

        Ok(())
    }
}

macro_rules! test_buf_bit_writer {
    ($f: ident, $word:ty) => {
        #[test]
        fn $f() -> Result<(), Box<dyn std::error::Error>> {
            use super::MemWordWriterVec;
            use crate::{
                codes::{GammaRead, GammaWrite},
                prelude::{
                    len_delta, len_gamma, BufBitReader, DeltaRead, DeltaWrite, MemWordReader,
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
                    big.write_bits(r.gen::<u64>() & u64::MAX >> 64 - n_bits, n_bits)?;
                }
                let n_bits = r.gen_range(0..=64);
                if n_bits == 0 {
                    little.write_bits(0, 0)?;
                } else {
                    little.write_bits(r.gen::<u64>() & u64::MAX >> 64 - n_bits, n_bits)?;
                }
                let value = r.gen_range(0..128);
                assert_eq!(big.write_unary(value)?, value as usize + 1);
                let value = r.gen_range(0..128);
                assert_eq!(little.write_unary(value)?, value as usize + 1);
            }

            drop(big);
            drop(little);

            type ReadWord = u16;
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

            let mut big_buff = BufBitReader::<BE, _>::new(MemWordReader::new(be_trans));
            let mut little_buff = BufBitReader::<LE, _>::new(MemWordReader::new(le_trans));

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
                    assert_eq!(
                        big_buff.read_bits(n_bits)?,
                        r.gen::<u64>() & u64::MAX >> 64 - n_bits
                    );
                }
                let n_bits = r.gen_range(0..=64);
                if n_bits == 0 {
                    assert_eq!(little_buff.read_bits(0)?, 0);
                } else {
                    assert_eq!(
                        little_buff.read_bits(n_bits)?,
                        r.gen::<u64>() & u64::MAX >> 64 - n_bits
                    );
                }

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

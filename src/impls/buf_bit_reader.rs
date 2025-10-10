/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use common_traits::*;

use crate::codes::params::{DefaultReadParams, ReadParams};
use crate::traits::*;
use core::convert::Infallible;
use core::error::Error;
use core::{mem, ptr};
#[cfg(feature = "mem_dbg")]
use mem_dbg::{MemDbg, MemSize};

/// An internal shortcut to the double type of the word of a
/// [`WordRead`].
type BB<WR> = <<WR as WordRead>::Word as DoubleType>::DoubleType;

/// An implementation of [`BitRead`] and [`BitSeek`] for a [`WordRead`] and a
/// [`WordSeek`].
///
/// This implementation uses a bit buffer to store bits that are not yet read.
/// The buffer is sized as twice the word size of the underlying [`WordRead`].
/// Typically, the best choice is to have a buffer that is sized as `usize`,
/// which means that the word of the underlying [`WordRead`] should be half of
/// that (i.e., `u32` for a 64-bit architecture). However, results will vary
/// depending on the CPU.
///
/// The peek word is equal to the bit buffer. The value returned
/// by [`peek_bits`](crate::traits::BitRead::peek_bits) contains at least as
/// many bits as the word size plus one (extended with zeros beyond end of
/// stream).
///
/// This implementation is usually faster than
/// [`BitReader`](crate::impls::BitReader).
///
/// The additional type parameter `RP` is used to select the parameters for the
/// instantaneous codes, but the casual user should be happy with the default
/// value. See [`ReadParams`] for more details.
///
/// For additional flexibility, this structures implements [`std::io::Read`].
/// Note that because of coherence rules it is not possible to implement
/// [`std::io::Read`] for a generic [`BitRead`].

#[derive(Debug)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
pub struct BufBitReader<E: Endianness, WR: WordRead, RP: ReadParams = DefaultReadParams>
where
    WR::Word: DoubleType,
{
    /// The [`WordRead`] used to fill the buffer.
    backend: WR,
    /// The 2-word bit buffer that is used to read the codes. It is never full,
    /// but it may be empty. Only the upper (BE) or lower (LE)
    /// `bits_in_buffer` bits are valid; the other bits are always zeroes.
    buffer: BB<WR>,
    /// Number of valid upper (BE) or lower (LE) bits in the buffer.
    /// It is always smaller than `BB::<WR>::BITS`.
    bits_in_buffer: usize,
    _marker: core::marker::PhantomData<(E, RP)>,
}

impl<E: Endianness, WR: WordRead + Clone, RP: ReadParams> core::clone::Clone
    for BufBitReader<E, WR, RP>
where
    WR::Word: DoubleType,
{
    fn clone(&self) -> Self {
        Self {
            backend: self.backend.clone(),
            buffer: self.buffer,
            bits_in_buffer: self.bits_in_buffer,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<E: Endianness, WR: WordRead, RP: ReadParams> BufBitReader<E, WR, RP>
where
    WR::Word: DoubleType,
{
    /// Create a new [`BufBitReader`] around a [`WordRead`].
    ///
    /// # Example
    /// ```
    /// use dsi_bitstream::prelude::*;
    /// let words: [u32; 2] = [0x0043b59f, 0xccf16077];
    /// let word_reader = MemWordReader::new(&words);
    /// let mut buf_bit_reader = <BufBitReader<BE, _>>::new(word_reader);
    /// ```
    #[must_use]
    pub fn new(backend: WR) -> Self {
        #[cfg(feature = "std")]
        check_tables(WR::Word::BITS + 1);
        Self {
            backend,
            buffer: BB::<WR>::ZERO,
            bits_in_buffer: 0,
            _marker: core::marker::PhantomData,
        }
    }

    ///  Return the backend, consuming this reader.
    pub fn into_inner(self) -> Result<WR, Infallible> {
        // SAFETY: forget(self) prevents double dropping backend
        let backend = unsafe { ptr::read(&self.backend) };
        mem::forget(self);
        Ok(backend)
    }
}

//
// Big-endian implementation
//

impl<WR: WordRead, RP: ReadParams> BufBitReader<BE, WR, RP>
where
    WR::Word: DoubleType,
{
    /// Ensure that in the buffer there are at least `WR::Word::BITS` bits to read.
    /// This method can be called only if there are at least
    /// `WR::Word::BITS` free bits in the buffer.
    #[inline(always)]
    fn refill(&mut self) -> Result<(), <WR as WordRead>::Error> {
        debug_assert!(BB::<WR>::BITS - self.bits_in_buffer >= WR::Word::BITS);

        let new_word: BB<WR> = self.backend.read_word()?.to_be().upcast();
        self.bits_in_buffer += WR::Word::BITS;
        self.buffer |= new_word << (BB::<WR>::BITS - self.bits_in_buffer);
        Ok(())
    }
}

impl<WR: WordRead, RP: ReadParams> BitRead<BE> for BufBitReader<BE, WR, RP>
where
    WR::Word: DoubleType + UpcastableInto<u64>,
    BB<WR>: CastableInto<u64>,
{
    type Error = <WR as WordRead>::Error;
    type PeekWord = BB<WR>;

    #[inline(always)]
    fn peek_bits(&mut self, n_bits: usize) -> Result<Self::PeekWord, Self::Error> {
        debug_assert!(n_bits > 0);
        debug_assert!(n_bits <= Self::PeekWord::BITS);

        // A peek can do at most one refill, otherwise we might lose data
        if n_bits > self.bits_in_buffer {
            self.refill()?;
        }

        debug_assert!(n_bits <= self.bits_in_buffer);

        // Move the n_bits highest bits of the buffer to the lowest
        Ok(self.buffer >> (BB::<WR>::BITS - n_bits))
    }

    #[inline(always)]
    fn skip_bits_after_peek(&mut self, n_bits: usize) {
        self.bits_in_buffer -= n_bits;
        self.buffer <<= n_bits;
    }

    #[inline]
    fn read_bits(&mut self, mut n_bits: usize) -> Result<u64, Self::Error> {
        debug_assert!(n_bits <= 64);
        debug_assert!(self.bits_in_buffer < BB::<WR>::BITS);

        // most common path, we just read the buffer
        if n_bits <= self.bits_in_buffer {
            // Valid right shift of BB::<WR>::BITS - n_bits, even when n_bits is zero
            let result: u64 = (self.buffer >> (BB::<WR>::BITS - n_bits - 1) >> 1_u32).cast();
            self.bits_in_buffer -= n_bits;
            self.buffer <<= n_bits;
            return Ok(result);
        }

        let mut result: u64 =
            (self.buffer >> (BB::<WR>::BITS - 1 - self.bits_in_buffer) >> 1_u8).cast();
        n_bits -= self.bits_in_buffer;

        // Directly read to the result without updating the buffer
        while n_bits > WR::Word::BITS {
            let new_word: u64 = self.backend.read_word()?.to_be().upcast();
            result = (result << WR::Word::BITS) | new_word;
            n_bits -= WR::Word::BITS;
        }

        debug_assert!(n_bits > 0);
        debug_assert!(n_bits <= WR::Word::BITS);

        // get the final word
        let new_word = self.backend.read_word()?.to_be();
        self.bits_in_buffer = WR::Word::BITS - n_bits;
        // compose the remaining bits
        let upcasted: u64 = new_word.upcast();
        let final_bits: u64 = (upcasted >> self.bits_in_buffer).downcast();
        result = (result << (n_bits - 1) << 1) | final_bits;
        // and put the rest in the buffer
        self.buffer = (UpcastableInto::<BB<WR>>::upcast(new_word)
            << (BB::<WR>::BITS - self.bits_in_buffer - 1))
            << 1;

        Ok(result)
    }

    #[inline]
    fn read_unary(&mut self) -> Result<u64, Self::Error> {
        debug_assert!(self.bits_in_buffer < BB::<WR>::BITS);

        // count the zeros from the left
        let zeros: usize = self.buffer.leading_zeros() as _;

        // if we encountered an 1 in the bits_in_buffer we can return
        if zeros < self.bits_in_buffer {
            self.buffer = self.buffer << zeros << 1;
            self.bits_in_buffer -= zeros + 1;
            return Ok(zeros as u64);
        }

        let mut result: u64 = self.bits_in_buffer as _;

        loop {
            let new_word = self.backend.read_word()?.to_be();

            if new_word != WR::Word::ZERO {
                let zeros: usize = new_word.leading_zeros() as _;
                self.buffer =
                    UpcastableInto::<BB<WR>>::upcast(new_word) << (WR::Word::BITS + zeros) << 1;
                self.bits_in_buffer = WR::Word::BITS - zeros - 1;
                return Ok(result + zeros as u64);
            }
            result += WR::Word::BITS as u64;
        }
    }

    #[inline]
    fn skip_bits(&mut self, mut n_bits: usize) -> Result<(), Self::Error> {
        debug_assert!(self.bits_in_buffer < BB::<WR>::BITS);
        // happy case, just shift the buffer
        if n_bits <= self.bits_in_buffer {
            self.bits_in_buffer -= n_bits;
            self.buffer <<= n_bits;
            return Ok(());
        }

        n_bits -= self.bits_in_buffer;

        // skip words as needed
        while n_bits > WR::Word::BITS {
            let _ = self.backend.read_word()?;
            n_bits -= WR::Word::BITS;
        }

        // get the final word
        let new_word = self.backend.read_word()?.to_be();
        self.bits_in_buffer = WR::Word::BITS - n_bits;

        self.buffer = UpcastableInto::<BB<WR>>::upcast(new_word)
            << (BB::<WR>::BITS - 1 - self.bits_in_buffer)
            << 1;

        Ok(())
    }

    #[cfg(not(feature = "no_copy_impls"))]
    fn copy_to<F: Endianness, W: BitWrite<F>>(
        &mut self,
        bit_write: &mut W,
        mut n: u64,
    ) -> Result<(), CopyError<Self::Error, W::Error>> {
        let from_buffer = Ord::min(n, self.bits_in_buffer as _);
        self.buffer = self.buffer.rotate_left(from_buffer as _);

        #[allow(unused_mut)]
        let mut self_buffer_u64: u64 = self.buffer.cast();

        #[cfg(feature = "checks")]
        {
            // Clean up in case checks are enabled
            if n < 64 {
                self_buffer_u64 &= (1_u64 << n) - 1;
            }
        }

        bit_write
            .write_bits(self_buffer_u64, from_buffer as usize)
            .map_err(CopyError::WriteError)?;
        n -= from_buffer;

        if n == 0 {
            self.bits_in_buffer -= from_buffer as usize;
            return Ok(());
        }

        while n > WR::Word::BITS as u64 {
            bit_write
                .write_bits(
                    self.backend
                        .read_word()
                        .map_err(CopyError::ReadError)?
                        .to_be()
                        .upcast(),
                    WR::Word::BITS,
                )
                .map_err(CopyError::WriteError)?;
            n -= WR::Word::BITS as u64;
        }

        assert!(n > 0);
        let new_word = self
            .backend
            .read_word()
            .map_err(CopyError::ReadError)?
            .to_be();
        self.bits_in_buffer = WR::Word::BITS - n as usize;
        bit_write
            .write_bits((new_word >> self.bits_in_buffer).upcast(), n as usize)
            .map_err(CopyError::WriteError)?;
        self.buffer = UpcastableInto::<BB<WR>>::upcast(new_word)
            .rotate_right(WR::Word::BITS as u32 - n as u32);

        Ok(())
    }
}

impl<
    E: Error + Send + Sync + 'static,
    WR: WordRead<Error = E> + WordSeek<Error = E>,
    RP: ReadParams,
> BitSeek for BufBitReader<BE, WR, RP>
where
    WR::Word: DoubleType,
{
    type Error = <WR as WordSeek>::Error;

    #[inline]
    fn bit_pos(&mut self) -> Result<u64, Self::Error> {
        Ok(self.backend.word_pos()? * WR::Word::BITS as u64 - self.bits_in_buffer as u64)
    }

    #[inline]
    fn set_bit_pos(&mut self, bit_index: u64) -> Result<(), Self::Error> {
        self.backend
            .set_word_pos(bit_index / WR::Word::BITS as u64)?;
        let bit_offset = (bit_index % WR::Word::BITS as u64) as usize;
        self.buffer = BB::<WR>::ZERO;
        self.bits_in_buffer = 0;
        if bit_offset != 0 {
            let new_word: BB<WR> = self.backend.read_word()?.to_be().upcast();
            self.bits_in_buffer = WR::Word::BITS - bit_offset;
            self.buffer = new_word << (BB::<WR>::BITS - self.bits_in_buffer);
        }
        Ok(())
    }
}

//
// Little-endian implementation
//

impl<WR: WordRead, RP: ReadParams> BufBitReader<LE, WR, RP>
where
    WR::Word: DoubleType,
{
    /// Ensure that in the buffer there are at least `WR::Word::BITS` bits to read.
    /// This method can be called only if there are at least
    /// `WR::Word::BITS` free bits in the buffer.
    #[inline(always)]
    fn refill(&mut self) -> Result<(), <WR as WordRead>::Error> {
        debug_assert!(BB::<WR>::BITS - self.bits_in_buffer >= WR::Word::BITS);

        let new_word: BB<WR> = self.backend.read_word()?.to_le().upcast();
        self.buffer |= new_word << self.bits_in_buffer;
        self.bits_in_buffer += WR::Word::BITS;
        Ok(())
    }
}

impl<WR: WordRead, RP: ReadParams> BitRead<LE> for BufBitReader<LE, WR, RP>
where
    WR::Word: DoubleType + UpcastableInto<u64>,
    BB<WR>: CastableInto<u64>,
{
    type Error = <WR as WordRead>::Error;
    type PeekWord = BB<WR>;

    #[inline(always)]
    fn peek_bits(&mut self, n_bits: usize) -> Result<Self::PeekWord, Self::Error> {
        debug_assert!(n_bits > 0);
        debug_assert!(n_bits <= Self::PeekWord::BITS);

        // A peek can do at most one refill, otherwise we might lose data
        if n_bits > self.bits_in_buffer {
            self.refill()?;
        }

        debug_assert!(n_bits <= self.bits_in_buffer);

        // Keep the n_bits lowest bits of the buffer
        let shamt = BB::<WR>::BITS - n_bits;
        Ok((self.buffer << shamt) >> shamt)
    }

    #[inline(always)]
    fn skip_bits_after_peek(&mut self, n_bits: usize) {
        self.bits_in_buffer -= n_bits;
        self.buffer >>= n_bits;
    }

    #[inline]
    fn read_bits(&mut self, mut n_bits: usize) -> Result<u64, Self::Error> {
        debug_assert!(n_bits <= 64);
        debug_assert!(self.bits_in_buffer < BB::<WR>::BITS);

        // most common path, we just read the buffer
        if n_bits <= self.bits_in_buffer {
            let result: u64 = (self.buffer & ((BB::<WR>::ONE << n_bits) - BB::<WR>::ONE)).cast();
            self.bits_in_buffer -= n_bits;
            self.buffer >>= n_bits;
            return Ok(result);
        }

        let mut result: u64 = self.buffer.cast();
        let mut bits_in_res = self.bits_in_buffer;

        // Directly read to the result without updating the buffer
        while n_bits > WR::Word::BITS + bits_in_res {
            let new_word: u64 = self.backend.read_word()?.to_le().upcast();
            result |= new_word << bits_in_res;
            bits_in_res += WR::Word::BITS;
        }

        n_bits -= bits_in_res;

        debug_assert!(n_bits > 0);
        debug_assert!(n_bits <= WR::Word::BITS);

        // get the final word
        let new_word = self.backend.read_word()?.to_le();
        self.bits_in_buffer = WR::Word::BITS - n_bits;
        // compose the remaining bits
        let shamt = 64 - n_bits;
        let upcasted: u64 = new_word.upcast();
        let final_bits: u64 = ((upcasted << shamt) >> shamt).downcast();
        result |= final_bits << bits_in_res;
        // and put the rest in the buffer
        self.buffer = UpcastableInto::<BB<WR>>::upcast(new_word) >> n_bits;

        Ok(result)
    }

    #[inline]
    fn read_unary(&mut self) -> Result<u64, Self::Error> {
        debug_assert!(self.bits_in_buffer < BB::<WR>::BITS);

        // count the zeros from the right
        let zeros: usize = self.buffer.trailing_zeros() as usize;

        // if we encountered an 1 in the bits_in_buffer we can return
        if zeros < self.bits_in_buffer {
            self.buffer = self.buffer >> zeros >> 1;
            self.bits_in_buffer -= zeros + 1;
            return Ok(zeros as u64);
        }

        let mut result: u64 = self.bits_in_buffer as _;

        loop {
            let new_word = self.backend.read_word()?.to_le();

            if new_word != WR::Word::ZERO {
                let zeros: usize = new_word.trailing_zeros() as _;
                self.buffer = UpcastableInto::<BB<WR>>::upcast(new_word) >> zeros >> 1;
                self.bits_in_buffer = WR::Word::BITS - zeros - 1;
                return Ok(result + zeros as u64);
            }
            result += WR::Word::BITS as u64;
        }
    }

    #[inline]
    fn skip_bits(&mut self, mut n_bits: usize) -> Result<(), Self::Error> {
        debug_assert!(self.bits_in_buffer < BB::<WR>::BITS);
        // happy case, just shift the buffer
        if n_bits <= self.bits_in_buffer {
            self.bits_in_buffer -= n_bits;
            self.buffer >>= n_bits;
            return Ok(());
        }

        n_bits -= self.bits_in_buffer;

        // skip words as needed
        while n_bits > WR::Word::BITS {
            let _ = self.backend.read_word()?;
            n_bits -= WR::Word::BITS;
        }

        // get the final word
        let new_word = self.backend.read_word()?.to_le();
        self.bits_in_buffer = WR::Word::BITS - n_bits;
        self.buffer = UpcastableInto::<BB<WR>>::upcast(new_word) >> n_bits;

        Ok(())
    }

    #[cfg(not(feature = "no_copy_impls"))]
    fn copy_to<F: Endianness, W: BitWrite<F>>(
        &mut self,
        bit_write: &mut W,
        mut n: u64,
    ) -> Result<(), CopyError<Self::Error, W::Error>> {
        let from_buffer = Ord::min(n, self.bits_in_buffer as _);

        #[allow(unused_mut)]
        let mut self_buffer_u64: u64 = self.buffer.cast();

        #[cfg(feature = "checks")]
        {
            // Clean up in case checks are enabled
            if n < 64 {
                self_buffer_u64 &= (1_u64 << n) - 1;
            }
        }

        bit_write
            .write_bits(self_buffer_u64, from_buffer as usize)
            .map_err(CopyError::WriteError)?;

        self.buffer >>= from_buffer;
        n -= from_buffer;

        if n == 0 {
            self.bits_in_buffer -= from_buffer as usize;
            return Ok(());
        }

        while n > WR::Word::BITS as u64 {
            bit_write
                .write_bits(
                    self.backend
                        .read_word()
                        .map_err(CopyError::ReadError)?
                        .to_le()
                        .upcast(),
                    WR::Word::BITS,
                )
                .map_err(CopyError::WriteError)?;
            n -= WR::Word::BITS as u64;
        }

        assert!(n > 0);
        let new_word = self
            .backend
            .read_word()
            .map_err(CopyError::ReadError)?
            .to_le();
        self.bits_in_buffer = WR::Word::BITS - n as usize;

        #[allow(unused_mut)]
        let mut new_word_u64: u64 = new_word.upcast();

        #[cfg(feature = "checks")]
        {
            // Clean up in case checks are enabled
            if n < 64 {
                new_word_u64 &= (1_u64 << n) - 1;
            }
        }

        bit_write
            .write_bits(new_word_u64, n as usize)
            .map_err(CopyError::WriteError)?;
        self.buffer = UpcastableInto::<BB<WR>>::upcast(new_word) >> n;
        Ok(())
    }
}

impl<
    E: Error + Send + Sync + 'static,
    WR: WordRead<Error = E> + WordSeek<Error = E>,
    RP: ReadParams,
> BitSeek for BufBitReader<LE, WR, RP>
where
    WR::Word: DoubleType,
{
    type Error = <WR as WordSeek>::Error;

    #[inline]
    fn bit_pos(&mut self) -> Result<u64, Self::Error> {
        Ok(self.backend.word_pos()? * WR::Word::BITS as u64 - self.bits_in_buffer as u64)
    }

    #[inline]
    fn set_bit_pos(&mut self, bit_index: u64) -> Result<(), Self::Error> {
        self.backend
            .set_word_pos(bit_index / WR::Word::BITS as u64)?;

        let bit_offset = (bit_index % WR::Word::BITS as u64) as usize;
        self.buffer = BB::<WR>::ZERO;
        self.bits_in_buffer = 0;
        if bit_offset != 0 {
            let new_word: BB<WR> = self.backend.read_word()?.to_le().upcast();
            self.bits_in_buffer = WR::Word::BITS - bit_offset;
            self.buffer = new_word >> bit_offset;
        }
        Ok(())
    }
}

#[cfg(feature = "std")]
impl<WR: WordRead, RP: ReadParams> std::io::Read for BufBitReader<LE, WR, RP>
where
    WR::Word: DoubleType + UpcastableInto<u64>,
    BB<WR>: CastableInto<u64>,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut iter = buf.chunks_exact_mut(8);

        for chunk in &mut iter {
            let word = self
                .read_bits(64)
                .map_err(|_| std::io::ErrorKind::UnexpectedEof)?;
            chunk.copy_from_slice(&word.to_le_bytes());
        }

        let rem = iter.into_remainder();
        if !rem.is_empty() {
            let word = self
                .read_bits(rem.len() * 8)
                .map_err(|_| std::io::ErrorKind::UnexpectedEof)?;
            rem.copy_from_slice(&word.to_le_bytes()[..rem.len()]);
        }

        Ok(buf.len())
    }
}

#[cfg(feature = "std")]
impl<WR: WordRead, RP: ReadParams> std::io::Read for BufBitReader<BE, WR, RP>
where
    WR::Word: DoubleType + UpcastableInto<u64>,
    BB<WR>: CastableInto<u64>,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut iter = buf.chunks_exact_mut(8);

        for chunk in &mut iter {
            let word = self
                .read_bits(64)
                .map_err(|_| std::io::ErrorKind::UnexpectedEof)?;
            chunk.copy_from_slice(&word.to_be_bytes());
        }

        let rem = iter.into_remainder();
        if !rem.is_empty() {
            let word = self
                .read_bits(rem.len() * 8)
                .map_err(|_| std::io::ErrorKind::UnexpectedEof)?;
            rem.copy_from_slice(&word.to_be_bytes()[8 - rem.len()..]);
        }

        Ok(buf.len())
    }
}

#[cfg(test)]
#[cfg(feature = "std")]
mod test {
    use super::*;
    use crate::prelude::{MemWordReader, MemWordWriterVec};
    use std::io::Read;

    #[test]
    fn test_read() {
        let data = [
            0x90, 0x2d, 0xd0, 0x26, 0xdf, 0x89, 0xbb, 0x7e, 0x3a, 0xd6, 0xc6, 0x96, 0x73, 0xe9,
            0x9d, 0xc9, 0x2a, 0x77, 0x82, 0xa9, 0xe6, 0x4b, 0x53, 0xcc, 0x83, 0x80, 0x4a, 0xf3,
            0xcd, 0xe3, 0x50, 0x4e, 0x45, 0x4a, 0x3a, 0x42, 0x00, 0x4b, 0x4d, 0xbe, 0x4c, 0x88,
            0x24, 0xf2, 0x4b, 0x6b, 0xbd, 0x79, 0xeb, 0x74, 0xbc, 0xe8, 0x7d, 0xff, 0x4b, 0x3d,
            0xa7, 0xd6, 0x0d, 0xef, 0x9c, 0x5b, 0xb3, 0xec, 0x94, 0x97, 0xcc, 0x8b, 0x41, 0xe1,
            0x9c, 0xcc, 0x1a, 0x03, 0x58, 0xc4, 0xfb, 0xd0, 0xc0, 0x10, 0xe2, 0xa0, 0xc9, 0xac,
            0xa7, 0xbb, 0x50, 0xf6, 0x5c, 0x87, 0x68, 0x0f, 0x42, 0x93, 0x3f, 0x2e, 0x28, 0x28,
            0x76, 0x83, 0x9b, 0xeb, 0x12, 0xe0, 0x4f, 0xc5, 0xb0, 0x8d, 0x14, 0xda, 0x3b, 0xdf,
            0xd3, 0x4b, 0x80, 0xd1, 0xfc, 0x87, 0x85, 0xae, 0x54, 0xc7, 0x45, 0xc9, 0x38, 0x43,
            0xa7, 0x9f, 0xdd, 0xa9, 0x71, 0xa7, 0x52, 0x36, 0x82, 0xff, 0x49, 0x55, 0xdb, 0x84,
            0xc2, 0x95, 0xad, 0x45, 0x80, 0xc6, 0x02, 0x80, 0xf8, 0xfc, 0x86, 0x79, 0xae, 0xb9,
            0x57, 0xe7, 0x3b, 0x33, 0x64, 0xa8,
        ];
        let data_u32 = unsafe { data.align_to::<u32>().1 };

        for i in 0..data.len() {
            let mut reader = BufBitReader::<LE, _>::new(MemWordReader::new(&data_u32));
            let mut buffer = vec![0; i];
            assert_eq!(reader.read(&mut buffer).unwrap(), i);
            assert_eq!(&buffer, &data[..i]);

            let mut reader = BufBitReader::<BE, _>::new(MemWordReader::new(&data_u32));
            let mut buffer = vec![0; i];
            assert_eq!(reader.read(&mut buffer).unwrap(), i);
            assert_eq!(&buffer, &data[..i]);
        }
    }

    macro_rules! test_buf_bit_reader {
        ($f: ident, $word:ty) => {
            #[test]
            fn $f() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
                #[allow(unused_imports)]
                use crate::{
                    codes::{GammaRead, GammaWrite},
                    prelude::{
                        BufBitWriter, DeltaRead, DeltaWrite, MemWordReader, len_delta, len_gamma,
                    },
                };
                use rand::Rng;
                use rand::{SeedableRng, rngs::SmallRng};

                let mut buffer_be: Vec<$word> = vec![];
                let mut buffer_le: Vec<$word> = vec![];
                let mut big = BufBitWriter::<BE, _>::new(MemWordWriterVec::new(&mut buffer_be));
                let mut little = BufBitWriter::<LE, _>::new(MemWordWriterVec::new(&mut buffer_le));

                let mut r = SmallRng::seed_from_u64(0);
                const ITER: usize = 1_000_000;

                for _ in 0..ITER {
                    let value = r.random_range(0..128);
                    assert_eq!(big.write_gamma(value)?, len_gamma(value));
                    let value = r.random_range(0..128);
                    assert_eq!(little.write_gamma(value)?, len_gamma(value));
                    let value = r.random_range(0..128);
                    assert_eq!(big.write_gamma(value)?, len_gamma(value));
                    let value = r.random_range(0..128);
                    assert_eq!(little.write_gamma(value)?, len_gamma(value));
                    let value = r.random_range(0..128);
                    assert_eq!(big.write_delta(value)?, len_delta(value));
                    let value = r.random_range(0..128);
                    assert_eq!(little.write_delta(value)?, len_delta(value));
                    let value = r.random_range(0..128);
                    assert_eq!(big.write_delta(value)?, len_delta(value));
                    let value = r.random_range(0..128);
                    assert_eq!(little.write_delta(value)?, len_delta(value));
                    let n_bits = r.random_range(0..=64);
                    if n_bits == 0 {
                        big.write_bits(0, 0)?;
                    } else {
                        big.write_bits(1, n_bits)?;
                    }
                    let n_bits = r.random_range(0..=64);
                    if n_bits == 0 {
                        little.write_bits(0, 0)?;
                    } else {
                        little.write_bits(1, n_bits)?;
                    }
                    let value = r.random_range(0..128);
                    assert_eq!(big.write_unary(value)?, value as usize + 1);
                    let value = r.random_range(0..128);
                    assert_eq!(little.write_unary(value)?, value as usize + 1);
                }

                drop(big);
                drop(little);

                type ReadWord = $word;

                #[allow(clippy::size_of_in_element_count)] // false positive
                let be_trans: &[ReadWord] = unsafe {
                    core::slice::from_raw_parts(
                        buffer_be.as_ptr() as *const ReadWord,
                        buffer_be.len()
                            * (core::mem::size_of::<$word>() / core::mem::size_of::<ReadWord>()),
                    )
                };

                #[allow(clippy::size_of_in_element_count)] // false positive
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
                    assert_eq!(big_buff.read_gamma()?, r.random_range(0..128));
                    assert_eq!(little_buff.read_gamma()?, r.random_range(0..128));
                    assert_eq!(big_buff.read_gamma()?, r.random_range(0..128));
                    assert_eq!(little_buff.read_gamma()?, r.random_range(0..128));
                    assert_eq!(big_buff.read_delta()?, r.random_range(0..128));
                    assert_eq!(little_buff.read_delta()?, r.random_range(0..128));
                    assert_eq!(big_buff.read_delta()?, r.random_range(0..128));
                    assert_eq!(little_buff.read_delta()?, r.random_range(0..128));
                    let n_bits = r.random_range(0..=64);
                    if n_bits == 0 {
                        assert_eq!(big_buff.read_bits(0)?, 0);
                    } else {
                        assert_eq!(big_buff.read_bits(n_bits)?, 1);
                    }
                    let n_bits = r.random_range(0..=64);
                    if n_bits == 0 {
                        assert_eq!(little_buff.read_bits(0)?, 0);
                    } else {
                        assert_eq!(little_buff.read_bits(n_bits)?, 1);
                    }

                    assert_eq!(big_buff.read_unary()?, r.random_range(0..128));
                    assert_eq!(little_buff.read_unary()?, r.random_range(0..128));
                }

                Ok(())
            }
        };
    }

    test_buf_bit_reader!(test_u64, u64);
    test_buf_bit_reader!(test_u32, u32);

    test_buf_bit_reader!(test_u16, u16);
}

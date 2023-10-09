/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use crate::codes::table_params::{DefaultReadParams, ReadParams};
use crate::codes::unary_tables;
use crate::traits::*;
use anyhow::{bail, Context, Result};
use common_traits::*;

/// An implementation of [`BitRead`] and [`BitSeek`] for a
/// [`WordRead`] and a [`WordSeek`].
///
/// Endianness can be selected using the parameter `E`.
///
/// This implementation uses a
/// bit buffer to store bits that are not yet read. The type of the bit buffer
/// is choosable using the type parameter `BB`, but it must at least twice
/// as large as the word type.
///
/// This implementation is usually faster than [`BitReader`](crate::impls::BitReader).
#[derive(Debug)]
pub struct BufBitReader<
    E: Endianness,
    BB: UnsignedInt,
    WR: WordRead,
    RCP: ReadParams = DefaultReadParams,
> {
    /// The [`WordRead`] used to fill the buffer.
    backend: WR,
    /// The bit buffer (at least 2 words) that is used to read the codes. It is never full,
    /// but it may be empty.
    buffer: BB,
    /// Number of bits valid left in the buffer. It is always smaller than `BB::BITS`.
    valid_bits: usize,
    _marker_endianness: core::marker::PhantomData<E>,
    _marker_default_codes: core::marker::PhantomData<RCP>,
}

impl<E: Endianness, BB: UnsignedInt, WR: WordRead + Clone, RCP: ReadParams> core::clone::Clone
    for BufBitReader<E, BB, WR, RCP>
{
    fn clone(&self) -> Self {
        Self {
            backend: self.backend.clone(),
            buffer: self.buffer,
            valid_bits: self.valid_bits,
            _marker_endianness: core::marker::PhantomData,
            _marker_default_codes: core::marker::PhantomData,
        }
    }
}

impl<E: Endianness, BB: UnsignedInt, WR: WordRead, RCP: ReadParams> BufBitReader<E, BB, WR, RCP> {
    /// Create a new [`BufBitReader`] on a generic backend
    ///
    /// ### Example
    /// ```
    /// use dsi_bitstream::prelude::*;
    /// let words: [u64; 1] = [0x0043b59fccf16077];
    /// let word_reader = MemWordReader::new(&words);
    /// let mut bitstream = <BufBitReader<BE, u128, _>>::new(word_reader);
    /// ```
    #[must_use]
    pub fn new(backend: WR) -> Self {
        Self {
            backend,
            buffer: BB::ZERO,
            valid_bits: 0,
            _marker_endianness: core::marker::PhantomData,
            _marker_default_codes: core::marker::PhantomData,
        }
    }
}

impl<BB: UnsignedInt, WR: WordRead, RCP: ReadParams> BufBitReader<BE, BB, WR, RCP>
where
    WR::Word: UpcastableInto<BB>,
{
    /// Ensure that in the buffer there are at least `WR::Word::BITS` bits to read
    /// The user has the responsability of guaranteeing that there are at least
    /// `WR::Word::BITS` free bits in the buffer.
    #[inline(always)]
    fn refill(&mut self) -> Result<()> {
        // if we have 64 valid bits, we don't have space for a new word
        // and by definition we can only read
        let free_bits = BB::BITS - self.valid_bits;
        debug_assert!(free_bits >= WR::Word::BITS);

        let new_word: BB = self
            .backend
            .read_word()
            .with_context(|| "Error while reflling BufBitReader")?
            .to_be()
            .upcast();
        self.valid_bits += WR::Word::BITS;
        self.buffer |= (new_word << (BB::BITS - self.valid_bits - 1)) << 1;
        Ok(())
    }
}

impl<BB: UnsignedInt, WR: WordRead + WordSeek, RCP: ReadParams> BitSeek
    for BufBitReader<BE, BB, WR, RCP>
where
    WR::Word: UpcastableInto<BB>,
{
    #[inline]
    fn get_bit_pos(&self) -> usize {
        self.backend.get_word_pos() * WR::Word::BITS - self.valid_bits
    }

    #[inline]
    fn set_bit_pos(&mut self, bit_index: usize) -> Result<()> {
        self.backend
            .set_word_pos(bit_index / WR::Word::BITS)
            .with_context(|| "BufBitReader was seeking_bit")?;
        let bit_offset = bit_index % WR::Word::BITS;
        self.buffer = BB::ZERO;
        self.valid_bits = 0;
        if bit_offset != 0 {
            let new_word: BB = self.backend.read_word()?.to_be().upcast();
            self.valid_bits = WR::Word::BITS - bit_offset;
            self.buffer = new_word << (BB::BITS - self.valid_bits);
        }
        Ok(())
    }
}

impl<BB: UnsignedInt, WR: WordRead, RCP: ReadParams> BitRead<BE> for BufBitReader<BE, BB, WR, RCP>
where
    BB: DowncastableInto<WR::Word> + CastableInto<u64>,
    WR::Word: UpcastableInto<BB> + UpcastableInto<u64>,
{
    type PeekWord = WR::Word;

    #[inline]
    fn peek_bits(&mut self, n_bits: usize) -> Result<Self::PeekWord> {
        if n_bits > Self::PeekWord::BITS {
            bail!(
                "The n of bits to peek has to be in [0, {}] and {} is not.",
                Self::PeekWord::BITS,
                n_bits
            );
        }
        if n_bits == 0 {
            return Ok(Self::PeekWord::ZERO);
        }
        // a peek can do at most one refill, otherwise we might loose data
        if n_bits > self.valid_bits {
            self.refill()?;
        }

        // read the `n_bits` highest bits of the buffer and shift them to
        // be the lowest
        Ok((self.buffer >> (BB::BITS - n_bits)).downcast())
    }

    #[inline]
    fn skip_bits(&mut self, mut n_bits: usize) -> Result<()> {
        // happy case, just shift the buffer
        if n_bits <= self.valid_bits {
            self.valid_bits -= n_bits;
            self.buffer <<= n_bits;
            return Ok(());
        }

        // clean the buffer data
        n_bits -= self.valid_bits;
        self.valid_bits = 0;
        self.buffer = BB::ZERO;
        // skip words as needed
        while n_bits > WR::Word::BITS {
            let _ = self.backend.read_word()?;
            n_bits -= WR::Word::BITS;
        }
        // read the new word and clear the final bits
        self.refill()?;
        self.valid_bits -= n_bits;
        self.buffer <<= n_bits;

        Ok(())
    }

    #[inline(always)]
    fn skip_bits_after_table_lookup(&mut self, n_bits: usize) -> Result<()> {
        self.valid_bits -= n_bits;
        self.buffer <<= n_bits;
        Ok(())
    }

    #[inline]
    fn read_bits(&mut self, mut n_bits: usize) -> Result<u64> {
        debug_assert!(self.valid_bits < BB::BITS);

        // most common path, we just read the buffer
        if n_bits <= self.valid_bits {
            // Valid right shift of BB::BITS - n_bits, even when n_bits is zero
            let result: u64 = (self.buffer >> (BB::BITS - n_bits - 1) >> 1_u32).cast();
            self.valid_bits -= n_bits;
            self.buffer <<= n_bits;
            return Ok(result);
        }

        if n_bits > 64 {
            bail!(
                "The n of bits to peek has to be in [0, 64] and {} is not.",
                n_bits
            );
        }

        let mut result: u64 = (self.buffer >> (BB::BITS - 1 - self.valid_bits) >> 1_u8).cast();
        n_bits -= self.valid_bits;

        // Directly read to the result without updating the buffer
        while n_bits > WR::Word::BITS {
            let new_word: u64 = self.backend.read_word()?.to_be().upcast();
            result = (result << WR::Word::BITS) | new_word;
            n_bits -= WR::Word::BITS;
        }
        // get the final word
        let new_word = self.backend.read_word()?.to_be();
        self.valid_bits = WR::Word::BITS - n_bits;
        // compose the remaining bits
        let upcasted: u64 = new_word.upcast();
        let final_bits: u64 = (upcasted >> self.valid_bits).downcast();
        result = (result << n_bits - 1 << 1) | final_bits;
        // and put the rest in the buffer
        self.buffer = new_word.upcast();
        self.buffer = (self.buffer << (BB::BITS - self.valid_bits - 1)) << 1;

        Ok(result)
    }

    #[inline]
    fn read_unary_param<const USE_TABLE: bool>(&mut self) -> Result<u64> {
        if USE_TABLE {
            if let Some((res, _)) = unary_tables::read_table_be(self)? {
                return Ok(res);
            }
        }
        let mut result: u64 = 0;
        loop {
            // count the zeros from the left
            let zeros: usize = self.buffer.leading_zeros() as usize;

            // if we encountered an 1 in the valid_bits we can return
            if zeros < self.valid_bits {
                result += zeros as u64;
                self.buffer = self.buffer << zeros << 1;
                self.valid_bits -= zeros + 1;
                return Ok(result);
            }

            result += self.valid_bits as u64;

            // otherwise we didn't encounter the ending 1 yet so we need to
            // refill and iter again
            let new_word: BB = self.backend.read_word()?.to_be().upcast();
            self.valid_bits = WR::Word::BITS;
            self.buffer = new_word << (BB::BITS - WR::Word::BITS);
        }
    }
}

impl<BB: UnsignedInt, WR: WordRead, RCP: ReadParams> BufBitReader<LE, BB, WR, RCP>
where
    WR::Word: UpcastableInto<BB>,
{
    /// Ensure that in the buffer there are at least `WR::Word::BITS` bits to read
    /// The user has the responsability of guaranteeing that there are at least
    /// `WR::Word::BITS` free bits in the buffer.
    #[inline(always)]
    fn refill(&mut self) -> Result<()> {
        // if we have 64 valid bits, we don't have space for a new word
        // and by definition we can only read
        let free_bits = BB::BITS - self.valid_bits;
        debug_assert!(free_bits >= WR::Word::BITS);

        let new_word: BB = self
            .backend
            .read_word()
            .with_context(|| "Error while reflling BufBitReader")?
            .to_le()
            .upcast();
        self.buffer |= new_word << self.valid_bits;
        self.valid_bits += WR::Word::BITS;
        Ok(())
    }
}

impl<BB: UnsignedInt, WR: WordRead + WordSeek, RCP: ReadParams> BitSeek
    for BufBitReader<LE, BB, WR, RCP>
where
    WR::Word: UpcastableInto<BB>,
{
    #[inline]
    fn get_bit_pos(&self) -> usize {
        self.backend.get_word_pos() * WR::Word::BITS - self.valid_bits
    }

    #[inline]
    fn set_bit_pos(&mut self, bit_index: usize) -> Result<()> {
        self.backend
            .set_word_pos(bit_index / WR::Word::BITS)
            .with_context(|| "BufBitReader was seeking_bit")?;
        let bit_offset = bit_index % WR::Word::BITS;
        self.buffer = BB::ZERO;
        self.valid_bits = 0;
        if bit_offset != 0 {
            let new_word: BB = self.backend.read_word()?.to_le().upcast();
            self.valid_bits = WR::Word::BITS - bit_offset;
            self.buffer = new_word >> bit_offset;
        }
        Ok(())
    }
}

impl<BB: UnsignedInt, WR: WordRead, RCP: ReadParams> BitRead<LE> for BufBitReader<LE, BB, WR, RCP>
where
    BB: DowncastableInto<WR::Word> + CastableInto<u64>,
    WR::Word: UpcastableInto<BB> + UpcastableInto<u64>,
{
    type PeekWord = WR::Word;

    #[inline]
    fn skip_bits(&mut self, mut n_bits: usize) -> Result<()> {
        // happy case, just shift the buffer
        if n_bits <= self.valid_bits {
            self.valid_bits -= n_bits;
            self.buffer >>= n_bits;
            return Ok(());
        }

        // clean the buffer data
        n_bits -= self.valid_bits;
        self.valid_bits = 0;
        self.buffer = BB::ZERO;
        // skip words as needed
        while n_bits > WR::Word::BITS {
            let _ = self.backend.read_word()?;
            n_bits -= WR::Word::BITS;
        }
        // read the new word and clear the final bits
        self.refill()?;
        self.valid_bits -= n_bits;
        self.buffer >>= n_bits;

        Ok(())
    }

    #[inline(always)]
    fn skip_bits_after_table_lookup(&mut self, n_bits: usize) -> Result<()> {
        self.valid_bits -= n_bits;
        self.buffer >>= n_bits;
        Ok(())
    }

    #[inline]
    fn read_bits(&mut self, mut n_bits: usize) -> Result<u64> {
        debug_assert!(self.valid_bits < BB::BITS);

        // most common path, we just read the buffer
        if n_bits <= self.valid_bits {
            let result: u64 = (self.buffer & ((BB::ONE << n_bits) - BB::ONE)).cast();
            self.valid_bits -= n_bits;
            self.buffer >>= n_bits;
            return Ok(result);
        }

        if n_bits > 64 {
            bail!(
                "The n of bits to peek has to be in [0, 64] and {} is not.",
                n_bits
            );
        }

        let mut result: u64 = self.buffer.cast();
        let mut bits_in_res = self.valid_bits;

        // Directly read to the result without updating the buffer
        while n_bits > WR::Word::BITS + bits_in_res {
            let new_word: u64 = self.backend.read_word()?.to_le().upcast();
            result |= new_word << bits_in_res;
            bits_in_res += WR::Word::BITS;
        }

        // get the final word
        n_bits -= bits_in_res;
        let new_word = self.backend.read_word()?.to_le();
        self.valid_bits = WR::Word::BITS - n_bits;
        // compose the remaining bits
        let shamt = 64 - n_bits;
        let upcasted: u64 = new_word.upcast();
        let final_bits: u64 = ((upcasted << shamt) >> shamt).downcast();
        result |= final_bits << bits_in_res;
        // and put the rest in the buffer
        self.buffer = new_word.upcast();
        // TODO: n_bits might be equal to buffer size (?!?)
        self.buffer = self.buffer >> n_bits - 1 >> 1;

        Ok(result)
    }

    #[inline]
    fn peek_bits(&mut self, n_bits: usize) -> Result<Self::PeekWord> {
        if n_bits > Self::PeekWord::BITS {
            bail!(
                "The n of bits to peek has to be in [0, {}] and {} is not.",
                Self::PeekWord::BITS,
                n_bits
            );
        }
        if n_bits == 0 {
            return Ok(Self::PeekWord::ZERO);
        }
        // a peek can do at most one refill, otherwise we might loose data
        if n_bits > self.valid_bits {
            self.refill()?;
        }

        // read the `n_bits` highest bits of the buffer and shift them to
        // be the lowest
        let shamt = BB::BITS - n_bits;
        Ok(((self.buffer << shamt) >> shamt).downcast())
    }

    #[inline]
    fn read_unary_param<const USE_TABLE: bool>(&mut self) -> Result<u64> {
        if USE_TABLE {
            if let Some((res, _)) = unary_tables::read_table_le(self)? {
                return Ok(res);
            }
        }
        let mut result: u64 = 0;
        loop {
            // count the zeros from the left
            let zeros: usize = self.buffer.trailing_zeros() as usize;

            // if we encountered an 1 in the valid_bits we can return
            if zeros < self.valid_bits {
                result += zeros as u64;
                self.buffer = self.buffer >> zeros >> 1;
                self.valid_bits -= zeros + 1;
                return Ok(result);
            }

            result += self.valid_bits as u64;

            // otherwise we didn't encounter the ending 1 yet so we need to
            // refill and iter again
            let new_word: BB = self.backend.read_word()?.to_le().upcast();
            self.valid_bits = WR::Word::BITS;
            self.buffer = new_word;
        }
    }
}

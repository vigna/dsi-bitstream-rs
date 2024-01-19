/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use core::convert::Infallible;
use std::error::Error;

use crate::codes::params::{DefaultReadParams, ReadParams};
use crate::codes::unary_tables;
use crate::traits::*;

/// An implementation of [`BitRead`] for a [`WordRead`] with word `u64` and of
/// [`BitSeek`] for a [`WordSeek`].
///
/// This implementation accesses randomly the underlying [`WordRead`] without
/// any buffering. It is usually slower than
/// [`BufBitReader`](crate::impls::BufBitReader).
///
/// The peek word is `u32`. The value returned by
/// [`peek_bits`](crate::traits::BitRead::peek_bits) contains at least 32 bits
/// (extended with zeros beyond end of stream), that is, a full peek word.
///
/// The additional type parameter `RP` is used to select the parameters for the
/// instantanous codes, but the casual user should be happy with the default
/// value. See [`ReadParams`] for more details.

#[derive(Debug, Clone)]
pub struct BitReader<E: Endianness, WR, RP: ReadParams = DefaultReadParams> {
    /// The stream which we will read words from.
    data: WR,
    /// The index of the current bit.
    bit_index: u64,
    _marker: core::marker::PhantomData<(E, RP)>,
}

impl<E: Endianness, WR, RP: ReadParams> BitReader<E, WR, RP> {
    pub fn new(data: WR) -> Self {
        check_tables(32);
        Self {
            data,
            bit_index: 0,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<
        E: Error + Send + Sync + 'static,
        WR: WordRead<Error = E, Word = u64> + WordSeek<Error = E>,
        RP: ReadParams,
    > BitRead<BE> for BitReader<BE, WR, RP>
{
    type Error = <WR as WordRead>::Error;
    type PeekWord = u32;

    #[inline]
    fn skip_bits(&mut self, n_bits: usize) -> Result<(), Self::Error> {
        self.bit_index += n_bits as u64;
        Ok(())
    }

    #[inline]
    fn read_bits(&mut self, n_bits: usize) -> Result<u64, Self::Error> {
        if n_bits == 0 {
            return Ok(0);
        }

        assert!(n_bits <= 64);

        self.data.set_word_pos(self.bit_index / 64)?;
        let in_word_offset = (self.bit_index % 64) as usize;

        let res = if (in_word_offset + n_bits) <= 64 {
            // single word access
            let word = self.data.read_word()?.to_be();
            (word << in_word_offset) >> (64 - n_bits)
        } else {
            // double word access
            let high_word = self.data.read_word()?.to_be();
            let low_word = self.data.read_word()?.to_be();
            let shamt1 = 64 - n_bits;
            let shamt2 = 128 - in_word_offset - n_bits;
            ((high_word << in_word_offset) >> shamt1) | (low_word >> shamt2)
        };
        self.bit_index += n_bits as u64;
        Ok(res)
    }

    #[inline]
    fn peek_bits(&mut self, n_bits: usize) -> Result<u32, Self::Error> {
        if n_bits == 0 {
            return Ok(0);
        }

        assert!(n_bits <= 32);

        self.data.set_word_pos(self.bit_index / 64)?;
        let in_word_offset = (self.bit_index % 64) as usize;

        let res = if (in_word_offset + n_bits) <= 64 {
            // single word access
            let word = self.data.read_word()?.to_be();
            (word << in_word_offset) >> (64 - n_bits)
        } else {
            // double word access
            let high_word = self.data.read_word()?.to_be();
            let low_word = self.data.read_word()?.to_be();
            let shamt1 = 64 - n_bits;
            let shamt2 = 128 - in_word_offset - n_bits;
            ((high_word << in_word_offset) >> shamt1) | (low_word >> shamt2)
        };
        Ok(res as u32)
    }

    #[inline]
    fn read_unary_param<const USE_TABLE: bool>(&mut self) -> Result<u64, Self::Error> {
        if USE_TABLE {
            if let Some((res, _)) = unary_tables::read_table_be(self) {
                return Ok(res);
            }
        }
        self.data.set_word_pos(self.bit_index / 64)?;
        let in_word_offset = self.bit_index % 64;
        let mut bits_in_word = 64 - in_word_offset;
        let mut total = 0;

        let mut word = self.data.read_word()?.to_be();
        word <<= in_word_offset;
        loop {
            let zeros = word.leading_zeros() as u64;
            // the unary code fits in the word
            if zeros < bits_in_word {
                self.bit_index += total + zeros + 1;
                return Ok(total + zeros);
            }
            total += bits_in_word;
            bits_in_word = 64;
            word = self.data.read_word()?.to_be();
        }
    }

    fn skip_bits_after_table_lookup(&mut self, n: usize) {
        self.bit_index += n as u64;
    }
}

impl<WR: WordSeek, RP: ReadParams> BitSeek for BitReader<LE, WR, RP> {
    type Error = Infallible;

    fn get_bit_pos(&mut self) -> Result<u64, Self::Error> {
        Ok(self.bit_index)
    }

    fn set_bit_pos(&mut self, bit_index: u64) -> Result<(), Self::Error> {
        self.bit_index = bit_index;
        Ok(())
    }
}

impl<WR: WordSeek, RP: ReadParams> BitSeek for BitReader<BE, WR, RP> {
    type Error = Infallible;

    fn get_bit_pos(&mut self) -> Result<u64, Self::Error> {
        Ok(self.bit_index)
    }

    fn set_bit_pos(&mut self, bit_index: u64) -> Result<(), Self::Error> {
        self.bit_index = bit_index;
        Ok(())
    }
}

impl<
        E: Error + Send + Sync + 'static,
        WR: WordRead<Error = E, Word = u64> + WordSeek<Error = E>,
        RP: ReadParams,
    > BitRead<LE> for BitReader<LE, WR, RP>
{
    type Error = <WR as WordRead>::Error;
    type PeekWord = u32;

    #[inline]
    fn skip_bits(&mut self, n_bits: usize) -> Result<(), Self::Error> {
        self.bit_index += n_bits as u64;
        Ok(())
    }

    #[inline]
    fn read_bits(&mut self, n_bits: usize) -> Result<u64, Self::Error> {
        #[cfg(feature = "checks")]
        assert!(n_bits <= 64);

        if n_bits == 0 {
            return Ok(0);
        }

        self.data.set_word_pos(self.bit_index / 64)?;
        let in_word_offset = (self.bit_index % 64) as usize;

        let res = if (in_word_offset + n_bits) <= 64 {
            // single word access
            let word = self.data.read_word()?.to_le();
            let shamt = 64 - n_bits;
            (word << (shamt - in_word_offset)) >> shamt
        } else {
            // double word access
            let low_word = self.data.read_word()?.to_le();
            let high_word = self.data.read_word()?.to_le();
            let shamt1 = 128 - in_word_offset - n_bits;
            let shamt2 = 64 - n_bits;
            ((high_word << shamt1) >> shamt2) | (low_word >> in_word_offset)
        };
        self.bit_index += n_bits as u64;
        Ok(res)
    }

    #[inline]
    fn peek_bits(&mut self, n_bits: usize) -> Result<u32, Self::Error> {
        if n_bits == 0 {
            return Ok(0);
        }

        assert!(n_bits <= 32);

        self.data.set_word_pos(self.bit_index / 64)?;
        let in_word_offset = (self.bit_index % 64) as usize;

        let res = if (in_word_offset + n_bits) <= 64 {
            // single word access
            let word = self.data.read_word()?.to_le();
            let shamt = 64 - n_bits;
            (word << (shamt - in_word_offset)) >> shamt
        } else {
            // double word access
            let low_word = self.data.read_word()?.to_le();
            let high_word = self.data.read_word()?.to_le();
            let shamt1 = 128 - in_word_offset - n_bits;
            let shamt2 = 64 - n_bits;
            ((high_word << shamt1) >> shamt2) | (low_word >> in_word_offset)
        };
        Ok(res as u32)
    }

    #[inline]
    fn read_unary_param<const USE_TABLE: bool>(&mut self) -> Result<u64, Self::Error> {
        if USE_TABLE {
            if let Some((res, _)) = unary_tables::read_table_le(self) {
                return Ok(res);
            }
        }
        self.data.set_word_pos(self.bit_index / 64)?;
        let in_word_offset = self.bit_index % 64;
        let mut bits_in_word = 64 - in_word_offset;
        let mut total = 0;

        let mut word = self.data.read_word()?.to_le();
        word >>= in_word_offset;
        loop {
            let zeros = word.trailing_zeros() as u64;
            // the unary code fits in the word
            if zeros < bits_in_word {
                self.bit_index += total + zeros + 1;
                return Ok(total + zeros);
            }
            total += bits_in_word;
            bits_in_word = 64;
            word = self.data.read_word()?.to_le();
        }
    }

    fn skip_bits_after_table_lookup(&mut self, n: usize) {
        self.bit_index += n as u64;
    }
}

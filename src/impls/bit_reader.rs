/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use core::convert::Infallible;
#[cfg(feature = "mem_dbg")]
use mem_dbg::{MemDbg, MemSize};

use crate::codes::params::{DefaultReadParams, ReadParams};
use crate::traits::*;

/// An implementation of [`BitRead`] for a [`WordRead`] with word `u64` and of
/// [`BitSeek`] for a [`WordSeek`].
///
/// This implementation randomly accesses the underlying [`WordRead`] without
/// any buffering. It is usually slower than
/// [`BufBitReader`](crate::impls::BufBitReader).
///
/// The peek word is `u32`. The value returned by
/// [`peek_bits`](crate::traits::BitRead::peek_bits) contains at least 32 bits
/// (extended with zeros beyond end of stream), that is, a full peek word.
///
/// The additional type parameter `RP` is used to select the parameters for the
/// instantaneous codes, but the casual user should be happy with the default
/// value. See [`ReadParams`] for more details.
///
/// For additional flexibility, when the `std` feature is enabled, this
/// structure implements [`std::io::Read`]. Note that because of coherence
/// rules it is not possible to implement [`std::io::Read`] for a generic
/// [`BitRead`].

#[derive(Debug, Clone)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
pub struct BitReader<E: Endianness, WR, RP: ReadParams = DefaultReadParams> {
    /// The backend from which we will read words.
    backend: WR,
    /// The index of the current bit.
    bit_index: u64,
    _marker: core::marker::PhantomData<(E, RP)>,
}

impl<E: Endianness, WR, RP: ReadParams> BitReader<E, WR, RP> {
    /// Creates a new [`BitReader`] with the given word reader.
    #[must_use]
    pub const fn new(backend: WR) -> Self {
        Self {
            backend,
            bit_index: 0,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<WR: WordRead<Word = u64> + WordSeek<Error = <WR as WordRead>::Error>, RP: ReadParams>
    BitRead<BE> for BitReader<BE, WR, RP>
{
    type Error = <WR as WordRead>::Error;
    type PeekWord = u32;
    const PEEK_BITS: usize = 32;

    #[inline]
    fn skip_bits(&mut self, n_bits: usize) -> Result<(), Self::Error> {
        self.bit_index += n_bits as u64;
        Ok(())
    }

    #[inline]
    fn read_bits(&mut self, num_bits: usize) -> Result<u64, Self::Error> {
        debug_assert!(num_bits <= 64);
        #[cfg(feature = "checks")]
        assert!(num_bits <= 64);

        if num_bits == 0 {
            return Ok(0);
        }

        self.backend.set_word_pos(self.bit_index / 64)?;
        let in_word_offset = (self.bit_index % 64) as usize;

        let res = if (in_word_offset + num_bits) <= 64 {
            // single word access
            let word = self.backend.read_word()?.to_be();
            (word << in_word_offset) >> (64 - num_bits)
        } else {
            // double word access
            let high_word = self.backend.read_word()?.to_be();
            let low_word = self.backend.read_word()?.to_be();
            let shamt1 = 64 - num_bits;
            let shamt2 = 128 - in_word_offset - num_bits;
            ((high_word << in_word_offset) >> shamt1) | (low_word >> shamt2)
        };
        self.bit_index += num_bits as u64;
        Ok(res)
    }

    #[inline]
    fn peek_bits(&mut self, n_bits: usize) -> Result<u32, Self::Error> {
        if n_bits == 0 {
            return Ok(0);
        }

        #[cfg(feature = "checks")]
        assert!(n_bits <= 32);

        self.backend.set_word_pos(self.bit_index / 64)?;
        let in_word_offset = (self.bit_index % 64) as usize;

        let res = if (in_word_offset + n_bits) <= 64 {
            // single word access
            let word = self.backend.read_word()?.to_be();
            (word << in_word_offset) >> (64 - n_bits)
        } else {
            // double word access
            let high_word = self.backend.read_word()?.to_be();
            let low_word = self.backend.read_word()?.to_be();
            let shamt1 = 64 - n_bits;
            let shamt2 = 128 - in_word_offset - n_bits;
            ((high_word << in_word_offset) >> shamt1) | (low_word >> shamt2)
        };
        Ok(res as u32)
    }

    #[inline]
    fn read_unary(&mut self) -> Result<u64, Self::Error> {
        self.backend.set_word_pos(self.bit_index / 64)?;
        let in_word_offset = self.bit_index % 64;
        let mut bits_in_word = 64 - in_word_offset;
        let mut total = 0;

        let mut word = self.backend.read_word()?.to_be();
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
            word = self.backend.read_word()?.to_be();
        }
    }

    #[inline(always)]
    fn skip_bits_after_peek(&mut self, n: usize) {
        self.bit_index += n as u64;
    }
}

impl<E: Endianness, WR: WordSeek, RP: ReadParams> BitSeek for BitReader<E, WR, RP> {
    type Error = Infallible;

    fn bit_pos(&mut self) -> Result<u64, Self::Error> {
        Ok(self.bit_index)
    }

    fn set_bit_pos(&mut self, bit_index: u64) -> Result<(), Self::Error> {
        self.bit_index = bit_index;
        Ok(())
    }
}

impl<WR: WordRead<Word = u64> + WordSeek<Error = <WR as WordRead>::Error>, RP: ReadParams>
    BitRead<LE> for BitReader<LE, WR, RP>
{
    type Error = <WR as WordRead>::Error;
    type PeekWord = u32;
    const PEEK_BITS: usize = 32;

    #[inline]
    fn skip_bits(&mut self, n_bits: usize) -> Result<(), Self::Error> {
        self.bit_index += n_bits as u64;
        Ok(())
    }

    #[inline]
    fn read_bits(&mut self, num_bits: usize) -> Result<u64, Self::Error> {
        #[cfg(feature = "checks")]
        assert!(num_bits <= 64);

        if num_bits == 0 {
            return Ok(0);
        }

        self.backend.set_word_pos(self.bit_index / 64)?;
        let in_word_offset = (self.bit_index % 64) as usize;

        let res = if (in_word_offset + num_bits) <= 64 {
            // single word access
            let word = self.backend.read_word()?.to_le();
            let shamt = 64 - num_bits;
            (word << (shamt - in_word_offset)) >> shamt
        } else {
            // double word access
            let low_word = self.backend.read_word()?.to_le();
            let high_word = self.backend.read_word()?.to_le();
            let shamt1 = 128 - in_word_offset - num_bits;
            let shamt2 = 64 - num_bits;
            ((high_word << shamt1) >> shamt2) | (low_word >> in_word_offset)
        };
        self.bit_index += num_bits as u64;
        Ok(res)
    }

    #[inline]
    fn peek_bits(&mut self, n_bits: usize) -> Result<u32, Self::Error> {
        if n_bits == 0 {
            return Ok(0);
        }

        #[cfg(feature = "checks")]
        assert!(n_bits <= 32);

        self.backend.set_word_pos(self.bit_index / 64)?;
        let in_word_offset = (self.bit_index % 64) as usize;

        let res = if (in_word_offset + n_bits) <= 64 {
            // single word access
            let word = self.backend.read_word()?.to_le();
            let shamt = 64 - n_bits;
            (word << (shamt - in_word_offset)) >> shamt
        } else {
            // double word access
            let low_word = self.backend.read_word()?.to_le();
            let high_word = self.backend.read_word()?.to_le();
            let shamt1 = 128 - in_word_offset - n_bits;
            let shamt2 = 64 - n_bits;
            ((high_word << shamt1) >> shamt2) | (low_word >> in_word_offset)
        };
        Ok(res as u32)
    }

    #[inline]
    fn read_unary(&mut self) -> Result<u64, Self::Error> {
        self.backend.set_word_pos(self.bit_index / 64)?;
        let in_word_offset = self.bit_index % 64;
        let mut bits_in_word = 64 - in_word_offset;
        let mut total = 0;

        let mut word = self.backend.read_word()?.to_le();
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
            word = self.backend.read_word()?.to_le();
        }
    }

    #[inline(always)]
    fn skip_bits_after_peek(&mut self, n: usize) {
        self.bit_index += n as u64;
    }
}

#[cfg(feature = "std")]
impl<WR: WordRead<Word = u64> + WordSeek<Error = <WR as WordRead>::Error>, RP: ReadParams>
    std::io::Read for BitReader<LE, WR, RP>
{
    /// Note that this implementation does not use `Ok(0)` to signal
    /// stream exhaustion (an empty `buf` still yields `Ok(0)` as required):
    /// the underlying [`WordRead`] error type cannot distinguish end of
    /// stream from a backend failure, so reading past the last available
    /// byte fails with
    /// [`std::io::ErrorKind::UnexpectedEof`] rather than silently reporting
    /// end of file (which would make a genuine backend error look like a
    /// clean EOF). To read a stream of known length to its end, bound the
    /// reader with [`std::io::Read::take`] or use exact-length reads.
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // Read byte by byte so the returned count reflects exactly the bytes
        // produced (read_bits(64) is not atomic on a narrow-word backend near
        // EOF); the reader's endianness handles bit order.
        for (i, slot) in buf.iter_mut().enumerate() {
            match self.read_bits(8) {
                Ok(byte) => {
                    debug_assert!(byte < 256, "read_bits(8) yields at most 8 bits");
                    *slot = byte as u8;
                }
                Err(_) if i > 0 => return Ok(i),
                Err(_) => return Err(std::io::ErrorKind::UnexpectedEof.into()),
            }
        }
        Ok(buf.len())
    }
}

#[cfg(feature = "std")]
impl<WR: WordRead<Word = u64> + WordSeek<Error = <WR as WordRead>::Error>, RP: ReadParams>
    std::io::Read for BitReader<BE, WR, RP>
{
    /// Does not use `Ok(0)` to signal stream exhaustion; see the
    /// little-endian implementation.
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // See the little-endian impl.
        for (i, slot) in buf.iter_mut().enumerate() {
            match self.read_bits(8) {
                Ok(byte) => {
                    debug_assert!(byte < 256, "read_bits(8) yields at most 8 bits");
                    *slot = byte as u8;
                }
                Err(_) if i > 0 => return Ok(i),
                Err(_) => return Err(std::io::ErrorKind::UnexpectedEof.into()),
            }
        }
        Ok(buf.len())
    }
}

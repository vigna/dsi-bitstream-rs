/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use core::error::Error;
use core::fmt::{Display, Formatter};

use crate::traits::*;
use num_traits::AsPrimitive;

/// The error returned by the bit copy methods [`BitRead::copy_to`]
/// and [`BitWrite::copy_from`].
///
/// It can be a read or a write error, depending on which stream (source or
/// destination) generated the error.
#[derive(Debug, Clone)]
pub enum CopyError<RE: Error + Send + Sync + 'static, WE: Error + Send + Sync + 'static> {
    ReadError(RE),
    WriteError(WE),
}

impl<RE: Error + Send + Sync + 'static, WE: Error + Send + Sync + 'static> Display
    for CopyError<RE, WE>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            CopyError::ReadError(e) => write!(f, "read error while copying: {}", e),
            CopyError::WriteError(e) => write!(f, "write error while copying: {}", e),
        }
    }
}

impl<RE: Error + Send + Sync + 'static, WE: Error + Send + Sync + 'static> Error
    for CopyError<RE, WE>
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CopyError::ReadError(e) => Some(e),
            CopyError::WriteError(e) => Some(e),
        }
    }
}

/// Sequential, streaming bit-by-bit reads.
///
/// This trait specifies basic operations over which codes can be implemented by
/// traits such as [`GammaReadParam`](crate::codes::GammaReadParam).
///
/// To read quickly complex codes, such traits may use the
/// [`peek_bits`](BitRead::peek_bits) method to read a few bits in advance and
/// then use a table to decode them. For this to happen correctly,
/// [`peek_bits`](BitRead::peek_bits) must return a sufficient number of bits.
/// Each table module provides a `check_read_table` const fn that can be used
/// in a `const { }` block to verify at compile time that the peek word is
/// large enough.
///
/// Please see the documentation of the [`impls`](crate::impls) module for more
/// details.
pub trait BitRead<E: Endianness> {
    type Error: Error + Send + Sync + 'static;

    /// The type we can read from the stream without advancing.
    type PeekWord: AsPrimitive<u64>;

    /// The number of bits that [`peek_bits`](BitRead::peek_bits) is guaranteed
    /// to return successfully (with zero-extended EOF).
    const PEEK_BITS: usize;

    /// Reads `n` bits and returns them in the lowest bits.
    ///
    /// Implementors should check the value of `n` when in test mode
    /// and panic if it is greater than 64.
    fn read_bits(&mut self, n: usize) -> Result<u64, Self::Error>;

    /// Peeks at `n` bits without advancing the stream position.
    /// `n` must be nonzero, and at most `PeekWord::BITS`.
    fn peek_bits(&mut self, n: usize) -> Result<Self::PeekWord, Self::Error>;

    /// Skip `n` bits from the stream.
    ///
    /// When moving forward by a small amount of bits, this method might be
    /// more efficient than [`BitSeek::set_bit_pos`].
    fn skip_bits(&mut self, n: usize) -> Result<(), Self::Error>;

    #[doc(hidden)]
    /// Skip bits from the stream after a call to [`BitRead::peek_bits`].
    ///
    /// This is an internal optimization used to skip bits we know
    /// are already in some internal buffer as we [peeked](BitRead::peek_bits)
    /// at them. Please don't use.
    fn skip_bits_after_peek(&mut self, n: usize);

    /// Reads a unary code.
    ///
    /// Implementations are required to support the range [0 . . 2⁶⁴ – 1).
    fn read_unary(&mut self) -> Result<u64, Self::Error>;

    fn copy_to<F: Endianness, W: BitWrite<F>>(
        &mut self,
        bit_write: &mut W,
        mut n: u64,
    ) -> Result<(), CopyError<Self::Error, W::Error>> {
        while n > 0 {
            let to_read = core::cmp::min(n, 64) as usize;
            let read = self.read_bits(to_read).map_err(CopyError::ReadError)?;
            bit_write
                .write_bits(read, to_read)
                .map_err(CopyError::WriteError)?;
            n -= to_read as u64;
        }
        Ok(())
    }
}

/// Sequential, streaming bit-by-bit writes.
///
/// This trait specifies basic operations over which codes can be implemented
/// by traits such as [`crate::codes::GammaWriteParam`].
pub trait BitWrite<E: Endianness> {
    type Error: Error + Send + Sync + 'static;

    /// Writes the lowest `n` bits of `value` to the stream and
    /// returns the number of bits written, that is, `n`.
    ///
    /// Implementors should check the value of `n` in test mode and panic if it
    /// is greater than 64. Moreover, if the feature `checks` is enabled they
    /// should check that the remaining bits of `value` are zero.
    fn write_bits(&mut self, value: u64, n: usize) -> Result<usize, Self::Error>;

    /// Writes `value` as a unary code to the stream and returns the number of
    /// bits written, that is, `value` plus one.
    ///
    /// Implementations are required to support the range [0 . . 2⁶⁴ – 1).
    fn write_unary(&mut self, value: u64) -> Result<usize, Self::Error>;

    /// Flush the buffer, consuming the bit stream.
    ///
    /// Returns the number of bits written from the bit buffer (not
    /// including padding).
    fn flush(&mut self) -> Result<usize, Self::Error>;

    fn copy_from<F: Endianness, R: BitRead<F>>(
        &mut self,
        bit_read: &mut R,
        mut n: u64,
    ) -> Result<(), CopyError<R::Error, Self::Error>> {
        while n > 0 {
            let to_read = core::cmp::min(n, 64) as usize;
            let read = bit_read.read_bits(to_read).map_err(CopyError::ReadError)?;
            self.write_bits(read, to_read)
                .map_err(CopyError::WriteError)?;
            n -= to_read as u64;
        }
        Ok(())
    }
}

/// Seekability for [`BitRead`] and [`BitWrite`] streams.
pub trait BitSeek {
    type Error: Error + Send + Sync + 'static;
    /// Gets the current position in bits from the start of the stream.
    fn bit_pos(&mut self) -> Result<u64, Self::Error>;

    /// Sets the current position in bits from the start of the
    /// stream to `bit_pos`.
    ///
    /// Note that moving forward by a small amount of bits may be accomplished
    /// more efficiently by calling [`BitRead::skip_bits`].
    fn set_bit_pos(&mut self, bit_pos: u64) -> Result<(), Self::Error>;
}

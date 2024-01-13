/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use std::error::Error;

use crate::traits::*;
use common_traits::UpcastableInto;

/// Sequential, streaming bit-by-bit reads.
///
/// This trait specify basic operation over which codes can be implemented by
/// traits such as [`crate::codes::GammaReadParam`].
pub trait BitRead<E: Endianness> {
    type Error: Error;

    /// The type we can read from the stream without advancing.
    type PeekWord: UpcastableInto<u64>;

    /// Read `n` bits and return them in the lowest bits.
    fn read_bits(&mut self, n: usize) -> Result<u64, Self::Error>;

    /// Peeks at `n` bits without advancing the stream position.
    /// `n` must be nonzero, and at most `PeekWord::BITS`.
    fn peek_bits(&mut self, n: usize) -> Result<Self::PeekWord, Self::Error>;

    /// Skip `n` bits from the stream.
    fn skip_bits(&mut self, n: usize) -> Result<(), Self::Error>;

    #[doc(hidden)]
    /// Skip bits form the stream after a call to [`BitRead::peek_bits`].
    ///
    /// This is an internal optimization used to skip bits we know
    /// are already in some internal buffer as we [peeked](BitRead::peek_bits)
    /// at them. Please don't use.
    fn skip_bits_after_table_lookup(&mut self, n: usize);

    /// Read a unary code.
    ///
    /// This version of the method has a constant parameter
    /// deciding whether to use a decoding table. You should rather use
    /// [`BitRead::read_unary`], which uses the default
    /// choice of the implementing type.
    fn read_unary_param<const USE_TABLE: bool>(&mut self) -> Result<u64, Self::Error>;

    /// Read a unary code.
    ///
    /// This version of the method uses the version of
    /// of [`BitRead::read_unary_param`] selected as default by
    /// the implementing type. The default implementation does
    /// not use a table.
    #[inline(always)]
    fn read_unary(&mut self) -> Result<u64, Self::Error> {
        self.read_unary_param::<false>()
    }

    /// Skip a unary code.
    #[inline(always)]
    fn skip_unary(&mut self) -> Result<(), Self::Error> {
        self.read_unary()?;
        Ok(())
    }
}

/// Sequential, streaming bit-by-bit writes.
///
/// This trait specify basic operation over which codes can be implemented
/// by traits such as [`crate::codes::GammaWriteParam`].
pub trait BitWrite<E: Endianness> {
    type Error: Error;

    /// Write the lowest `n` bits of `value` to the stream and return the number of
    /// bits written, that is, `n`.
    ///
    /// The remaining bits must be ignored.
    fn write_bits(&mut self, value: u64, n: usize) -> Result<usize, Self::Error>;

    /// Write `value` as a unary code to the stream and return the number of
    /// bits written, that is, `value` plus one.
    ///     
    /// This version of the method has a constant parameter
    /// deciding whether to use an encoding table. You should rather use
    /// [`BitWrite::write_unary`], which uses the default
    /// choice of the implementing type.
    fn write_unary_param<const USE_TABLE: bool>(
        &mut self,
        value: u64,
    ) -> Result<usize, Self::Error>;

    /// Write `value` as a unary code to the stream and return the number of
    /// bits written, that is, `value` plus one.
    ///
    /// This version of the method uses the version of
    /// of [`BitWrite::write_unary_param`] selected as default by
    /// the implementing type. The default implementation
    /// uses a table.
    fn write_unary(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.write_unary_param::<true>(value)
    }

    /// Flush the buffer, consuming the bit stream.
    fn flush(self) -> Result<(), Self::Error>;
}

/// Seekability for [`BitRead`] and [`BitWrite`] streams.
pub trait BitSeek {
    type Error: Error;
    /// Get the current position in bits from the start of the file.
    fn get_bit_pos(&mut self) -> Result<u64, Self::Error>;

    /// Set the current position in bits from the start of the file to `bit_pos`.
    fn set_bit_pos(&mut self, bit_pos: u64) -> Result<(), Self::Error>;
}

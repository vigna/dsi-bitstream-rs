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
    /// On buffered readers this is usually half the buffer size,
    /// which is equal to the word size of the underlying [`WordRead`].
    type PeekWord: UpcastableInto<u64>;

    /// Read `n` bits and return them in the lowest bits.
    fn read_bits(&mut self, n: usize) -> Result<u64, Self::Error>;

    /// Peeks at `n` bits without advancing the stream position.
    /// `n` must be nonzero, and at most `PeekWord::BITS`.
    fn peek_bits(&mut self, n: usize) -> Result<Self::PeekWord, Self::Error>;

    /// Skip `n` bits from the stream.
    fn skip_bits(&mut self, n: usize) -> Result<(), Self::Error>;

    /// Skip bits form the stream after a call to [`BitRead::peek_bits`].
    ///
    /// This is an internal optimization used to skip bits we know
    /// are already in some internal buffer as we [peeked](BitRead::peek_bits)
    /// at them.
    #[inline(always)]
    fn skip_bits_after_table_lookup(&mut self, n: usize) -> Result<(), Self::Error> {
        self.skip_bits(n)
    }

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
    /// the implementing type.
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
/// by traits such as [`crate::codes::GammaWrite`].
///
/// Note that the endianness parameter `E` is used only to specify the
/// endianness of the bit stream, and not that of the method arguments.
pub trait BitWrite<E: Endianness> {
    type Error: Error;

    /// Write the lowest `n` bits of value to the stream and return the number of
    /// bits written, that is, `n`.
    ///
    /// The other bits should be ignored, but it is allow to check them
    /// and panic in test mode if they are not zero.
    fn write_bits(&mut self, value: u64, n: usize) -> Result<usize, Self::Error>;

    /// Write `value` as an unary code to the stream and return the number of
    /// bits written, that is, `values` plus one.
    ///     
    /// This version of the method has a constant parameter
    /// deciding whether to use an encoding table. You should rather use
    /// [`BitWrite::write_unary`], which uses the default
    /// choice of the implementing type.
    fn write_unary_param<const USE_TABLE: bool>(
        &mut self,
        value: u64,
    ) -> Result<usize, Self::Error>;

    /// Write `value` as an unary code to the stream and return the number of
    /// bits written, that is, `values` plus one.
    ///
    /// This version of the method uses the version of
    /// of [`BitWrite::write_unary_param`] selected as default by
    /// the implementing type.
    fn write_unary(&mut self, value: u64) -> Result<usize, Self::Error>;

    /// Flush the buffer, consuming the bit stream.
    fn flush(self) -> Result<(), Self::Error>;
}

/// Seekability for [`BitRead`] and [`BitWrite`] streams.
pub trait BitSeek {
    type Error: Error;

    fn get_bit_pos(&mut self) -> Result<u64, Self::Error>;

    fn set_bit_pos(&mut self, bit_pos: u64) -> Result<(), Self::Error>;
}

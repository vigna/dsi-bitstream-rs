/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

/*!

Traits for bit-by-bit I/O.

The traits in this module define the basic bit access operations, including
fixed-width bit input/output and handling of unary codes. More sophisticated
operations, such as reading and writing instantaneous codes, are built on this traits.

Traits depend on a parameter that specifies the endianness of the stream.

 */

use crate::traits::*;
use anyhow::Result;
use common_traits::UpcastableInto;

/// Sequential, streaming bit-by-bit reads.
///
/// This trait specify basic operation over which codes can be implemented
/// by traits such as [`crate::codes::GammaRead`].
///
/// Note that the endianness parameter `E` is used only to specify the
/// endianness of the bit stream, and not that of the returned values.
pub trait BitRead<E: Endianness> {
    /// The type we can read from the stream without advancing.
    /// On buffered readers this is usually half the buffer size,
    /// which is equal to the word size of the underlying [`WordRead`].
    type PeekWord: UpcastableInto<u64>;

    /// Read `n` bits and return them in the lowest bits.
    fn read_bits(&mut self, n: usize) -> Result<u64>;

    /// Peeks at `n` bits without advancing the stream position.
    /// `n` must be nonzero, and at most `PeekWord::BITS`.
    fn peek_bits(&mut self, n: usize) -> Result<Self::PeekWord>;

    /// Skip `n` bits from the stream.
    fn skip_bits(&mut self, n: usize) -> Result<()>;

    /// Skip bits form the stream after a call to [`BitRead::peek_bits`].
    ///
    /// This is an internal optimization used to skip bits we know
    /// are already in some internal buffer as we [peeked](BitRead::peek_bits)
    /// at them.
    #[inline(always)]
    fn skip_bits_after_table_lookup(&mut self, n: usize) -> Result<()> {
        self.skip_bits(n)
    }

    /// Read a unary code.
    ///
    /// This version of the method has a constant parameter
    /// deciding whether to use a decoding table. You should rather use
    /// [`BitRead::read_unary`], which uses the default
    /// choice of the implementing type.
    fn read_unary_param<const USE_TABLE: bool>(&mut self) -> Result<u64>;

    /// Read a unary code.
    ///
    /// This version of the method uses the version of
    /// of [`BitRead::read_unary_param`] selected as default by
    /// the implementing type.
    #[inline(always)]
    fn read_unary(&mut self) -> Result<u64> {
        self.read_unary_param::<false>()
    }

    /// Skip a unary code.
    #[inline(always)]
    fn skip_unary(&mut self) -> Result<()> {
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
    /// Write the lowest `n` bits of value to the stream and return the number of
    /// bits written, that is, `n`.
    fn write_bits(&mut self, value: u64, n: usize) -> Result<usize>;

    /// Write `value` as an unary code to the stream and return the number of
    /// bits written, that is, `values` plus one.
    ///     
    /// This version of the method has a constant parameter
    /// deciding whether to use an encoding table. You should rather use
    /// [`BitWrite::write_unary`], which uses the default
    /// choice of the implementing type.
    fn write_unary_param<const USE_TABLE: bool>(&mut self, value: u64) -> Result<usize>;

    /// Write `value` as an unary code to the stream and return the number of
    /// bits written, that is, `values` plus one.
    ///
    /// This version of the method uses the version of
    /// of [`BitWrite::write_unary_param`] selected as default by
    /// the implementing type.
    fn write_unary(&mut self, value: u64) -> Result<usize>;

    /// Flush the buffer, consuming the bit stream.
    fn flush(self) -> Result<()>;
}

/// Seekability for [`BitRead`] and [`BitWrite`] streams.
pub trait BitSeek {
    #[must_use]
    fn get_bit_pos(&self) -> usize;

    fn set_bit_pos(&mut self, bit_pos: usize) -> Result<()>;
}

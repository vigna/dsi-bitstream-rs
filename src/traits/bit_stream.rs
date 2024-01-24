/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use std::error::Error;

use crate::{
    prelude::{delta_tables, gamma_tables, unary_tables, zeta_tables},
    traits::*,
};
use common_traits::CastableInto;

pub trait Peekable<const N: usize> {}
macro_rules! impl_peekable {
    ($($n:literal),*) => {$(
        impl<T: Peekable<{$n + 1}>> Peekable<$n> for T {}
    )*};
}

impl_peekable!(
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32
);

/// Sequential, streaming bit-by-bit reads.
///
/// This trait specify basic operation over which codes can be implemented by
/// traits such as [`GammaReadParam`](crate::codes::GammaReadParam).
///
/// To read quickly complex codes, such traits may use the
/// [`peek_bits`](BitRead::peek_bits) method to read a few bits in advance and
/// then use a table to decode them. For this to happen correctly,
/// [`peek_bits`](BitRead::peek_bits) must return a sufficient number of bits.
/// It is unfortunately difficult at the time being to check statically that
/// this is the case, but in test mode an assertion will be triggered if the
/// number of bits returned by [`peek_bits`](BitRead::peek_bits) is not
/// sufficient.
///
/// Implementors are invited to call [`check_tables`] at construction time to
/// provide a warning to the user if the peek word is not large enough.
///
/// Please see the documentation of the [`impls`](crate::impls) module for more
/// details.
pub trait BitRead<E: Endianness> {
    type Error: Error + Send + Sync + 'static;

    /// The type we can read from the stream without advancing.
    type PeekWord: CastableInto<u64>;

    /// Read `n` bits and return them in the lowest bits.
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
}

/// Sequential, streaming bit-by-bit writes.
///
/// This trait specify basic operation over which codes can be implemented
/// by traits such as [`crate::codes::GammaWriteParam`].
pub trait BitWrite<E: Endianness> {
    type Error: Error + Send + Sync + 'static;

    /// Write the lowest `n` bits of `value` to the stream and return the number
    /// of bits written, that is, `n`.
    ///
    ///
    /// Implementors should check the value of `n` in test mode and panic if it
    /// is greater than 64. Moreover, if the feature `checks` is enabled they
    /// should check that the remaining bits of `value` are zero.
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
    fn flush(&mut self) -> Result<(), Self::Error>;
}

/// Seekability for [`BitRead`] and [`BitWrite`] streams.
pub trait BitSeek {
    type Error: Error + Send + Sync + 'static;
    /// Get the current position in bits from the start of the file.
    fn get_bit_pos(&mut self) -> Result<u64, Self::Error>;

    /// Set the current position in bits from the start of the file to `bit_pos`.
    ///
    /// Note that moving forward by a small amount of bits may be accomplished
    /// more efficiently by calling [`BitRead::skip_bits`].
    fn set_bit_pos(&mut self, bit_pos: u64) -> Result<(), Self::Error>;
}

/// Utility function to check that the peek word is large enough.
///
/// It **strongly suggested** that this function is called by the
/// creation methods of types implementing [`BitRead`].
pub fn check_tables(peek_bits: usize) {
    if peek_bits < gamma_tables::READ_BITS {
        eprintln!(
            "DANGER: your BitRead can peek at {} bits, but the tables for γ codes use {} bits",
            peek_bits,
            gamma_tables::READ_BITS
        );
    }
    if peek_bits < delta_tables::READ_BITS {
        eprintln!(
            "DANGER: your BitRead can peek at {} bits, but the tables for δ codes use {} bits",
            peek_bits,
            delta_tables::READ_BITS
        );
    }
    if peek_bits < zeta_tables::READ_BITS {
        eprintln!(
            "DANGER: your BitRead can peek at {} bits, but the tables for ζ₃ codes use {} bits",
            peek_bits,
            zeta_tables::READ_BITS
        );
    }
    if peek_bits < unary_tables::READ_BITS {
        eprintln!(
            "DANGER: your BitRead can peek at {} bits, but the tables for unary codes use {} bits",
            peek_bits,
            unary_tables::READ_BITS
        );
    }
}

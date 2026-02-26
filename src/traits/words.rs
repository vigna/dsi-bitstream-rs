/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use core::error::Error;

use num_primitive::PrimitiveUnsigned;
use num_traits::{AsPrimitive, ConstOne, ConstZero};

/// This is a convenience trait bundling the bounds required for words read and
/// written by either a [`WordRead`] or [`WordWrite`], respectively.
pub trait Word: PrimitiveUnsigned + ConstZero + ConstOne {}
impl<W: PrimitiveUnsigned + ConstZero + ConstOne> Word for W {}

/// Trait providing the double-width type for a given unsigned integer type.
///
/// This is used by [`crate::impls::BufBitReader`] to provide a bit buffer
/// that is twice the width of the word read from the backend.
///
/// The methods
/// [`as_double`](Self::as_double)/[`as_u64`](Self::as_u64) can be
/// used to convert a word into its double-width type or to a `u64`,
/// respectively, without loss of precision.
pub trait DoubleType {
    type DoubleType: Word + AsPrimitive<u64>;

    /// Converts a word into its double-width type without loss of precision.
    fn as_double(&self) -> Self::DoubleType;

    /// Converts a word into a `u64` without loss of precision.
    fn as_u64(&self) -> u64;
}

macro_rules! impl_double_type {
    ($($t:ty => $d:ty),*) => {
        $(
            impl DoubleType for $t {
                type DoubleType = $d;

                fn as_double(&self) -> Self::DoubleType {
                    *self as Self::DoubleType
                }

                fn as_u64(&self) -> u64 {
                    *self as u64
                }
            }
        )*
    };
}

impl_double_type!(
    u8 => u16,
    u16 => u32,
    u32 => u64,
    u64 => u128
);

/// Sequential, streaming word-by-word reads.
pub trait WordRead {
    type Error: Error + Send + Sync + 'static;

    /// The word type (the type of the result of [`WordRead::read_word`]).
    type Word: Word;

    /// Reads a word and advances the current position.
    fn read_word(&mut self) -> Result<Self::Word, Self::Error>;
}

/// Sequential, streaming word-by-word writes.
pub trait WordWrite {
    type Error: Error + Send + Sync + 'static;

    /// The word type (the type of the argument of [`WordWrite::write_word`]).
    type Word: Word;

    /// Writes a word and advances the current position.
    fn write_word(&mut self, word: Self::Word) -> Result<(), Self::Error>;

    /// Flush the stream.
    fn flush(&mut self) -> Result<(), Self::Error>;
}

/// Seekability for [`WordRead`] and [`WordWrite`] streams.
pub trait WordSeek {
    type Error: Error + Send + Sync + 'static;
    /// Gets the current position in words from the start of the stream.
    ///
    /// Note that, consistently with
    /// [`Seek::stream_position`](https://doc.rust-lang.org/beta/std/io/trait.Seek.html#method.stream_position),
    /// this method takes a mutable reference to `self`.
    fn word_pos(&mut self) -> Result<u64, Self::Error>;

    /// Sets the current position in words from the start of the stream to `word_pos`.
    fn set_word_pos(&mut self, word_pos: u64) -> Result<(), Self::Error>;
}

/// Replacement of [`std::io::Error`] for `no_std` environments
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WordError {
    UnexpectedEof { word_pos: usize },
}

impl core::error::Error for WordError {}
impl core::fmt::Display for WordError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            WordError::UnexpectedEof { word_pos } => {
                write!(f, "unexpected end of data at word position {}", word_pos)
            }
        }
    }
}

/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use core::error::Error;

use common_traits::*;

/// This is a trait alias for all the properties that we need words of memory
/// read and wrote by either a [`WordRead`] or [`WordWrite`], respectively.
pub trait Word: UnsignedInt + ToBytes + FromBytes + FiniteRangeNumber {}
impl<W: UnsignedInt + ToBytes + FromBytes + FiniteRangeNumber> Word for W {}

/// Sequential, streaming word-by-word reads.
pub trait WordRead {
    type Error: Error + Send + Sync + 'static;

    /// The word type (the type of the result of [`WordRead::read_word`]).
    type Word: Word;

    /// Read a word and advance the current position.
    fn read_word(&mut self) -> Result<Self::Word, Self::Error>;
}

/// Sequential, streaming word-by-word writes.
pub trait WordWrite {
    type Error: Error + Send + Sync + 'static;

    /// The word type (the type of the argument of [`WordWrite::write_word`]).
    type Word: Word;

    /// Write a word and advance the current position.
    fn write_word(&mut self, word: Self::Word) -> Result<(), Self::Error>;

    /// Flush the stream.
    fn flush(&mut self) -> Result<(), Self::Error>;
}

/// Seekability for [`WordRead`] and [`WordWrite`] streams.
pub trait WordSeek {
    type Error: Error + Send + Sync + 'static;
    /// Get the current position in words from the start of the file.
    fn word_pos(&mut self) -> Result<u64, Self::Error>;

    /// Set the current position in words from the start of the file to `word_pos`.
    fn set_word_pos(&mut self, word_pos: u64) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone, PartialEq)]
/// Replacement of [`std::io::Error`] for `no_std` environments
pub enum WordError {
    UnexpectedEof { word_pos: usize },
}

impl core::error::Error for WordError {}
impl core::fmt::Display for WordError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            WordError::UnexpectedEof { word_pos } => {
                write!(f, "Unexpected end of data at word position {}", word_pos)
            }
        }
    }
}

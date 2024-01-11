/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use std::{
    error::Error,
    io::{Read, Seek, SeekFrom, Write},
};

use common_traits::*;

pub trait WordRead {
    type Error: Error + Send + Sync;
    /// The word type (the type of the result of [`WordRead::read_word`]).
    type Word: UnsignedInt;
    /// Read a word and advance the current position.
    fn read_word(&mut self) -> Result<Self::Word, Self::Error>;
}

/// Sequential, streaming word-by-word writes.
pub trait WordWrite {
    type Error: Error + Send + Sync;
    /// The word type (the type of the argument of [`WordWrite::write_word`]).
    type Word: UnsignedInt;
    /// Write a word and advance the current position.
    fn write_word(&mut self, word: Self::Word) -> Result<(), Self::Error>;
}

/// Seekability for [`WordRead`] and [`WordWrite`] streams.
pub trait WordSeek {
    type Error: Error + Send + Sync;
    #[must_use]
    fn get_word_pos(&mut self) -> Result<u64, Self::Error>;

    fn set_word_pos(&mut self, word_index: u64) -> Result<(), Self::Error>;
}

impl<R: Read> WordRead for R {
    type Error = std::io::Error;
    type Word = u8;

    #[inline(always)]
    fn read_word(&mut self) -> Result<Self::Word, std::io::Error> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }
}

impl<W: Write> WordWrite for W {
    type Error = std::io::Error;
    type Word = u8;

    #[inline(always)]
    fn write_word(&mut self, word: Self::Word) -> Result<(), std::io::Error> {
        self.write(&[word])?;
        Ok(())
    }
}

impl<S: Seek> WordSeek for S {
    type Error = std::io::Error;

    fn get_word_pos(&mut self) -> Result<u64, std::io::Error> {
        Ok(self.stream_position()?)
    }

    fn set_word_pos(&mut self, word_index: u64) -> Result<(), std::io::Error> {
        self.seek(SeekFrom::Start(word_index as u64))?;
        Ok(())
    }
}

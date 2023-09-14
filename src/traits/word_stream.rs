/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use anyhow::Result;
use common_traits::*;

/// A Seekable word stream
pub trait WordStream {
    /// Return the index of the **next** word that  will be
    /// read on the next [`WordRead::read_next_word`] call.
    #[must_use]
    fn get_position(&self) -> usize;

    /// Set the position in the stream so that the `word_index`-th word will be
    /// read on the next [`WordRead::read_next_word`] call.
    ///
    /// # Errors
    /// This function fails if the given `word_index` is out of bound of the
    /// underneath backend memory.
    fn set_position(&mut self, word_index: usize) -> Result<()>;
}

/// An object we can read words from sequentially
pub trait WordRead {
    /// The type that can be read
    type Word: Word;
    /// Read a [`u64`] word from the stream and advance the position by 8 bytes.
    ///
    /// # Errors
    /// This function fails if we cannot read the next word in the stream,
    /// usually this happens when the stream ended.
    fn read_next_word(&mut self) -> Result<Self::Word>;
}

/// An object that we can write words to sequentially
pub trait WordWrite {
    /// The type that can be wrote
    type Word: Word;
    /// Write a [`u64`] word from the stream and advance the position by 8 bytes.
    ///
    /// # Errors
    /// This function fails if we cannot write a word to the stream,
    /// usually this happens when the stream ended.
    fn write_word(&mut self, word: Self::Word) -> Result<()>;
}

/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use anyhow::Result;
use common_traits::*;

/// Sequential, streaming word-by-word reads.
pub trait WordRead {
    /// The word type (the type of the result of [`WordRead::read_word`]).
    type Word: UnsignedInt;
    /// Read a word and advance the current position.
    fn read_word(&mut self) -> Result<Self::Word>;
}

/// Sequential, streaming word-by-word writes.
pub trait WordWrite {
    /// The word type (the type of the argument of [`WordWrite::write_word`]).
    type Word: UnsignedInt;
    /// Write a word and advance the current position.
    fn write_word(&mut self, word: Self::Word) -> Result<()>;
}

/// Seekability for [`WordRead`] and [`WordWrite`] streams.
pub trait WordSeek {
    #[must_use]
    fn get_word_pos(&self) -> usize;

    fn set_word_pos(&mut self, word_index: usize) -> Result<()>;
}

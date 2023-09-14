/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

/*!

Traits for word-by-word I/O.

The traits in this module are used by all bit-based backends to access the underlying
source of data (e.g., memory or file) word-by-word with a programmable word size.

 */

use anyhow::Result;
use common_traits::*;

/// Sequential, streaming word-by-word reads.
pub trait WordRead {
    /// The word type (the type of the result of [`WordRead::read`]).
    type Word: Word;
    /// Read a word and advance the current position.
    fn read(&mut self) -> Result<Self::Word>;
}

/// Sequential, streaming word-by-word writes.
pub trait WordWrite {
    /// The word type (the type of the argument of [`WordWrite::write`]).
    type Word: Word;
    /// Write a word and advance the current position.
    fn write(&mut self, word: Self::Word) -> Result<()>;
}
/// Seekability for [`WordRead`] and [`WordWrite`] streams.
pub trait WordSeek {
    #[must_use]
    fn get_pos(&self) -> usize;

    fn set_pos(&mut self, word_index: usize) -> Result<()>;
}

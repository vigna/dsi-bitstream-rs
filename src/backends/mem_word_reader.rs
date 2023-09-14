/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use crate::traits::*;
use anyhow::{bail, Result};
use common_traits::Word;

/// An Implementation of [`WordRead`] for a slice.
///
/// # Example
/// ```
/// use dsi_bitstream::prelude::*;
///
/// let words: [u64; 2] = [
///     0x0043b59fcdf16077,
///     0x702863e6f9739b86,
/// ];
///
/// let mut word_reader = MemWordReader::new(&words);
///
/// // the stream is read sequentially
/// assert_eq!(word_reader.get_word_pos(), 0);
/// assert_eq!(word_reader.read_word().unwrap(), 0x0043b59fcdf16077);
/// assert_eq!(word_reader.get_word_pos(), 1);
/// assert_eq!(word_reader.read_word().unwrap(), 0x702863e6f9739b86);
/// assert_eq!(word_reader.get_word_pos(), 2);
/// assert!(word_reader.read_word().is_err());
///
/// // you can change position
/// assert!(word_reader.set_word_pos(1).is_ok());
/// assert_eq!(word_reader.get_word_pos(), 1);
/// assert_eq!(word_reader.read_word().unwrap(), 0x702863e6f9739b86);
///
/// // errored set position doesn't change the current position
/// assert_eq!(word_reader.get_word_pos(), 2);
/// assert!(word_reader.set_word_pos(100).is_err());
/// assert_eq!(word_reader.get_word_pos(), 2);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct MemWordReader<W: Word, B: AsRef<[W]>> {
    data: B,
    word_index: usize,
    _marker: core::marker::PhantomData<W>,
}

impl<W: Word, B: AsRef<[W]>> MemWordReader<W, B> {
    /// Create a new [`MemWordReader`] from a slice of data
    #[must_use]
    pub fn new(data: B) -> Self {
        Self {
            data,
            word_index: 0,
            _marker: Default::default(),
        }
    }

    pub fn len(&self) -> usize {
        self.data.as_ref().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

///
#[derive(Debug, Clone, PartialEq)]
pub struct MemWordReaderInf<W: Word, B: AsRef<[W]>> {
    data: B,
    word_index: usize,
    _marker: core::marker::PhantomData<W>,
}

impl<W: Word, B: AsRef<[W]>> MemWordReaderInf<W, B> {
    /// Create a new [`MemWordReaderInfinite`] from a slice of data
    #[must_use]
    pub fn new(data: B) -> Self {
        Self {
            data,
            word_index: 0,
            _marker: Default::default(),
        }
    }
}
impl<W: Word, B: AsRef<[W]>> WordRead for MemWordReaderInf<W, B> {
    type Word = W;

    #[inline(always)]
    fn read_word(&mut self) -> Result<W> {
        let res = self
            .data
            .as_ref()
            .get(self.word_index)
            .copied()
            .unwrap_or(W::ZERO);
        self.word_index += 1;
        Ok(res)
    }
}

impl<W: Word, B: AsRef<[W]>> WordSeek for MemWordReaderInf<W, B> {
    #[inline(always)]
    #[must_use]
    fn get_word_pos(&self) -> usize {
        self.word_index
    }

    #[inline(always)]
    fn set_word_pos(&mut self, word_index: usize) -> Result<()> {
        self.word_index = word_index;
        Ok(())
    }
}

impl<W: Word, B: AsRef<[W]>> WordRead for MemWordReader<W, B> {
    type Word = W;

    #[inline]
    fn read_word(&mut self) -> Result<W> {
        match self.data.as_ref().get(self.word_index) {
            Some(word) => {
                self.word_index += 1;
                Ok(*word)
            }
            None => {
                bail!("Cannot read next word as the underlying memory ended",);
            }
        }
    }
}

impl<W: Word, B: AsRef<[W]>> WordSeek for MemWordReader<W, B> {
    #[inline]
    #[must_use]
    fn get_word_pos(&self) -> usize {
        self.word_index
    }

    #[inline]
    fn set_word_pos(&mut self, word_index: usize) -> Result<()> {
        if word_index >= self.data.as_ref().len() {
            bail!(
                "Index {} is out of bound on a MemWordReader of length {}",
                word_index,
                self.data.as_ref().len()
            );
        }
        self.word_index = word_index;
        Ok(())
    }
}

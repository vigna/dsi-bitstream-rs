/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use crate::traits::*;
use anyhow::{bail, Result};
use common_traits::UnsignedInt;

/// An Implementation of [`WordRead`], [`WordWrite`], and [`WordSeek`] for a
/// mutable slice of memory.
///
/// # Example
/// ```
/// use dsi_bitstream::prelude::*;
///
/// let mut words: [u64; 2] = [
///     0x0043b59fcdf16077,
///     0x702863e6f9739b86,
/// ];
///
/// let mut word_writer = MemWordWriter::new(&mut words);
///
/// // the stream is read sequentially
/// assert_eq!(word_writer.get_word_pos(), 0);
/// assert_eq!(word_writer.read_word().unwrap(), 0x0043b59fcdf16077);
/// assert_eq!(word_writer.get_word_pos(), 1);
/// assert_eq!(word_writer.read_word().unwrap(), 0x702863e6f9739b86);
/// assert_eq!(word_writer.get_word_pos(), 2);
/// assert!(word_writer.read_word().is_err());
///
/// // you can change position
/// assert!(word_writer.set_word_pos(1).is_ok());
/// assert_eq!(word_writer.get_word_pos(), 1);
/// assert_eq!(word_writer.read_word().unwrap(), 0x702863e6f9739b86);
///
/// // errored set position doesn't change the current position
/// assert_eq!(word_writer.get_word_pos(), 2);
/// assert!(word_writer.set_word_pos(100).is_err());
/// assert_eq!(word_writer.get_word_pos(), 2);
///
/// // we can write and read back!
/// assert!(word_writer.set_word_pos(0).is_ok());
/// assert!(word_writer.write_word(0x0b801b2bf696e8d2).is_ok());
/// assert_eq!(word_writer.get_word_pos(), 1);
/// assert!(word_writer.set_word_pos(0).is_ok());
/// assert_eq!(word_writer.read_word().unwrap(), 0x0b801b2bf696e8d2);
/// assert_eq!(word_writer.get_word_pos(), 1);
/// ```
#[derive(Debug, PartialEq)]
pub struct MemWordWriter<W: UnsignedInt, B: AsMut<[W]>> {
    data: B,
    word_index: usize,
    _marker: core::marker::PhantomData<W>,
}

impl<W: UnsignedInt, B: AsMut<[W]> + AsRef<[W]>> MemWordWriter<W, B> {
    /// Create a new [`MemWordWriter`] from a slice of **ZERO INITIALIZED** data
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

/// An Implementation of [`WordSeek`], [`WordRead`], [`WordWrite`]
/// for a mutable [`Vec<u64>`]. The core difference between [`MemWordWriter`]
/// and [`MemWordWriterVec`] is that the former allocates new memory
/// if the stream writes out of bound by 1.
///
/// # Example
/// ```
/// use dsi_bitstream::prelude::*;
///
/// let mut words: Vec<u64> = vec![
///     0x0043b59fcdf16077,
/// ];
///
/// let mut word_writer = MemWordWriterVec::new(&mut words);
///
/// // the stream is read sequentially
/// assert_eq!(word_writer.get_word_pos(), 0);
/// assert!(word_writer.write_word(0).is_ok());
/// assert_eq!(word_writer.get_word_pos(), 1);
/// assert!(word_writer.write_word(1).is_ok());
/// assert_eq!(word_writer.get_word_pos(), 2);
/// ```
#[derive(Debug, PartialEq)]
#[cfg(feature = "alloc")]
pub struct MemWordWriterVec<W: UnsignedInt, B: AsMut<alloc::vec::Vec<W>>> {
    data: B,
    word_index: usize,
    _marker: core::marker::PhantomData<W>,
}

#[cfg(feature = "alloc")]
impl<W: UnsignedInt, B: AsMut<alloc::vec::Vec<W>> + AsRef<alloc::vec::Vec<W>>>
    MemWordWriterVec<W, B>
{
    /// Create a new [`MemWordWriter`] from a slice of **ZERO INITIALIZED** data
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

impl<W: UnsignedInt, B: AsMut<[W]>> WordRead for MemWordWriter<W, B> {
    type Word = W;

    #[inline]
    fn read_word(&mut self) -> Result<W> {
        match self.data.as_mut().get(self.word_index) {
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

impl<W: UnsignedInt, B: AsRef<[W]> + AsMut<[W]>> WordSeek for MemWordWriter<W, B> {
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

impl<W: UnsignedInt, B: AsMut<[W]>> WordWrite for MemWordWriter<W, B> {
    type Word = W;

    #[inline]
    fn write_word(&mut self, word: W) -> Result<()> {
        match self.data.as_mut().get_mut(self.word_index) {
            Some(word_ref) => {
                self.word_index += 1;
                *word_ref = word;
                Ok(())
            }
            None => {
                bail!("Cannot write next word as the underlying memory ended",);
            }
        }
    }
}

#[cfg(feature = "alloc")]
impl<W: UnsignedInt, B: AsMut<alloc::vec::Vec<W>>> WordWrite for MemWordWriterVec<W, B> {
    type Word = W;

    #[inline]
    fn write_word(&mut self, word: W) -> Result<()> {
        if self.word_index >= self.data.as_mut().len() {
            self.data.as_mut().resize(self.word_index + 1, W::ZERO);
        }
        self.data.as_mut()[self.word_index] = word;
        self.word_index += 1;
        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl<W: UnsignedInt, B: AsMut<alloc::vec::Vec<W>>> WordRead for MemWordWriterVec<W, B> {
    type Word = W;

    #[inline]
    fn read_word(&mut self) -> Result<W> {
        match self.data.as_mut().get(self.word_index) {
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

#[cfg(feature = "alloc")]
impl<W: UnsignedInt, B: AsMut<alloc::vec::Vec<W>> + AsRef<alloc::vec::Vec<W>>> WordSeek
    for MemWordWriterVec<W, B>
{
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

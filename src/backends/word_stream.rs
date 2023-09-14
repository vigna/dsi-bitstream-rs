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

/// An Implementation of [`WordRead`] for a slice of memory `&[u64]`
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
/// let mut word_reader = MemWordRead::new(&words);
///
/// // the stream is read sequentially
/// assert_eq!(word_reader.get_position(), 0);
/// assert_eq!(word_reader.read().unwrap(), 0x0043b59fcdf16077);
/// assert_eq!(word_reader.get_position(), 1);
/// assert_eq!(word_reader.read().unwrap(), 0x702863e6f9739b86);
/// assert_eq!(word_reader.get_position(), 2);
/// assert!(word_reader.read().is_err());
///
/// // you can change position
/// assert!(word_reader.set_position(1).is_ok());
/// assert_eq!(word_reader.get_position(), 1);
/// assert_eq!(word_reader.read().unwrap(), 0x702863e6f9739b86);
///
/// // errored set position doesn't change the current position
/// assert_eq!(word_reader.get_position(), 2);
/// assert!(word_reader.set_position(100).is_err());
/// assert_eq!(word_reader.get_position(), 2);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct MemWordRead<W: Word, B: AsRef<[W]>> {
    data: B,
    word_index: usize,
    _marker: core::marker::PhantomData<W>,
}

impl<W: Word, B: AsRef<[W]>> MemWordRead<W, B> {
    /// Create a new [`MemWordRead`] from a slice of data
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
pub struct MemWordReadInfinite<W: Word, B: AsRef<[W]>> {
    data: B,
    word_index: usize,
    _marker: core::marker::PhantomData<W>,
}

impl<W: Word, B: AsRef<[W]>> MemWordReadInfinite<W, B> {
    /// Create a new [`MemWordReadInfinite`] from a slice of data
    #[must_use]
    pub fn new(data: B) -> Self {
        Self {
            data,
            word_index: 0,
            _marker: Default::default(),
        }
    }
}
impl<W: Word, B: AsRef<[W]>> WordRead for MemWordReadInfinite<W, B> {
    type Word = W;

    #[inline(always)]
    fn read(&mut self) -> Result<W> {
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

impl<W: Word, B: AsRef<[W]>> WordSeek for MemWordReadInfinite<W, B> {
    #[inline(always)]
    #[must_use]
    fn get_position(&self) -> usize {
        self.word_index
    }

    #[inline(always)]
    fn set_position(&mut self, word_index: usize) -> Result<()> {
        self.word_index = word_index;
        Ok(())
    }
}

/// An Implementation of [`WordSeek`], [`WordRead`], [`WordWrite`] for a
/// mutable slice of memory `&mut [u64]`
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
/// let mut word_writer = MemWordWrite::new(&mut words);
///
/// // the stream is read sequentially
/// assert_eq!(word_writer.get_position(), 0);
/// assert_eq!(word_writer.read().unwrap(), 0x0043b59fcdf16077);
/// assert_eq!(word_writer.get_position(), 1);
/// assert_eq!(word_writer.read().unwrap(), 0x702863e6f9739b86);
/// assert_eq!(word_writer.get_position(), 2);
/// assert!(word_writer.read().is_err());
///
/// // you can change position
/// assert!(word_writer.set_position(1).is_ok());
/// assert_eq!(word_writer.get_position(), 1);
/// assert_eq!(word_writer.read().unwrap(), 0x702863e6f9739b86);
///
/// // errored set position doesn't change the current position
/// assert_eq!(word_writer.get_position(), 2);
/// assert!(word_writer.set_position(100).is_err());
/// assert_eq!(word_writer.get_position(), 2);
///
/// // we can write and read back!
/// assert!(word_writer.set_position(0).is_ok());
/// assert!(word_writer.write(0x0b801b2bf696e8d2).is_ok());
/// assert_eq!(word_writer.get_position(), 1);
/// assert!(word_writer.set_position(0).is_ok());
/// assert_eq!(word_writer.read().unwrap(), 0x0b801b2bf696e8d2);
/// assert_eq!(word_writer.get_position(), 1);
/// ```
#[derive(Debug, PartialEq)]
pub struct MemWordWrite<W: Word, B: AsMut<[W]>> {
    data: B,
    word_index: usize,
    _marker: core::marker::PhantomData<W>,
}

impl<W: Word, B: AsMut<[W]> + AsRef<[W]>> MemWordWrite<W, B> {
    /// Create a new [`MemWordWrite`] from a slice of **ZERO INITIALIZED** data
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
/// for a mutable [`Vec<u64>`]. The core difference between [`MemWordWrite`]
/// and [`MemWordWriteVec`] is that the former allocates new memory
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
/// let mut word_writer = MemWordWriteVec::new(&mut words);
///
/// // the stream is read sequentially
/// assert_eq!(word_writer.get_position(), 0);
/// assert!(word_writer.write(0).is_ok());
/// assert_eq!(word_writer.get_position(), 1);
/// assert!(word_writer.write(1).is_ok());
/// assert_eq!(word_writer.get_position(), 2);
/// ```
#[derive(Debug, PartialEq)]
#[cfg(feature = "alloc")]
pub struct MemWordWriteVec<W: Word, B: AsMut<alloc::vec::Vec<W>>> {
    data: B,
    word_index: usize,
    _marker: core::marker::PhantomData<W>,
}

#[cfg(feature = "alloc")]
impl<W: Word, B: AsMut<alloc::vec::Vec<W>> + AsRef<alloc::vec::Vec<W>>> MemWordWriteVec<W, B> {
    /// Create a new [`MemWordWrite`] from a slice of **ZERO INITIALIZED** data
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

impl<W: Word, B: AsRef<[W]>> WordRead for MemWordRead<W, B> {
    type Word = W;

    #[inline]
    fn read(&mut self) -> Result<W> {
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

impl<W: Word, B: AsRef<[W]>> WordSeek for MemWordRead<W, B> {
    #[inline]
    #[must_use]
    fn get_position(&self) -> usize {
        self.word_index
    }

    #[inline]
    fn set_position(&mut self, word_index: usize) -> Result<()> {
        if word_index >= self.data.as_ref().len() {
            bail!(
                "Index {} is out of bound on a MemWordRead of length {}",
                word_index,
                self.data.as_ref().len()
            );
        }
        self.word_index = word_index;
        Ok(())
    }
}

impl<W: Word, B: AsMut<[W]>> WordRead for MemWordWrite<W, B> {
    type Word = W;

    #[inline]
    fn read(&mut self) -> Result<W> {
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

impl<W: Word, B: AsRef<[W]> + AsMut<[W]>> WordSeek for MemWordWrite<W, B> {
    #[inline]
    #[must_use]
    fn get_position(&self) -> usize {
        self.word_index
    }

    #[inline]
    fn set_position(&mut self, word_index: usize) -> Result<()> {
        if word_index >= self.data.as_ref().len() {
            bail!(
                "Index {} is out of bound on a MemWordRead of length {}",
                word_index,
                self.data.as_ref().len()
            );
        }
        self.word_index = word_index;
        Ok(())
    }
}

impl<W: Word, B: AsMut<[W]>> WordWrite for MemWordWrite<W, B> {
    type Word = W;

    #[inline]
    fn write(&mut self, word: W) -> Result<()> {
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
impl<W: Word, B: AsMut<alloc::vec::Vec<W>>> WordWrite for MemWordWriteVec<W, B> {
    type Word = W;

    #[inline]
    fn write(&mut self, word: W) -> Result<()> {
        if self.word_index >= self.data.as_mut().len() {
            self.data.as_mut().resize(self.word_index + 1, W::ZERO);
        }
        self.data.as_mut()[self.word_index] = word;
        self.word_index += 1;
        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl<W: Word, B: AsMut<alloc::vec::Vec<W>>> WordRead for MemWordWriteVec<W, B> {
    type Word = W;

    #[inline]
    fn read(&mut self) -> Result<W> {
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
impl<W: Word, B: AsMut<alloc::vec::Vec<W>> + AsRef<alloc::vec::Vec<W>>> WordSeek
    for MemWordWriteVec<W, B>
{
    #[inline]
    #[must_use]
    fn get_position(&self) -> usize {
        self.word_index
    }

    #[inline]
    fn set_position(&mut self, word_index: usize) -> Result<()> {
        if word_index >= self.data.as_ref().len() {
            bail!(
                "Index {} is out of bound on a MemWordRead of length {}",
                word_index,
                self.data.as_ref().len()
            );
        }
        self.word_index = word_index;
        Ok(())
    }
}

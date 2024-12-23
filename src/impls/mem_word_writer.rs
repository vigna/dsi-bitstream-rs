/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use core::convert::Infallible;
#[cfg(feature = "mem_dbg")]
use mem_dbg::{MemDbg, MemSize};

use crate::traits::*;

/// An implementation of [`WordRead`], [`WordWrite`], and [`WordSeek`] for a
/// mutable slice.
///
/// Writing beyond the end of the slice will return an error.
///
/// # Example
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use dsi_bitstream::prelude::*;
///
/// let mut words: [u64; 2] = [
///     0x0043b59fcdf16077,
///     0x702863e6f9739b86,
/// ];
///
/// let mut word_writer = MemWordWriterSlice::new(&mut words);
///
/// // the stream is read sequentially
/// assert_eq!(word_writer.word_pos()?, 0);
/// assert_eq!(word_writer.read_word()?, 0x0043b59fcdf16077);
/// assert_eq!(word_writer.word_pos()?, 1);
/// assert_eq!(word_writer.read_word()?, 0x702863e6f9739b86);
/// assert_eq!(word_writer.word_pos()?, 2);
/// assert!(word_writer.read_word().is_err());
///
/// // you can change position
/// assert!(word_writer.set_word_pos(1).is_ok());
/// assert_eq!(word_writer.word_pos()?, 1);
/// assert_eq!(word_writer.read_word()?, 0x702863e6f9739b86);
///
/// // errored set position doesn't change the current position
/// assert_eq!(word_writer.word_pos()?, 2);
/// assert!(word_writer.set_word_pos(100).is_err());
/// assert_eq!(word_writer.word_pos()?, 2);
///
/// // we can write and read back!
/// assert!(word_writer.set_word_pos(0).is_ok());
/// assert!(word_writer.write_word(0x0b801b2bf696e8d2).is_ok());
/// assert_eq!(word_writer.word_pos()?, 1);
/// assert!(word_writer.set_word_pos(0).is_ok());
/// assert_eq!(word_writer.read_word()?, 0x0b801b2bf696e8d2);
/// assert_eq!(word_writer.word_pos()?, 1);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
pub struct MemWordWriterSlice<W: Word, B: AsMut<[W]>> {
    data: B,
    word_index: usize,
    _marker: core::marker::PhantomData<W>,
}

impl<W: Word, B: AsMut<[W]> + AsRef<[W]>> MemWordWriterSlice<W, B> {
    /// Create a new [`MemWordWriterSlice`] from a slice of **ZERO INITIALIZED** data
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

    pub fn into_inner(self) -> B {
        self.data
    }
}

/// An implementation of [`WordRead`], [`WordWrite`], and [`WordSeek`]
/// for a mutable vector.
///
/// The vector will be extended as new data is written.
///
/// # Example
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use dsi_bitstream::prelude::*;
///
/// let mut words: Vec<u64> = vec![
///     0x0043b59fcdf16077,
/// ];
///
/// let mut word_writer = MemWordWriterVec::new(&mut words);
///
/// // the stream is read sequentially
/// assert_eq!(word_writer.word_pos()?, 0);
/// assert!(word_writer.write_word(0).is_ok());
/// assert_eq!(word_writer.word_pos()?, 1);
/// assert!(word_writer.write_word(1).is_ok());
/// assert_eq!(word_writer.word_pos()?, 2);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
#[cfg(feature = "alloc")]
pub struct MemWordWriterVec<W: Word, B: AsMut<alloc::vec::Vec<W>>> {
    data: B,
    word_index: usize,
    _marker: core::marker::PhantomData<W>,
}

#[cfg(feature = "alloc")]
impl<W: Word, B: AsMut<alloc::vec::Vec<W>> + AsRef<alloc::vec::Vec<W>>> MemWordWriterVec<W, B> {
    /// Create a new [`MemWordWriterSlice`] from a slice of **ZERO INITIALIZED** data
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

    pub fn into_inner(self) -> B {
        self.data
    }
}

impl<W: Word, B: AsMut<[W]>> WordRead for MemWordWriterSlice<W, B> {
    type Error = std::io::Error;
    type Word = W;

    #[inline]
    fn read_word(&mut self) -> Result<W, std::io::Error> {
        match self.data.as_mut().get(self.word_index) {
            Some(word) => {
                self.word_index += 1;
                Ok(*word)
            }
            None => Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Cannot read next word as the underlying memory ended",
            )),
        }
    }
}

impl<W: Word, B: AsRef<[W]> + AsMut<[W]>> WordSeek for MemWordWriterSlice<W, B> {
    type Error = std::io::Error;

    #[inline(always)]
    fn word_pos(&mut self) -> Result<u64, std::io::Error> {
        Ok(self.word_index as u64)
    }

    #[inline(always)]
    fn set_word_pos(&mut self, word_index: u64) -> Result<(), std::io::Error> {
        if word_index > self.data.as_ref().len() as u64 {
            Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                format_args!(
                    "Position beyond end of vector: {} > {}",
                    word_index,
                    self.data.as_ref().len()
                )
                .to_string(),
            ))
        } else {
            self.word_index = word_index as usize;
            Ok(())
        }
    }
}

impl<W: Word, B: AsMut<[W]>> WordWrite for MemWordWriterSlice<W, B> {
    type Error = std::io::Error;
    type Word = W;

    #[inline]
    fn write_word(&mut self, word: W) -> Result<(), std::io::Error> {
        match self.data.as_mut().get_mut(self.word_index) {
            Some(word_ref) => {
                self.word_index += 1;
                *word_ref = word;
                Ok(())
            }
            None => Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Cannot write next word as the underlying memory ended",
            )),
        }
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl<W: Word, B: AsMut<alloc::vec::Vec<W>>> WordWrite for MemWordWriterVec<W, B> {
    type Error = Infallible;
    type Word = W;

    #[inline]
    fn write_word(&mut self, word: W) -> Result<(), Infallible> {
        if self.word_index >= self.data.as_mut().len() {
            self.data.as_mut().resize(self.word_index + 1, W::ZERO);
        }
        self.data.as_mut()[self.word_index] = word;
        self.word_index += 1;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl<W: Word, B: AsMut<alloc::vec::Vec<W>>> WordRead for MemWordWriterVec<W, B> {
    type Error = std::io::Error;
    type Word = W;

    #[inline]
    fn read_word(&mut self) -> Result<W, std::io::Error> {
        match self.data.as_mut().get(self.word_index) {
            Some(word) => {
                self.word_index += 1;
                Ok(*word)
            }
            None => Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Cannot read next word as the underlying memory ended",
            )),
        }
    }
}

#[cfg(feature = "alloc")]
impl<W: Word, B: AsMut<alloc::vec::Vec<W>> + AsRef<alloc::vec::Vec<W>>> WordSeek
    for MemWordWriterVec<W, B>
{
    type Error = std::io::Error;

    #[inline(always)]
    fn word_pos(&mut self) -> Result<u64, std::io::Error> {
        Ok(self.word_index as u64)
    }

    #[inline(always)]
    fn set_word_pos(&mut self, word_index: u64) -> Result<(), std::io::Error> {
        if word_index > self.data.as_ref().len() as u64 {
            Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                format_args!(
                    "Position beyond end of vector: {} > {}",
                    word_index,
                    self.data.as_ref().len()
                )
                .to_string(),
            ))
        } else {
            self.word_index = word_index as usize;
            Ok(())
        }
    }
}

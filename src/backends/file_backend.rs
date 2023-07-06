/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use crate::traits::*;
use anyhow::Result;

/// A word backend implementation of [`WordStream`], [`WordRead`], [`WordWrite`]
/// for a generic file, this could transparently handle [`std::fs::File`],
/// [`std::io::BufReader`], [`std::io::BufWriter`], and sockets.
///
/// # Implementation details and decisions
/// While we could write blanket implementations for any generic type that
/// implements [`std::io::Read`], [`std::io::Write`], or [`std::io::Seek`],
/// doing so would force us to set an unique word `W`, while this wrapper allows
/// to choose the read and wite words at the cost of a more complicated type.
/// The alternative is to modify the WordSteam to have a generic type instead of
/// an associated one, but that would require the memory slices we read to
/// always be aligned to 16 bytes (u128). For memory mapped regions it's ok,
/// but we can't enforce it by types.
///
/// TODO!: maybe FileBackend is not the best name, as it's more generic than
/// that
pub struct FileBackend<W: Word, B> {
    file: B,
    position: usize,
    _marker: core::marker::PhantomData<W>,
}

impl<W: Word, B> FileBackend<W, B> {
    /// Create a new FileBackend
    pub fn new(file: B) -> Self {
        Self {
            file,
            position: 0,
            _marker: core::marker::PhantomData,
        }
    }
}

/// forward [`Clone`] if the backend supports it
impl<W: Word, B: Clone> Clone for FileBackend<W, B> {
    fn clone(&self) -> Self {
        Self {
            file: self.file.clone(),
            position: self.position,
            _marker: core::marker::PhantomData,
        }
    }
}

/// forward [`core::fmt::Debug`] if the backend supports it
impl<W: Word, B: core::fmt::Debug> core::fmt::Debug for FileBackend<W, B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.file.fmt(f)
    }
}

/// Convert [`std::io::Read`] to [`WordRead`]
impl<W: Word, B: std::io::Read> WordRead for FileBackend<W, B> {
    type Word = W;

    #[inline]
    fn read_next_word(&mut self) -> Result<W> {
        let mut res: W::BytesForm = Default::default();
        let _ = self.file.read(res.as_mut())?;
        self.position += W::BYTES;
        Ok(W::from_ne_bytes(res))
    }
}

/// Convert [`std::io::Write`] to [`WordWrite`]
impl<W: Word, B: std::io::Write> WordWrite for FileBackend<W, B> {
    type Word = W;

    #[inline]
    fn write_word(&mut self, word: W) -> Result<()> {
        let _ = self.file.write(word.to_ne_bytes().as_ref())?;
        self.position += W::BYTES;
        Ok(())
    }
}

/// Convert [`std::io::Seek`] to [`WordStream`]
impl<W: Word, B: std::io::Seek> WordStream for FileBackend<W, B> {
    #[inline(always)]
    fn get_position(&self) -> usize {
        self.position
    }

    #[inline(always)]
    fn set_position(&mut self, word_index: usize) -> Result<()> {
        self.position = word_index * W::BYTES;
        self.file
            .seek(std::io::SeekFrom::Start(self.position as u64))?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;
    #[test]
    fn test_file_backend() {
        let data: Vec<u32> = vec![
            0xa6032421, 0xc9d01b28, 0x168b4ecd, 0xc5ccbed9, 0xfd007100, 0x08469d41, 0x989fd8c2,
            0x954d351a, 0x3225ec9f, 0xbca253f9, 0x915aad84, 0x274c0de1, 0x4bfc6982, 0x59a47341,
            0x4e32a33a, 0x9e0d2208,
        ];
        let path = std::env::temp_dir().join("test_file_backend");
        {
            let mut writer = <FileBackend<u32, _>>::new(std::fs::File::create(&path).unwrap());
            for value in &data {
                writer.write_word(*value).unwrap();
            }
        }
        {
            let mut reader = <FileBackend<u32, _>>::new(std::fs::File::open(&path).unwrap());
            for value in &data {
                assert_eq!(*value, reader.read_next_word().unwrap());
            }
        }
    }

    #[test]
    fn test_file_backend_codes() {
        let data: Vec<u8> = vec![
            0x5f, 0x68, 0xdb, 0xca, 0x79, 0x17, 0xf3, 0x37, 0x2c, 0x46, 0x63, 0xf7, 0xf3, 0x28,
            0xa4, 0x8d, 0x29, 0x3b, 0xb6, 0xd5, 0xc7, 0xe2, 0x22, 0x3f, 0x6e, 0xb5, 0xf2, 0xda,
            0x13, 0x1d, 0x37, 0x18, 0x5b, 0xf8, 0x45, 0x59, 0x33, 0x38, 0xaf, 0xc4, 0x8a, 0x1d,
            0x78, 0x81, 0xc8, 0xc3, 0xdb, 0xab, 0x23, 0xe1, 0x13, 0xb0, 0x04, 0xd7, 0x3c, 0x21,
            0x0e, 0xba, 0x5d, 0xfc, 0xac, 0x4f, 0x04, 0x2d,
        ];
        let path = std::env::temp_dir().join("test_file_backend_codes");
        {
            let mut writer = <BufferedBitStreamWrite<BE, _>>::new(<FileBackend<u64, _>>::new(
                std::fs::File::create(&path).unwrap(),
            ));
            for value in &data {
                writer.write_gamma(*value as _).unwrap();
            }
        }
        {
            let mut reader = <BufferedBitStreamRead<BE, u64, _>>::new(<FileBackend<u32, _>>::new(
                std::fs::File::open(&path).unwrap(),
            ));
            for value in &data {
                assert_eq!(*value as u64, reader.read_gamma().unwrap());
            }
        }
        {
            let mut writer = <BufferedBitStreamWrite<LE, _>>::new(<FileBackend<u64, _>>::new(
                std::fs::File::create(&path).unwrap(),
            ));
            for value in &data {
                writer.write_gamma(*value as _).unwrap();
            }
        }
        {
            let mut reader = <BufferedBitStreamRead<LE, u64, _>>::new(<FileBackend<u32, _>>::new(
                std::fs::File::open(&path).unwrap(),
            ));
            for value in &data {
                assert_eq!(*value as u64, reader.read_gamma().unwrap());
            }
        }
    }
}

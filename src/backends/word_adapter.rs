/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use crate::traits::*;
use anyhow::Result;
use common_traits::*;

/// An adapter from [`std::io::Read`], [`std::io::Write`], and [`std::io::Seek`],
/// to [`WordRead`], [`WordWrite`], and [`WordSeek`], respectively.
///
/// Instances of this struct can be created using [`WordAdapter::new`]. They
/// turn every standard (possibly seekable) source or destination of
/// bytes (such as [`std::fs::File`], [`std::io::BufReader`], sockets, etc.)
/// into a source or destination of words.
#[derive(Clone)]
pub struct WordAdapter<W: Word, B> {
    file: B,
    position: usize,
    _marker: core::marker::PhantomData<W>,
}

impl<W: Word, B> WordAdapter<W, B> {
    /// Create a new WordAdapter
    pub fn new(file: B) -> Self {
        Self {
            file,
            position: 0,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<W: Word, B> core::fmt::Debug for WordAdapter<W, B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("WordAdapter")
            .field("file", &"CAN'T PRINT FILE")
            .field("position", &self.position)
            .finish()
    }
}

/// Convert [`std::io::Read`] to [`WordRead`]
impl<W: Word, B: std::io::Read> WordRead for WordAdapter<W, B> {
    type Word = W;

    #[inline]
    fn read_word(&mut self) -> Result<W> {
        let mut res: W::Bytes = Default::default();
        let _ = self.file.read(res.as_mut())?;
        self.position += 1;
        Ok(W::from_ne_bytes(res))
    }
}

/// Convert [`std::io::Write`] to [`WordWrite`]
impl<W: Word, B: std::io::Write> WordWrite for WordAdapter<W, B> {
    type Word = W;

    #[inline]
    fn write_word(&mut self, word: W) -> Result<()> {
        let _ = self.file.write(word.to_ne_bytes().as_ref())?;
        self.position += 1;
        Ok(())
    }
}

/// Convert [`std::io::Seek`] to [`WordSeek`]
impl<W: Word, B: std::io::Seek> WordSeek for WordAdapter<W, B> {
    #[inline(always)]
    fn get_word_pos(&self) -> usize {
        self.position
    }

    #[inline(always)]
    fn set_word_pos(&mut self, word_index: usize) -> Result<()> {
        self.position = word_index;
        self.file
            .seek(std::io::SeekFrom::Start((word_index * W::BYTES) as u64))?;
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
            let mut writer = <WordAdapter<u32, _>>::new(std::fs::File::create(&path).unwrap());
            for value in &data {
                writer.write_word(*value).unwrap();
            }
        }
        {
            let mut reader = <WordAdapter<u32, _>>::new(std::fs::File::open(&path).unwrap());
            for value in &data {
                assert_eq!(*value, reader.read_word().unwrap());
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
            let mut writer = <BufBitWriter<BE, _>>::new(<WordAdapter<u64, _>>::new(
                std::fs::File::create(&path).unwrap(),
            ));
            for value in &data {
                writer.write_gamma(*value as _).unwrap();
            }
        }
        {
            let mut reader = <BufBitReader<BE, u64, _>>::new(<WordAdapter<u32, _>>::new(
                std::fs::File::open(&path).unwrap(),
            ));
            for value in &data {
                assert_eq!(*value as u64, reader.read_gamma().unwrap());
            }
        }
        {
            let mut writer = <BufBitWriter<LE, _>>::new(<WordAdapter<u64, _>>::new(
                std::fs::File::create(&path).unwrap(),
            ));
            for value in &data {
                writer.write_gamma(*value as _).unwrap();
            }
        }
        {
            let mut reader = <BufBitReader<LE, u64, _>>::new(<WordAdapter<u32, _>>::new(
                std::fs::File::open(&path).unwrap(),
            ));
            for value in &data {
                assert_eq!(*value as u64, reader.read_gamma().unwrap());
            }
        }
    }
}

/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use crate::traits::*;
use common_traits::*;
#[cfg(feature = "mem_dbg")]
use mem_dbg::{MemDbg, MemSize};
use std::io::{Read, Seek, SeekFrom, Write};

/// An adapter from [`Read`], [`Write`], and [`Seek`], to [`WordRead`],
/// [`WordWrite`], and [`WordSeek`], respectively.
///
/// Instances of this struct can be created using [`WordAdapter::new`]. They
/// turn every standard (possibly seekable) source or destination of bytes (such
/// as [`std::fs::File`], [`std::io::BufReader`], sockets, etc.) into a source
/// or destination of words.
///
/// Due to the necessity of managing files whose length is not a multiple of the
/// word length, [`read_word`](WordAdapter::read_word) will return a partially
/// read word extended with zeros at the end of such files.
///
/// To provide a sensible value after such a read,
/// [`word_pos`](WordAdapter::word_pos) will always return the position
/// of the underlying [`Seek`] rounded up to the next multiple of `W::Bytes`.
/// This approach, however, requires that if you adapt a [`Seek`], its current position must be
/// a multiple of `W::Bytes`, or the results of [`word_pos`](WordAdapter::word_pos)
/// will be shifted by the rounding.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
pub struct WordAdapter<W: UnsignedInt + FromBytes + ToBytes, B> {
    backend: B,
    _marker: core::marker::PhantomData<W>,
}

impl<W: UnsignedInt + FromBytes + ToBytes, B> WordAdapter<W, B> {
    /// Create a new WordAdapter
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            _marker: core::marker::PhantomData,
        }
    }

    pub fn into_inner(self) -> B {
        self.backend
    }
}

impl<W: UnsignedInt + ToBytes + FromBytes + FiniteRangeNumber, B: Read> WordRead
    for WordAdapter<W, B>
{
    type Error = std::io::Error;
    type Word = W;

    #[inline(always)]
    fn read_word(&mut self) -> Result<W, Self::Error> {
        let mut res: W::Bytes = Default::default();
        self.backend
            .read_exact(res.as_mut())
            .map_err(|e|
                match e.kind() {
                std::io::ErrorKind::UnexpectedEof => {
                    std::io::Error::new(
                        e.kind(),
                        format!(concat!(
                            "Unexpected end of file. ",
                            "This might happen because the file length is not a multiple of the word size used for reading ({} bytes). ",
                            "In this case, please pad with zeros at the end of the file so that the file length is a multiple of {0}. ",
                            "The inner std::io::Error was {:?}"), W::BYTES, e),
                    )
                }
                _ => e,
            })?;
        Ok(W::from_ne_bytes(res))
    }
}

impl<W: UnsignedInt + ToBytes + FromBytes + FiniteRangeNumber, B: Write> WordWrite
    for WordAdapter<W, B>
{
    type Error = std::io::Error;
    type Word = W;

    #[inline(always)]
    fn write_word(&mut self, word: W) -> Result<(), std::io::Error> {
        let _ = self.backend.write(word.to_ne_bytes().as_ref())?;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.backend.flush()
    }
}

impl<W: UnsignedInt + ToBytes + FromBytes + FiniteRangeNumber, B: Seek> WordSeek
    for WordAdapter<W, B>
{
    type Error = std::io::Error;

    #[inline(always)]
    fn word_pos(&mut self) -> Result<u64, std::io::Error> {
        Ok(self.backend.stream_position()?.div_ceil(W::BYTES as u64))
    }

    #[inline(always)]
    fn set_word_pos(&mut self, word_index: u64) -> Result<(), std::io::Error> {
        self.backend
            .seek(SeekFrom::Start(word_index * W::BYTES as u64))?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;
    #[test]
    fn test_word_adapter() {
        let data: Vec<u32> = vec![
            0xa6032421, 0xc9d01b28, 0x168b4ecd, 0xc5ccbed9, 0xfd007100, 0x08469d41, 0x989fd8c2,
            0x954d351a, 0x3225ec9f, 0xbca253f9, 0x915aad84, 0x274c0de1, 0x4bfc6982, 0x59a47341,
            0x4e32a33a, 0x9e0d2208,
        ];
        let path = std::env::temp_dir().join("test_file_adapter");
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
    fn test_word_adapter_codes() {
        let data: Vec<u8> = vec![
            0x5f, 0x68, 0xdb, 0xca, 0x79, 0x17, 0xf3, 0x37, 0x2c, 0x46, 0x63, 0xf7, 0xf3, 0x28,
            0xa4, 0x8d, 0x29, 0x3b, 0xb6, 0xd5, 0xc7, 0xe2, 0x22, 0x3f, 0x6e, 0xb5, 0xf2, 0xda,
            0x13, 0x1d, 0x37, 0x18, 0x5b, 0xf8, 0x45, 0x59, 0x33, 0x38, 0xaf, 0xc4, 0x8a, 0x1d,
            0x78, 0x81, 0xc8, 0xc3, 0xdb, 0xab, 0x23, 0xe1, 0x13, 0xb0, 0x04, 0xd7, 0x3c, 0x21,
            0x0e, 0xba, 0x5d, 0xfc, 0xac, 0x4f, 0x04, 0x2d,
        ];
        let path = std::env::temp_dir().join("test_file_adapter_codes");
        {
            let mut writer = <BufBitWriter<BE, _>>::new(<WordAdapter<u64, _>>::new(
                std::fs::File::create(&path).unwrap(),
            ));
            for value in &data {
                writer.write_gamma(*value as _).unwrap();
            }
        }
        {
            let mut reader = <BufBitReader<BE, _>>::new(<WordAdapter<u32, _>>::new(
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
            let mut reader = <BufBitReader<LE, _>>::new(<WordAdapter<u32, _>>::new(
                std::fs::File::open(&path).unwrap(),
            ));
            for value in &data {
                assert_eq!(*value as u64, reader.read_gamma().unwrap());
            }
        }
    }
}

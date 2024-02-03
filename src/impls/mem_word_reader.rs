/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use core::convert::Infallible;

use crate::prelude::*;

/**

An implementation of [`WordRead`] and [`WordSeek`] for a slice.

The implementation depends on the `INF` parameter: if true, the reader will
behave as if the slice is infinitely extended with zeros.
If false, the reader will return an error when reading
beyond the end of the slice. You can create a zero-extended
reader with [`MemWordReader::new`] and a strict reader with
[`MemWordReader::new_strict`].

The zero-extended reader is usually much faster than the strict reader, but
it might loop infinitely when reading beyond the end of the slice.

# Examples

```rust
use dsi_bitstream::prelude::*;

let words: [u32; 2] = [1, 2];

let mut word_reader = MemWordReader::new(&words);
assert_eq!(1, word_reader.read_word().unwrap());
assert_eq!(2, word_reader.read_word().unwrap());
assert_eq!(0, word_reader.read_word().unwrap());
assert_eq!(0, word_reader.read_word().unwrap());

let mut word_reader = MemWordReader::new_strict(&words);
assert_eq!(1, word_reader.read_word().unwrap());
assert_eq!(2, word_reader.read_word().unwrap());
assert!(word_reader.read_word().is_err());
```
*/
#[derive(Debug, Clone, PartialEq)]
pub struct MemWordReader<W: Word, B: AsRef<[W]>, const INF: bool = true> {
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

    pub fn into_inner(self) -> B {
        self.data
    }
}

impl<W: Word, B: AsRef<[W]>> MemWordReader<W, B, false> {
    /// Create a new [`MemWordReader`] from a slice of data
    #[must_use]
    pub fn new_strict(data: B) -> Self {
        Self {
            data,
            word_index: 0,
            _marker: Default::default(),
        }
    }
}

impl<W: Word, B: AsRef<[W]>> WordRead for MemWordReader<W, B, true> {
    type Error = Infallible;
    type Word = W;

    #[inline(always)]
    fn read_word(&mut self) -> Result<W, Infallible> {
        let res = self
            .data
            .as_ref()
            .get(self.word_index)
            .copied()
            .unwrap_or(Self::Word::ZERO);

        self.word_index += 1;
        Ok(res)
    }
}

impl<W: Word, B: AsRef<[W]>> WordRead for MemWordReader<W, B, false> {
    type Error = std::io::Error;
    type Word = W;

    #[inline(always)]
    fn read_word(&mut self) -> Result<W, std::io::Error> {
        let res = self
            .data
            .as_ref()
            .get(self.word_index)
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Unexpected end of slice",
            ))?;

        self.word_index += 1;
        Ok(*res)
    }
}

impl<W: Word, B: AsRef<[W]>> WordSeek for MemWordReader<W, B, true> {
    type Error = Infallible;

    #[inline(always)]
    fn get_word_pos(&mut self) -> Result<u64, Infallible> {
        Ok(self.word_index as u64)
    }

    #[inline(always)]
    fn set_word_pos(&mut self, word_index: u64) -> Result<(), Infallible> {
        // This is dirty but it's infallible
        self.word_index = word_index.min(usize::MAX as u64) as usize;
        Ok(())
    }
}

impl<W: Word, B: AsRef<[W]>> WordSeek for MemWordReader<W, B, false> {
    type Error = std::io::Error;

    #[inline(always)]
    fn get_word_pos(&mut self) -> Result<u64, std::io::Error> {
        Ok(self.word_index as u64)
    }
    #[inline(always)]
    fn set_word_pos(&mut self, word_index: u64) -> Result<(), std::io::Error> {
        return if word_index > self.data.as_ref().len() as u64 {
            Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                format_args!(
                    "Position beyond end of slice: {} > {}",
                    word_index,
                    self.data.as_ref().len()
                )
                .to_string(),
            ))
        } else {
            self.word_index = word_index as usize;
            Ok(())
        };
    }
}

#[test]

fn test_eof_table_read() {
    use crate::codes::{DeltaReadParam, DeltaWrite};
    let mut words: [u64; 1] = [0];
    let mut writer = crate::prelude::BufBitWriter::<crate::prelude::LE, _>::new(
        MemWordWriterSlice::new(&mut words),
    );
    for _ in 0..16 {
        writer.write_delta(1).unwrap();
    }
    writer.flush().unwrap();
    drop(writer);

    let mut reader =
        crate::prelude::BufBitReader::<crate::prelude::LE, _>::new(MemWordReader::new(&words));
    for _ in 0..16 {
        // Here the last table read make peek_bits return Ok(None)
        assert_eq!(1, reader.read_delta_param::<true, true>().unwrap());
    }
}

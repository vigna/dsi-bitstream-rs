/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use anyhow::Result;
use common_traits::UnsignedInt;

use crate::prelude::*;

/// A zero-extended implementation of [`WordRead`] and [`WordSeek`] for a slice.
///
/// Trying to read beyond the end of the slice will return zeros.
/// Use [`MemWordReaderStrict`] if you wanna obtain an error instead.
///
/// Note that zero extension is usually what you want, as memory written with
/// a certain word size will cause end-of-stream errors when read with a larger
/// word size.
#[derive(Debug, Clone, PartialEq)]
pub struct MemWordReader<W: UnsignedInt, B: AsRef<[W]>> {
    data: B,
    word_index: usize,
    _marker: core::marker::PhantomData<W>,
}

impl<W: UnsignedInt, B: AsRef<[W]>> MemWordReader<W, B> {
    /// Create a new [`MemWordReader`] from a slice of data
    #[must_use]
    pub fn new(data: B) -> Self {
        Self {
            data,
            word_index: 0,
            _marker: Default::default(),
        }
    }
}
impl<W: UnsignedInt, B: AsRef<[W]>> WordRead for MemWordReader<W, B> {
    type Word = W;

    #[inline(always)]
    fn read_word(&mut self) -> Result<W> {
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

impl<W: UnsignedInt, B: AsRef<[W]>> WordSeek for MemWordReader<W, B> {
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

    let mut reader =
        crate::prelude::BufBitReader::<crate::prelude::LE, _>::new(MemWordReader::new(&words));
    for _ in 0..16 {
        // Here the last table read make peek_bits return Ok(None)
        assert_eq!(1, reader.read_delta_param::<true, true>().unwrap());
    }
}

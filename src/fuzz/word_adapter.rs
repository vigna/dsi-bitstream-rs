/*
* SPDX-FileCopyrightText: 2024 Tommaso Fontana
*
* SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
*/

use crate::prelude::*;
use arbitrary::Arbitrary;
use common_traits::{DowncastableFrom, DowncastableInto};
use std::io::Cursor;

#[derive(Arbitrary, Debug)]
pub enum WordSize {
    U8,
    U16,
    U32,
    U64,
}

#[derive(Arbitrary, Debug)]
pub struct FuzzCase {
    word_size: WordSize,
    init: Vec<u8>,
    commands: Vec<RandomCommand>,
}

#[derive(Arbitrary, Debug)]
pub enum RandomCommand {
    GetPosition,
    SetPosition(usize),
    ReadNextWord,
    WriteWord(u64),
}

fn fuzz<W: Word + DowncastableFrom<u64>>(mut data: FuzzCase) {
    let mut idx: usize = 0;
    let mut truth_data = data.init.clone();
    let data_len = data.init.len();
    let read_word_limit = data.init.len().div_ceil(W::BYTES);
    let write_word_limit = data.init.len() / W::BYTES;
    let mut adapter = WordAdapter::<W, _>::new(Cursor::new(&mut data.init));
    for command in data.commands {
        match command {
            RandomCommand::GetPosition => {
                assert_eq!(adapter.word_pos().unwrap(), idx as u64);
            }
            RandomCommand::SetPosition(word_index) => {
                if word_index <= read_word_limit {
                    // this is inside because doing a seek on std::io::Cursor works
                    // for any seek offset, even if out of bound.
                    let _ = adapter.set_word_pos(word_index as u64);
                    idx = word_index;
                }
            }
            RandomCommand::ReadNextWord => {
                if adapter.word_pos().unwrap() < read_word_limit as u64 {
                    // manual implementation of read next word from the vector
                    let mut buffer = W::Bytes::default();
                    // in vector indices
                    let start = idx * W::BYTES;
                    let end = (start + W::BYTES).min(data_len);
                    // this implicitly zero extends the data
                    let buffer_end = end - start;
                    buffer.as_mut()[0..buffer_end].copy_from_slice(&truth_data[start..end]);

                    assert_eq!(adapter.read_word().unwrap(), W::from_ne_bytes(buffer),);
                    idx += 1;
                }
            }
            RandomCommand::WriteWord(new_word) => {
                if adapter.word_pos().unwrap() < write_word_limit as u64 {
                    let new_word: W = new_word.downcast();
                    adapter.write_word(new_word).unwrap();
                    let start = idx * W::BYTES;
                    let end = start + W::BYTES;
                    truth_data[start..end].copy_from_slice(new_word.to_ne_bytes().as_ref());
                    let _ = adapter.flush();
                    idx += 1;
                }
            }
        };
    }
}

pub fn harness(data: FuzzCase) {
    match data.word_size {
        WordSize::U8 => fuzz::<u8>(data),
        WordSize::U16 => fuzz::<u16>(data),
        WordSize::U32 => fuzz::<u32>(data),
        WordSize::U64 => fuzz::<u64>(data),
    }
}

/*
 * SPDX-FileCopyrightText: 2023 Inria
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use crate::prelude::*;
use arbitrary::Arbitrary;

#[derive(Arbitrary, Debug)]
pub struct FuzzCase {
    init: Vec<u64>,
    commands: Vec<RandomCommand>,
}

#[derive(Arbitrary, Debug)]
pub enum RandomCommand {
    Len,
    GetPosition,
    SetPosition(usize),
    ReadWord,
    WriteWord(u64),
}

pub fn harness(data: FuzzCase) {
    let mut idx = 0;
    let mut buffer = data.init.clone();
    let mut buffer2 = data.init.clone();

    let mut writer = MemWordWriterSlice::new(&mut buffer2);
    for command in data.commands {
        match command {
            RandomCommand::Len => {
                assert_eq!(writer.len(), buffer.len());
            }
            RandomCommand::GetPosition => {
                assert_eq!(writer.get_word_pos().unwrap(), idx);
            }
            RandomCommand::SetPosition(word_index) => {
                let _ = writer.set_word_pos(word_index as u64);
                if buffer.get(word_index).is_some() {
                    idx = word_index as u64;
                }
            }
            RandomCommand::ReadWord => {
                assert_eq!(writer.read_word().ok(), buffer.get(idx as usize).copied());
                if buffer.get(idx as usize).is_some() {
                    idx += 1;
                }
            }
            RandomCommand::WriteWord(word) => {
                let can_write = if let Some(w) = buffer.get_mut(idx as usize) {
                    *w = word;
                    true
                } else {
                    false
                };
                assert_eq!(writer.write_word(word).is_ok(), can_write);
                if can_write {
                    idx += 1;
                }
            }
        };
    }
}

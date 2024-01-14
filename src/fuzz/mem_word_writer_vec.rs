/*
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use crate::prelude::*;
use arbitrary::Arbitrary;

#[derive(Arbitrary, Debug)]
pub struct FuzzCase {
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
    let mut buffer = vec![];
    let mut buffer2 = vec![];

    let mut writer = MemWordWriterVec::new(&mut buffer2);
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
                if word_index <= buffer.len() {
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
                if idx >= buffer.len() as u64 {
                    buffer.resize(idx as usize + 1, 0);
                }
                assert!(writer.write_word(word).is_ok());
                buffer[idx as usize] = word;
                idx += 1;
            }
        };
    }
}

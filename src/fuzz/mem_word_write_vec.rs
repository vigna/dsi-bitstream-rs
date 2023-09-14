/*
 * SPDX-FileCopyrightText: 2023 Inria
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
    ReadNextWord,
    WriteWord(u64),
}

pub fn harness(data: FuzzCase) {
    let mut idx = 0;
    let mut buffer = vec![];
    let mut buffer2 = vec![];

    let mut writer = MemWordWriteVec::new(&mut buffer2);
    for command in data.commands {
        match command {
            RandomCommand::Len => {
                assert_eq!(writer.len(), buffer.len());
            }
            RandomCommand::GetPosition => {
                assert_eq!(writer.get_position(), idx);
            }
            RandomCommand::SetPosition(word_index) => {
                let _ = writer.set_position(word_index);
                if buffer.get(word_index).is_some() {
                    idx = word_index;
                }
            }
            RandomCommand::ReadNextWord => {
                assert_eq!(writer.read().ok(), buffer.get(idx).copied());
                if buffer.get(idx).is_some() {
                    idx += 1;
                }
            }
            RandomCommand::WriteWord(word) => {
                if idx >= buffer.len() {
                    buffer.resize(idx + 1, 0);
                }
                assert!(writer.write(word).is_ok());
                buffer[idx] = word;
                idx += 1;
            }
        };
    }
}

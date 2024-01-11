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
    GetPosition,
    SetPosition(usize),
    ReadNextWord,
}

pub fn harness(data: FuzzCase) {
    let mut idx = 0;
    let mut reader = MemWordReader::new(&data.init);
    for command in data.commands {
        match command {
            RandomCommand::GetPosition => {
                assert_eq!(reader.get_word_pos().unwrap(), idx);
            }
            RandomCommand::SetPosition(word_index) => {
                let _ = reader.set_word_pos(word_index);
                if data.init.get(word_index).is_some() {
                    idx = word_index;
                }
            }
            RandomCommand::ReadNextWord => {
                assert_eq!(reader.read_word().ok(), data.init.get(idx).copied());
                if data.init.get(idx).is_some() {
                    idx += 1;
                }
            }
        };
    }
}

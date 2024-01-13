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
                let _ = reader.set_word_pos(word_index as u64);
                if data.init.get(word_index).is_some() {
                    idx = word_index as u64;
                }
            }
            RandomCommand::ReadNextWord => {
                if reader.get_word_pos().unwrap() != u64::MAX {
                    assert_eq!(
                        reader.read_word().ok(),
                        Some(data.init.get(idx as usize).copied().unwrap_or(0))
                    );
                    idx += 1;
                }
            }
        };
    }
}

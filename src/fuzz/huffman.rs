/*
* SPDX-FileCopyrightText: 2023 Tommaso Fontana
*
* SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
*/

use crate::codes::huffman::HuffmanTree;
use crate::prelude::*;
use arbitrary::Arbitrary;
use std::io::Seek;

#[derive(Arbitrary, Debug)]
pub struct FuzzCase {
    counts: Vec<u16>,
}

pub fn harness(data: FuzzCase) {
    let counts = data.counts.iter().map(|x| *x as usize).collect::<Vec<_>>();
    let huffman = HuffmanTree::new(&counts);
    if huffman.is_err() {
        return;
    }
    let huffman = huffman.unwrap();

    let mut writer = BufBitWriter::<BigEndian, _>::new(WordAdapter::<u32, _>::new(
        std::io::Cursor::new(Vec::new()),
    ));

    let mut bit_off = 0;
    for i in 0..counts.len() {
        bit_off += huffman.encode(i as u64, &mut writer).unwrap();
    }
    let mut data = writer.into_inner().unwrap().into_inner();
    data.seek(std::io::SeekFrom::Start(0)).unwrap();

    let mut reader = BufBitReader::<BigEndian, _>::new(WordAdapter::<u32, _>::new(data));

    for i in 0..counts.len() {
        assert_eq!(
            huffman.write_table[i].code,
            reader
                .read_bits(huffman.write_table[i].len as usize)
                .unwrap() as usize
        );
    }

    reader.set_bit_pos(0).unwrap();

    for i in 0..counts.len() {
        assert_eq!(huffman.decode(&mut reader).unwrap(), i as u64);
    }
}

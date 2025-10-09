/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */
#![cfg(feature = "std")]

use anyhow::Result;
use dsi_bitstream::prelude::*;
use std::io::Cursor;
use std::io::{BufReader, BufWriter};

#[test]
fn test_word_pos() -> Result<()> {
    let mut data: Vec<u8> = vec![];

    // create a bit writer on the file
    let mut writer = <BufBitWriter<NE, _>>::new(<WordAdapter<u64, _>>::new(
        BufWriter::with_capacity(1 << 20, Cursor::new(&mut data)),
    ));

    println!("Write Gammas...");
    for n in 0..100 {
        writer.write_gamma(n)?;
    }
    writer.flush()?;
    drop(writer);

    let reader = BufBitReader::<NE, _>::new(WordAdapter::<u32, _>::new(BufReader::new(
        Cursor::new(&mut data),
    )));
    let mut reader = CountBitReader::<_, _, false>::new(reader);
    for _ in 0..100 {
        let _ = reader.read_gamma()?;
        assert_eq!(
            reader.bits_read as u64,
            reader.bit_pos()?,
            "Number of bits read and position in bit stream are different"
        );
    }
    Ok(())
}

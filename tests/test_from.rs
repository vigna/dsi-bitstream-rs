/*
 * SPDX-FileCopyrightText: 2025 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */
#![cfg(feature = "std")]

use std::{env, path::PathBuf};

use dsi_bitstream::{
    codes::{DeltaRead, DeltaWrite},
    impls::{buf_bit_reader, buf_bit_writer},
    traits::LE,
};

#[test]
fn test_from() -> Result<(), Box<dyn core::error::Error>> {
    let temp_dir: PathBuf = env::temp_dir();
    let mut writer = buf_bit_writer::from_path::<LE, u64>(&temp_dir.join("test.bin"))?;
    for i in 0..100 {
        writer.write_delta(i)?;
    }
    drop(writer);
    let mut reader = buf_bit_reader::from_path::<LE, u64>(&temp_dir.join("test.bin"))?;
    for i in 0..100 {
        assert_eq!(reader.read_delta()?, i);
    }
    Ok(())
}

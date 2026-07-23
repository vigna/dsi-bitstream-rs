/*
 * SPDX-FileCopyrightText: 2025 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */
#![cfg(feature = "std")]

use std::env;

use dsi_bitstream::{
    codes::{DeltaRead, DeltaWrite},
    impls::{buf_bit_reader, buf_bit_writer},
    traits::LE,
};

#[test]
fn test_from() -> Result<(), Box<dyn core::error::Error>> {
    // Unique per-process path so concurrent test binaries do not clobber one
    // another (and to avoid reusing a predictable, possibly pre-existing file).
    let path = env::temp_dir().join(format!(
        "dsi_bitstream_test_from_{}.bin",
        std::process::id()
    ));
    let mut writer = buf_bit_writer::from_path::<LE, u64>(&path)?;
    for i in 0..100 {
        writer.write_delta(i)?;
    }
    drop(writer);
    let mut reader = buf_bit_reader::from_path::<LE, u64>(&path)?;
    for i in 0..100 {
        assert_eq!(reader.read_delta()?, i);
    }
    drop(reader);
    let _ = std::fs::remove_file(&path);
    Ok(())
}

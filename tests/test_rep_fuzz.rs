/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

#[cfg(feature = "fuzz")]
use std::error::Error;

macro_rules! impl_fuzz_repr {
    ($func_name:ident, $func_zip_name:ident, $fuzz_name:ident) => {
        #[cfg(feature = "fuzz")]
        #[test]
        fn $func_name() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
            use arbitrary::Arbitrary;
            use dsi_bitstream::fuzz::$fuzz_name::*;
            use std::io::Read;
            let dir = format!("fuzz/corpus/{}", stringify!($fuzz_name));
            if !std::path::Path::new(&dir).exists() {
                eprintln!("The corpus directory {} does not exist", dir);
                return Ok(());
            }
            for file in std::fs::read_dir(&dir)? {
                let file = file?;

                if file.file_type()?.is_dir() {
                    continue;
                }

                let filename = format!("{}/{}", dir, file.file_name().to_string_lossy());
                let mut file = std::fs::File::open(&filename)?;
                let metadata = std::fs::metadata(&filename)?;
                let mut file_bytes = vec![0; metadata.len() as usize];
                file.read(&mut file_bytes)?;

                let mut unstructured = arbitrary::Unstructured::new(&file_bytes);
                let data = FuzzCase::arbitrary(&mut unstructured)?;
                dsi_bitstream::fuzz::$fuzz_name::harness(data);
            }

            Ok(())
        }

        #[cfg(feature = "fuzz")]
        #[test]
        fn $func_zip_name() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
            use arbitrary::Arbitrary;
            use dsi_bitstream::fuzz::$fuzz_name::*;
            use std::io::prelude::*;

            let zip_path = format!("tests/corpus/{}.zip", stringify!($fuzz_name));
            let zip_file = std::fs::File::open(&zip_path)?;
            let mut zip = zip::ZipArchive::new(zip_file)?;

            for i in 0..zip.len() {
                let mut file = zip.by_index(i)?;

                if file.is_dir() {
                    continue;
                }

                let mut file_bytes = vec![0; file.size() as usize];
                file.read(&mut file_bytes)?;

                let mut unstructured = arbitrary::Unstructured::new(&file_bytes);
                let data = FuzzCase::arbitrary(&mut unstructured)?;
                dsi_bitstream::fuzz::$fuzz_name::harness(data);
            }

            Ok(())
        }
    };
}

impl_fuzz_repr!(fuzz_codes, fuzz_codes_zip, codes);

impl_fuzz_repr!(
    fuzz_mem_word_reader,
    fuzz_mem_word_reader_zip,
    mem_word_reader
);
impl_fuzz_repr!(
    fuzz_mem_word_writer,
    fuzz_mem_word_writer_zip,
    mem_word_writer
);
impl_fuzz_repr!(
    fuzz_mem_word_writer_vec,
    fuzz_mem_word_writer_vec_zip,
    mem_word_writer_vec
);
impl_fuzz_repr!(
    fuzz_word_adapter,
    fuzz_word_adapter_zip,
    word_adapter
);

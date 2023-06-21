#[cfg(feature = "fuzz")]
use anyhow::Result;

macro_rules! impl_fuzz_repr {
    ($func_name:ident, $fuzz_name:ident) => {
        #[cfg(feature = "fuzz")]
        #[test]
        fn $func_name() -> Result<()> {
            use arbitrary::Arbitrary;
            use dsi_bitstream::fuzz::$fuzz_name::*;
            use std::io::Read;
            let dir = format!("fuzz/corpus/{}", stringify!($fuzz_name));
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
                dsi_bitstream::fuzz::$fuzz_name::$fuzz_name(data);
            }

            Ok(())
        }
    };
}

impl_fuzz_repr!(test_rep_fuzz_codes, codes);
impl_fuzz_repr!(test_rep_fuzz_mem_word_read, mem_word_read);
impl_fuzz_repr!(test_rep_fuzz_mem_word_write, mem_word_write);
impl_fuzz_repr!(test_rep_fuzz_mem_word_write_vec, mem_word_write_vec);

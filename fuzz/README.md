# Fuzzing

This crate provides fuzzing for `dsi-bitstream` using `cargo-fuzz` (you will
need the nightly compiler).

The fuzzing harnesses, however, can be found in `dsi-bitstream::fuzz`, 
so you can easily replace `cargo-fuzz` with any other fuzzing framework.

## Precomputed corpora

We distribute fuzzing-generated precomputed corpora that will 
be used during testing of the main crate when the feature `fuzz` is enabled, 
but it is possible to regenerate them.

To update one of the selected corpus zip files, e.g., `codes.zip`:
```shell
TARGET="codes"
# temp dir
mkdir tmp
# Extract the files
unzip "tests/corpus/${TARGET}.zip" -d tmp
# Merge and deduplicate the current corpus 
cargo fuzz run ${TARGET} -- -merge=1 tmp fuzz/corpus/${TARGET}
# Recompress from inside tmp so entries have no tmp/ prefix, into a fresh
# archive (remove the old one first so stale entries are not kept)
rm -f "tests/corpus/${TARGET}.zip"
(cd tmp && zip -r "../tests/corpus/${TARGET}.zip" .)
# Delete tmp folder
rm -rfd tmp
```

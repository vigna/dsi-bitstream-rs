# dsi-bistream

A Rust implementation of read/write bit streams supporting several types
of instantaneous codes. It mimics the behavior of the analogous classes in
the [DSI Utilities](https://dsiutils.di.unimi.it/).

```rust
use dsi_bitstream::prelude::*;
// where the codes will be written to, this can also be a file, or a memory slice
let mut data = Vec::<u64>::new();
// write some data
{
    // create a codes writer
    let mut writer = BufBitWriter::<BigEndian, _>::new(MemWordWriterVec::new(&mut data));
    // write 0 using 10 bits
    writer.write_bits(0, 10).unwrap();
    // write 1 in unary
    writer.write_unary(1).unwrap();
    // write 2 in gamma
    writer.write_gamma(2).unwrap();
    // write 3 in delta
    writer.write_delta(3).unwrap();
    // write 4 in zeta 3
    writer.write_zeta(4, 3).unwrap();
}
// read them back
{
    // create a codes reader
    let mut reader = BufBitReader::<BigEndian, u128, _>::new(MemWordReader::new(&data));
    // read back the data
    assert_eq!(reader.read_bits(10).unwrap(), 0);
    assert_eq!(reader.read_unary().unwrap(), 1);
    assert_eq!(reader.read_gamma().unwrap(), 2);
    assert_eq!(reader.read_delta().unwrap(), 3);
    assert_eq!(reader.read_zeta(3).unwrap(), 4);
}
```

## Coverage
```shell
cargo tarpaulin --engine llvm
```
If you also want to run the fuzzing test cases use:
```shell
cargo tarpaulin --engine llvm --features="fuzz"
```
This will reproduce our selected corpus zip files at `tests/corpus/` and
run your local data corpus in `fuzz/corpus/`.

## Fuzzing
The fuzzing harnesses can be found in `dsi-bitstream::fuzz`, so you can use 
any fuzzing framework you want. The simplest is `cargo-fuzz`, which
can be installed as:
```shell
cargo install cargo-fuzz
```
To find the current targets:
```shell
cargo fuzz list
```
To start the fuzzing
```shell
cargo fuzz run codes
```
### Coverage

To compute the coverage in `lcov` format:
```shell
cargo tarpaulin --engine llvm --features="fuzz" -o lcov
```
### Corpus.zip

To update one of the selected corpus zip files:
```shell
TARGET="codes"
# temp dir
mkdir tmp
# Extract the files
unzip "tests/corpus/${TARGET}.zip" -d tmp
# Merge and deduplicate the current corpus 
cargo fuzz run ${TARGET} -- -merge=1 tmp fuzz/corpus/${TARGET}
# Recompress
zip tests/corpus/${TARGET}.zip tmp/*
# Delete tmp folder
rm -rfd tmp
```

## Benchmarking

The implementation has several tunable parameters that can be used to improve performance 
on certain platforms. The default values are set to work well on most platforms, but you can
customize them creating your own copy of the library.

You can run benchmarks and generate SVG plots with
```shell
./python/gen_plots.sh
```
which starts a few Python scripts (you can run selectively the scripts
for a more fine-grained control).
The cargo options in `benchmarks`select aggressive optimizations, and the 
the python scripts run the benchmarks `--target-cpu=native`.

The resulting figures report the performance of read and write operation
on all codes, in both little-ending and big-endiang format. The code may
use or not decoding tables, and in the first case results are reported
for different sizes of the decoding tables.

There are three entry points for altering the behavior of the code:

- The size of the tables can be set in the source of the script
  `gen_code_tables.py`. Running the script will generate new tables
   with the provided parameters.
- Whether to use tables for unary code can only be configured in the source
  of the [`BitRead::read_unary`](crate::traits::BitRead::read_unary) and 
  [`BitWrite::write_unary`](crate::traits::BitWrite::write_unary) functions, but the
  default (no table) is the best choice on all architetures we are
  aware of.
- Whether to use tables for all other codes can be configured by
  passing around a different implementations of 
  [`ReadParams`](crate::codes::table_params::ReadParams)
  and [`WriteParams`](crate::codes::table_params::WriteParams)
  in place of the default 
  [`DefaultReadParams`](crate::codes::table_params::DefaultReadParams) and
  [`DefaultWriteParams`](crate::codes::table_params::DefaultWriteParams).

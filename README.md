# dsi-bitstream

[![downloads](https://img.shields.io/crates/d/dsi-bitstream)](https://crates.io/crates/dsi-bitstream)
[![dependents](https://img.shields.io/librariesio/dependents/cargo/dsi-bitstream)](https://crates.io/crates/dsi-bitstream/reverse_dependencies)
![GitHub CI](https://github.com/vigna/dsi-bitstream-rs/actions/workflows/rust.yml/badge.svg)
![license](https://img.shields.io/crates/l/dsi-bitstream)
[![](https://tokei.rs/b1/github/vigna/dsi-bitstream-rs?type=Rust,Python)](https://github.com/vigna/dsi-bitstream-rs)
[![Latest version](https://img.shields.io/crates/v/dsi-bitstream.svg)](https://crates.io/crates/dsi-bitstream)
[![Documentation](https://docs.rs/dsi-bitstream/badge.svg)](https://docs.rs/dsi-bitstream)
[![Coverage Status](https://coveralls.io/repos/github/vigna/dsi-bitstream-rs/badge.svg?branch=main)](https://coveralls.io/github/vigna/dsi-bitstream-rs?branch=main)

A Rust implementation of bit streams supporting several types of instantaneous
codes for compression.

This library mimics the behavior of the analogous classes in the [DSI
Utilities], but it aims at being much more flexible and (hopefully) efficient.

The two main traits are [`BitRead`] and [`BitWrite`], with which are associated
two main implementations [`BufBitReader`] and [`BufBitWriter`]. Additional
traits make it possible to read and write instantaneous codes, like the
[exponential Golomb codes] used in [H.264 (MPEG-4)] and [H.265].

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use dsi_bitstream::prelude::*;
// To write a bit stream, we need first a WordWrite around an output backend
// (in this case, a vector), which is word-based for efficiency.
// It could be a file, etc. 
let mut word_write = MemWordWriterVec::new(Vec::<u64>::new());
// Let us create a little-endian bit writer. The write word size will be inferred.
let mut writer = BufBitWriter::<LE, _>::new(word_write);
// Write 0 using 10 bits
writer.write_bits(0, 10)?;
// Write 1 in unary code
writer.write_unary(0)?;
// Write 2 in γ code
writer.write_gamma(1)?;
// Write 3 in δ code
writer.write_delta(2)?;
writer.flush();

// Let's recover the data
let data = writer.into_inner()?.into_inner();

// Reading back the data is similar, but since a reader has a bit buffer
// twice as large as the read word size, it is more efficient to use a 
// u32 as read word, so we need to recreate the vector from its pointer.
let data = std::mem::ManuallyDrop::new(data);
let data = unsafe {
    let ptr = data.as_ptr() as *mut u32;
    Vec::from_raw_parts(ptr, data.len() * 2, data.capacity() * 2)
};
let mut reader = BufBitReader::<LE, _>::new(MemWordReader::new(data));
assert_eq!(reader.read_bits(10)?, 0);
assert_eq!(reader.read_unary()?, 0);
assert_eq!(reader.read_gamma()?, 1);
assert_eq!(reader.read_delta()?, 2);

# Ok(())
# }
```

In this case, the backend is already word-based, but if you have a byte-based
backend such as a file [`WordAdapter`] can be used to adapt it to a word-based
backend.

You can also use references to backends instead of owned values,
but this approach is less efficient:

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use dsi_bitstream::prelude::*;
let mut data = Vec::<u64>::new();
let mut word_write = MemWordWriterVec::new(&mut data);
let mut writer = BufBitWriter::<LE, _>::new(word_write);
writer.write_bits(0, 10)?;
writer.write_unary(0)?;
writer.write_gamma(1)?;
writer.write_delta(2)?;
writer.flush();
drop(writer); // We must drop the writer release the borrow on data

let data = std::mem::ManuallyDrop::new(data);
let data = unsafe {
    let ptr = data.as_ptr() as *mut u32;
    Vec::from_raw_parts(ptr, data.len() * 2, data.capacity() * 2)
};
let mut reader = BufBitReader::<LE, _>::new(MemWordReader::new(&data));
assert_eq!(reader.read_bits(10)?, 0);
assert_eq!(reader.read_unary()?, 0);
assert_eq!(reader.read_gamma()?, 1);
assert_eq!(reader.read_delta()?, 2);
# Ok(())
# }
```

Please read the documentation of the [`traits`] module and the [`impls`] module
for more details.

## Options

There are a few options to modify the behavior of the bit read/write traits:

- Endianness can be selected using the [`BE`] or [`LE`] types as the first
  parameter. The native endianness is usually the best choice, albeit sometimes
  the lack of some low-level instructions (first bit set, last bit etc, etc.)
  may make the non-native endianness more efficient.
- Data is read from or written to the backend one word at a time, and the size
  of the word can be selected using the second parameter, but it must match the
  word size of the backend, so it is usually inferred. Currently, we suggest
  `usize` for writing and a type that is half of `usize` for reading.

More in-depth (and much more complicated) tuning can be obtained by modifying
the default values for the parameters of instantaneous codes. Methods reading or
writing instantaneous codes are defined in supporting traits and usually have
const type parameters, in particular, whether to use decoding tables or not
(e.g., [`GammaReadParam::read_gamma_param`]). Such traits are implemented for
[`BitRead`]/[`BitWrite`]. The only exception is unary code, which is implemented
by [`BitRead::read_unary`] and [`BitWrite::write_unary`].

However, there are traits with non-parametric methods (e.g.,
[`GammaRead::read_gamma`]) that are the standard entry points for the user.
These traits are implemented for [`BufBitReader`]/[`BufBitWriter`] depending on
a selector type implementing [`ReadParams`]/[`WriteParams`], respectively.
The default value for the parameter is
[`DefaultReadParams`]/[`DefaultWriteParams`], which uses choices we tested on
several platforms and that we believe are good defaults, but by passing a
different implementation of [`ReadParams`]/[`WriteParams`] you can change the
default behavior. See [`params`] for more details.

Finally, if you choose to use tables, the size of the tables is hardwired in the
source code (in particular, in the files `*_tables.rs` in the `codes` source
directory) and can be changed only by regenerating the tables using the script
`gen_code_tables.py` in the `python` directory. You will need to modify the
values hardwired at the end of the script.

## Benchmarks

To evaluate the performance on your hardware you can run the
benchmarks in the `benchmarks` directory, which test the speed of read/write
operations under several combinations of parameters. Please refer to the crate
documentation therein. The `svg` directory contains reference results of these
benchmarks of a few architectures.

## Testing

Besides unit tests, we provide zipped precomputed corpora generated by fuzzing.
You can run the tests on the zipped precomputed corpora by enabling the `fuzz`
feature:

```shell
cargo test --features fuzz
```

When the feature is enabled, tests will be also run on local corpora found in
the top-level `fuzz` directory, if any are present.

## Acknowledgments

This software has been partially supported by project SERICS (PE00000014) under
the NRRP MUR program funded by the EU - NGEU, and by project ANR COREGRAPHIE,
grant ANR-20-CE23-0002 of the French Agence Nationale de la Recherche. Views and
opinions expressed are however those of the authors only and do not necessarily
reflect those of the European Union or the Italian MUR. Neither the European
Union nor the Italian MUR can be held responsible for them.

[`BitRead`]: <https://docs.rs/dsi-bitstream/latest/dsi_bitstream/traits/trait.BitRead.html>
[`BitWrite`]: <https://docs.rs/dsi-bitstream/latest/dsi_bitstream/traits/trait.BitWrite.html>
[`BufBitReader`]: <https://docs.rs/dsi-bitstream/latest/dsi_bitstream/impls/struct.BufBitReader.html>
[`BufBitWriter`]: <https://docs.rs/dsi-bitstream/latest/dsi_bitstream/impls/struct.BufBitWriter.html>
[`ReadParams`]: <https://docs.rs/dsi-bitstream/latest/dsi_bitstream/codes/params/trait.ReadParams.html>
[`WriteParams`]: <https://docs.rs/dsi-bitstream/latest/dsi_bitstream/codes/params/trait.WriteParams.html>
[`GammaReadParam::read_gamma_param`]: <https://docs.rs/dsi-bitstream/latest/dsi_bitstream/codes/gamma/trait.GammaReadParam.html#tymethod.read_gamma_param>
[`WordAdapter`]: <https://docs.rs/dsi-bitstream/latest/dsi_bitstream/impls/struct.WordAdapter.html>
[`traits`]: <https://docs.rs/dsi-bitstream/latest/dsi_bitstream/traits/index.html>
[`impls`]: <https://docs.rs/dsi-bitstream/latest/dsi_bitstream/impls/index.html>
[`params`]: <https://docs.rs/dsi-bitstream/latest/dsi_bitstream/codes/params/index.html>
[`GammaRead::read_gamma`]: <https://docs.rs/dsi-bitstream/latest/dsi_bitstream/codes/gamma/trait.GammaRead.html#tymethod.read_gamma>
[`BE`]: <https://docs.rs/dsi-bitstream/latest/dsi_bitstream/traits/type.BE.html>
[`LE`]: <https://docs.rs/dsi-bitstream/latest/dsi_bitstream/traits/type.LE.html>
[`DefaultReadParams`]: <https://docs.rs/dsi-bitstream/latest/dsi_bitstream/codes/params/struct.DefaultReadParams.html>
[`DefaultWriteParams`]: <https://docs.rs/dsi-bitstream/latest/dsi_bitstream/codes/params/struct.DefaultWriteParams.html>
[`BitRead::read_unary`]: <https://docs.rs/dsi-bitstream/latest/dsi_bitstream/traits/trait.BitRead.html#method.read_unary>
[`BitWrite::write_unary`]: <https://docs.rs/dsi-bitstream/latest/dsi_bitstream/traits/trait.BitWrite.html#method.write_unary>
[DSI Utilities]: <https://dsiutils.di.unimi.it/>
[exponential Golomb codes]: <https://docs.rs/dsi-bitstream/latest/dsi_bitstream/codes/exp_golomb/index.html>
[H.264 (MPEG-4)]: <https://en.wikipedia.org/wiki/Advanced_Video_Coding>
[H.265]: <https://en.wikipedia.org/wiki/High_Efficiency_Video_Coding>

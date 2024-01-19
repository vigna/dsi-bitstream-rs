/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

/*!

Implementations of bit and word (seekable) streams.

Implementations of bit streams read from word streams, that is,
implementations of [`WordRead`](crate::traits::WordRead) and
[`WordWrite`](crate::traits::WordWrite). If you have a standard
[`Read`](std::io::Read) or [`Write`](std::io::Write) byte stream
you can wrap it into a [`WordAdapter`] to turn it into a word stream.

In instead you want to read or write words directly from memory, you can use
[`MemWordReader`] and [`MemWordWriterVec`]/[`MemWordWriterSlice`],
which read from a slice and write to a vector/slice.

In all cases, you must specify a word type, which is the type of the words
you want to read or write. In the case of [`WordAdapter`], the word type
is arbitrary; in the case of [`MemWordReader`] and
[`MemWordWriterVec`]/[`MemWordWriterSlice`],
it must match the type of the elements of the slice or vector,
and will be usually filled in by type inference.

Once you have a way to read or write by words, you can use [`BufBitReader`] and
[`BufBitWriter`] to read or write bits. Both have a statically
selectable endianness and use an internal bit buffer to store bits that are not
yet read or written. In the case of [`BufBitReader`], the bit buffer is
twice large as the word type, so we suggest to use a type that is half of `usize` as word type,
whereas in the case of [`BufBitWriter`] the bit buffer is as large as the word,
so we suggest to use `usize` as word type.

[`BitReader`] reads memory directly, without using a bit buffer, but it is
usually significantly slower than [`BufBitReader`].

If you want to optimize these choices for your architecture, we suggest to
run the benchmarks in the `benchmarks` directory.

## Examples

### Reading from a file

```rust
use dsi_bitstream::prelude::*;
use std::io::BufReader;

let file = std::fs::File::open("README.md").unwrap();
// Adapt to word type u32, select little endianness
let mut reader = BufBitReader::<LE, _>::new(WordAdapter::<u32, _>::new(BufReader::new(file)));
reader.read_gamma().unwrap();
```

### Writing to and reading from a vector

```rust
use dsi_bitstream::prelude::*;

let mut v: Vec<u64> = vec![];
// Automatically chooses word type u64, select big endianness
let mut writer = BufBitWriter::<BE, _>::new(MemWordWriterVec::new(&mut v));
writer.write_gamma(42).unwrap();
writer.flush().unwrap();
drop(writer); // We must drop the writer release the borrow on v

let mut reader = BufBitReader::<BE, _>::new(MemWordReader::new(&v));
assert_eq!(reader.read_gamma().unwrap(), 42);
```

*/

mod mem_word_reader;
pub use mem_word_reader::*;

mod mem_word_writer;
pub use mem_word_writer::*;

#[cfg(feature = "std")]
mod word_adapter;
#[cfg(feature = "std")]
pub use word_adapter::*;

mod bit_reader;
pub use bit_reader::BitReader;

mod buf_bit_reader;
pub use buf_bit_reader::BufBitReader;

mod buf_bit_writer;
pub use buf_bit_writer::{BufBitWriter, DropHelper};
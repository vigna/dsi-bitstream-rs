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
[`WordWrite`](crate::traits::WordWrite).

If you have a standard, [`Read`](std::io::Read) or [`Write`](std::io::Write)
stream you can wrap it into a [`WordAdapter`] to turn it into a word stream.

In instead you want to read or write words directly from memory, you can use
[`MemWordReader`] and [`MemWordWriter`], which read from a slice and write
to a vector.

In all cases, you must specify a word type, which is the type of the words
you want to read or write. In the case of [`WordAdapter`], the word type
is arbitrary; in the case of [`MemWordReader`] and [`MemWordWriter`] it must
match the type of the elements of the slice or vector, and will be usually filled in
by type inference.

Once you have a way to read or write by words, you can use [`BufBitReader`] and
[`BufBitWriter`] to read or write bits. Both have a statically
selectable endianness and use an internal bit buffer to store bits that are not
yet read or written.

[`BitReader`] reads memory directly, without using a bit buffer, but it is
usually significantly slower than [`BufBitReader`]. It is however easy to benchmark
your application with both wrappers, and choose the fastest one.

*/

mod mem_word_reader;
pub use mem_word_reader::*;

mod mem_word_writer;
pub use mem_word_writer::*;

#[cfg(feature = "std")]
mod word_adapter;
#[cfg(feature = "std")]
pub use word_adapter::*;

pub mod bit_reader;
pub use bit_reader::BitReader;

pub mod buf_bit_reader;
pub use buf_bit_reader::BufBitReader;

pub mod buf_bit_writer;
pub use buf_bit_writer::BufBitWriter;

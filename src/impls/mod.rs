/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

/*!

Implementations of bit and word (seekable) streams.

If you need to read or write words from a file or any backend implementing
[`std::io::Read`] or [`std::io::Write`] you just need to wrap it in a
[`WordAdapter`].

In instead you want to read or write directly from memory, you can use
[`MemWordReader`] and [`MemWordWriter`].

In all cases, you must specify a word type, which is the type of the words
you want to read or write. In the case of [`WordAdapter`], the word type
is arbitrary; in the case of [`MemWordReader`] and [`MemWordWriter`] it must
match the type of the elements of the slice.

Once you have a way to access words, you need can use [`BufBitReader`] and
[`BufBitWriter`] to read or write bits from a word stream. Both have a statically
selectable endianness and use an internal bit buffer to store bits that are not 
yet read or written: in the case of [`BufBitReader`] the type of the bit buffer
is choosable, but it must at least twice as large as the word type; in the case
of [`BufBitWriter`] the type of the bit buffer is fixed to `u32`.

[`BitReader`] and reads memory directly, without using a bit buffer, but it is
usually significantly slower than [`BufBitReader`]. It is however easy to benchmark
your application with both wrappers, and choose the fastest one.

*/

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
pub use buf_bit_writer::BufBitWriter;

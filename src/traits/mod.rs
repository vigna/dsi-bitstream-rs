/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

/*!

Traits for operating on streams of bits.

We provide three bit-based traits, [`BitRead`], [`BitWrite`], and
[`BitSeek`], analogous to [`std::io::Read`], [`std::io::Write`],
and [`std::io::Seek`], respectively. They provide read/write operations
on fixed-width blocks of bits and unary codes. More complex operations,
such as [reading instantaneous codes](crate::codes::GammaReadParam),
are built on these basic traits.

The [`Endianness`] trait is used to specify the endianness
of a bit stream by using its only two implementations, [`BigEndian`] (AKA [`LE`]) and
[`LittleEndian`] (AKA [`BE`]).

The implementations we provide for these traits (e.g.,
[`BufBitReader`](crate::impls::BufBitReader)) are based on
[`WordRead`], [`WordWrite`], and [`WordSeek`], which provide word-based operations,
as reading or writing multiple bytes at a time is usually much faster than
reading or writing single bytes, in particular when interacting with memory.
For example, [`MemWordRead`](crate::impls::MemWordReader) is a [`WordRead`]
that reads word-by-word from a slice.

All traits have an internal error type `Error`, which usually propagates the
error of the underlying backend. However, in some cases (e.g., [`MemWordRead`](crate::impls::MemWordReader)
with infinite zero extension) the error type is [`Infallible`](core::convert::Infallible),
in which case the compiler is able to perform several further optimizations.

## Bit and byte order

The endianness parameter specifies at the same byte the endianness of the byte
stream and of the bits in each byte: in the little-endian case, the first bit
of the stream is the least significant bit of the first byte, while in the
big-endian case it is the most significant bit of the first byte. Albeit in principle
one can mix independently the two orders, having the same order for both bytes
and bits is usually more convenient and makes for more efficient implementations.

Byte-level endianness is used to read memory word-by-word, greatly reducing the number
of memory accesses when reading from slices. However, it is important to note that
values have thair least significant bit always stored at the lowest bit position,
independently of endianness, as current CPUs always use big-endian bit order.
In particular, reversing the order of the bits of each byte of a file will not
in general give a file with complementary endianness.

For example, if we write just the value 6 to a big-endian bit stream, we will
get as first byte `110xxxxx`, while if we write it to a little-endian bit stream
we will obtain the byte `xxxxx110`. Clearly, reversing the order of the bits
of each byte will not give the other byte. (There are however a few exceptions
for which reversing works, such as palindromic bit writes and unary codes.)

*/

mod bit_stream;
pub use bit_stream::*;

mod word_stream;
pub use word_stream::*;

mod endianness;
pub use endianness::*;

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

*/

mod bit_stream;
pub use bit_stream::*;

mod word_stream;
pub use word_stream::*;

mod endianness;
pub use endianness::*;

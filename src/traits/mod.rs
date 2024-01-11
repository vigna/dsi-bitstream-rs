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

The implementations we provide for these traits (e.g.,
[`BufBitReader`](crate::impls::BufBitReader)) are based on
[`WordRead`], [`WordWrite`], and [`WordSeek`], which provide word-based operations,
as reading or writing multiple bytes at a time is usually much faster than
reading or writing single bytes, in particular when interacting with memory.

The [`Endianness`] trait is used by implementations to specify the endianness
of a bit stream by using its only two implementations, [`BigEndian`] (AKA [`LE`]) and
[`LittleEndian`] (AKA [`BE`]).

*/

mod bit_stream;
pub use bit_stream::*;

mod word_stream;
pub use word_stream::*;

mod endianness;
pub use endianness::*;

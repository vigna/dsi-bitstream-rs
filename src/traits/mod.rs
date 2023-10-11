/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

/*!

Traits to describe bit operations.

- The [`Endianness`] trait is used by implementations to specify the endianness of a bit stream.
- The traits [`BitRead`], [`BitWrite`], and [`BitSeek`] provide bit-based traits analogous to
  [`std::io::Read`], [`std::io::Write`], and [`std::io::Seek`].
- The traits [`WordRead`], [`WordWrite`], and [`WordSeek`] provide word-based traits analogous to
  [`std::io::Read`], [`std::io::Write`], and [`std::io::Seek`]. They provide the backend for
  the bit-based buffered implementations [`crate::impls::BufBitReader`] and [`crate::impls::BufBitWriter`].

*/

mod bit_stream;
pub use bit_stream::*;

mod word_stream;
pub use word_stream::*;

mod endianness;
pub use endianness::*;

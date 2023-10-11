/*
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

/*!

Debug helpers.

[`CountBitReader`] and [`CountBitWriter`] keep track of the number
of bits read or written to a [`BitRead`](crate::traits::BitRead)
and [`BitWrite`](crate::traits::BitWrite), respectively,
optionally printing on standard error the operations performed on the stream.

 [`DbgCodeReader`] and [`DbgCodeWriter`] print on standard error all code-based
operation beformed by a [`CodeRead`](crate::codes::CodeRead)
or [`CodeWrite`](crate::codes::CodeWrite).

*/

pub mod count;
pub use count::*;

pub mod dbg_codes;
pub use dbg_codes::*;

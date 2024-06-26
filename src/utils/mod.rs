/*
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

/*!

Debug helpers and statistics.

[`CountBitReader`] and [`CountBitWriter`] keep track of the number
of bits read or written to a [`BitRead`](crate::traits::BitRead)
and [`BitWrite`](crate::traits::BitWrite), respectively,
optionally printing on standard error the operations performed on the stream.

[`DbgBitReader`] and [`DbgBitWriter`] print on standard error all
operation beformed by a [`BitRead`](crate::traits::BitRead) or
[`BitWrite`](crate::traits::BitWrite).

[`CodesStats`] keeps track of the space needed to store a stream of
integers using different codes.

*/

mod count;
pub use count::*;

mod dbg_codes;
pub use dbg_codes::*;

pub mod stats;
pub use stats::CodesStats;

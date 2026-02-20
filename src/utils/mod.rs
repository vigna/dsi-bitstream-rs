/*
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 * SPDX-FileCopyrightText: 2025 Tommaso Fontana
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Helpers and statistics.
//!
//! [`CountBitReader`] and [`CountBitWriter`] keep track of the number
//! of bits read or written to a [`BitRead`](crate::traits::BitRead)
//! and [`BitWrite`](crate::traits::BitWrite), respectively,
//! optionally printing on standard error the operations performed on the stream.
//!
//! [`DbgBitReader`] and [`DbgBitWriter`] print on standard error all
//! operations performed by a [`BitRead`](crate::traits::BitRead) or
//! [`BitWrite`](crate::traits::BitWrite).
//!
//! [`CodesStats`] keeps track of the space needed to store a stream of
//! integers using different codes.
//!
//! With the `implied` feature, it also provides
//! `sample_implied_distribution`, an infinite iterator that returns samples
//! from the implied distribution of a code, and the helper function
//! `get_implied_distribution`.
//!
//! [`FindChangePoints`] finds, using exponential search, the points where a
//! non-decreasing monotonic function changes value.

mod count;
pub use count::*;

mod dbg_codes;
pub use dbg_codes::*;

mod find_change;
pub use find_change::*;

#[cfg(feature = "implied")]
mod implied;
#[cfg(feature = "implied")]
pub use implied::*;

pub mod stats;
pub use stats::CodesStats;
#[cfg(feature = "std")]
pub use stats::CodesStatsWrapper;

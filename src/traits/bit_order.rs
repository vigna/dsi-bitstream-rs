/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Marker types and trait used to conditionally implement MSB to LSB or LSB to
//! MSB bit orders in readers and writers.
//!
//! Note that we use an inner private trait `EndiannessCore` so that an user can
//! use [`Endianness`] for its generics, but cannot implement it, so all the
//! types that will ever implement [`Endianness`] are defined in this file.
//!
//! Apparently this pattern is a [SealedTrait](https://predr.ag/blog/definitive-guide-to-sealed-traits-in-rust/).

/// Inner private trait used to remove the possibility that anyone could
/// implement [`Endianness`] on other structs
mod private {
    pub trait Endianness {}
}
impl<T: private::Endianness> Endianness for T {}

/// Marker trait to require that something is either [`LE`] or
/// [`BE`]
pub trait Endianness: private::Endianness {}

/// Marker type that represents LSB to MSB bit order
pub struct LittleEndian;
/// Marker type that represents MSB to LSB bit order
pub struct BigEndian;

/// Alias for [`BigEndian`]
pub type BE = BigEndian;

/// Alias for [`LittleEndian`]
pub type LE = LittleEndian;

impl private::Endianness for LittleEndian {}
impl private::Endianness for BigEndian {}

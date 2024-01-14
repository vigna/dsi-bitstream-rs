/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

/// Inner private trait used to make implementing [`Endianness`]
/// impossible for other structs.
mod private {
    /// This is a [SealedTrait](https://predr.ag/blog/definitive-guide-to-sealed-traits-in-rust/).
    pub trait Endianness {}
}

impl<T: private::Endianness> Endianness for T {}

/// Marker trait for endianness selector types.
///
/// Its only implementations are [`LittleEndian`] and [`BigEndian`]
///
/// Note that in principle marker traits are not necessary to use
/// selector types, but they are useful to avoid that the user specifies
/// a nonsensical type, and to document the meaning of type parameters.
pub trait Endianness: private::Endianness {}

/// Selector type for little-endian streams.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LittleEndian;

/// Selector type for big-endian streams.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BigEndian;

/// Alias for [`BigEndian`]
pub type BE = BigEndian;

/// Alias for [`LittleEndian`]
pub type LE = LittleEndian;

impl private::Endianness for LittleEndian {}
impl private::Endianness for BigEndian {}

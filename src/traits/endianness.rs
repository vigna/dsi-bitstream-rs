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
    pub trait Endianness: 'static + Send + Sync + Copy {
        /// The name of the endianness.
        const _NAME: &'static str;
        /// Whether the endianness is little-endian.
        const _IS_LITTLE: bool;
        /// Whether the endianness is big-endian.
        const _IS_BIG: bool;
    }
}

impl<T: private::Endianness> Endianness for T {
    const NAME: &'static str = T::_NAME;
    const IS_LITTLE: bool = T::_IS_LITTLE;
    const IS_BIG: bool = T::_IS_BIG;
}

/// Marker trait for endianness selector types.
///
/// Its only implementations are [`LittleEndian`] and [`BigEndian`]
///
/// Note that in principle marker traits are not necessary to use
/// selector types, but they are useful to avoid that the user specifies
/// a nonsensical type, and to document the meaning of type parameters.
pub trait Endianness: private::Endianness {
    /// The name of the endianness.
    const NAME: &'static str;
    /// Whether the endianness is little-endian.
    const IS_LITTLE: bool;
    /// Whether the endianness is big-endian.
    const IS_BIG: bool;
}

impl core::fmt::Display for LE {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(LE::NAME)
    }
}

impl core::fmt::Display for BE {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(BE::NAME)
    }
}

/// Selector type for little-endian streams.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LittleEndian;

/// Selector type for big-endian streams.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BigEndian;

impl private::Endianness for LittleEndian {
    const _NAME: &'static str = "little";
    const _IS_LITTLE: bool = true;
    const _IS_BIG: bool = false;
}

impl private::Endianness for BigEndian {
    const _NAME: &'static str = "big";
    const _IS_LITTLE: bool = false;
    const _IS_BIG: bool = true;
}

/// Alias for [`BigEndian`]
pub type BE = BigEndian;

/// Alias for [`LittleEndian`]
pub type LE = LittleEndian;

#[cfg(target_endian = "little")]
/// A type alias for the native endianness of the target platform.
pub type NativeEndian = LittleEndian;
#[cfg(target_endian = "big")]
/// A type alias for the native endianness of the target platform.
pub type NativeEndian = BigEndian;

/// An Alias for [`NativeEndian`]
pub type NE = NativeEndian;

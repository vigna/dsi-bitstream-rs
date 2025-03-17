/*
 * SPDX-FileCopyrightText: 2025 Tommaso Fontana
 * SPDX-FileCopyrightText: 2025 Inria
 * SPDX-FileCopyrightText: 2025 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

 //! Programmable static and dynamic dispatch for codes.
//!
//! The code traits in the submodules of this module, such as
//! [`Omega`](super::omega), extend [`BitRead`] and [`BitWrite`] to provide a
//! way to read and write codes from a bitstream. The user can thus select at
//! compile time the desired trait and use the associated codes.
//!
//! In many contexts, however, one does not want to commit to a specific set of
//! codes, but rather would like to write generic methods that accept some code
//! as an input and then use it to read or write values. For example, a stream
//! encoder might let the user choose between different codes, depending on the
//! user's knowledge of the distribution of the values to be encoded.
//!
//! Having dynamic selection of a code, however, entails a performance cost, as,
//! for example, a match statement must be used to select the correct code. To
//! mitigate this cost, we provide two types of dispatch traits and three types
//! of implementations based on them.
//!
//! # Dispatch Traits
//!
//! The traits [`DynamicCodeRead`] and [`DynamicCodeWrite`] are the most generic
//! ones, and provide a method to read and write a code from a bitstream. By
//! implementing them, you can write a method accepting one or more unspecified
//! codes, and operate with them. For example, in this function we read twice a
//! code and return the sum of the two values, but make no committment on which
//! code we will be using:
//!```rust
//! use dsi_bitstream::prelude::*;
//! use dsi_bitstream::codes::dispatch::{CodesRead, DynamicCodeRead};
//! use std::fmt::Debug;
//!
//! fn read_two_codes_and_sum<
//!     E: Endianness,
//!     R: CodesRead<E> + ?Sized,
//!     GR: DynamicCodeRead
//! >(
//!     reader: &mut R,
//!     code: GR,
//! ) -> Result<u64, R::Error> {
//!     Ok(code.read(reader)? + code.read(reader)?)
//! }
//!```
//! On the other hand, the traits [`StaticCodeRead`] and [`StaticCodeWrite`]
//! are specialized for a reader or writer of given endianness. This means that
//! they can in principle be implemented for a specific code by storing a
//! function pointer, with much less runtime overhead.
//!```rust
//! use dsi_bitstream::prelude::*;
//! use dsi_bitstream::codes::dispatch::{CodesRead, StaticCodeRead};
//! use std::fmt::Debug;
//!
//! fn read_two_codes_and_sum<
//!     E: Endianness,
//!     R: CodesRead<E> + ?Sized,
//!     SR: StaticCodeRead<E, R>
//! >(
//!     reader: &mut R,
//!     code: SR,
//! ) -> Result<u64, R::Error> {
//!     Ok(code.read(reader)? + code.read(reader)?)
//! }
//!```
//!
//! Note that the syntax for invoking the methods in the two groups of traits
//! is identical, but the type variables are on the method in the first
//! case, and on the trait in the second case.
//!
//! # Implementations
//!
//! The [`Codes`] enum is an enum whose variants represent all the available
//! codes. It implements all the dispatch traits, so it can be used to read or
//! write any code both in a generic and in a specific way. It also implements
//! the [`CodeLen`] trait, which provides a method to compute the length of a
//! codeword.
//!
//! If Rust would support const enums in traits, one could create structures
//! with const enum type parameters of type [`Codes`], and then the compiler
//! would be able to optimize away the code selection at compile time. However,
//! this is not currently possible, so we provide a workaround using a
//! zero-sized struct with a `const usize` parameter, [`ConstCode`], that
//! implements all the dispatch traits and [`CodeLen`], and can be used to
//! select the code at compile time. The parameter must be taken from the
//! [`code_consts`] module, which contains constants for all parameterless
//! codes, and for the codes with parameters up to 10. For example, here at
//! execution time there will be no test to select a code, even if
//! `read_two_codes_and_sum` is generic:
//!```rust
//! use dsi_bitstream::prelude::*;
//! use dsi_bitstream::codes::dispatch::{code_consts, CodesRead, DynamicCodeRead};
//! use std::fmt::Debug;
//!
//! fn read_two_codes_and_sum<
//!     E: Endianness,
//!     R: CodesRead<E> + ?Sized,
//!     GR: DynamicCodeRead
//! >(
//!     reader: &mut R,
//!     code: GR,
//! ) -> Result<u64, R::Error> {
//!     Ok(code.read(reader)? + code.read(reader)?)
//! }
//!
//! fn call_read_two_codes_and_sum<E: Endianness, R: CodesRead<E> + ?Sized>(
//!     reader: &mut R,
//! ) -> Result<u64, R::Error> {
//!     read_two_codes_and_sum(reader, ConstCode::<{code_consts::GAMMA}>)
//! }
//!```
//!
//! Working with [`ConstCode`] is very efficient, but it forces the choice of a
//! code at compile time. If you need to read or write a code multiple times on
//! the same type of bitstream, you can use the structs [`FuncCodesReader`] and
//! [`FuncCodeWriter`], which implement [`StaticCodeRead`] and
//! [`StaticCodeWrite`] by storing a function pointer.
//!
//! A value of type [`FuncCodesReader`] or [`FuncCodeWriter`] can be created by calling
//! their `new` method with a variant of the [`Codes`] enum. As in the case of
//! [`ConstCode`], there are pointers for all parameterless codes, and for the
//! codes with parameters up to 10, and the method will return an error if the
//! code is not supported.
//!
//! For example:
//!```rust
//! use dsi_bitstream::prelude::*;
//! use dsi_bitstream::codes::dispatch::{CodesRead, StaticCodeRead, FuncCodesReader};
//! use std::fmt::Debug;
//!
//! fn read_two_codes_and_sum<
//!     E: Endianness,
//!     R: CodesRead<E> + ?Sized,
//!     SR: StaticCodeRead<E, R>
//! >(
//!     reader: &mut R,
//!     code: SR,
//! ) -> Result<u64, R::Error> {
//!     Ok(code.read(reader)? + code.read(reader)?)
//! }
//!
//! fn call_read_two_codes_and_sum<E: Endianness, R: CodesRead<E> + ?Sized>(
//!     reader: &mut R,
//! ) -> Result<u64, R::Error> {
//!     read_two_codes_and_sum(reader, FuncCodesReader::new(Codes::Gamma).unwrap())
//! }
//!```
//! Note that we [`unwrap`](core::result::Result::unwrap) the result of the
//! [`new`](FuncCodesReader::new) method, as we know that a function pointer exists
//! for the γ code.
//!
//! # Workaround to Limitations
//!
//! Both [`ConstCode`] and [`FuncCodesReader`] / [`FuncCodeWriter`] are limited to a
//! fixed set of codes. If you need to work with a code that is not supported by
//! them, you can implement your own version. For example, here we define a
//! zero-sized struct that represent a Rice code with a fixed parameter
//! `LOG2_B`:
//! ```rust
//! use dsi_bitstream::prelude::*;
//! use dsi_bitstream::codes::dispatch::{CodesRead, CodesWrite};
//! use dsi_bitstream::codes::dispatch::{DynamicCodeRead, DynamicCodeWrite};
//! use std::fmt::Debug;
//!
//! #[derive(Clone, Copy, Debug, Default)]
//! pub struct Rice<const LOG2_B: usize>;
//!
//! impl<const LOG2_B: usize> DynamicCodeRead for Rice<LOG2_B> {
//!     fn read<E: Endianness, CR: CodesRead<E> + ?Sized>(
//!         &self,
//!         reader: &mut CR,
//!     ) -> Result<u64, CR::Error> {
//!         reader.read_rice(LOG2_B)
//!     }
//! }
//!
//! impl<const LOG2_B: usize> DynamicCodeWrite for Rice<LOG2_B> {
//!     fn write<E: Endianness, CW: CodesWrite<E> + ?Sized>(
//!         &self,
//!         writer: &mut CW,
//!         value: u64,
//!     ) -> Result<usize, CW::Error> {
//!         writer.write_rice(value, LOG2_B)
//!     }
//! }
//!
//! impl<const LOG2_B: usize> CodeLen for Rice<LOG2_B> {
//!     #[inline]
//!     fn len(&self, value: u64) -> usize {
//!         len_rice(value, LOG2_B)
//!     }
//! }
//! ```
//!
//! Suppose instead you need to pass a [`StaticCodeRead`] to a method using a
//! code that is not supported directly by [`FuncCodesReader`]. You can create a new
//! [`FuncCodesReader`] using a provided function:
//!```rust
//! use dsi_bitstream::prelude::*;
//! use dsi_bitstream::codes::dispatch::{CodesRead, StaticCodeRead, FuncCodesReader};
//! use std::fmt::Debug;
//!
//! fn read_two_codes_and_sum<
//!     E: Endianness,
//!     R: CodesRead<E> + ?Sized,
//!     SR: StaticCodeRead<E, R>
//! >(
//!     reader: &mut R,
//!     code: SR,
//! ) -> Result<u64, R::Error> {
//!     Ok(code.read(reader)? + code.read(reader)?)
//! }
//!
//! fn call_read_two_codes_and_sum<E: Endianness, R: CodesRead<E> + ?Sized>(
//!     reader: &mut R,
//! ) -> Result<u64, R::Error> {
//!     read_two_codes_and_sum(reader, FuncCodesReader::new_with_func(|r: &mut R| r.read_rice(20)))
//! }
//!```

use crate::prelude::Endianness;
use crate::prelude::{BitRead, BitWrite};

use crate::codes::*;
use anyhow::Result;

pub mod codes;
pub use codes::*;

pub mod constant;
pub use constant::{code_consts, ConstCode};

pub mod func;
pub use func::{FuncCodeLen, FuncCodeWriter, FuncCodesReader};

pub mod factory;
pub use factory::{CodesReaderFactory, FuncCodesReaderFactory};

/// Convenience extension trait for reading all the codes supported by the
/// library.
///
/// A blanket implementation is provided for all types that implement the
/// necessary traits.
///
/// This trait is mainly useful internally to implement the dispatch
/// traits [`DynamicCodeRead`], [`StaticCodeRead`], [`DynamicCodeWrite`], and
/// [`StaticCodeWrite`]. The user might find more useful to define its own
/// convenience trait that includes only the codes they need.
pub trait CodesRead<E: Endianness>:
    BitRead<E>
    + GammaRead<E>
    + GammaReadParam<E>
    + DeltaRead<E>
    + DeltaReadParam<E>
    + ZetaRead<E>
    + ZetaReadParam<E>
    + OmegaRead<E>
    + MinimalBinaryRead<E>
    + PiRead<E>
    + GolombRead<E>
    + RiceRead<E>
    + ExpGolombRead<E>
    + VByteBeRead<E>
    + VByteLeRead<E>
{
}

impl<E: Endianness, B> CodesRead<E> for B where
    B: BitRead<E>
        + GammaRead<E>
        + GammaReadParam<E>
        + DeltaRead<E>
        + DeltaReadParam<E>
        + ZetaRead<E>
        + ZetaReadParam<E>
        + OmegaRead<E>
        + MinimalBinaryRead<E>
        + PiRead<E>
        + GolombRead<E>
        + RiceRead<E>
        + ExpGolombRead<E>
        + VByteBeRead<E>
        + VByteLeRead<E>
{
}

/// Convenience extension trait for writing all the codes supported by the
/// library.
///
/// A blanket implementation is provided for all types that implement the
/// necessary traits.
///
/// This trait is mainly useful internally to implement the dispatch
/// traits [`DynamicCodeRead`], [`StaticCodeRead`], [`DynamicCodeWrite`], and
/// [`StaticCodeWrite`]. The user might find more useful to define its own
/// convenience trait that includes only the codes they need.
pub trait CodesWrite<E: Endianness>:
    BitWrite<E>
    + GammaWrite<E>
    + DeltaWrite<E>
    + ZetaWrite<E>
    + OmegaWrite<E>
    + MinimalBinaryWrite<E>
    + PiWrite<E>
    + GolombWrite<E>
    + RiceWrite<E>
    + ExpGolombWrite<E>
    + VByteBeWrite<E>
    + VByteLeWrite<E>
{
}

impl<E: Endianness, B> CodesWrite<E> for B where
    B: BitWrite<E>
        + GammaWrite<E>
        + DeltaWrite<E>
        + ZetaWrite<E>
        + OmegaWrite<E>
        + MinimalBinaryWrite<E>
        + PiWrite<E>
        + GolombWrite<E>
        + RiceWrite<E>
        + ExpGolombWrite<E>
        + VByteBeWrite<E>
        + VByteLeWrite<E>
{
}

/// A trait providing a method to read a code from a generic [`CodesRead`].
///
/// The difference with [`StaticCodeRead`] is that this trait is more generic,
/// as the [`CodesRead`] is a parameter of the method, and not of the trait.
pub trait DynamicCodeRead {
    fn read<E: Endianness, CR: CodesRead<E> + ?Sized>(
        &self,
        reader: &mut CR,
    ) -> Result<u64, CR::Error>;
}

/// A trait providing a method to write a code to a generic [`CodesWrite`].
///
/// The difference with [`StaticCodeWrite`] is that this trait is more generic,
/// as the [`CodesWrite`] is a parameter of the method, and not of the trait.
pub trait DynamicCodeWrite {
    fn write<E: Endianness, CW: CodesWrite<E> + ?Sized>(
        &self,
        writer: &mut CW,
        value: u64,
    ) -> Result<usize, CW::Error>;
}

/// A trait providing a method to read a code from a [`CodesRead`] specified as
/// trait type parameter.
///
/// The difference with [`DynamicCodeRead`] is that this trait is more specialized,
/// as the [`CodesRead`] is a parameter of the trait.
///
/// For a fixed code this trait may be implemented by storing
/// a function pointer.
pub trait StaticCodeRead<E: Endianness, CR: CodesRead<E> + ?Sized> {
    fn read(&self, reader: &mut CR) -> Result<u64, CR::Error>;
}

/// A trait providing a method to write a code to a [`CodesWrite`] specified as
/// a trait type parameter.
///
/// The difference with [`DynamicCodeWrite`] is that this trait is more specialized,
/// as the [`CodesWrite`] is a parameter of the trait.
///
/// For a fixed code this trait may be implemented by storing a function
/// pointer.
pub trait StaticCodeWrite<E: Endianness, CW: CodesWrite<E> + ?Sized> {
    fn write(&self, writer: &mut CW, value: u64) -> Result<usize, CW::Error>;
}

/// A trait providing a generic method to compute the length of a codeword.
pub trait CodeLen {
    /// Return the length of the codeword for `value`.
    fn len(&self, value: u64) -> usize;
}

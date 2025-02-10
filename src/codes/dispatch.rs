/*
 * SPDX-FileCopyrightText: 2024 Tommaso Fontana
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
//! The traits [`GenericCodeRead`] and [`GenericCodeWrite`] are the most generic
//! ones, and provide a method to read and write a code from a bitstream. By
//! implementing them, you can write a method accepting one or more unspecified
//! codes, and operate with them. For example, in this function we read twice a
//! code and return the sum of the two values, but make no committment on which
//! code we will be using:
//!```rust
//! use dsi_bitstream::prelude::*;
//! use dsi_bitstream::codes::dispatch::{CodesRead, GenericCodeRead};
//! use std::fmt::Debug;
//!
//! fn read_two_codes_and_sum<
//!     E: Endianness,
//!     R: CodesRead<E> + ?Sized,
//!     GR: GenericCodeRead
//! >(
//!     reader: &mut R,
//!     code: GR,
//! ) -> Result<u64, GR::Error<R::Error>> {
//!     Ok(code.generic_read(reader)? + code.generic_read(reader)?)
//! }
//!```
//! On the other hand, the traits [`SpecificCodeRead`] and [`SpecificCodeWrite`]
//! are specialized for a reader or writer of given endianness. This means that
//! they can in principle be implemented for a specific code by storing a
//! function pointer, with much less runtime overhead.
//!```rust
//! use dsi_bitstream::prelude::*;
//! use dsi_bitstream::codes::dispatch::{CodesRead, SpecificCodeRead};
//! use std::fmt::Debug;
//!
//! fn read_two_codes_and_sum<
//!     E: Endianness,
//!     R: CodesRead<E> + ?Sized,
//!     SR: SpecificCodeRead<E, R>
//! >(
//!     reader: &mut R,
//!     code: SR,
//! ) -> Result<u64, SR::Error<R::Error>> {
//!     Ok(code.specific_read(reader)? + code.specific_read(reader)?)
//! }
//!```
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
//! use dsi_bitstream::codes::dispatch::{code_consts, CodesRead, GenericCodeRead};
//! use std::fmt::Debug;
//!
//! fn read_two_codes_and_sum<
//!     E: Endianness,
//!     R: CodesRead<E> + ?Sized,
//!     GR: GenericCodeRead
//! >(
//!     reader: &mut R,
//!     code: GR,
//! ) -> Result<u64, GR::Error<R::Error>> {
//!     Ok(code.generic_read(reader)? + code.generic_read(reader)?)
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
//! the same type of bitstream, you can use the structs [`FuncReader`] and
//! [`FuncWriter`], which implement [`SpecificCodeRead`] and
//! [`SpecificCodeWrite`] by storing a function pointer.
//!
//! A value of type [`FuncReader`] or [`FuncWriter`] can be created by calling
//! their `new` method with a variant of the [`Codes`] enum. As in the case of
//! [`ConstCode`], there are pointers for all parameterless codes, and for the
//! codes with parameters up to 10, and the method will return an error if the
//! code is not supported.
//!
//! For example:
//!```rust
//! use dsi_bitstream::prelude::*;
//! use dsi_bitstream::codes::dispatch::{CodesRead, SpecificCodeRead, FuncReader};
//! use std::fmt::Debug;
//!
//! fn read_two_codes_and_sum<
//!     E: Endianness,
//!     R: CodesRead<E> + ?Sized,
//!     SR: SpecificCodeRead<E, R>
//! >(
//!     reader: &mut R,
//!     code: SR,
//! ) -> Result<u64, SR::Error<R::Error>> {
//!     Ok(code.specific_read(reader)? + code.specific_read(reader)?)
//! }
//!
//! fn call_read_two_codes_and_sum<E: Endianness, R: CodesRead<E> + ?Sized>(
//!     reader: &mut R,
//! ) -> Result<u64, R::Error> {
//!     read_two_codes_and_sum(reader, FuncReader::new(Codes::Gamma).unwrap())
//! }
//!```
//! Note that we [`unwrap`](core::result::Result::unwrap) the result of the
//! [`new`](FuncReader::new) method, as we know that a function pointer exists
//! for the γ code.
//!
//! # Workaround to Limitations
//!
//! Both [`ConstCode`] and [`FuncReader`] / [`FuncWriter`] are limited to a
//! fixed set of codes. If you need to work with a code that is not supported by
//! them, you can implement your own version. For example, here we define a
//! zero-sized struct that represent a Rice code with a fixed parameter
//! `LOG2_B`:
//! ```rust
//! use dsi_bitstream::prelude::*;
//! use dsi_bitstream::codes::dispatch::{CodesRead, CodesWrite};
//! use dsi_bitstream::codes::dispatch::{GenericCodeRead, GenericCodeWrite};
//! use std::fmt::Debug;
//!
//! #[derive(Clone, Copy, Debug, Default)]
//! pub struct Rice<const LOG2_B: usize>;
//!
//! impl<const LOG2_B: usize> GenericCodeRead for Rice<LOG2_B> {
//!     type Error<CRE:  Debug + Send + Sync + 'static> = CRE;
//!
//!     fn generic_read<E: Endianness, CR: CodesRead<E> + ?Sized>(
//!         &self,
//!         reader: &mut CR,
//!     ) -> Result<u64, Self::Error<CR::Error>> {
//!         reader.read_rice(LOG2_B)
//!     }
//! }
//!
//! impl<const LOG2_B: usize> GenericCodeWrite for Rice<LOG2_B> {
//!     type Error<CWE:  Debug + Send + Sync + 'static> = CWE;
//!
//!     fn generic_write<E: Endianness, CW: CodesWrite<E> + ?Sized>(
//!         &self,
//!         writer: &mut CW,
//!         value: u64,
//!     ) -> Result<usize, Self::Error<CW::Error>> {
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
//! Suppose instead you need to pass a [`SpecificCodeRead`] to a method using a
//! code that is not supported directly by [`FuncReader`]. You can create a new
//! [`FuncReader`] using a provided function:
//!```rust
//! use dsi_bitstream::prelude::*;
//! use dsi_bitstream::codes::dispatch::{CodesRead, SpecificCodeRead, FuncReader};
//! use std::fmt::Debug;
//!
//! fn read_two_codes_and_sum<
//!     E: Endianness,
//!     R: CodesRead<E> + ?Sized,
//!     SR: SpecificCodeRead<E, R>
//! >(
//!     reader: &mut R,
//!     code: SR,
//! ) -> Result<u64, SR::Error<R::Error>> {
//!     Ok(code.specific_read(reader)? + code.specific_read(reader)?)
//! }
//!
//! fn call_read_two_codes_and_sum<E: Endianness, R: CodesRead<E> + ?Sized>(
//!     reader: &mut R,
//! ) -> Result<u64, R::Error> {
//!     read_two_codes_and_sum(reader, FuncReader::new_with_func(|r: &mut R| r.read_rice(20)))
//! }
//!```

use super::*;
use anyhow::Result;
use core::fmt::Debug;
use core::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
#[non_exhaustive]
/// An enum whose variants represent all the available codes.
///
/// This enum is kept in sync with implementations in the
/// [`codes`](crate::codes) module.
///
/// Both [`Display`](std::fmt::Display) and [`FromStr`](std::str::FromStr) are
/// implemented for this enum in a dual way, which makes it possible to store a
/// code as a string in a configuration file, and then parse it back.
pub enum Codes {
    Unary,
    Gamma,
    Delta,
    Omega,
    VByte,
    Zeta { k: usize },
    Pi { k: usize },
    Golomb { b: usize },
    ExpGolomb { k: usize },
    Rice { log2_b: usize },
}

/// A trait providing a method to read a code from a generic [`CodesRead`].
///
/// The difference with [`SpecificCodeRead`] is that this trait is more generic,
/// as the [`CodesRead`] is a parameter of the method, and not of the trait.
pub trait GenericCodeRead {
    type Error<CRE: Debug + Send + Sync + 'static>: Debug + Send + Sync + 'static;

    /// Read a value
    fn generic_read<E: Endianness, CR: CodesRead<E> + ?Sized>(
        &self,
        reader: &mut CR,
    ) -> Result<u64, Self::Error<CR::Error>>;
}

/// A trait providing a method to read a code from a [`CodesRead`] specified as
/// trait type parameter.
///
/// The difference with [`GenericCodeRead`] is that this trait is more specialized,
/// as the [`CodesRead`] is a parameter of the trait.
///
/// For a fixed code this trait may be implemented by storing
/// a function pointer.
pub trait SpecificCodeRead<E: Endianness, CR: CodesRead<E> + ?Sized> {
    type Error<CRE: Debug + Send + Sync + 'static>: Debug + Send + Sync + 'static;

    fn specific_read(&self, reader: &mut CR) -> Result<u64, Self::Error<CR::Error>>;
}

/// A trait providing a method to write a code to a generic [`CodesWrite`].
///
/// The difference with [`SpecificCodeWrite`] is that this trait is more generic,
/// as the [`CodesWrite`] is a parameter of the method, and not of the trait.
pub trait GenericCodeWrite {
    type Error<CWE: Debug + Send + Sync + 'static>: Debug + Send + Sync + 'static;

    fn generic_write<E: Endianness, CW: CodesWrite<E> + ?Sized>(
        &self,
        writer: &mut CW,
        value: u64,
    ) -> Result<usize, Self::Error<CW::Error>>;
}

/// A trait providing a method to write a code to a [`CodesWrite`] specified as
/// a trait type parameter.
///
/// The difference with [`GenericCodeWrite`] is that this trait is more specialized,
/// as the [`CodesWrite`] is a parameter of the trait.
///
/// For a fixed code this trait may be implemented by storing a function
/// pointer.
pub trait SpecificCodeWrite<E: Endianness, CW: CodesWrite<E> + ?Sized> {
    type Error<CWE: Debug + Send + Sync + 'static>: Debug + Send + Sync + 'static;

    fn specific_write(&self, writer: &mut CW, value: u64) -> Result<usize, Self::Error<CW::Error>>;
}

/// A trait providing a generic method to compute the length of a codeword.
pub trait CodeLen {
    /// Return the length of the codeword for `value`.
    fn len(&self, value: u64) -> usize;
}

#[derive(Debug)]
/// Error type for parsing a code from a string.
pub enum CodeError {
    ParseError(core::num::ParseIntError),
    UnknownCode(String),
}
impl std::error::Error for CodeError {}
impl core::fmt::Display for CodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            CodeError::ParseError(e) => write!(f, "Parse error: {}", e),
            CodeError::UnknownCode(s) => write!(f, "Unknown code: {}", s),
        }
    }
}

impl From<core::num::ParseIntError> for CodeError {
    fn from(e: core::num::ParseIntError) -> Self {
        CodeError::ParseError(e)
    }
}

/// Convenience extension trait for reading all the codes supported by the
/// library.
///
/// A blanket implementation is provided for all types that implement the
/// necessary traits.
///
/// This trait is mainly useful internally to implement the dispatch
/// traits [`GenericCodeRead`], [`SpecificCodeRead`], [`GenericCodeWrite`], and
/// [`SpecificCodeWrite`]. The user might find more useful to define its own
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
    + VByteRead<E>
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
        + VByteRead<E>
{
}

/// Convenience extension trait for writing all the codes supported by the
/// library.
///
/// A blanket implementation is provided for all types that implement the
/// necessary traits.
///
/// This trait is mainly useful internally to implement the dispatch
/// traits [`GenericCodeRead`], [`SpecificCodeRead`], [`GenericCodeWrite`], and
/// [`SpecificCodeWrite`]. The user might find more useful to define its own
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
    + VByteWrite<E>
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
        + VByteWrite<E>
{
}

impl GenericCodeRead for Codes {
    type Error<CRE: Debug + Send + Sync + 'static> = CRE;

    #[inline]
    fn generic_read<E: Endianness, CR: CodesRead<E> + ?Sized>(
        &self,
        reader: &mut CR,
    ) -> Result<u64, Self::Error<CR::Error>> {
        Ok(match self {
            Codes::Unary => reader.read_unary()?,
            Codes::Gamma => reader.read_gamma()?,
            Codes::Delta => reader.read_delta()?,
            Codes::Omega => reader.read_omega()?,
            Codes::VByte => reader.read_vbyte()?,
            Codes::Zeta { k: 3 } => reader.read_zeta3()?,
            Codes::Zeta { k } => reader.read_zeta(*k)?,
            Codes::Pi { k } => reader.read_pi(*k)?,
            Codes::Golomb { b } => reader.read_golomb(*b as u64)?,
            Codes::ExpGolomb { k } => reader.read_exp_golomb(*k)?,
            Codes::Rice { log2_b } => reader.read_rice(*log2_b)?,
        })
    }
}

impl<E: Endianness, CR: CodesRead<E> + ?Sized> SpecificCodeRead<E, CR> for Codes {
    type Error<CRE: Debug + Send + Sync + 'static> = CRE;

    #[inline(always)]
    fn specific_read(&self, reader: &mut CR) -> Result<u64, Self::Error<CR::Error>> {
        <Self as GenericCodeRead>::generic_read(self, reader)
    }
}

impl GenericCodeWrite for Codes {
    type Error<CWE: Debug + Send + Sync + 'static> = CWE;

    #[inline]
    fn generic_write<E: Endianness, CW: CodesWrite<E> + ?Sized>(
        &self,
        writer: &mut CW,
        value: u64,
    ) -> Result<usize, Self::Error<CW::Error>> {
        Ok(match self {
            Codes::Unary => writer.write_unary(value)?,
            Codes::Gamma => writer.write_gamma(value)?,
            Codes::Delta => writer.write_delta(value)?,
            Codes::Omega => writer.write_omega(value)?,
            Codes::VByte => writer.write_vbyte(value)?,
            Codes::Zeta { k: 3 } => writer.write_zeta3(value)?,
            Codes::Zeta { k } => writer.write_zeta(value, *k)?,
            Codes::Pi { k } => writer.write_pi(value, *k)?,
            Codes::Golomb { b } => writer.write_golomb(value, *b as u64)?,
            Codes::ExpGolomb { k } => writer.write_exp_golomb(value, *k)?,
            Codes::Rice { log2_b } => writer.write_rice(value, *log2_b)?,
        })
    }
}

impl<E: Endianness, CW: CodesWrite<E> + ?Sized> SpecificCodeWrite<E, CW> for Codes {
    type Error<CWE: Debug + Send + Sync + 'static> = CWE;

    #[inline(always)]
    fn specific_write(&self, writer: &mut CW, value: u64) -> Result<usize, Self::Error<CW::Error>> {
        <Self as GenericCodeWrite>::generic_write(self, writer, value)
    }
}

impl CodeLen for Codes {
    #[inline]
    fn len(&self, value: u64) -> usize {
        match self {
            Codes::Unary => value as usize + 1,
            Codes::Gamma => len_gamma(value),
            Codes::Delta => len_delta(value),
            Codes::Omega => len_omega(value),
            Codes::VByte => bit_len_vbyte(value),
            Codes::Zeta { k } => len_zeta(value, *k),
            Codes::Pi { k } => len_pi(value, *k),
            Codes::Golomb { b } => len_golomb(value, *b as u64),
            Codes::ExpGolomb { k } => len_exp_golomb(value, *k),
            Codes::Rice { log2_b } => len_rice(value, *log2_b),
        }
    }
}

impl core::fmt::Display for Codes {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Codes::Unary => write!(f, "Unary"),
            Codes::Gamma => write!(f, "Gamma"),
            Codes::Delta => write!(f, "Delta"),
            Codes::Omega => write!(f, "Omega"),
            Codes::VByte => write!(f, "VByte"),
            Codes::Zeta { k } => write!(f, "Zeta({})", k),
            Codes::Pi { k } => write!(f, "Pi({})", k),
            Codes::Golomb { b } => write!(f, "Golomb({})", b),
            Codes::ExpGolomb { k } => write!(f, "ExpGolomb({})", k),
            Codes::Rice { log2_b } => write!(f, "Rice({})", log2_b),
        }
    }
}

impl std::str::FromStr for Codes {
    type Err = CodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Unary" => Ok(Codes::Unary),
            "Gamma" => Ok(Codes::Gamma),
            "Delta" => Ok(Codes::Delta),
            "Omega" => Ok(Codes::Omega),
            "VByte" => Ok(Codes::VByte),
            _ => {
                let mut parts = s.split('(');
                let name = parts
                    .next()
                    .ok_or_else(|| CodeError::UnknownCode(format!("Could not parse {}", s)))?;
                let k = parts
                    .next()
                    .ok_or_else(|| CodeError::UnknownCode(format!("Could not parse {}", s)))?
                    .split(')')
                    .next()
                    .ok_or_else(|| CodeError::UnknownCode(format!("Could not parse {}", s)))?;
                match name {
                    "Zeta" => Ok(Codes::Zeta { k: k.parse()? }),
                    "Pi" => Ok(Codes::Pi { k: k.parse()? }),
                    "Golomb" => Ok(Codes::Golomb { b: k.parse()? }),
                    "ExpGolomb" => Ok(Codes::ExpGolomb { k: k.parse()? }),
                    "Rice" => Ok(Codes::Rice { log2_b: k.parse()? }),
                    _ => Err(CodeError::UnknownCode(format!("Could not parse {}", name))),
                }
            }
        }
    }
}

type ReadFn<E, CR> = fn(&mut CR) -> Result<u64, <CR as BitRead<E>>::Error>;

/// A structure containing a function pointer dispatching the read method for a
/// code.
///
/// This is a more efficient way to pass a [`SpecificCodeRead`] to a method, as
/// a [`FuncReader`] does not need to do a runtime test to dispatch the correct
/// code.
///
/// Instances can be obtained by calling the [`new`](FuncReader::new) method with
///  method with a variant of the [`Codes`] enum, or by calling the
/// [`new_with_func`](FuncReader::new_with_func) method with a function pointer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
pub struct FuncReader<E: Endianness, CR: CodesRead<E> + ?Sized> {
    read_func: ReadFn<E, CR>,
}

impl<E: Endianness, CR: CodesRead<E> + ?Sized> FuncReader<E, CR> {
    const UNARY: ReadFn<E, CR> = |reader: &mut CR| reader.read_unary();
    const GAMMA: ReadFn<E, CR> = |reader: &mut CR| reader.read_gamma();
    const DELTA: ReadFn<E, CR> = |reader: &mut CR| reader.read_delta();
    const OMEGA: ReadFn<E, CR> = |reader: &mut CR| reader.read_omega();
    const VBYTE: ReadFn<E, CR> = |reader: &mut CR| reader.read_vbyte();
    const ZETA2: ReadFn<E, CR> = |reader: &mut CR| reader.read_zeta(2);
    const ZETA3: ReadFn<E, CR> = |reader: &mut CR| reader.read_zeta3();
    const ZETA4: ReadFn<E, CR> = |reader: &mut CR| reader.read_zeta(4);
    const ZETA5: ReadFn<E, CR> = |reader: &mut CR| reader.read_zeta(5);
    const ZETA6: ReadFn<E, CR> = |reader: &mut CR| reader.read_zeta(6);
    const ZETA7: ReadFn<E, CR> = |reader: &mut CR| reader.read_zeta(7);
    const ZETA8: ReadFn<E, CR> = |reader: &mut CR| reader.read_zeta(8);
    const ZETA9: ReadFn<E, CR> = |reader: &mut CR| reader.read_zeta(9);
    const ZETA10: ReadFn<E, CR> = |reader: &mut CR| reader.read_zeta(10);
    const RICE1: ReadFn<E, CR> = |reader: &mut CR| reader.read_rice(1);
    const RICE2: ReadFn<E, CR> = |reader: &mut CR| reader.read_rice(2);
    const RICE3: ReadFn<E, CR> = |reader: &mut CR| reader.read_rice(3);
    const RICE4: ReadFn<E, CR> = |reader: &mut CR| reader.read_rice(4);
    const RICE5: ReadFn<E, CR> = |reader: &mut CR| reader.read_rice(5);
    const RICE6: ReadFn<E, CR> = |reader: &mut CR| reader.read_rice(6);
    const RICE7: ReadFn<E, CR> = |reader: &mut CR| reader.read_rice(7);
    const RICE8: ReadFn<E, CR> = |reader: &mut CR| reader.read_rice(8);
    const RICE9: ReadFn<E, CR> = |reader: &mut CR| reader.read_rice(9);
    const RICE10: ReadFn<E, CR> = |reader: &mut CR| reader.read_rice(10);
    const PI2: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi(2);
    const PI3: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi(3);
    const PI4: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi(4);
    const PI5: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi(5);
    const PI6: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi(6);
    const PI7: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi(7);
    const PI8: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi(8);
    const PI9: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi(9);
    const PI10: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi(10);
    const GOLOMB2: ReadFn<E, CR> = |reader: &mut CR| reader.read_golomb(2);
    const GOLOMB3: ReadFn<E, CR> = |reader: &mut CR| reader.read_golomb(3);
    const GOLOMB4: ReadFn<E, CR> = |reader: &mut CR| reader.read_golomb(4);
    const GOLOMB5: ReadFn<E, CR> = |reader: &mut CR| reader.read_golomb(5);
    const GOLOMB6: ReadFn<E, CR> = |reader: &mut CR| reader.read_golomb(6);
    const GOLOMB7: ReadFn<E, CR> = |reader: &mut CR| reader.read_golomb(7);
    const GOLOMB8: ReadFn<E, CR> = |reader: &mut CR| reader.read_golomb(8);
    const GOLOMB9: ReadFn<E, CR> = |reader: &mut CR| reader.read_golomb(9);
    const GOLOMB10: ReadFn<E, CR> = |reader: &mut CR| reader.read_golomb(10);
    const EXP_GOLOMB1: ReadFn<E, CR> = |reader: &mut CR| reader.read_exp_golomb(1);
    const EXP_GOLOMB2: ReadFn<E, CR> = |reader: &mut CR| reader.read_exp_golomb(2);
    const EXP_GOLOMB3: ReadFn<E, CR> = |reader: &mut CR| reader.read_exp_golomb(3);
    const EXP_GOLOMB4: ReadFn<E, CR> = |reader: &mut CR| reader.read_exp_golomb(4);
    const EXP_GOLOMB5: ReadFn<E, CR> = |reader: &mut CR| reader.read_exp_golomb(5);
    const EXP_GOLOMB6: ReadFn<E, CR> = |reader: &mut CR| reader.read_exp_golomb(6);
    const EXP_GOLOMB7: ReadFn<E, CR> = |reader: &mut CR| reader.read_exp_golomb(7);
    const EXP_GOLOMB8: ReadFn<E, CR> = |reader: &mut CR| reader.read_exp_golomb(8);
    const EXP_GOLOMB9: ReadFn<E, CR> = |reader: &mut CR| reader.read_exp_golomb(9);
    const EXP_GOLOMB10: ReadFn<E, CR> = |reader: &mut CR| reader.read_exp_golomb(10);
    /// Return a new [`FuncReader`] for the given code.
    ///
    /// # Errors
    ///
    /// The method will return an error if there is no constant
    /// for the given code in [`FuncReader`].
    pub fn new(code: Codes) -> anyhow::Result<Self> {
        let read_func = match code {
            Codes::Unary => Self::UNARY,
            Codes::Gamma => Self::GAMMA,
            Codes::Delta => Self::DELTA,
            Codes::Omega => Self::OMEGA,
            Codes::VByte => Self::VBYTE,
            Codes::Zeta { k: 1 } => Self::GAMMA,
            Codes::Zeta { k: 2 } => Self::ZETA2,
            Codes::Zeta { k: 3 } => Self::ZETA3,
            Codes::Zeta { k: 4 } => Self::ZETA4,
            Codes::Zeta { k: 5 } => Self::ZETA5,
            Codes::Zeta { k: 6 } => Self::ZETA6,
            Codes::Zeta { k: 7 } => Self::ZETA7,
            Codes::Zeta { k: 8 } => Self::ZETA8,
            Codes::Zeta { k: 9 } => Self::ZETA9,
            Codes::Zeta { k: 10 } => Self::ZETA10,
            Codes::Rice { log2_b: 0 } => Self::UNARY,
            Codes::Rice { log2_b: 1 } => Self::RICE1,
            Codes::Rice { log2_b: 2 } => Self::RICE2,
            Codes::Rice { log2_b: 3 } => Self::RICE3,
            Codes::Rice { log2_b: 4 } => Self::RICE4,
            Codes::Rice { log2_b: 5 } => Self::RICE5,
            Codes::Rice { log2_b: 6 } => Self::RICE6,
            Codes::Rice { log2_b: 7 } => Self::RICE7,
            Codes::Rice { log2_b: 8 } => Self::RICE8,
            Codes::Rice { log2_b: 9 } => Self::RICE9,
            Codes::Rice { log2_b: 10 } => Self::RICE10,
            Codes::Pi { k: 0 } => Self::GAMMA,
            Codes::Pi { k: 1 } => Self::ZETA2,
            Codes::Pi { k: 2 } => Self::PI2,
            Codes::Pi { k: 3 } => Self::PI3,
            Codes::Pi { k: 4 } => Self::PI4,
            Codes::Pi { k: 5 } => Self::PI5,
            Codes::Pi { k: 6 } => Self::PI6,
            Codes::Pi { k: 7 } => Self::PI7,
            Codes::Pi { k: 8 } => Self::PI8,
            Codes::Pi { k: 9 } => Self::PI9,
            Codes::Pi { k: 10 } => Self::PI10,
            Codes::Golomb { b: 1 } => Self::UNARY,
            Codes::Golomb { b: 2 } => Self::GOLOMB2,
            Codes::Golomb { b: 3 } => Self::GOLOMB3,
            Codes::Golomb { b: 4 } => Self::GOLOMB4,
            Codes::Golomb { b: 5 } => Self::GOLOMB5,
            Codes::Golomb { b: 6 } => Self::GOLOMB6,
            Codes::Golomb { b: 7 } => Self::GOLOMB7,
            Codes::Golomb { b: 8 } => Self::GOLOMB8,
            Codes::Golomb { b: 9 } => Self::GOLOMB9,
            Codes::Golomb { b: 10 } => Self::GOLOMB10,
            Codes::ExpGolomb { k: 0 } => Self::GAMMA,
            Codes::ExpGolomb { k: 1 } => Self::EXP_GOLOMB1,
            Codes::ExpGolomb { k: 2 } => Self::EXP_GOLOMB2,
            Codes::ExpGolomb { k: 3 } => Self::EXP_GOLOMB3,
            Codes::ExpGolomb { k: 4 } => Self::EXP_GOLOMB4,
            Codes::ExpGolomb { k: 5 } => Self::EXP_GOLOMB5,
            Codes::ExpGolomb { k: 6 } => Self::EXP_GOLOMB6,
            Codes::ExpGolomb { k: 7 } => Self::EXP_GOLOMB7,
            Codes::ExpGolomb { k: 8 } => Self::EXP_GOLOMB8,
            Codes::ExpGolomb { k: 9 } => Self::EXP_GOLOMB9,
            Codes::ExpGolomb { k: 10 } => Self::EXP_GOLOMB10,
            _ => anyhow::bail!("Unsupported read dispatch for code {:?}", code),
        };
        Ok(Self { read_func })
    }

    /// Return a new [`FuncReader`] for the given function.
    pub fn new_with_func(read_func: ReadFn<E, CR>) -> Self {
        Self { read_func }
    }
}

impl<E: Endianness, CR: CodesRead<E> + ?Sized> SpecificCodeRead<E, CR> for FuncReader<E, CR> {
    type Error<CRE: Debug + Send + Sync + 'static> = CRE;

    #[inline(always)]
    fn specific_read(&self, reader: &mut CR) -> Result<u64, Self::Error<CR::Error>> {
        (self.read_func)(reader).map_err(Into::into)
    }
}

type WriteFn<E, CW> = fn(&mut CW, u64) -> Result<usize, <CW as BitWrite<E>>::Error>;

/// A structure containing a function pointer dispatching the write method for a
/// code.
///
/// This is a more efficient way to pass a [`SpecificCodeWrite`] to a method, as a
/// [`FuncWriter`] does not need to do a runtime test to dispatch the correct
/// code.
///
/// Instances can be obtained by calling the [`new`](FuncWriter::new) method with
///  method with a variant of the [`Codes`] enum, or by calling the
/// [`new_with_func`](FuncWriter::new_with_func) method with a function pointer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
pub struct FuncWriter<E: Endianness, CW: CodesWrite<E> + ?Sized> {
    write_func: WriteFn<E, CW>,
}

impl<E: Endianness, CW: CodesWrite<E> + ?Sized> FuncWriter<E, CW> {
    const UNARY: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_unary(value);
    const GAMMA: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_gamma(value);
    const DELTA: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_delta(value);
    const OMEGA: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_omega(value);
    const VBYTE: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_vbyte(value);
    const ZETA2: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_zeta(value, 2);
    const ZETA3: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_zeta3(value);
    const ZETA4: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_zeta(value, 4);
    const ZETA5: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_zeta(value, 5);
    const ZETA6: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_zeta(value, 6);
    const ZETA7: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_zeta(value, 7);
    const ZETA8: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_zeta(value, 8);
    const ZETA9: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_zeta(value, 9);
    const ZETA10: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_zeta(value, 10);
    const RICE1: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_rice(value, 1);
    const RICE2: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_rice(value, 2);
    const RICE3: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_rice(value, 3);
    const RICE4: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_rice(value, 4);
    const RICE5: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_rice(value, 5);
    const RICE6: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_rice(value, 6);
    const RICE7: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_rice(value, 7);
    const RICE8: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_rice(value, 8);
    const RICE9: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_rice(value, 9);
    const RICE10: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_rice(value, 10);
    const PI2: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi(value, 2);
    const PI3: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi(value, 3);
    const PI4: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi(value, 4);
    const PI5: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi(value, 5);
    const PI6: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi(value, 6);
    const PI7: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi(value, 7);
    const PI8: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi(value, 8);
    const PI9: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi(value, 9);
    const PI10: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi(value, 10);
    const GOLOMB2: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_golomb(value, 2);
    const GOLOMB3: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_golomb(value, 3);
    const GOLOMB4: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_golomb(value, 4);
    const GOLOMB5: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_golomb(value, 5);
    const GOLOMB6: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_golomb(value, 6);
    const GOLOMB7: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_golomb(value, 7);
    const GOLOMB8: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_golomb(value, 8);
    const GOLOMB9: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_golomb(value, 9);
    const GOLOMB10: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_golomb(value, 10);
    const EXP_GOLOMB1: WriteFn<E, CW> =
        |writer: &mut CW, value: u64| writer.write_exp_golomb(value, 1);
    const EXP_GOLOMB2: WriteFn<E, CW> =
        |writer: &mut CW, value: u64| writer.write_exp_golomb(value, 2);
    const EXP_GOLOMB3: WriteFn<E, CW> =
        |writer: &mut CW, value: u64| writer.write_exp_golomb(value, 3);
    const EXP_GOLOMB4: WriteFn<E, CW> =
        |writer: &mut CW, value: u64| writer.write_exp_golomb(value, 4);
    const EXP_GOLOMB5: WriteFn<E, CW> =
        |writer: &mut CW, value: u64| writer.write_exp_golomb(value, 5);
    const EXP_GOLOMB6: WriteFn<E, CW> =
        |writer: &mut CW, value: u64| writer.write_exp_golomb(value, 6);
    const EXP_GOLOMB7: WriteFn<E, CW> =
        |writer: &mut CW, value: u64| writer.write_exp_golomb(value, 7);
    const EXP_GOLOMB8: WriteFn<E, CW> =
        |writer: &mut CW, value: u64| writer.write_exp_golomb(value, 8);
    const EXP_GOLOMB9: WriteFn<E, CW> =
        |writer: &mut CW, value: u64| writer.write_exp_golomb(value, 9);
    const EXP_GOLOMB10: WriteFn<E, CW> =
        |writer: &mut CW, value: u64| writer.write_exp_golomb(value, 10);

    /// Return a new [`FuncWriter`] for the given code.
    ///
    /// # Errors
    ///
    /// The method will return an error if there is no constant
    /// for the given code in [`FuncWriter`].
    pub fn new(code: Codes) -> anyhow::Result<Self> {
        let write = match code {
            Codes::Unary => Self::UNARY,
            Codes::Gamma => Self::GAMMA,
            Codes::Delta => Self::DELTA,
            Codes::Omega => Self::OMEGA,
            Codes::VByte => Self::VBYTE,
            Codes::Zeta { k: 1 } => Self::GAMMA,
            Codes::Zeta { k: 2 } => Self::ZETA2,
            Codes::Zeta { k: 3 } => Self::ZETA3,
            Codes::Zeta { k: 4 } => Self::ZETA4,
            Codes::Zeta { k: 5 } => Self::ZETA5,
            Codes::Zeta { k: 6 } => Self::ZETA6,
            Codes::Zeta { k: 7 } => Self::ZETA7,
            Codes::Zeta { k: 8 } => Self::ZETA8,
            Codes::Zeta { k: 9 } => Self::ZETA9,
            Codes::Zeta { k: 10 } => Self::ZETA10,
            Codes::Rice { log2_b: 0 } => Self::UNARY,
            Codes::Rice { log2_b: 1 } => Self::RICE1,
            Codes::Rice { log2_b: 2 } => Self::RICE2,
            Codes::Rice { log2_b: 3 } => Self::RICE3,
            Codes::Rice { log2_b: 4 } => Self::RICE4,
            Codes::Rice { log2_b: 5 } => Self::RICE5,
            Codes::Rice { log2_b: 6 } => Self::RICE6,
            Codes::Rice { log2_b: 7 } => Self::RICE7,
            Codes::Rice { log2_b: 8 } => Self::RICE8,
            Codes::Rice { log2_b: 9 } => Self::RICE9,
            Codes::Rice { log2_b: 10 } => Self::RICE10,
            Codes::Pi { k: 0 } => Self::GAMMA,
            Codes::Pi { k: 1 } => Self::ZETA2,
            Codes::Pi { k: 2 } => Self::PI2,
            Codes::Pi { k: 3 } => Self::PI3,
            Codes::Pi { k: 4 } => Self::PI4,
            Codes::Pi { k: 5 } => Self::PI5,
            Codes::Pi { k: 6 } => Self::PI6,
            Codes::Pi { k: 7 } => Self::PI7,
            Codes::Pi { k: 8 } => Self::PI8,
            Codes::Pi { k: 9 } => Self::PI9,
            Codes::Pi { k: 10 } => Self::PI10,
            Codes::Golomb { b: 1 } => Self::UNARY,
            Codes::Golomb { b: 2 } => Self::GOLOMB2,
            Codes::Golomb { b: 3 } => Self::GOLOMB3,
            Codes::Golomb { b: 4 } => Self::GOLOMB4,
            Codes::Golomb { b: 5 } => Self::GOLOMB5,
            Codes::Golomb { b: 6 } => Self::GOLOMB6,
            Codes::Golomb { b: 7 } => Self::GOLOMB7,
            Codes::Golomb { b: 8 } => Self::GOLOMB8,
            Codes::Golomb { b: 9 } => Self::GOLOMB9,
            Codes::Golomb { b: 10 } => Self::GOLOMB10,
            Codes::ExpGolomb { k: 0 } => Self::GAMMA,
            Codes::ExpGolomb { k: 1 } => Self::EXP_GOLOMB1,
            Codes::ExpGolomb { k: 2 } => Self::EXP_GOLOMB2,
            Codes::ExpGolomb { k: 3 } => Self::EXP_GOLOMB3,
            Codes::ExpGolomb { k: 4 } => Self::EXP_GOLOMB4,
            Codes::ExpGolomb { k: 5 } => Self::EXP_GOLOMB5,
            Codes::ExpGolomb { k: 6 } => Self::EXP_GOLOMB6,
            Codes::ExpGolomb { k: 7 } => Self::EXP_GOLOMB7,
            Codes::ExpGolomb { k: 8 } => Self::EXP_GOLOMB8,
            Codes::ExpGolomb { k: 9 } => Self::EXP_GOLOMB9,
            Codes::ExpGolomb { k: 10 } => Self::EXP_GOLOMB10,
            _ => anyhow::bail!("Unsupported write dispatch for code {:?}", code),
        };
        Ok(Self { write_func: write })
    }

    /// Return a new [`FuncWriter`] for the given function.
    pub fn new_with_func(write_func: WriteFn<E, CW>) -> Self {
        Self { write_func }
    }
}

impl<E: Endianness, CW: CodesWrite<E> + ?Sized> SpecificCodeWrite<E, CW> for FuncWriter<E, CW> {
    type Error<CWE: Debug + Send + Sync + 'static> = CWE;

    #[inline(always)]
    fn specific_write(&self, writer: &mut CW, value: u64) -> Result<usize, Self::Error<CW::Error>> {
        (self.write_func)(writer, value).map_err(Into::into)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
/// A zero-sized struct with a const generic parameter representing a code using
/// the values exported by the
/// [`code_consts`](crate::codes::dispatch::code_consts) module.
///
/// Methods for all traits are implemented for this struct using a match on the
/// value of the const type parameter. Since the parameter is a constant, the
/// match is resolved at compile time, so there will be no runtime overhead.
///
/// If the value is not among those defined in the
/// [`code_consts`](crate::codes::dispatch::code_consts) module, the methods
/// will panic.
///
/// See the [module documentation](crate::codes::dispatch) for more information.
pub struct ConstCode<const CODE: usize>;

/// The constants to use as generic parameter for the [`ConstCode`] struct.
pub mod code_consts {
    pub const UNARY: usize = 0;
    pub const GAMMA: usize = 1;
    pub const DELTA: usize = 2;
    pub const OMEGA: usize = 3;
    pub const VBYTE: usize = 4;
    pub const ZETA1: usize = GAMMA;
    pub const ZETA2: usize = 5;
    pub const ZETA3: usize = 6;
    pub const ZETA4: usize = 7;
    pub const ZETA5: usize = 8;
    pub const ZETA6: usize = 9;
    pub const ZETA7: usize = 10;
    pub const ZETA8: usize = 11;
    pub const ZETA9: usize = 12;
    pub const ZETA10: usize = 13;
    pub const RICE0: usize = UNARY;
    pub const RICE1: usize = 14;
    pub const RICE2: usize = 15;
    pub const RICE3: usize = 16;
    pub const RICE4: usize = 17;
    pub const RICE5: usize = 18;
    pub const RICE6: usize = 19;
    pub const RICE7: usize = 20;
    pub const RICE8: usize = 21;
    pub const RICE9: usize = 22;
    pub const RICE10: usize = 23;
    pub const PI0: usize = GAMMA;
    pub const PI1: usize = ZETA2;
    pub const PI2: usize = 24;
    pub const PI3: usize = 25;
    pub const PI4: usize = 26;
    pub const PI5: usize = 27;
    pub const PI6: usize = 28;
    pub const PI7: usize = 29;
    pub const PI8: usize = 30;
    pub const PI9: usize = 31;
    pub const PI10: usize = 32;
    pub const GOLOMB1: usize = UNARY;
    pub const GOLOMB2: usize = 33;
    pub const GOLOMB3: usize = 34;
    pub const GOLOMB4: usize = 35;
    pub const GOLOMB5: usize = 36;
    pub const GOLOMB6: usize = 37;
    pub const GOLOMB7: usize = 38;
    pub const GOLOMB8: usize = 39;
    pub const GOLOMB9: usize = 40;
    pub const GOLOMB10: usize = 41;
    pub const EXP_GOLOMB0: usize = GAMMA;
    pub const EXP_GOLOMB1: usize = 42;
    pub const EXP_GOLOMB2: usize = 43;
    pub const EXP_GOLOMB3: usize = 44;
    pub const EXP_GOLOMB4: usize = 45;
    pub const EXP_GOLOMB5: usize = 46;
    pub const EXP_GOLOMB6: usize = 47;
    pub const EXP_GOLOMB7: usize = 48;
    pub const EXP_GOLOMB8: usize = 49;
    pub const EXP_GOLOMB9: usize = 50;
    pub const EXP_GOLOMB10: usize = 51;
}

impl<const CODE: usize> GenericCodeRead for ConstCode<CODE> {
    type Error<CRE: Debug + Send + Sync + 'static> = CRE;

    fn generic_read<E: Endianness, CR: CodesRead<E> + ?Sized>(
        &self,
        reader: &mut CR,
    ) -> Result<u64, Self::Error<CR::Error>> {
        match CODE {
            code_consts::UNARY => reader.read_unary(),
            code_consts::GAMMA => reader.read_gamma(),
            code_consts::DELTA => reader.read_delta(),
            code_consts::OMEGA => reader.read_omega(),
            code_consts::VBYTE => reader.read_vbyte(),
            code_consts::ZETA2 => reader.read_zeta(2),
            code_consts::ZETA3 => reader.read_zeta3(),
            code_consts::ZETA4 => reader.read_zeta(4),
            code_consts::ZETA5 => reader.read_zeta(5),
            code_consts::ZETA6 => reader.read_zeta(6),
            code_consts::ZETA7 => reader.read_zeta(7),
            code_consts::ZETA8 => reader.read_zeta(8),
            code_consts::ZETA9 => reader.read_zeta(9),
            code_consts::ZETA10 => reader.read_zeta(10),
            code_consts::RICE1 => reader.read_rice(1),
            code_consts::RICE2 => reader.read_rice(2),
            code_consts::RICE3 => reader.read_rice(3),
            code_consts::RICE4 => reader.read_rice(4),
            code_consts::RICE5 => reader.read_rice(5),
            code_consts::RICE6 => reader.read_rice(6),
            code_consts::RICE7 => reader.read_rice(7),
            code_consts::RICE8 => reader.read_rice(8),
            code_consts::RICE9 => reader.read_rice(9),
            code_consts::RICE10 => reader.read_rice(10),
            code_consts::PI2 => reader.read_pi(2),
            code_consts::PI3 => reader.read_pi(3),
            code_consts::PI4 => reader.read_pi(4),
            code_consts::PI5 => reader.read_pi(5),
            code_consts::PI6 => reader.read_pi(6),
            code_consts::PI7 => reader.read_pi(7),
            code_consts::PI8 => reader.read_pi(8),
            code_consts::PI9 => reader.read_pi(9),
            code_consts::PI10 => reader.read_pi(10),
            code_consts::GOLOMB2 => reader.read_golomb(2),
            code_consts::GOLOMB3 => reader.read_golomb(3),
            code_consts::GOLOMB4 => reader.read_golomb(4),
            code_consts::GOLOMB5 => reader.read_golomb(5),
            code_consts::GOLOMB6 => reader.read_golomb(6),
            code_consts::GOLOMB7 => reader.read_golomb(7),
            code_consts::GOLOMB8 => reader.read_golomb(8),
            code_consts::GOLOMB9 => reader.read_golomb(9),
            code_consts::GOLOMB10 => reader.read_golomb(10),
            code_consts::EXP_GOLOMB1 => reader.read_exp_golomb(1),
            code_consts::EXP_GOLOMB2 => reader.read_exp_golomb(2),
            code_consts::EXP_GOLOMB3 => reader.read_exp_golomb(3),
            code_consts::EXP_GOLOMB4 => reader.read_exp_golomb(4),
            code_consts::EXP_GOLOMB5 => reader.read_exp_golomb(5),
            code_consts::EXP_GOLOMB6 => reader.read_exp_golomb(6),
            code_consts::EXP_GOLOMB7 => reader.read_exp_golomb(7),
            code_consts::EXP_GOLOMB8 => reader.read_exp_golomb(8),
            code_consts::EXP_GOLOMB9 => reader.read_exp_golomb(9),
            code_consts::EXP_GOLOMB10 => reader.read_exp_golomb(10),
            _ => panic!("Unknown code index: {}", CODE),
        }
    }
}

impl<E: Endianness, CR: CodesRead<E> + ?Sized, const CODE: usize> SpecificCodeRead<E, CR>
    for ConstCode<CODE>
{
    type Error<CRE: Debug + Send + Sync + 'static> = CRE;

    #[inline(always)]
    fn specific_read(&self, reader: &mut CR) -> Result<u64, Self::Error<CR::Error>> {
        <Self as GenericCodeRead>::generic_read(self, reader)
    }
}

impl<const CODE: usize> GenericCodeWrite for ConstCode<CODE> {
    type Error<CWE: Debug + Send + Sync + 'static> = CWE;

    fn generic_write<E: Endianness, CW: CodesWrite<E> + ?Sized>(
        &self,
        writer: &mut CW,
        value: u64,
    ) -> Result<usize, Self::Error<CW::Error>> {
        match CODE {
            code_consts::UNARY => writer.write_unary(value),
            code_consts::GAMMA => writer.write_gamma(value),
            code_consts::DELTA => writer.write_delta(value),
            code_consts::OMEGA => writer.write_omega(value),
            code_consts::VBYTE => writer.write_vbyte(value),
            code_consts::ZETA2 => writer.write_zeta(value, 2),
            code_consts::ZETA3 => writer.write_zeta3(value),
            code_consts::ZETA4 => writer.write_zeta(value, 4),
            code_consts::ZETA5 => writer.write_zeta(value, 5),
            code_consts::ZETA6 => writer.write_zeta(value, 6),
            code_consts::ZETA7 => writer.write_zeta(value, 7),
            code_consts::ZETA8 => writer.write_zeta(value, 8),
            code_consts::ZETA9 => writer.write_zeta(value, 9),
            code_consts::ZETA10 => writer.write_zeta(value, 10),
            code_consts::RICE1 => writer.write_rice(value, 1),
            code_consts::RICE2 => writer.write_rice(value, 2),
            code_consts::RICE3 => writer.write_rice(value, 3),
            code_consts::RICE4 => writer.write_rice(value, 4),
            code_consts::RICE5 => writer.write_rice(value, 5),
            code_consts::RICE6 => writer.write_rice(value, 6),
            code_consts::RICE7 => writer.write_rice(value, 7),
            code_consts::RICE8 => writer.write_rice(value, 8),
            code_consts::RICE9 => writer.write_rice(value, 9),
            code_consts::RICE10 => writer.write_rice(value, 10),
            code_consts::PI2 => writer.write_pi(value, 2),
            code_consts::PI3 => writer.write_pi(value, 3),
            code_consts::PI4 => writer.write_pi(value, 4),
            code_consts::PI5 => writer.write_pi(value, 5),
            code_consts::PI6 => writer.write_pi(value, 6),
            code_consts::PI7 => writer.write_pi(value, 7),
            code_consts::PI8 => writer.write_pi(value, 8),
            code_consts::PI9 => writer.write_pi(value, 9),
            code_consts::PI10 => writer.write_pi(value, 10),
            code_consts::GOLOMB2 => writer.write_golomb(value, 2),
            code_consts::GOLOMB3 => writer.write_golomb(value, 3),
            code_consts::GOLOMB4 => writer.write_golomb(value, 4),
            code_consts::GOLOMB5 => writer.write_golomb(value, 5),
            code_consts::GOLOMB6 => writer.write_golomb(value, 6),
            code_consts::GOLOMB7 => writer.write_golomb(value, 7),
            code_consts::GOLOMB8 => writer.write_golomb(value, 8),
            code_consts::GOLOMB9 => writer.write_golomb(value, 9),
            code_consts::GOLOMB10 => writer.write_golomb(value, 10),
            code_consts::EXP_GOLOMB1 => writer.write_exp_golomb(value, 1),
            code_consts::EXP_GOLOMB2 => writer.write_exp_golomb(value, 2),
            code_consts::EXP_GOLOMB3 => writer.write_exp_golomb(value, 3),
            code_consts::EXP_GOLOMB4 => writer.write_exp_golomb(value, 4),
            code_consts::EXP_GOLOMB5 => writer.write_exp_golomb(value, 5),
            code_consts::EXP_GOLOMB6 => writer.write_exp_golomb(value, 6),
            code_consts::EXP_GOLOMB7 => writer.write_exp_golomb(value, 7),
            code_consts::EXP_GOLOMB8 => writer.write_exp_golomb(value, 8),
            code_consts::EXP_GOLOMB9 => writer.write_exp_golomb(value, 9),
            code_consts::EXP_GOLOMB10 => writer.write_exp_golomb(value, 10),
            _ => panic!("Unknown code: {}", CODE),
        }
    }
}

impl<E: Endianness, CW: CodesWrite<E> + ?Sized, const CODE: usize> SpecificCodeWrite<E, CW>
    for ConstCode<CODE>
{
    type Error<CWE: Debug + Send + Sync + 'static> = CWE;

    #[inline(always)]
    fn specific_write(&self, writer: &mut CW, value: u64) -> Result<usize, Self::Error<CW::Error>> {
        <Self as GenericCodeWrite>::generic_write(self, writer, value)
    }
}

impl<const CODE: usize> CodeLen for ConstCode<CODE> {
    #[inline]
    fn len(&self, value: u64) -> usize {
        match CODE {
            code_consts::UNARY => value as usize + 1,
            code_consts::GAMMA => len_gamma(value),
            code_consts::DELTA => len_delta(value),
            code_consts::OMEGA => len_omega(value),
            code_consts::VBYTE => bit_len_vbyte(value),
            code_consts::ZETA2 => len_zeta(value, 2),
            code_consts::ZETA3 => len_zeta(value, 3),
            code_consts::ZETA4 => len_zeta(value, 4),
            code_consts::ZETA5 => len_zeta(value, 5),
            code_consts::ZETA6 => len_zeta(value, 6),
            code_consts::ZETA7 => len_zeta(value, 7),
            code_consts::ZETA8 => len_zeta(value, 8),
            code_consts::ZETA9 => len_zeta(value, 9),
            code_consts::ZETA10 => len_zeta(value, 10),
            code_consts::RICE1 => len_rice(value, 1),
            code_consts::RICE2 => len_rice(value, 2),
            code_consts::RICE3 => len_rice(value, 3),
            code_consts::RICE4 => len_rice(value, 4),
            code_consts::RICE5 => len_rice(value, 5),
            code_consts::RICE6 => len_rice(value, 6),
            code_consts::RICE7 => len_rice(value, 7),
            code_consts::RICE8 => len_rice(value, 8),
            code_consts::RICE9 => len_rice(value, 9),
            code_consts::RICE10 => len_rice(value, 10),
            code_consts::PI2 => len_pi(value, 2),
            code_consts::PI3 => len_pi(value, 3),
            code_consts::PI4 => len_pi(value, 4),
            code_consts::PI5 => len_pi(value, 5),
            code_consts::PI6 => len_pi(value, 6),
            code_consts::PI7 => len_pi(value, 7),
            code_consts::PI8 => len_pi(value, 8),
            code_consts::PI9 => len_pi(value, 9),
            code_consts::PI10 => len_pi(value, 10),
            code_consts::GOLOMB2 => len_golomb(value, 2),
            code_consts::GOLOMB3 => len_golomb(value, 3),
            code_consts::GOLOMB4 => len_golomb(value, 4),
            code_consts::GOLOMB5 => len_golomb(value, 5),
            code_consts::GOLOMB6 => len_golomb(value, 6),
            code_consts::GOLOMB7 => len_golomb(value, 7),
            code_consts::GOLOMB8 => len_golomb(value, 8),
            code_consts::GOLOMB9 => len_golomb(value, 9),
            code_consts::GOLOMB10 => len_golomb(value, 10),
            code_consts::EXP_GOLOMB1 => len_exp_golomb(value, 1),
            code_consts::EXP_GOLOMB2 => len_exp_golomb(value, 2),
            code_consts::EXP_GOLOMB3 => len_exp_golomb(value, 3),
            code_consts::EXP_GOLOMB4 => len_exp_golomb(value, 4),
            code_consts::EXP_GOLOMB5 => len_exp_golomb(value, 5),
            code_consts::EXP_GOLOMB6 => len_exp_golomb(value, 6),
            code_consts::EXP_GOLOMB7 => len_exp_golomb(value, 7),
            code_consts::EXP_GOLOMB8 => len_exp_golomb(value, 8),
            code_consts::EXP_GOLOMB9 => len_exp_golomb(value, 9),
            code_consts::EXP_GOLOMB10 => len_exp_golomb(value, 10),
            _ => panic!("Unknown code: {}", CODE),
        }
    }
}

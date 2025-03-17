/*
 * SPDX-FileCopyrightText: 2025 Tommaso Fontana
 * SPDX-FileCopyrightText: 2025 Inria
 * SPDX-FileCopyrightText: 2025 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Dynamic-dispatching factories for readers with a lifetime.
//! 
//! # Motivation
//! 
//! [`FuncCodeReader`] already provides dynamic dispatching of read functions,
//! but in some uses cases the reader has to reference some data (e.g., readers
//! based on the same memory buffer). In this case, one would need to create a
//! dispatching function pointer for each code and each reader because the
//! lifetime of different readers make the function pointers incompatible.
//! 
//! The trait [`CodesReaderFactory`] solves this problem by providing a way to
//! create a [`CodesRead`] with a lifetime that can reference data owned by the
//! factory. This trait must be implemented by client applications.
//! 
//! At the point, one can create a [`FactoryFuncCodeReader`] depending on a
//! specific [`CodesReaderFactory`]. The [`FactoryFuncCodeReader`] will store a
//! function pointer with a generic lifetime that can be downcast to a specific
//! lifetime. Thus, the function pointer is created just once at the creation of
//! the [`FactoryFuncCodeReader`], and can be reused to create
//! [`FuncCodeReader`]s with any lifetime using [`FactoryFuncCodeReader::get`].
//! 
//! # Implementation Notes
//! 
//! In principle, we would like to have inside a [`FactoryFuncCodeReader`] a
//! field with type
//! 
//! ```ignore
//! for<'a> FuncCodeReader<E, CRF::CodesReader<'a>>
//! ```
//! 
//! However, this is not possible in the Rust type system. We can however write
//! the type
//! 
//! ```ignore
//! for<'a> fn(&mut CRF::CodesReader<'a>) -> Result<u64>
//! ```
//! 
//! This workaround is not perfect as we cannot properly specify the error type:
//! ```ignore
//! Result<u64, <CRF::CodesReader<'a> as BitRead<E>>::Error>
//! ```
//! The compiler here complains that the return type has a lifetime not
//! constrained by the input arguments.
//!
//! To work around this problem, we could add an otherwise useless associated
//! type `CodesReaderFactory::Error` to the [`CodesReaderFactory`] trait,
//! imposing that the error type of [`CodesReaderFactory::CodesReader`] is the
//! same. Unfortunately, this requires that all users of the factory add a `where`
//! constraint in which the error type is written explicitly.
//!
//! To mitigate this problem, we provide instead a helper trait
//! [`CodesReaderFactoryHelper`] that extends [`CodesReaderFactory`]; the helper
//! trait contains an `Error` associated type and [uses higher-rank trait
//! bounds](https://users.rust-lang.org/t/extracting-static-associated-type-from-type-with-lifetime/126880)
//! to bind the associated type to the error type of the
//! [`CodesReaderFactory::CodesReader`]. The user can implement
//! [`CodesReaderFactory`] on its own types and write trait bounds using
//! [`CodesReaderFactoryHelper`]:
//! ```ignore
//! fn test<E: Endianness, CRF: CodesReaderFactoryHelper<E>>(factory: CRF)
//! {
//!     let reader = factory.new_reader();
//!     // do something with the reader
//!     // CRF::Error is the error type of CRF::CodesReader<'a>
//! }
//! ```

use super::*;
use anyhow::Result;
use core::fmt::Debug;
/// A trait that models a type that can return a [`CodesRead`] that can reference
/// data owned by the factory. The typical case is a factory that owns the
/// bit stream, and returns a [`CodesRead`] that can read from it.
pub trait CodesReaderFactory<E: Endianness> {
    type CodesReader<'a>
    where
        Self: 'a;

    /// Create a new code reader that can reference data owned by the factory.
    fn new_reader(&self) -> Self::CodesReader<'_>;
}

/// Extension helper trait for [`CodesReaderFactory`].
/// 
/// By writing trait bounds using this helper instead of [`CodesReaderFactory`],
/// you can access the error type of the [`CodesReaderFactory::CodesReader`] through
/// [`CodesReaderFactoryHelper::Error`].
pub trait CodesReaderFactoryHelper<E: Endianness>:
    for<'a> CodesReaderFactory<E, CodesReader<'a>: CodesRead<E, Error = Self::Error>>
{
    type Error;
}

impl<E: Endianness, F, ERR> CodesReaderFactoryHelper<E> for F
where
    F: ?Sized + for<'a> CodesReaderFactory<E, CodesReader<'a>: CodesRead<E, Error = ERR>>,
{
    type Error = ERR;
}

/// The function type stored in a [`FactoryFuncCodeReader`].
/// 
/// The role of this type is analogous to that of `ReadFn` in [`FuncCodeReader`],
/// but we have an extra lifetime parameter to handle the lifetime
/// of the [`CodesReaderFactory::CodesReader`].
type FactoryReadFn<E, CRF> = for<'a> fn(
    &mut <CRF as CodesReaderFactory<E>>::CodesReader<'a>,
) -> Result<u64, <CRF as CodesReaderFactoryHelper<E>>::Error>;

/// A newtype depending on a [`CodesReaderFactory`] and containing a function
/// pointer dispatching the read method for a code.
/// 
/// It is essentially a version of [`FuncCodeReader`] that depends on a
/// [`CodesReaderFactory`] and its associated
/// [`CodesReaderFactory::CodesReader`] instead of a generic [`CodesRead`].
#[derive(Debug, Copy, PartialEq, Eq)]
pub struct FactoryFuncCodeReader<E: Endianness, CRF: CodesReaderFactoryHelper<E> + ?Sized>(
    FactoryReadFn<E, CRF>,
);

/// Manually implement [`Clone`] to avoid the [`Clone`] bound on `CR` and `E`.
impl<E: Endianness, CRF: CodesReaderFactoryHelper<E> + ?Sized> Clone
    for FactoryFuncCodeReader<E, CRF>
{
    #[inline(always)]
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<E: Endianness, CRF: CodesReaderFactoryHelper<E> + ?Sized> FactoryFuncCodeReader<E, CRF> {
    // due to the added lifetime generic we cannot just re-use the FuncCodeReader definitions
    const UNARY: FactoryReadFn<E, CRF> = |reader| reader.read_unary();
    const GAMMA: FactoryReadFn<E, CRF> = |reader| reader.read_gamma();
    const DELTA: FactoryReadFn<E, CRF> = |reader| reader.read_delta();
    const OMEGA: FactoryReadFn<E, CRF> = |reader| reader.read_omega();
    const VBYTE_BE: FactoryReadFn<E, CRF> = |reader| reader.read_vbyte_be();
    const VBYTE_LE: FactoryReadFn<E, CRF> = |reader| reader.read_vbyte_le();
    const ZETA2: FactoryReadFn<E, CRF> = |reader| reader.read_zeta(2);
    const ZETA3: FactoryReadFn<E, CRF> = |reader| reader.read_zeta3();
    const ZETA4: FactoryReadFn<E, CRF> = |reader| reader.read_zeta(4);
    const ZETA5: FactoryReadFn<E, CRF> = |reader| reader.read_zeta(5);
    const ZETA6: FactoryReadFn<E, CRF> = |reader| reader.read_zeta(6);
    const ZETA7: FactoryReadFn<E, CRF> = |reader| reader.read_zeta(7);
    const ZETA8: FactoryReadFn<E, CRF> = |reader| reader.read_zeta(8);
    const ZETA9: FactoryReadFn<E, CRF> = |reader| reader.read_zeta(9);
    const ZETA10: FactoryReadFn<E, CRF> = |reader| reader.read_zeta(10);
    const RICE1: FactoryReadFn<E, CRF> = |reader| reader.read_rice(1);
    const RICE2: FactoryReadFn<E, CRF> = |reader| reader.read_rice(2);
    const RICE3: FactoryReadFn<E, CRF> = |reader| reader.read_rice(3);
    const RICE4: FactoryReadFn<E, CRF> = |reader| reader.read_rice(4);
    const RICE5: FactoryReadFn<E, CRF> = |reader| reader.read_rice(5);
    const RICE6: FactoryReadFn<E, CRF> = |reader| reader.read_rice(6);
    const RICE7: FactoryReadFn<E, CRF> = |reader| reader.read_rice(7);
    const RICE8: FactoryReadFn<E, CRF> = |reader| reader.read_rice(8);
    const RICE9: FactoryReadFn<E, CRF> = |reader| reader.read_rice(9);
    const RICE10: FactoryReadFn<E, CRF> = |reader| reader.read_rice(10);
    const PI1: FactoryReadFn<E, CRF> = |reader| reader.read_pi(1);
    const PI2: FactoryReadFn<E, CRF> = |reader| reader.read_pi(2);
    const PI3: FactoryReadFn<E, CRF> = |reader| reader.read_pi(3);
    const PI4: FactoryReadFn<E, CRF> = |reader| reader.read_pi(4);
    const PI5: FactoryReadFn<E, CRF> = |reader| reader.read_pi(5);
    const PI6: FactoryReadFn<E, CRF> = |reader| reader.read_pi(6);
    const PI7: FactoryReadFn<E, CRF> = |reader| reader.read_pi(7);
    const PI8: FactoryReadFn<E, CRF> = |reader| reader.read_pi(8);
    const PI9: FactoryReadFn<E, CRF> = |reader| reader.read_pi(9);
    const PI10: FactoryReadFn<E, CRF> = |reader| reader.read_pi(10);
    const GOLOMB3: FactoryReadFn<E, CRF> = |reader| reader.read_golomb(3);
    const GOLOMB5: FactoryReadFn<E, CRF> = |reader| reader.read_golomb(5);
    const GOLOMB6: FactoryReadFn<E, CRF> = |reader| reader.read_golomb(6);
    const GOLOMB7: FactoryReadFn<E, CRF> = |reader| reader.read_golomb(7);
    const GOLOMB9: FactoryReadFn<E, CRF> = |reader| reader.read_golomb(9);
    const GOLOMB10: FactoryReadFn<E, CRF> = |reader| reader.read_golomb(10);
    const EXP_GOLOMB1: FactoryReadFn<E, CRF> = |reader| reader.read_exp_golomb(1);
    const EXP_GOLOMB2: FactoryReadFn<E, CRF> = |reader| reader.read_exp_golomb(2);
    const EXP_GOLOMB3: FactoryReadFn<E, CRF> = |reader| reader.read_exp_golomb(3);
    const EXP_GOLOMB4: FactoryReadFn<E, CRF> = |reader| reader.read_exp_golomb(4);
    const EXP_GOLOMB5: FactoryReadFn<E, CRF> = |reader| reader.read_exp_golomb(5);
    const EXP_GOLOMB6: FactoryReadFn<E, CRF> = |reader| reader.read_exp_golomb(6);
    const EXP_GOLOMB7: FactoryReadFn<E, CRF> = |reader| reader.read_exp_golomb(7);
    const EXP_GOLOMB8: FactoryReadFn<E, CRF> = |reader| reader.read_exp_golomb(8);
    const EXP_GOLOMB9: FactoryReadFn<E, CRF> = |reader| reader.read_exp_golomb(9);
    const EXP_GOLOMB10: FactoryReadFn<E, CRF> = |reader| reader.read_exp_golomb(10);

    /// Return a new [`FactoryFuncCodeReader`] for the given code.
    ///
    /// # Errors
    ///
    /// The method will return an error if there is no constant
    /// for the given code in [`FactoryFuncCodeReader`].
    pub fn new(code: Codes) -> anyhow::Result<Self> {
        let read_func = match code {
            Codes::Unary => Self::UNARY,
            Codes::Gamma => Self::GAMMA,
            Codes::Delta => Self::DELTA,
            Codes::Omega => Self::OMEGA,
            Codes::VByteBe => Self::VBYTE_BE,
            Codes::VByteLe => Self::VBYTE_LE,
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
            Codes::Pi { k: 1 } => Self::PI1,
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
            Codes::Golomb { b: 2 } => Self::RICE1,
            Codes::Golomb { b: 3 } => Self::GOLOMB3,
            Codes::Golomb { b: 4 } => Self::RICE2,
            Codes::Golomb { b: 5 } => Self::GOLOMB5,
            Codes::Golomb { b: 6 } => Self::GOLOMB6,
            Codes::Golomb { b: 7 } => Self::GOLOMB7,
            Codes::Golomb { b: 8 } => Self::RICE3,
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
        Ok(Self(read_func))
    }

    /// Returns a new [`FactoryFuncCodeReader`] for the given function.
    #[inline(always)]
    pub fn new_with_func(read_func: FactoryReadFn<E, CRF>) -> Self {
        Self(read_func)
    }

    /// Returns the function pointer for the code.
    #[inline(always)]
    pub fn inner(&self) -> FactoryReadFn<E, CRF> {
        self.0
    }

    /// Returns a [`FuncCodeReader`] compatible with `CRF`'s
    /// [`CodesReaderFactory::CodesReader`] for a given lifetime `'a`.
    #[inline(always)]
    pub fn get<'a>(
        &self,
    ) -> super::FuncCodeReader<E, <CRF as CodesReaderFactory<E>>::CodesReader<'a>> {
        super::FuncCodeReader::new_with_func(self.0)
    }
}

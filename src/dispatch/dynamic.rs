/*
 * SPDX-FileCopyrightText: 2025 Tommaso Fontana
 * SPDX-FileCopyrightText: 2025 Inria
 * SPDX-FileCopyrightText: 2025 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Dynamic dispatching for codes based on function pointers.
//!
//! This kind of dispatch is resolved at runtime, but just once, at construction
//! time, against a specific [`CodesRead`]. The code is stored in a function
//! pointer, so it cannot be inlined like in the [static
//! case](crate::dispatch::r#static), but the approach is more flexible.

use super::*;
#[cfg(feature = "mem_dbg")]
use mem_dbg::{MemDbg, MemSize};

type ReadFn<E, CR> = fn(&mut CR) -> Result<u64, <CR as BitRead<E>>::Error>;

/// A newtype containing a function pointer dispatching the read
/// method for a code.
///
/// This is a more efficient way to pass a [`StaticCodeRead`] to a method, as a
/// [`FuncCodeReader`] does not need to do a runtime test to dispatch the
/// correct code.
///
/// Instances can be obtained by calling the [`new`](FuncCodeReader::new) method
/// with a variant of the [`Codes`] enum, or by calling the
/// [`new_with_func`](FuncCodeReader::new_with_func) method with a function
/// pointer.
///
/// Note that since selection of the code happens in the
/// [`new`](FuncCodeReader::new) method, it is more efficient to clone a
/// [`FuncCodeReader`] than to create a new one.
#[derive(Debug, Copy)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
pub struct FuncCodeReader<E: Endianness, CR: CodesRead<E> + ?Sized>(ReadFn<E, CR>);

/// Manually implement [`Clone`] to avoid the [`Clone`] bound on CR and E
impl<E: Endianness, CR: CodesRead<E> + ?Sized> Clone for FuncCodeReader<E, CR> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<E: Endianness, CR: CodesRead<E> + ?Sized> FuncCodeReader<E, CR> {
    const UNARY: ReadFn<E, CR> = |reader: &mut CR| reader.read_unary();
    const GAMMA: ReadFn<E, CR> = |reader: &mut CR| reader.read_gamma();
    const DELTA: ReadFn<E, CR> = |reader: &mut CR| reader.read_delta();
    const OMEGA: ReadFn<E, CR> = |reader: &mut CR| reader.read_omega();
    const VBYTE_BE: ReadFn<E, CR> = |reader: &mut CR| reader.read_vbyte_be();
    const VBYTE_LE: ReadFn<E, CR> = |reader: &mut CR| reader.read_vbyte_le();
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
    const PI1: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi(1);
    const PI2: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi2();
    const PI3: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi(3);
    const PI4: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi(4);
    const PI5: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi(5);
    const PI6: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi(6);
    const PI7: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi(7);
    const PI8: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi(8);
    const PI9: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi(9);
    const PI10: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi(10);
    const GOLOMB3: ReadFn<E, CR> = |reader: &mut CR| reader.read_golomb(3);
    const GOLOMB5: ReadFn<E, CR> = |reader: &mut CR| reader.read_golomb(5);
    const GOLOMB6: ReadFn<E, CR> = |reader: &mut CR| reader.read_golomb(6);
    const GOLOMB7: ReadFn<E, CR> = |reader: &mut CR| reader.read_golomb(7);
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

    /// Returns a new [`FuncCodeReader`] for the given code.
    ///
    /// The code is [canonicalized](Codes::canonicalize) before
    /// the lookup, so equivalent codes yield the same reader.
    ///
    /// # Errors
    ///
    /// The method will return an error if there is no constant
    /// for the given code in [`FuncCodeReader`].
    pub const fn new(code: Codes) -> Result<Self, DispatchError> {
        let code = code.canonicalize();
        let read_func = match code {
            Codes::Unary => Self::UNARY,
            Codes::Gamma => Self::GAMMA,
            Codes::Delta => Self::DELTA,
            Codes::Omega => Self::OMEGA,
            Codes::VByteBe => Self::VBYTE_BE,
            Codes::VByteLe => Self::VBYTE_LE,
            Codes::Zeta(2) => Self::ZETA2,
            Codes::Zeta(3) => Self::ZETA3,
            Codes::Zeta(4) => Self::ZETA4,
            Codes::Zeta(5) => Self::ZETA5,
            Codes::Zeta(6) => Self::ZETA6,
            Codes::Zeta(7) => Self::ZETA7,
            Codes::Zeta(8) => Self::ZETA8,
            Codes::Zeta(9) => Self::ZETA9,
            Codes::Zeta(10) => Self::ZETA10,
            Codes::Rice(1) => Self::RICE1,
            Codes::Rice(2) => Self::RICE2,
            Codes::Rice(3) => Self::RICE3,
            Codes::Rice(4) => Self::RICE4,
            Codes::Rice(5) => Self::RICE5,
            Codes::Rice(6) => Self::RICE6,
            Codes::Rice(7) => Self::RICE7,
            Codes::Rice(8) => Self::RICE8,
            Codes::Rice(9) => Self::RICE9,
            Codes::Rice(10) => Self::RICE10,
            Codes::Pi(1) => Self::PI1,
            Codes::Pi(2) => Self::PI2,
            Codes::Pi(3) => Self::PI3,
            Codes::Pi(4) => Self::PI4,
            Codes::Pi(5) => Self::PI5,
            Codes::Pi(6) => Self::PI6,
            Codes::Pi(7) => Self::PI7,
            Codes::Pi(8) => Self::PI8,
            Codes::Pi(9) => Self::PI9,
            Codes::Pi(10) => Self::PI10,
            Codes::Golomb(3) => Self::GOLOMB3,
            Codes::Golomb(5) => Self::GOLOMB5,
            Codes::Golomb(6) => Self::GOLOMB6,
            Codes::Golomb(7) => Self::GOLOMB7,
            Codes::Golomb(9) => Self::GOLOMB9,
            Codes::Golomb(10) => Self::GOLOMB10,
            Codes::ExpGolomb(1) => Self::EXP_GOLOMB1,
            Codes::ExpGolomb(2) => Self::EXP_GOLOMB2,
            Codes::ExpGolomb(3) => Self::EXP_GOLOMB3,
            Codes::ExpGolomb(4) => Self::EXP_GOLOMB4,
            Codes::ExpGolomb(5) => Self::EXP_GOLOMB5,
            Codes::ExpGolomb(6) => Self::EXP_GOLOMB6,
            Codes::ExpGolomb(7) => Self::EXP_GOLOMB7,
            Codes::ExpGolomb(8) => Self::EXP_GOLOMB8,
            Codes::ExpGolomb(9) => Self::EXP_GOLOMB9,
            Codes::ExpGolomb(10) => Self::EXP_GOLOMB10,
            _ => return Err(DispatchError::UnsupportedCode(code)),
        };
        Ok(Self(read_func))
    }

    /// Returns a new [`FuncCodeReader`] for the given function.
    #[inline(always)]
    pub fn new_with_func(read_func: ReadFn<E, CR>) -> Self {
        Self(read_func)
    }

    /// Returns the function pointer for the code.
    #[must_use]
    #[inline(always)]
    pub fn get_func(&self) -> ReadFn<E, CR> {
        self.0
    }
}

impl<E: Endianness, CR: CodesRead<E> + ?Sized> StaticCodeRead<E, CR> for FuncCodeReader<E, CR> {
    #[inline(always)]
    fn read(&self, reader: &mut CR) -> Result<u64, CR::Error> {
        (self.0)(reader)
    }
}

type WriteFn<E, CW> = fn(&mut CW, u64) -> Result<usize, <CW as BitWrite<E>>::Error>;

/// A newtype containing a function pointer dispatching the write method for a
/// code.
///
/// This is a more efficient way to pass a [`StaticCodeWrite`] to a method, as
/// a [`FuncCodeWriter`] does not need to do a runtime test to dispatch the
/// correct code.
///
/// Instances can be obtained by calling the [`new`](FuncCodeWriter::new) method
/// with a variant of the [`Codes`] enum, or by calling the
/// [`new_with_func`](FuncCodeWriter::new_with_func) method with a function
/// pointer.
///
/// Note that since selection of the code happens in the
/// [`new`](FuncCodeWriter::new) method, it is more efficient to clone a
/// [`FuncCodeWriter`] than to create a new one.
#[derive(Debug, Copy)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
pub struct FuncCodeWriter<E: Endianness, CW: CodesWrite<E> + ?Sized>(WriteFn<E, CW>);

/// Manually implement [`Clone`] to avoid the [`Clone`] bound on CW and E.
impl<E: Endianness, CW: CodesWrite<E> + ?Sized> Clone for FuncCodeWriter<E, CW> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<E: Endianness, CW: CodesWrite<E> + ?Sized> FuncCodeWriter<E, CW> {
    const UNARY: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_unary(n);
    const GAMMA: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_gamma(n);
    const DELTA: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_delta(n);
    const OMEGA: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_omega(n);
    const VBYTE_BE: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_vbyte_be(n);
    const VBYTE_LE: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_vbyte_le(n);
    const ZETA2: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_zeta(n, 2);
    const ZETA3: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_zeta3(n);
    const ZETA4: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_zeta(n, 4);
    const ZETA5: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_zeta(n, 5);
    const ZETA6: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_zeta(n, 6);
    const ZETA7: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_zeta(n, 7);
    const ZETA8: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_zeta(n, 8);
    const ZETA9: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_zeta(n, 9);
    const ZETA10: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_zeta(n, 10);
    const RICE1: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_rice(n, 1);
    const RICE2: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_rice(n, 2);
    const RICE3: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_rice(n, 3);
    const RICE4: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_rice(n, 4);
    const RICE5: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_rice(n, 5);
    const RICE6: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_rice(n, 6);
    const RICE7: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_rice(n, 7);
    const RICE8: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_rice(n, 8);
    const RICE9: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_rice(n, 9);
    const RICE10: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_rice(n, 10);
    const PI1: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_pi(n, 1);
    const PI2: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_pi2(n);
    const PI3: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_pi(n, 3);
    const PI4: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_pi(n, 4);
    const PI5: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_pi(n, 5);
    const PI6: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_pi(n, 6);
    const PI7: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_pi(n, 7);
    const PI8: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_pi(n, 8);
    const PI9: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_pi(n, 9);
    const PI10: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_pi(n, 10);
    const GOLOMB3: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_golomb(n, 3);
    const GOLOMB5: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_golomb(n, 5);
    const GOLOMB6: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_golomb(n, 6);
    const GOLOMB7: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_golomb(n, 7);
    const GOLOMB9: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_golomb(n, 9);
    const GOLOMB10: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_golomb(n, 10);
    const EXP_GOLOMB1: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_exp_golomb(n, 1);
    const EXP_GOLOMB2: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_exp_golomb(n, 2);
    const EXP_GOLOMB3: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_exp_golomb(n, 3);
    const EXP_GOLOMB4: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_exp_golomb(n, 4);
    const EXP_GOLOMB5: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_exp_golomb(n, 5);
    const EXP_GOLOMB6: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_exp_golomb(n, 6);
    const EXP_GOLOMB7: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_exp_golomb(n, 7);
    const EXP_GOLOMB8: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_exp_golomb(n, 8);
    const EXP_GOLOMB9: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_exp_golomb(n, 9);
    const EXP_GOLOMB10: WriteFn<E, CW> = |writer: &mut CW, n: u64| writer.write_exp_golomb(n, 10);

    /// Returns a new [`FuncCodeWriter`] for the given code.
    ///
    /// The code is [canonicalized](Codes::canonicalize) before
    /// the lookup, so equivalent codes yield the same writer.
    ///
    /// # Errors
    ///
    /// The method will return an error if there is no constant
    /// for the given code in [`FuncCodeWriter`].
    pub const fn new(code: Codes) -> Result<Self, DispatchError> {
        let code = code.canonicalize();
        let write_func = match code {
            Codes::Unary => Self::UNARY,
            Codes::Gamma => Self::GAMMA,
            Codes::Delta => Self::DELTA,
            Codes::Omega => Self::OMEGA,
            Codes::VByteBe => Self::VBYTE_BE,
            Codes::VByteLe => Self::VBYTE_LE,
            Codes::Zeta(2) => Self::ZETA2,
            Codes::Zeta(3) => Self::ZETA3,
            Codes::Zeta(4) => Self::ZETA4,
            Codes::Zeta(5) => Self::ZETA5,
            Codes::Zeta(6) => Self::ZETA6,
            Codes::Zeta(7) => Self::ZETA7,
            Codes::Zeta(8) => Self::ZETA8,
            Codes::Zeta(9) => Self::ZETA9,
            Codes::Zeta(10) => Self::ZETA10,
            Codes::Rice(1) => Self::RICE1,
            Codes::Rice(2) => Self::RICE2,
            Codes::Rice(3) => Self::RICE3,
            Codes::Rice(4) => Self::RICE4,
            Codes::Rice(5) => Self::RICE5,
            Codes::Rice(6) => Self::RICE6,
            Codes::Rice(7) => Self::RICE7,
            Codes::Rice(8) => Self::RICE8,
            Codes::Rice(9) => Self::RICE9,
            Codes::Rice(10) => Self::RICE10,
            Codes::Pi(1) => Self::PI1,
            Codes::Pi(2) => Self::PI2,
            Codes::Pi(3) => Self::PI3,
            Codes::Pi(4) => Self::PI4,
            Codes::Pi(5) => Self::PI5,
            Codes::Pi(6) => Self::PI6,
            Codes::Pi(7) => Self::PI7,
            Codes::Pi(8) => Self::PI8,
            Codes::Pi(9) => Self::PI9,
            Codes::Pi(10) => Self::PI10,
            Codes::Golomb(3) => Self::GOLOMB3,
            Codes::Golomb(5) => Self::GOLOMB5,
            Codes::Golomb(6) => Self::GOLOMB6,
            Codes::Golomb(7) => Self::GOLOMB7,
            Codes::Golomb(9) => Self::GOLOMB9,
            Codes::Golomb(10) => Self::GOLOMB10,
            Codes::ExpGolomb(1) => Self::EXP_GOLOMB1,
            Codes::ExpGolomb(2) => Self::EXP_GOLOMB2,
            Codes::ExpGolomb(3) => Self::EXP_GOLOMB3,
            Codes::ExpGolomb(4) => Self::EXP_GOLOMB4,
            Codes::ExpGolomb(5) => Self::EXP_GOLOMB5,
            Codes::ExpGolomb(6) => Self::EXP_GOLOMB6,
            Codes::ExpGolomb(7) => Self::EXP_GOLOMB7,
            Codes::ExpGolomb(8) => Self::EXP_GOLOMB8,
            Codes::ExpGolomb(9) => Self::EXP_GOLOMB9,
            Codes::ExpGolomb(10) => Self::EXP_GOLOMB10,
            _ => return Err(DispatchError::UnsupportedCode(code)),
        };
        Ok(Self(write_func))
    }

    /// Returns a new [`FuncCodeWriter`] for the given function.
    #[inline(always)]
    pub fn new_with_func(write_func: WriteFn<E, CW>) -> Self {
        Self(write_func)
    }

    /// Returns the function pointer for the code.
    #[must_use]
    #[inline(always)]
    pub fn get_func(&self) -> WriteFn<E, CW> {
        self.0
    }
}

impl<E: Endianness, CW: CodesWrite<E> + ?Sized> StaticCodeWrite<E, CW> for FuncCodeWriter<E, CW> {
    #[inline(always)]
    fn write(&self, writer: &mut CW, n: u64) -> Result<usize, CW::Error> {
        (self.0)(writer, n)
    }
}

type LenFn = fn(u64) -> usize;

/// A newtype containing a function pointer dispatching the length method for a
/// code.
///
/// This is a more efficient way to pass a [`CodeLen`] to a method, as
/// a [`FuncCodeLen`] does not need to do a runtime test to dispatch the correct
/// method.
///
/// Instances can be obtained by calling the [`new`](FuncCodeLen::new) method
/// with a variant of the [`Codes`] enum, or by calling the
/// [`new_with_func`](FuncCodeLen::new_with_func) method with a function pointer.
///
/// Note that since selection of the code happens in the [`new`](FuncCodeLen::new)
/// method, it is more efficient to clone a [`FuncCodeLen`] than to create a new one.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
#[cfg_attr(feature = "mem_dbg", mem_size_flat)]
pub struct FuncCodeLen(LenFn);

impl FuncCodeLen {
    const UNARY: LenFn = |n| n as usize + 1;
    const GAMMA: LenFn = |n| len_gamma(n);
    const DELTA: LenFn = |n| len_delta(n);
    const OMEGA: LenFn = |n| len_omega(n);
    const VBYTE_BE: LenFn = |n| bit_len_vbyte(n);
    const VBYTE_LE: LenFn = |n| bit_len_vbyte(n);
    const ZETA2: LenFn = |n| len_zeta(n, 2);
    const ZETA3: LenFn = |n| len_zeta(n, 3);
    const ZETA4: LenFn = |n| len_zeta(n, 4);
    const ZETA5: LenFn = |n| len_zeta(n, 5);
    const ZETA6: LenFn = |n| len_zeta(n, 6);
    const ZETA7: LenFn = |n| len_zeta(n, 7);
    const ZETA8: LenFn = |n| len_zeta(n, 8);
    const ZETA9: LenFn = |n| len_zeta(n, 9);
    const ZETA10: LenFn = |n| len_zeta(n, 10);
    const RICE1: LenFn = |n| len_rice(n, 1);
    const RICE2: LenFn = |n| len_rice(n, 2);
    const RICE3: LenFn = |n| len_rice(n, 3);
    const RICE4: LenFn = |n| len_rice(n, 4);
    const RICE5: LenFn = |n| len_rice(n, 5);
    const RICE6: LenFn = |n| len_rice(n, 6);
    const RICE7: LenFn = |n| len_rice(n, 7);
    const RICE8: LenFn = |n| len_rice(n, 8);
    const RICE9: LenFn = |n| len_rice(n, 9);
    const RICE10: LenFn = |n| len_rice(n, 10);
    const PI1: LenFn = |n| len_pi(n, 1);
    const PI2: LenFn = |n| len_pi(n, 2);
    const PI3: LenFn = |n| len_pi(n, 3);
    const PI4: LenFn = |n| len_pi(n, 4);
    const PI5: LenFn = |n| len_pi(n, 5);
    const PI6: LenFn = |n| len_pi(n, 6);
    const PI7: LenFn = |n| len_pi(n, 7);
    const PI8: LenFn = |n| len_pi(n, 8);
    const PI9: LenFn = |n| len_pi(n, 9);
    const PI10: LenFn = |n| len_pi(n, 10);
    const GOLOMB3: LenFn = |n| len_golomb(n, 3);
    const GOLOMB5: LenFn = |n| len_golomb(n, 5);
    const GOLOMB6: LenFn = |n| len_golomb(n, 6);
    const GOLOMB7: LenFn = |n| len_golomb(n, 7);
    const GOLOMB9: LenFn = |n| len_golomb(n, 9);
    const GOLOMB10: LenFn = |n| len_golomb(n, 10);
    const EXP_GOLOMB1: LenFn = |n| len_exp_golomb(n, 1);
    const EXP_GOLOMB2: LenFn = |n| len_exp_golomb(n, 2);
    const EXP_GOLOMB3: LenFn = |n| len_exp_golomb(n, 3);
    const EXP_GOLOMB4: LenFn = |n| len_exp_golomb(n, 4);
    const EXP_GOLOMB5: LenFn = |n| len_exp_golomb(n, 5);
    const EXP_GOLOMB6: LenFn = |n| len_exp_golomb(n, 6);
    const EXP_GOLOMB7: LenFn = |n| len_exp_golomb(n, 7);
    const EXP_GOLOMB8: LenFn = |n| len_exp_golomb(n, 8);
    const EXP_GOLOMB9: LenFn = |n| len_exp_golomb(n, 9);
    const EXP_GOLOMB10: LenFn = |n| len_exp_golomb(n, 10);

    /// Returns a new [`FuncCodeLen`] for the given code.
    ///
    /// The code is [canonicalized](Codes::canonicalize) before
    /// the lookup, so equivalent codes yield the same length
    /// function.
    ///
    /// # Errors
    ///
    /// The method will return an error if there is no constant
    /// for the given code in [`FuncCodeLen`].
    pub const fn new(code: Codes) -> Result<Self, DispatchError> {
        let code = code.canonicalize();
        let len_func = match code {
            Codes::Unary => Self::UNARY,
            Codes::Gamma => Self::GAMMA,
            Codes::Delta => Self::DELTA,
            Codes::Omega => Self::OMEGA,
            Codes::VByteBe => Self::VBYTE_BE,
            Codes::VByteLe => Self::VBYTE_LE,
            Codes::Zeta(2) => Self::ZETA2,
            Codes::Zeta(3) => Self::ZETA3,
            Codes::Zeta(4) => Self::ZETA4,
            Codes::Zeta(5) => Self::ZETA5,
            Codes::Zeta(6) => Self::ZETA6,
            Codes::Zeta(7) => Self::ZETA7,
            Codes::Zeta(8) => Self::ZETA8,
            Codes::Zeta(9) => Self::ZETA9,
            Codes::Zeta(10) => Self::ZETA10,
            Codes::Rice(1) => Self::RICE1,
            Codes::Rice(2) => Self::RICE2,
            Codes::Rice(3) => Self::RICE3,
            Codes::Rice(4) => Self::RICE4,
            Codes::Rice(5) => Self::RICE5,
            Codes::Rice(6) => Self::RICE6,
            Codes::Rice(7) => Self::RICE7,
            Codes::Rice(8) => Self::RICE8,
            Codes::Rice(9) => Self::RICE9,
            Codes::Rice(10) => Self::RICE10,
            Codes::Pi(1) => Self::PI1,
            Codes::Pi(2) => Self::PI2,
            Codes::Pi(3) => Self::PI3,
            Codes::Pi(4) => Self::PI4,
            Codes::Pi(5) => Self::PI5,
            Codes::Pi(6) => Self::PI6,
            Codes::Pi(7) => Self::PI7,
            Codes::Pi(8) => Self::PI8,
            Codes::Pi(9) => Self::PI9,
            Codes::Pi(10) => Self::PI10,
            Codes::Golomb(3) => Self::GOLOMB3,
            Codes::Golomb(5) => Self::GOLOMB5,
            Codes::Golomb(6) => Self::GOLOMB6,
            Codes::Golomb(7) => Self::GOLOMB7,
            Codes::Golomb(9) => Self::GOLOMB9,
            Codes::Golomb(10) => Self::GOLOMB10,
            Codes::ExpGolomb(1) => Self::EXP_GOLOMB1,
            Codes::ExpGolomb(2) => Self::EXP_GOLOMB2,
            Codes::ExpGolomb(3) => Self::EXP_GOLOMB3,
            Codes::ExpGolomb(4) => Self::EXP_GOLOMB4,
            Codes::ExpGolomb(5) => Self::EXP_GOLOMB5,
            Codes::ExpGolomb(6) => Self::EXP_GOLOMB6,
            Codes::ExpGolomb(7) => Self::EXP_GOLOMB7,
            Codes::ExpGolomb(8) => Self::EXP_GOLOMB8,
            Codes::ExpGolomb(9) => Self::EXP_GOLOMB9,
            Codes::ExpGolomb(10) => Self::EXP_GOLOMB10,
            _ => return Err(DispatchError::UnsupportedCode(code)),
        };
        Ok(Self(len_func))
    }

    /// Returns a new [`FuncCodeLen`] for the given function.
    #[inline(always)]
    pub fn new_with_func(len_func: LenFn) -> Self {
        Self(len_func)
    }
    /// Returns the function pointer for the code.
    #[must_use]
    #[inline(always)]
    pub fn get_func(&self) -> LenFn {
        self.0
    }
}

/// Here we do not depend on the bitstream, so there is no need for a "static"
/// version of the trait.
impl CodeLen for FuncCodeLen {
    #[inline(always)]
    fn len(&self, n: u64) -> usize {
        (self.0)(n)
    }
}

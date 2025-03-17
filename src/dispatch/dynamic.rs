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
//! time. The code is stored in a function pointer, so it cannot be inlined like
//! in the [static case](crate::dispatch::static), but the approach is more
//! flexible.

use super::*;
#[cfg(feature = "mem_dbg")]
use mem_dbg::{MemDbg, MemSize};

type ReadFn<E, CR> = fn(&mut CR) -> Result<u64, <CR as BitRead<E>>::Error>;

/// A newtype containing a [function pointer](ReadFn) dispatching the read
/// method for a code.
///
/// This is a more efficient way to pass a [`StaticCodeRead`] to a method, as a
/// [`FuncCodeReader`] does not need to do a runtime test to dispatch the
/// correct code.
///
/// Instances can be obtained by calling the [`new`](FuncCodeReader::new) method
///  with method with a variant of the [`Codes`] enum, or by calling the
/// [`new_with_func`](FuncCodeReader::new_with_func) method with a function
/// pointer.
///
/// Note that since selection of the code happens in the
/// [`new`](FuncCodeReader::new) method, it is more efficient to clone a
/// [`FuncCodeReader`] than to create a new one.
#[derive(Debug, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
pub struct FuncCodeReader<E: Endianness, CR: CodesRead<E> + ?Sized>(ReadFn<E, CR>);

/// manually implement Clone to avoid the Clone bound on CR and E
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
    const PI2: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi(2);
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
    /// Return a new [`FuncCodeReader`] for the given code.
    ///
    /// # Errors
    ///
    /// The method will return an error if there is no constant
    /// for the given code in [`FuncCodeReader`].
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

    /// Return a new [`FuncCodeReader`] for the given function.
    #[inline(always)]
    pub fn new_with_func(read_func: ReadFn<E, CR>) -> Self {
        Self(read_func)
    }

    /// Get the function pointer for the code.
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
///  with method with a variant of the [`Codes`] enum, or by calling the
/// [`new_with_func`](FuncCodeWriter::new_with_func) method with a function
/// pointer.
///
/// Note that since selection of the code happens in the
/// [`new`](FuncCodeReader::new) method, it is more efficient to clone a
/// [`FuncCodeWriter`] than to create a new one.
#[derive(Debug, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
pub struct FuncCodeWriter<E: Endianness, CW: CodesWrite<E> + ?Sized>(WriteFn<E, CW>);

/// manually implement Clone to avoid the Clone bound on CR and E
impl<E: Endianness, CR: CodesWrite<E> + ?Sized> Clone for FuncCodeWriter<E, CR> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<E: Endianness, CW: CodesWrite<E> + ?Sized> FuncCodeWriter<E, CW> {
    const UNARY: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_unary(value);
    const GAMMA: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_gamma(value);
    const DELTA: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_delta(value);
    const OMEGA: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_omega(value);
    const VBYTE_BE: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_vbyte_be(value);
    const VBYTE_LE: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_vbyte_le(value);
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
    const PI1: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi(value, 1);
    const PI2: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi(value, 2);
    const PI3: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi(value, 3);
    const PI4: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi(value, 4);
    const PI5: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi(value, 5);
    const PI6: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi(value, 6);
    const PI7: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi(value, 7);
    const PI8: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi(value, 8);
    const PI9: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi(value, 9);
    const PI10: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi(value, 10);
    const GOLOMB3: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_golomb(value, 3);
    const GOLOMB5: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_golomb(value, 5);
    const GOLOMB6: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_golomb(value, 6);
    const GOLOMB7: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_golomb(value, 7);
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

    /// Return a new [`FuncCodeWriter`] for the given code.
    ///
    /// # Errors
    ///
    /// The method will return an error if there is no constant
    /// for the given code in [`FuncCodeWriter`].
    pub fn new(code: Codes) -> anyhow::Result<Self> {
        let write_func = match code {
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
            _ => anyhow::bail!("Unsupported write dispatch for code {:?}", code),
        };
        Ok(Self(write_func))
    }

    /// Return a new [`FuncCodeWriter`] for the given function.
    #[inline(always)]
    pub fn new_with_func(write_func: WriteFn<E, CW>) -> Self {
        Self(write_func)
    }

    /// Get the function pointer for the code.
    #[inline(always)]
    pub fn get_func(&self) -> WriteFn<E, CW> {
        self.0
    }
}

impl<E: Endianness, CW: CodesWrite<E> + ?Sized> StaticCodeWrite<E, CW> for FuncCodeWriter<E, CW> {
    #[inline(always)]
    fn write(&self, writer: &mut CW, value: u64) -> Result<usize, CW::Error> {
        (self.0)(writer, value)
    }
}

type LenFn = fn(u64) -> usize;

/// A newtype containing a function pointer dispatching the read method for a
/// code.
///
/// This is a more efficient way to pass a [`StaticCodeRead`] to a method, as
/// a [`FuncCodeReader`] does not need to do a runtime test to dispatch the correct
/// code.
///
/// Instances can be obtained by calling the [`new`](FuncCodeReader::new) method with
///  method with a variant of the [`Codes`] enum, or by calling the
/// [`new_with_func`](FuncCodeReader::new_with_func) method with a function pointer.
///
/// Note that since selection of the code happens in the [`new`](FuncCodeReader::new)
/// method, it is more efficient to clone a [`FuncCodeReader`] than to create a new one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
pub struct FuncCodeLen(LenFn);

impl FuncCodeLen {
    const UNARY: LenFn = |value| value as usize + 1;
    const GAMMA: LenFn = |value| len_gamma(value);
    const DELTA: LenFn = |value| len_delta(value);
    const OMEGA: LenFn = |value| len_omega(value);
    const VBYTE_BE: LenFn = |value| bit_len_vbyte(value);
    const VBYTE_LE: LenFn = |value| bit_len_vbyte(value);
    const ZETA2: LenFn = |value| len_zeta(value, 2);
    const ZETA3: LenFn = |value| len_zeta(value, 3);
    const ZETA4: LenFn = |value| len_zeta(value, 4);
    const ZETA5: LenFn = |value| len_zeta(value, 5);
    const ZETA6: LenFn = |value| len_zeta(value, 6);
    const ZETA7: LenFn = |value| len_zeta(value, 7);
    const ZETA8: LenFn = |value| len_zeta(value, 8);
    const ZETA9: LenFn = |value| len_zeta(value, 9);
    const ZETA10: LenFn = |value| len_zeta(value, 10);
    const RICE1: LenFn = |value| len_rice(value, 1);
    const RICE2: LenFn = |value| len_rice(value, 2);
    const RICE3: LenFn = |value| len_rice(value, 3);
    const RICE4: LenFn = |value| len_rice(value, 4);
    const RICE5: LenFn = |value| len_rice(value, 5);
    const RICE6: LenFn = |value| len_rice(value, 6);
    const RICE7: LenFn = |value| len_rice(value, 7);
    const RICE8: LenFn = |value| len_rice(value, 8);
    const RICE9: LenFn = |value| len_rice(value, 9);
    const RICE10: LenFn = |value| len_rice(value, 10);
    const PI1: LenFn = |value| len_pi(value, 1);
    const PI2: LenFn = |value| len_pi(value, 2);
    const PI3: LenFn = |value| len_pi(value, 3);
    const PI4: LenFn = |value| len_pi(value, 4);
    const PI5: LenFn = |value| len_pi(value, 5);
    const PI6: LenFn = |value| len_pi(value, 6);
    const PI7: LenFn = |value| len_pi(value, 7);
    const PI8: LenFn = |value| len_pi(value, 8);
    const PI9: LenFn = |value| len_pi(value, 9);
    const PI10: LenFn = |value| len_pi(value, 10);
    const GOLOMB3: LenFn = |value| len_golomb(value, 3);
    const GOLOMB5: LenFn = |value| len_golomb(value, 5);
    const GOLOMB6: LenFn = |value| len_golomb(value, 6);
    const GOLOMB7: LenFn = |value| len_golomb(value, 7);
    const GOLOMB9: LenFn = |value| len_golomb(value, 9);
    const GOLOMB10: LenFn = |value| len_golomb(value, 10);
    const EXP_GOLOMB1: LenFn = |value| len_exp_golomb(value, 1);
    const EXP_GOLOMB2: LenFn = |value| len_exp_golomb(value, 2);
    const EXP_GOLOMB3: LenFn = |value| len_exp_golomb(value, 3);
    const EXP_GOLOMB4: LenFn = |value| len_exp_golomb(value, 4);
    const EXP_GOLOMB5: LenFn = |value| len_exp_golomb(value, 5);
    const EXP_GOLOMB6: LenFn = |value| len_exp_golomb(value, 6);
    const EXP_GOLOMB7: LenFn = |value| len_exp_golomb(value, 7);
    const EXP_GOLOMB8: LenFn = |value| len_exp_golomb(value, 8);
    const EXP_GOLOMB9: LenFn = |value| len_exp_golomb(value, 9);
    const EXP_GOLOMB10: LenFn = |value| len_exp_golomb(value, 10);
    /// Return a new [`FuncCodeLen`] for the given code.
    ///
    /// # Errors
    ///
    /// The method will return an error if there is no constant
    /// for the given code in [`FuncCodeLen`].
    pub fn new(code: Codes) -> anyhow::Result<Self> {
        let len_func = match code {
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
        Ok(Self(len_func))
    }

    /// Return a new [`FuncCodeReader`] for the given function.
    #[inline(always)]
    pub fn new_with_func(len_func: LenFn) -> Self {
        Self(len_func)
    }
    /// Get the function pointer for the code.
    #[inline(always)]
    pub fn get_func(&self) -> LenFn {
        self.0
    }
}

/// Here we do not depend on the bitstream, so there is no need for a "static"
/// version of the trait.
impl CodeLen for FuncCodeLen {
    #[inline(always)]
    fn len(&self, value: u64) -> usize {
        (self.0)(value)
    }
}

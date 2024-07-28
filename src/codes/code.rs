/*
 * SPDX-FileCopyrightText: 2024 Tommaso Fontana
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! A module for selecting the code to use either dynamically or at compile time.
//!
//! - [`Code`] is an enum with all the supported codes and can be used
//!   to test different codes in a generic way.
//! - [`ConstCode`] is a zero-sized struct with a const generic parameter that can be used
//!   to select the code at compile time.
//!
//! [`CodeRead`], [`CodeWrite`] and [`CodeLen`] are traits that both
//! [`Code`] and [`ConstCode`] implement to allow for generic code selection
//! over a generic bitstream.
//!
//! If you need to read or write a code multiple times on the same type of bitstream,
//! you can use [`CodeReadDispatch`] and [`CodeWriteDispatch`] which are
//! specialized versions of [`CodeRead`] and [`CodeWrite`] that are implemented
//! by [`Code`], [`ConstCode`] and [`CodeReadDispatcher`], [`CodeWriteDispatcher`]
//! which are more efficient for multiple reads and writes because they do
//! not need to do dynamic dispatch.
//!
//! [`CodeStatsWrapper`] is a struct that can wrap any struct implementing
//! [`CodeRead`] or [`CodeWrite`], and keep track of the space it would need to
//! store the same sequence using different codes. This can be used as a
//! transparent wrapper to figure out which code is the best for a given sequence.
//!

use super::*;
use core::error::Error;
use core::marker::PhantomData;

/// Something that can decode a value form any bitstream.
pub trait CodeRead {
    type Error<CRE>: Error + Send + Sync + 'static
    where
        CRE: Error + Send + Sync + 'static;
    /// Read a value
    fn read<E: Endianness, CR: ReadCodes<E> + ?Sized>(
        &self,
        reader: &mut CR,
    ) -> Result<u64, Self::Error<CR::Error>>;
}

/// Like [`CodeRead`] but with a specific endianness and reader.
/// This more specialized version allows also to do single static dispatch
/// of the read method for a code.
pub trait CodeReadDispatch<E, CR>
where
    E: Endianness,
    CR: ReadCodes<E> + ?Sized,
{
    type Error<CRE>: Error + Send + Sync + 'static
    where
        CRE: Error + Send + Sync + 'static;
    fn read_dispatch(&self, reader: &mut CR) -> Result<u64, Self::Error<CR::Error>>;
}

/// Something that can encode a value to any bitstream.
pub trait CodeWrite {
    type Error<CWE>: Error + Send + Sync + 'static
    where
        CWE: Error + Send + Sync + 'static;
    /// Write a value
    fn write<E: Endianness, CW: WriteCodes<E> + ?Sized>(
        &self,
        writer: &mut CW,
        value: u64,
    ) -> Result<usize, Self::Error<CW::Error>>;
}

/// Like [`CodeWrite`] but with a specific endianness and writer.
/// This more specialized version allows also to do single static dispatch
/// of the write method for a code.
pub trait CodeWriteDispatch<E, CW>
where
    E: Endianness,
    CW: WriteCodes<E> + ?Sized,
{
    type Error<CWE>: Error + Send + Sync + 'static
    where
        CWE: Error + Send + Sync + 'static;
    fn write_dispatch(&self, writer: &mut CW, value: u64) -> Result<usize, Self::Error<CW::Error>>;
}

/// Something that can compute the length of a value encoded with a code.
pub trait CodeLen {
    /// Compute how many bits it takes to encode a value with this code.
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
#[non_exhaustive]
/// An enum of all the codes supported by this library, with their parameters.
/// This can be used to test different codes in a generic way.
pub enum Code {
    Unary,
    Gamma,
    Delta,
    Omega,
    VByte,
    Zeta { k: usize },
    Pi { k: usize },
    PiWeb { k: usize },
    Golomb { b: usize },
    ExpGolomb { k: usize },
    Rice { log2_b: usize },
}

impl CodeRead for Code {
    type Error<CRE> = CRE
    where
        CRE: Error + Send + Sync + 'static;
    #[inline]
    fn read<E: Endianness, CR: ReadCodes<E> + ?Sized>(
        &self,
        reader: &mut CR,
    ) -> Result<u64, Self::Error<CR::Error>> {
        Ok(match self {
            Code::Unary => reader.read_unary()?,
            Code::Gamma => reader.read_gamma()?,
            Code::Delta => reader.read_delta()?,
            Code::Omega => reader.read_omega()?,
            Code::VByte => reader.read_vbyte()?,
            Code::Zeta { k: 3 } => reader.read_zeta3()?,
            Code::Zeta { k } => reader.read_zeta(*k as u64)?,
            Code::Pi { k } => reader.read_pi(*k as u64)?,
            Code::PiWeb { k } => reader.read_pi_web(*k as u64)?,
            Code::Golomb { b } => reader.read_golomb(*b as u64)?,
            Code::ExpGolomb { k } => reader.read_exp_golomb(*k)?,
            Code::Rice { log2_b } => reader.read_rice(*log2_b)?,
        })
    }
}

impl<E, CR> CodeReadDispatch<E, CR> for Code
where
    E: Endianness,
    CR: ReadCodes<E> + ?Sized,
{
    type Error<CRE> = CRE
    where
        CRE: Error + Send + Sync + 'static;
    #[inline(always)]
    fn read_dispatch(&self, reader: &mut CR) -> Result<u64, Self::Error<CR::Error>> {
        <Self as CodeRead>::read(self, reader)
    }
}

impl CodeWrite for Code {
    type Error<CWE> = CWE
    where
        CWE: Error + Send + Sync + 'static;
    #[inline]
    fn write<E: Endianness, CW: WriteCodes<E> + ?Sized>(
        &self,
        writer: &mut CW,
        value: u64,
    ) -> Result<usize, Self::Error<CW::Error>> {
        Ok(match self {
            Code::Unary => writer.write_unary(value)?,
            Code::Gamma => writer.write_gamma(value)?,
            Code::Delta => writer.write_delta(value)?,
            Code::Omega => writer.write_omega(value)?,
            Code::VByte => writer.write_vbyte(value)?,
            Code::Zeta { k: 3 } => writer.write_zeta3(value)?,
            Code::Zeta { k } => writer.write_zeta(value, *k as u64)?,
            Code::Pi { k } => writer.write_pi(value, *k as u64)?,
            Code::PiWeb { k } => writer.write_pi_web(value, *k as u64)?,
            Code::Golomb { b } => writer.write_golomb(value, *b as u64)?,
            Code::ExpGolomb { k } => writer.write_exp_golomb(value, *k)?,
            Code::Rice { log2_b } => writer.write_rice(value, *log2_b)?,
        })
    }
}

impl<E, CW> CodeWriteDispatch<E, CW> for Code
where
    E: Endianness,
    CW: WriteCodes<E> + ?Sized,
{
    type Error<CWE> = CWE
    where
        CWE: Error + Send + Sync + 'static;
    #[inline(always)]
    fn write_dispatch(&self, writer: &mut CW, value: u64) -> Result<usize, Self::Error<CW::Error>> {
        <Self as CodeWrite>::write(self, writer, value)
    }
}

impl CodeLen for Code {
    #[inline]
    fn len(&self, value: u64) -> usize {
        match self {
            Code::Unary => value as usize + 1,
            Code::Gamma => len_gamma(value),
            Code::Delta => len_delta(value),
            Code::Omega => len_omega(value),
            Code::VByte => len_vbyte(value),
            Code::Zeta { k } => len_zeta(value, *k as u64),
            Code::Pi { k } => len_pi(value, *k as u64),
            Code::PiWeb { k } => len_pi_web(value, *k as u64),
            Code::Golomb { b } => len_golomb(value, *b as u64),
            Code::ExpGolomb { k } => len_exp_golomb(value, *k),
            Code::Rice { log2_b } => len_rice(value, *log2_b),
        }
    }
}

impl core::fmt::Display for Code {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Code::Unary => write!(f, "Unary"),
            Code::Gamma => write!(f, "Gamma"),
            Code::Delta => write!(f, "Delta"),
            Code::Omega => write!(f, "Omega"),
            Code::VByte => write!(f, "VByte"),
            Code::Zeta { k } => write!(f, "Zeta({})", k),
            Code::Pi { k } => write!(f, "Pi({})", k),
            Code::PiWeb { k } => write!(f, "PiWeb({})", k),
            Code::Golomb { b } => write!(f, "Golomb({})", b),
            Code::ExpGolomb { k } => write!(f, "ExpGolomb({})", k),
            Code::Rice { log2_b } => write!(f, "Rice({})", log2_b),
        }
    }
}

impl std::str::FromStr for Code {
    type Err = CodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Unary" => Ok(Code::Unary),
            "Gamma" => Ok(Code::Gamma),
            "Delta" => Ok(Code::Delta),
            "Omega" => Ok(Code::Omega),
            "VByte" => Ok(Code::VByte),
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
                    "Zeta" => Ok(Code::Zeta { k: k.parse()? }),
                    "Pi" => Ok(Code::Pi { k: k.parse()? }),
                    "PiWeb" => Ok(Code::PiWeb { k: k.parse()? }),
                    "Golomb" => Ok(Code::Golomb { b: k.parse()? }),
                    "ExpGolomb" => Ok(Code::ExpGolomb { k: k.parse()? }),
                    "Rice" => Ok(Code::Rice { log2_b: k.parse()? }),
                    _ => Err(CodeError::UnknownCode(format!("Could not parse {}", name))),
                }
            }
        }
    }
}

type ReadFn<E, CR> = fn(&mut CR) -> Result<u64, <CR as BitRead<E>>::Error>;

/// Single static dispatch of the read method for a code.
/// This is a more efficient way to read codes that are initialized once dynamically,
/// and then used multiple times.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
pub struct CodeReadDispatcher<E, CR>
where
    E: Endianness,
    CR: ReadCodes<E> + ?Sized,
{
    read: ReadFn<E, CR>,
    _marker: PhantomData<E>,
}

impl<E, CR> CodeReadDispatcher<E, CR>
where
    E: Endianness,
    CR: ReadCodes<E> + ?Sized,
{
    pub const UNARY: ReadFn<E, CR> = |reader: &mut CR| reader.read_unary();
    pub const GAMMA: ReadFn<E, CR> = |reader: &mut CR| reader.read_gamma();
    pub const DELTA: ReadFn<E, CR> = |reader: &mut CR| reader.read_delta();
    pub const OMEGA: ReadFn<E, CR> = |reader: &mut CR| reader.read_omega();
    pub const VBYTE: ReadFn<E, CR> = |reader: &mut CR| reader.read_vbyte();
    pub const ZETA2: ReadFn<E, CR> = |reader: &mut CR| reader.read_zeta(2);
    pub const ZETA3: ReadFn<E, CR> = |reader: &mut CR| reader.read_zeta3();
    pub const ZETA4: ReadFn<E, CR> = |reader: &mut CR| reader.read_zeta(4);
    pub const ZETA5: ReadFn<E, CR> = |reader: &mut CR| reader.read_zeta(5);
    pub const ZETA6: ReadFn<E, CR> = |reader: &mut CR| reader.read_zeta(6);
    pub const ZETA7: ReadFn<E, CR> = |reader: &mut CR| reader.read_zeta(7);
    pub const ZETA8: ReadFn<E, CR> = |reader: &mut CR| reader.read_zeta(8);
    pub const ZETA9: ReadFn<E, CR> = |reader: &mut CR| reader.read_zeta(9);
    pub const ZETA10: ReadFn<E, CR> = |reader: &mut CR| reader.read_zeta(10);
    pub const PI2: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi(2);
    pub const PI3: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi(3);
    pub const PI4: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi(4);
    pub const PI5: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi(5);
    pub const PI6: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi(6);
    pub const PI7: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi(7);
    pub const PI8: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi(8);
    pub const PI9: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi(9);
    pub const PI10: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi(10);
    pub const PI_WEB2: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi_web(2);
    pub const PI_WEB3: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi_web(3);
    pub const PI_WEB4: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi_web(4);
    pub const PI_WEB5: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi_web(5);
    pub const PI_WEB6: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi_web(6);
    pub const PI_WEB7: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi_web(7);
    pub const PI_WEB8: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi_web(8);
    pub const PI_WEB9: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi_web(9);
    pub const PI_WEB10: ReadFn<E, CR> = |reader: &mut CR| reader.read_pi_web(10);
    pub const GOLOMB2: ReadFn<E, CR> = |reader: &mut CR| reader.read_golomb(2);
    pub const GOLOMB3: ReadFn<E, CR> = |reader: &mut CR| reader.read_golomb(3);
    pub const GOLOMB4: ReadFn<E, CR> = |reader: &mut CR| reader.read_golomb(4);
    pub const GOLOMB5: ReadFn<E, CR> = |reader: &mut CR| reader.read_golomb(5);
    pub const GOLOMB6: ReadFn<E, CR> = |reader: &mut CR| reader.read_golomb(6);
    pub const GOLOMB7: ReadFn<E, CR> = |reader: &mut CR| reader.read_golomb(7);
    pub const GOLOMB8: ReadFn<E, CR> = |reader: &mut CR| reader.read_golomb(8);
    pub const GOLOMB9: ReadFn<E, CR> = |reader: &mut CR| reader.read_golomb(9);
    pub const GOLOMB10: ReadFn<E, CR> = |reader: &mut CR| reader.read_golomb(10);
    pub const EXP_GOLOMB2: ReadFn<E, CR> = |reader: &mut CR| reader.read_exp_golomb(2);
    pub const EXP_GOLOMB3: ReadFn<E, CR> = |reader: &mut CR| reader.read_exp_golomb(3);
    pub const EXP_GOLOMB4: ReadFn<E, CR> = |reader: &mut CR| reader.read_exp_golomb(4);
    pub const EXP_GOLOMB5: ReadFn<E, CR> = |reader: &mut CR| reader.read_exp_golomb(5);
    pub const EXP_GOLOMB6: ReadFn<E, CR> = |reader: &mut CR| reader.read_exp_golomb(6);
    pub const EXP_GOLOMB7: ReadFn<E, CR> = |reader: &mut CR| reader.read_exp_golomb(7);
    pub const EXP_GOLOMB8: ReadFn<E, CR> = |reader: &mut CR| reader.read_exp_golomb(8);
    pub const EXP_GOLOMB9: ReadFn<E, CR> = |reader: &mut CR| reader.read_exp_golomb(9);
    pub const EXP_GOLOMB10: ReadFn<E, CR> = |reader: &mut CR| reader.read_exp_golomb(10);
    pub const RICE2: ReadFn<E, CR> = |reader: &mut CR| reader.read_rice(2);
    pub const RICE3: ReadFn<E, CR> = |reader: &mut CR| reader.read_rice(3);
    pub const RICE4: ReadFn<E, CR> = |reader: &mut CR| reader.read_rice(4);
    pub const RICE5: ReadFn<E, CR> = |reader: &mut CR| reader.read_rice(5);
    pub const RICE6: ReadFn<E, CR> = |reader: &mut CR| reader.read_rice(6);
    pub const RICE7: ReadFn<E, CR> = |reader: &mut CR| reader.read_rice(7);
    pub const RICE8: ReadFn<E, CR> = |reader: &mut CR| reader.read_rice(8);
    pub const RICE9: ReadFn<E, CR> = |reader: &mut CR| reader.read_rice(9);
    pub const RICE10: ReadFn<E, CR> = |reader: &mut CR| reader.read_rice(10);

    /// Create a new dispatch for the given code.
    pub fn new(code: Code) -> Result<Self> {
        let read = match code {
            Code::Unary => Self::UNARY,
            Code::Gamma => Self::GAMMA,
            Code::Delta => Self::DELTA,
            Code::Omega => Self::OMEGA,
            Code::VByte => Self::VBYTE,
            Code::Zeta { k: 2 } => Self::ZETA2,
            Code::Zeta { k: 3 } => Self::ZETA3,
            Code::Zeta { k: 4 } => Self::ZETA4,
            Code::Zeta { k: 5 } => Self::ZETA5,
            Code::Zeta { k: 6 } => Self::ZETA6,
            Code::Zeta { k: 7 } => Self::ZETA7,
            Code::Zeta { k: 8 } => Self::ZETA8,
            Code::Zeta { k: 9 } => Self::ZETA9,
            Code::Zeta { k: 10 } => Self::ZETA10,
            Code::Pi { k: 2 } => Self::PI2,
            Code::Pi { k: 3 } => Self::PI3,
            Code::Pi { k: 4 } => Self::PI4,
            Code::Pi { k: 5 } => Self::PI5,
            Code::Pi { k: 6 } => Self::PI6,
            Code::Pi { k: 7 } => Self::PI7,
            Code::Pi { k: 8 } => Self::PI8,
            Code::Pi { k: 9 } => Self::PI9,
            Code::Pi { k: 10 } => Self::PI10,
            Code::PiWeb { k: 2 } => Self::PI_WEB2,
            Code::PiWeb { k: 3 } => Self::PI_WEB3,
            Code::PiWeb { k: 4 } => Self::PI_WEB4,
            Code::PiWeb { k: 5 } => Self::PI_WEB5,
            Code::PiWeb { k: 6 } => Self::PI_WEB6,
            Code::PiWeb { k: 7 } => Self::PI_WEB7,
            Code::PiWeb { k: 8 } => Self::PI_WEB8,
            Code::PiWeb { k: 9 } => Self::PI_WEB9,
            Code::PiWeb { k: 10 } => Self::PI_WEB10,
            Code::Golomb { b: 2 } => Self::GOLOMB2,
            Code::Golomb { b: 3 } => Self::GOLOMB3,
            Code::Golomb { b: 4 } => Self::GOLOMB4,
            Code::Golomb { b: 5 } => Self::GOLOMB5,
            Code::Golomb { b: 6 } => Self::GOLOMB6,
            Code::Golomb { b: 7 } => Self::GOLOMB7,
            Code::Golomb { b: 8 } => Self::GOLOMB8,
            Code::Golomb { b: 9 } => Self::GOLOMB9,
            Code::Golomb { b: 10 } => Self::GOLOMB10,
            Code::ExpGolomb { k: 2 } => Self::EXP_GOLOMB2,
            Code::ExpGolomb { k: 3 } => Self::EXP_GOLOMB3,
            Code::ExpGolomb { k: 4 } => Self::EXP_GOLOMB4,
            Code::ExpGolomb { k: 5 } => Self::EXP_GOLOMB5,
            Code::ExpGolomb { k: 6 } => Self::EXP_GOLOMB6,
            Code::ExpGolomb { k: 7 } => Self::EXP_GOLOMB7,
            Code::ExpGolomb { k: 8 } => Self::EXP_GOLOMB8,
            Code::ExpGolomb { k: 9 } => Self::EXP_GOLOMB9,
            Code::ExpGolomb { k: 10 } => Self::EXP_GOLOMB10,
            Code::Rice { log2_b: 2 } => Self::RICE2,
            Code::Rice { log2_b: 3 } => Self::RICE3,
            Code::Rice { log2_b: 4 } => Self::RICE4,
            Code::Rice { log2_b: 5 } => Self::RICE5,
            Code::Rice { log2_b: 6 } => Self::RICE6,
            Code::Rice { log2_b: 7 } => Self::RICE7,
            Code::Rice { log2_b: 8 } => Self::RICE8,
            Code::Rice { log2_b: 9 } => Self::RICE9,
            Code::Rice { log2_b: 10 } => Self::RICE10,
            _ => anyhow::bail!("Unsupported read dispatch for code {:?}", code),
        };
        Ok(Self {
            read,
            _marker: PhantomData,
        })
    }
}

impl<E, CR> CodeReadDispatch<E, CR> for CodeReadDispatcher<E, CR>
where
    E: Endianness,
    CR: ReadCodes<E> + ?Sized,
{
    type Error<CRE> = CRE
    where
        CRE: Error + Send + Sync + 'static;

    #[inline(always)]
    fn read_dispatch(&self, reader: &mut CR) -> Result<u64, Self::Error<CR::Error>> {
        (self.read)(reader).map_err(Into::into)
    }
}

type WriteFn<E, CW> = fn(&mut CW, u64) -> Result<usize, <CW as BitWrite<E>>::Error>;

/// Single static dispatch of the write method for a code.
/// This is a more efficient way to write codes that are initialized once dynamically,
/// and then used multiple times.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
pub struct CodeWriteDispatcher<E, CW>
where
    E: Endianness,
    CW: WriteCodes<E> + ?Sized,
{
    write: WriteFn<E, CW>,
    _marker: PhantomData<E>,
}

impl<E, CW> CodeWriteDispatcher<E, CW>
where
    E: Endianness,
    CW: WriteCodes<E> + ?Sized,
{
    pub const UNARY: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_unary(value);
    pub const GAMMA: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_gamma(value);
    pub const DELTA: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_delta(value);
    pub const OMEGA: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_omega(value);
    pub const VBYTE: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_vbyte(value);
    pub const ZETA2: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_zeta(value, 2);
    pub const ZETA3: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_zeta3(value);
    pub const ZETA4: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_zeta(value, 4);
    pub const ZETA5: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_zeta(value, 5);
    pub const ZETA6: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_zeta(value, 6);
    pub const ZETA7: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_zeta(value, 7);
    pub const ZETA8: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_zeta(value, 8);
    pub const ZETA9: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_zeta(value, 9);
    pub const ZETA10: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_zeta(value, 10);
    pub const PI2: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi(value, 2);
    pub const PI3: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi(value, 3);
    pub const PI4: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi(value, 4);
    pub const PI5: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi(value, 5);
    pub const PI6: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi(value, 6);
    pub const PI7: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi(value, 7);
    pub const PI8: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi(value, 8);
    pub const PI9: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi(value, 9);
    pub const PI10: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi(value, 10);
    pub const PI_WEB2: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi_web(value, 2);
    pub const PI_WEB3: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi_web(value, 3);
    pub const PI_WEB4: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi_web(value, 4);
    pub const PI_WEB5: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi_web(value, 5);
    pub const PI_WEB6: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi_web(value, 6);
    pub const PI_WEB7: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi_web(value, 7);
    pub const PI_WEB8: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi_web(value, 8);
    pub const PI_WEB9: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_pi_web(value, 9);
    pub const PI_WEB10: WriteFn<E, CW> =
        |writer: &mut CW, value: u64| writer.write_pi_web(value, 10);
    pub const GOLOMB2: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_golomb(value, 2);
    pub const GOLOMB3: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_golomb(value, 3);
    pub const GOLOMB4: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_golomb(value, 4);
    pub const GOLOMB5: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_golomb(value, 5);
    pub const GOLOMB6: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_golomb(value, 6);
    pub const GOLOMB7: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_golomb(value, 7);
    pub const GOLOMB8: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_golomb(value, 8);
    pub const GOLOMB9: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_golomb(value, 9);
    pub const GOLOMB10: WriteFn<E, CW> =
        |writer: &mut CW, value: u64| writer.write_golomb(value, 10);
    pub const EXP_GOLOMB2: WriteFn<E, CW> =
        |writer: &mut CW, value: u64| writer.write_exp_golomb(value, 2);
    pub const EXP_GOLOMB3: WriteFn<E, CW> =
        |writer: &mut CW, value: u64| writer.write_exp_golomb(value, 3);
    pub const EXP_GOLOMB4: WriteFn<E, CW> =
        |writer: &mut CW, value: u64| writer.write_exp_golomb(value, 4);
    pub const EXP_GOLOMB5: WriteFn<E, CW> =
        |writer: &mut CW, value: u64| writer.write_exp_golomb(value, 5);
    pub const EXP_GOLOMB6: WriteFn<E, CW> =
        |writer: &mut CW, value: u64| writer.write_exp_golomb(value, 6);
    pub const EXP_GOLOMB7: WriteFn<E, CW> =
        |writer: &mut CW, value: u64| writer.write_exp_golomb(value, 7);
    pub const EXP_GOLOMB8: WriteFn<E, CW> =
        |writer: &mut CW, value: u64| writer.write_exp_golomb(value, 8);
    pub const EXP_GOLOMB9: WriteFn<E, CW> =
        |writer: &mut CW, value: u64| writer.write_exp_golomb(value, 9);
    pub const EXP_GOLOMB10: WriteFn<E, CW> =
        |writer: &mut CW, value: u64| writer.write_exp_golomb(value, 10);
    pub const RICE2: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_rice(value, 2);
    pub const RICE3: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_rice(value, 3);
    pub const RICE4: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_rice(value, 4);
    pub const RICE5: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_rice(value, 5);
    pub const RICE6: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_rice(value, 6);
    pub const RICE7: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_rice(value, 7);
    pub const RICE8: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_rice(value, 8);
    pub const RICE9: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_rice(value, 9);
    pub const RICE10: WriteFn<E, CW> = |writer: &mut CW, value: u64| writer.write_rice(value, 10);

    pub fn new(code: Code) -> Result<Self> {
        let write = match code {
            Code::Unary => Self::UNARY,
            Code::Gamma => Self::GAMMA,
            Code::Delta => Self::DELTA,
            Code::Omega => Self::OMEGA,
            Code::VByte => Self::VBYTE,
            Code::Zeta { k: 2 } => Self::ZETA2,
            Code::Zeta { k: 3 } => Self::ZETA3,
            Code::Zeta { k: 4 } => Self::ZETA4,
            Code::Zeta { k: 5 } => Self::ZETA5,
            Code::Zeta { k: 6 } => Self::ZETA6,
            Code::Zeta { k: 7 } => Self::ZETA7,
            Code::Zeta { k: 8 } => Self::ZETA8,
            Code::Zeta { k: 9 } => Self::ZETA9,
            Code::Zeta { k: 10 } => Self::ZETA10,
            Code::Pi { k: 2 } => Self::PI2,
            Code::Pi { k: 3 } => Self::PI3,
            Code::Pi { k: 4 } => Self::PI4,
            Code::Pi { k: 5 } => Self::PI5,
            Code::Pi { k: 6 } => Self::PI6,
            Code::Pi { k: 7 } => Self::PI7,
            Code::Pi { k: 8 } => Self::PI8,
            Code::Pi { k: 9 } => Self::PI9,
            Code::Pi { k: 10 } => Self::PI10,
            Code::PiWeb { k: 2 } => Self::PI_WEB2,
            Code::PiWeb { k: 3 } => Self::PI_WEB3,
            Code::PiWeb { k: 4 } => Self::PI_WEB4,
            Code::PiWeb { k: 5 } => Self::PI_WEB5,
            Code::PiWeb { k: 6 } => Self::PI_WEB6,
            Code::PiWeb { k: 7 } => Self::PI_WEB7,
            Code::PiWeb { k: 8 } => Self::PI_WEB8,
            Code::PiWeb { k: 9 } => Self::PI_WEB9,
            Code::PiWeb { k: 10 } => Self::PI_WEB10,
            Code::Golomb { b: 2 } => Self::GOLOMB2,
            Code::Golomb { b: 3 } => Self::GOLOMB3,
            Code::Golomb { b: 4 } => Self::GOLOMB4,
            Code::Golomb { b: 5 } => Self::GOLOMB5,
            Code::Golomb { b: 6 } => Self::GOLOMB6,
            Code::Golomb { b: 7 } => Self::GOLOMB7,
            Code::Golomb { b: 8 } => Self::GOLOMB8,
            Code::Golomb { b: 9 } => Self::GOLOMB9,
            Code::Golomb { b: 10 } => Self::GOLOMB10,
            Code::ExpGolomb { k: 2 } => Self::EXP_GOLOMB2,
            Code::ExpGolomb { k: 3 } => Self::EXP_GOLOMB3,
            Code::ExpGolomb { k: 4 } => Self::EXP_GOLOMB4,
            Code::ExpGolomb { k: 5 } => Self::EXP_GOLOMB5,
            Code::ExpGolomb { k: 6 } => Self::EXP_GOLOMB6,
            Code::ExpGolomb { k: 7 } => Self::EXP_GOLOMB7,
            Code::ExpGolomb { k: 8 } => Self::EXP_GOLOMB8,
            Code::ExpGolomb { k: 9 } => Self::EXP_GOLOMB9,
            Code::ExpGolomb { k: 10 } => Self::EXP_GOLOMB10,
            Code::Rice { log2_b: 2 } => Self::RICE2,
            Code::Rice { log2_b: 3 } => Self::RICE3,
            Code::Rice { log2_b: 4 } => Self::RICE4,
            Code::Rice { log2_b: 5 } => Self::RICE5,
            Code::Rice { log2_b: 6 } => Self::RICE6,
            Code::Rice { log2_b: 7 } => Self::RICE7,
            Code::Rice { log2_b: 8 } => Self::RICE8,
            Code::Rice { log2_b: 9 } => Self::RICE9,
            Code::Rice { log2_b: 10 } => Self::RICE10,
            _ => anyhow::bail!("Unsupported write dispatch for code {:?}", code),
        };
        Ok(Self {
            write,
            _marker: PhantomData,
        })
    }
}

impl<E, CW> CodeWriteDispatch<E, CW> for CodeWriteDispatcher<E, CW>
where
    E: Endianness,
    CW: WriteCodes<E> + ?Sized,
{
    type Error<CWE> = CWE
    where
        CWE: Error + Send + Sync + 'static;
    #[inline(always)]
    fn write_dispatch(&self, writer: &mut CW, value: u64) -> Result<usize, Self::Error<CW::Error>> {
        (self.write)(writer, value).map_err(Into::into)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
/// A zero-sized struct with a const generic parameter that can be used to
/// select the code at compile time.
pub struct ConstCode<const CODE: usize>;

/// The constants ot use as generic parameter of the [`ConstCode`] struct.
pub mod const_codes {
    /// This macro just define a bunch of constants with progressive values.
    /// It is used to assign a unique integer to each code.
    macro_rules! impl_codes {
        ($code:ident, $($tail:ident),*) => {
            pub const $code: usize = 0 $( + 1 + (0 * $tail))*;
            impl_codes!($($tail),*);
        };
        ($code:ident) => {
            pub const $code: usize = 0;
        };
    }

    impl_codes!(
        ZETA1,
        ZETA2,
        ZETA3,
        ZETA4,
        ZETA5,
        ZETA6,
        ZETA7,
        ZETA8,
        ZETA9,
        ZETA10,
        PI2,
        PI3,
        PI4,
        PI5,
        PI6,
        PI7,
        PI8,
        PI9,
        PI10,
        PI_WEB2,
        PI_WEB3,
        PI_WEB4,
        PI_WEB5,
        PI_WEB6,
        PI_WEB7,
        PI_WEB8,
        PI_WEB9,
        PI_WEB10,
        GOLOMB2,
        GOLOMB3,
        GOLOMB4,
        GOLOMB5,
        GOLOMB6,
        GOLOMB7,
        GOLOMB8,
        GOLOMB9,
        GOLOMB10,
        EXP_GOLOMB2,
        EXP_GOLOMB3,
        EXP_GOLOMB4,
        EXP_GOLOMB5,
        EXP_GOLOMB6,
        EXP_GOLOMB7,
        EXP_GOLOMB8,
        EXP_GOLOMB9,
        EXP_GOLOMB10,
        RICE2,
        RICE3,
        RICE4,
        RICE5,
        RICE6,
        RICE7,
        RICE8,
        RICE9,
        RICE10,
        VBYTE,
        OMEGA,
        DELTA,
        GAMMA,
        UNARY
    );
}

impl<const CODE: usize> CodeRead for ConstCode<CODE> {
    type Error<CRE> = CRE
    where
        CRE: Error + Send + Sync + 'static;
    fn read<E: Endianness, CR: ReadCodes<E> + ?Sized>(
        &self,
        reader: &mut CR,
    ) -> Result<u64, Self::Error<CR::Error>> {
        match CODE {
            const_codes::UNARY => reader.read_unary(),
            const_codes::GAMMA => reader.read_gamma(),
            const_codes::DELTA => reader.read_delta(),
            const_codes::OMEGA => reader.read_omega(),
            const_codes::VBYTE => reader.read_vbyte(),
            const_codes::ZETA1 => reader.read_zeta(1),
            const_codes::ZETA2 => reader.read_zeta(2),
            const_codes::ZETA3 => reader.read_zeta3(),
            const_codes::ZETA4 => reader.read_zeta(4),
            const_codes::ZETA5 => reader.read_zeta(5),
            const_codes::ZETA6 => reader.read_zeta(6),
            const_codes::ZETA7 => reader.read_zeta(7),
            const_codes::ZETA8 => reader.read_zeta(8),
            const_codes::ZETA9 => reader.read_zeta(9),
            const_codes::ZETA10 => reader.read_zeta(10),
            const_codes::PI2 => reader.read_pi(2),
            const_codes::PI3 => reader.read_pi(3),
            const_codes::PI4 => reader.read_pi(4),
            const_codes::PI5 => reader.read_pi(5),
            const_codes::PI6 => reader.read_pi(6),
            const_codes::PI7 => reader.read_pi(7),
            const_codes::PI8 => reader.read_pi(8),
            const_codes::PI9 => reader.read_pi(9),
            const_codes::PI10 => reader.read_pi(10),
            const_codes::PI_WEB2 => reader.read_pi_web(2),
            const_codes::PI_WEB3 => reader.read_pi_web(3),
            const_codes::PI_WEB4 => reader.read_pi_web(4),
            const_codes::PI_WEB5 => reader.read_pi_web(5),
            const_codes::PI_WEB6 => reader.read_pi_web(6),
            const_codes::PI_WEB7 => reader.read_pi_web(7),
            const_codes::PI_WEB8 => reader.read_pi_web(8),
            const_codes::PI_WEB9 => reader.read_pi_web(9),
            const_codes::PI_WEB10 => reader.read_pi_web(10),
            const_codes::GOLOMB2 => reader.read_golomb(2),
            const_codes::GOLOMB3 => reader.read_golomb(3),
            const_codes::GOLOMB4 => reader.read_golomb(4),
            const_codes::GOLOMB5 => reader.read_golomb(5),
            const_codes::GOLOMB6 => reader.read_golomb(6),
            const_codes::GOLOMB7 => reader.read_golomb(7),
            const_codes::GOLOMB8 => reader.read_golomb(8),
            const_codes::GOLOMB9 => reader.read_golomb(9),
            const_codes::GOLOMB10 => reader.read_golomb(10),
            const_codes::EXP_GOLOMB2 => reader.read_exp_golomb(2),
            const_codes::EXP_GOLOMB3 => reader.read_exp_golomb(3),
            const_codes::EXP_GOLOMB4 => reader.read_exp_golomb(4),
            const_codes::EXP_GOLOMB5 => reader.read_exp_golomb(5),
            const_codes::EXP_GOLOMB6 => reader.read_exp_golomb(6),
            const_codes::EXP_GOLOMB7 => reader.read_exp_golomb(7),
            const_codes::EXP_GOLOMB8 => reader.read_exp_golomb(8),
            const_codes::EXP_GOLOMB9 => reader.read_exp_golomb(9),
            const_codes::EXP_GOLOMB10 => reader.read_exp_golomb(10),
            const_codes::RICE2 => reader.read_rice(2),
            const_codes::RICE3 => reader.read_rice(3),
            const_codes::RICE4 => reader.read_rice(4),
            const_codes::RICE5 => reader.read_rice(5),
            const_codes::RICE6 => reader.read_rice(6),
            const_codes::RICE7 => reader.read_rice(7),
            const_codes::RICE8 => reader.read_rice(8),
            const_codes::RICE9 => reader.read_rice(9),
            const_codes::RICE10 => reader.read_rice(10),
            _ => panic!("Unknown code: {}", CODE),
        }
    }
}

impl<E, CR, const CODE: usize> CodeReadDispatch<E, CR> for ConstCode<CODE>
where
    E: Endianness,
    CR: ReadCodes<E> + ?Sized,
{
    type Error<CRE> = CRE
    where
        CRE: Error + Send + Sync + 'static;
    #[inline(always)]
    fn read_dispatch(&self, reader: &mut CR) -> Result<u64, Self::Error<CR::Error>> {
        <Self as CodeRead>::read(self, reader)
    }
}

impl<const CODE: usize> CodeWrite for ConstCode<CODE> {
    type Error<CWE> = CWE
    where
        CWE: Error + Send + Sync + 'static;
    fn write<E: Endianness, CW: WriteCodes<E> + ?Sized>(
        &self,
        writer: &mut CW,
        value: u64,
    ) -> Result<usize, Self::Error<CW::Error>> {
        match CODE {
            const_codes::UNARY => writer.write_unary(value),
            const_codes::GAMMA => writer.write_gamma(value),
            const_codes::DELTA => writer.write_delta(value),
            const_codes::OMEGA => writer.write_omega(value),
            const_codes::VBYTE => writer.write_vbyte(value),
            const_codes::ZETA1 => writer.write_zeta(value, 1),
            const_codes::ZETA2 => writer.write_zeta(value, 2),
            const_codes::ZETA3 => writer.write_zeta3(value),
            const_codes::ZETA4 => writer.write_zeta(value, 4),
            const_codes::ZETA5 => writer.write_zeta(value, 5),
            const_codes::ZETA6 => writer.write_zeta(value, 6),
            const_codes::ZETA7 => writer.write_zeta(value, 7),
            const_codes::ZETA8 => writer.write_zeta(value, 8),
            const_codes::ZETA9 => writer.write_zeta(value, 9),
            const_codes::ZETA10 => writer.write_zeta(value, 10),
            const_codes::PI2 => writer.write_pi(value, 2),
            const_codes::PI3 => writer.write_pi(value, 3),
            const_codes::PI4 => writer.write_pi(value, 4),
            const_codes::PI5 => writer.write_pi(value, 5),
            const_codes::PI6 => writer.write_pi(value, 6),
            const_codes::PI7 => writer.write_pi(value, 7),
            const_codes::PI8 => writer.write_pi(value, 8),
            const_codes::PI9 => writer.write_pi(value, 9),
            const_codes::PI10 => writer.write_pi(value, 10),
            const_codes::PI_WEB2 => writer.write_pi_web(value, 2),
            const_codes::PI_WEB3 => writer.write_pi_web(value, 3),
            const_codes::PI_WEB4 => writer.write_pi_web(value, 4),
            const_codes::PI_WEB5 => writer.write_pi_web(value, 5),
            const_codes::PI_WEB6 => writer.write_pi_web(value, 6),
            const_codes::PI_WEB7 => writer.write_pi_web(value, 7),
            const_codes::PI_WEB8 => writer.write_pi_web(value, 8),
            const_codes::PI_WEB9 => writer.write_pi_web(value, 9),
            const_codes::PI_WEB10 => writer.write_pi_web(value, 10),
            const_codes::GOLOMB2 => writer.write_golomb(value, 2),
            const_codes::GOLOMB3 => writer.write_golomb(value, 3),
            const_codes::GOLOMB4 => writer.write_golomb(value, 4),
            const_codes::GOLOMB5 => writer.write_golomb(value, 5),
            const_codes::GOLOMB6 => writer.write_golomb(value, 6),
            const_codes::GOLOMB7 => writer.write_golomb(value, 7),
            const_codes::GOLOMB8 => writer.write_golomb(value, 8),
            const_codes::GOLOMB9 => writer.write_golomb(value, 9),
            const_codes::GOLOMB10 => writer.write_golomb(value, 10),
            const_codes::EXP_GOLOMB2 => writer.write_exp_golomb(value, 2),
            const_codes::EXP_GOLOMB3 => writer.write_exp_golomb(value, 3),
            const_codes::EXP_GOLOMB4 => writer.write_exp_golomb(value, 4),
            const_codes::EXP_GOLOMB5 => writer.write_exp_golomb(value, 5),
            const_codes::EXP_GOLOMB6 => writer.write_exp_golomb(value, 6),
            const_codes::EXP_GOLOMB7 => writer.write_exp_golomb(value, 7),
            const_codes::EXP_GOLOMB8 => writer.write_exp_golomb(value, 8),
            const_codes::EXP_GOLOMB9 => writer.write_exp_golomb(value, 9),
            const_codes::EXP_GOLOMB10 => writer.write_exp_golomb(value, 10),
            const_codes::RICE2 => writer.write_rice(value, 2),
            const_codes::RICE3 => writer.write_rice(value, 3),
            const_codes::RICE4 => writer.write_rice(value, 4),
            const_codes::RICE5 => writer.write_rice(value, 5),
            const_codes::RICE6 => writer.write_rice(value, 6),
            const_codes::RICE7 => writer.write_rice(value, 7),
            const_codes::RICE8 => writer.write_rice(value, 8),
            const_codes::RICE9 => writer.write_rice(value, 9),
            const_codes::RICE10 => writer.write_rice(value, 10),
            _ => panic!("Unknown code: {}", CODE),
        }
    }
}

impl<E, CW, const CODE: usize> CodeWriteDispatch<E, CW> for ConstCode<CODE>
where
    E: Endianness,
    CW: WriteCodes<E> + ?Sized,
{
    type Error<CWE> = CWE
    where
        CWE: Error + Send + Sync + 'static;
    #[inline(always)]
    fn write_dispatch(&self, writer: &mut CW, value: u64) -> Result<usize, Self::Error<CW::Error>> {
        <Self as CodeWrite>::write(self, writer, value)
    }
}

impl<const CODE: usize> CodeLen for ConstCode<CODE> {
    #[inline]
    fn len(&self, value: u64) -> usize {
        match CODE {
            const_codes::UNARY => value as usize + 1,
            const_codes::GAMMA => len_gamma(value),
            const_codes::DELTA => len_delta(value),
            const_codes::OMEGA => len_omega(value),
            const_codes::VBYTE => len_vbyte(value),
            const_codes::ZETA1 => len_zeta(value, 1),
            const_codes::ZETA2 => len_zeta(value, 2),
            const_codes::ZETA3 => len_zeta(value, 3),
            const_codes::ZETA4 => len_zeta(value, 4),
            const_codes::ZETA5 => len_zeta(value, 5),
            const_codes::ZETA6 => len_zeta(value, 6),
            const_codes::ZETA7 => len_zeta(value, 7),
            const_codes::ZETA8 => len_zeta(value, 8),
            const_codes::ZETA9 => len_zeta(value, 9),
            const_codes::ZETA10 => len_zeta(value, 10),
            const_codes::PI2 => len_pi(value, 2),
            const_codes::PI3 => len_pi(value, 3),
            const_codes::PI4 => len_pi(value, 4),
            const_codes::PI5 => len_pi(value, 5),
            const_codes::PI6 => len_pi(value, 6),
            const_codes::PI7 => len_pi(value, 7),
            const_codes::PI8 => len_pi(value, 8),
            const_codes::PI9 => len_pi(value, 9),
            const_codes::PI10 => len_pi(value, 10),
            const_codes::PI_WEB2 => len_pi_web(value, 2),
            const_codes::PI_WEB3 => len_pi_web(value, 3),
            const_codes::PI_WEB4 => len_pi_web(value, 4),
            const_codes::PI_WEB5 => len_pi_web(value, 5),
            const_codes::PI_WEB6 => len_pi_web(value, 6),
            const_codes::PI_WEB7 => len_pi_web(value, 7),
            const_codes::PI_WEB8 => len_pi_web(value, 8),
            const_codes::PI_WEB9 => len_pi_web(value, 9),
            const_codes::PI_WEB10 => len_pi_web(value, 10),
            const_codes::GOLOMB2 => len_golomb(value, 2),
            const_codes::GOLOMB3 => len_golomb(value, 3),
            const_codes::GOLOMB4 => len_golomb(value, 4),
            const_codes::GOLOMB5 => len_golomb(value, 5),
            const_codes::GOLOMB6 => len_golomb(value, 6),
            const_codes::GOLOMB7 => len_golomb(value, 7),
            const_codes::GOLOMB8 => len_golomb(value, 8),
            const_codes::GOLOMB9 => len_golomb(value, 9),
            const_codes::GOLOMB10 => len_golomb(value, 10),
            const_codes::EXP_GOLOMB2 => len_exp_golomb(value, 2),
            const_codes::EXP_GOLOMB3 => len_exp_golomb(value, 3),
            const_codes::EXP_GOLOMB4 => len_exp_golomb(value, 4),
            const_codes::EXP_GOLOMB5 => len_exp_golomb(value, 5),
            const_codes::EXP_GOLOMB6 => len_exp_golomb(value, 6),
            const_codes::EXP_GOLOMB7 => len_exp_golomb(value, 7),
            const_codes::EXP_GOLOMB8 => len_exp_golomb(value, 8),
            const_codes::EXP_GOLOMB9 => len_exp_golomb(value, 9),
            const_codes::EXP_GOLOMB10 => len_exp_golomb(value, 10),
            const_codes::RICE2 => len_rice(value, 2),
            const_codes::RICE3 => len_rice(value, 3),
            const_codes::RICE4 => len_rice(value, 4),
            const_codes::RICE5 => len_rice(value, 5),
            const_codes::RICE6 => len_rice(value, 6),
            const_codes::RICE7 => len_rice(value, 7),
            const_codes::RICE8 => len_rice(value, 8),
            const_codes::RICE9 => len_rice(value, 9),
            const_codes::RICE10 => len_rice(value, 10),
            _ => panic!("Unknown code: {}", CODE),
        }
    }
}

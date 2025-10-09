/*
 * SPDX-FileCopyrightText: 2025 Tommaso Fontana
 * SPDX-FileCopyrightText: 2025 Inria
 * SPDX-FileCopyrightText: 2025 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Enumeration of all available codes, with associated read and write methods.
//!
//! This is the slower and more generic form of dispatching, mostly used for
//! testing and writing examples. For faster dispatching, consider using
//! [dynamic] or [static] dispatch.

use super::*;
#[cfg(feature = "mem_dbg")]
use mem_dbg::{MemDbg, MemSize};

#[derive(Debug, Clone, Copy, Eq)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
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
    VByteLe,
    VByteBe,
    Zeta { k: usize },
    Pi { k: usize },
    Golomb { b: u64 },
    ExpGolomb { k: usize },
    Rice { log2_b: usize },
}

/// Some codes are equivalent, so we implement [`PartialEq`] to make them
/// interchangeable so `Codes::Unary == Codes::Rice{log2_b: 0}`.
impl PartialEq for Codes {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // First we check the equivalence classes
            (
                Self::Unary | Self::Rice { log2_b: 0 } | Self::Golomb { b: 1 },
                Self::Unary | Self::Rice { log2_b: 0 } | Self::Golomb { b: 1 },
            ) => true,
            (
                Self::Gamma | Self::Zeta { k: 1 } | Self::ExpGolomb { k: 0 },
                Self::Gamma | Self::Zeta { k: 1 } | Self::ExpGolomb { k: 0 },
            ) => true,
            (
                Self::Golomb { b: 2 } | Self::Rice { log2_b: 1 },
                Self::Golomb { b: 2 } | Self::Rice { log2_b: 1 },
            ) => true,
            (
                Self::Golomb { b: 4 } | Self::Rice { log2_b: 2 },
                Self::Golomb { b: 4 } | Self::Rice { log2_b: 2 },
            ) => true,
            (
                Self::Golomb { b: 8 } | Self::Rice { log2_b: 3 },
                Self::Golomb { b: 8 } | Self::Rice { log2_b: 3 },
            ) => true,
            // we know that we are not in a special case, so we can directly
            // compare them naively
            (Self::Delta, Self::Delta) => true,
            (Self::Omega, Self::Omega) => true,
            (Self::VByteLe, Self::VByteLe) => true,
            (Self::VByteBe, Self::VByteBe) => true,
            (Self::Zeta { k }, Self::Zeta { k: k2 }) => k == k2,
            (Self::Pi { k }, Self::Pi { k: k2 }) => k == k2,
            (Self::Golomb { b }, Self::Golomb { b: b2 }) => b == b2,
            (Self::ExpGolomb { k }, Self::ExpGolomb { k: k2 }) => k == k2,
            (Self::Rice { log2_b }, Self::Rice { log2_b: log2_b2 }) => log2_b == log2_b2,
            _ => false,
        }
    }
}

impl Codes {
    /// Delegate to the [`DynamicCodeRead`] implementation.
    ///
    /// This inherent method is provided to reduce ambiguity in method
    /// resolution.
    #[inline(always)]
    pub fn read<E: Endianness, CR: CodesRead<E> + ?Sized>(
        &self,
        reader: &mut CR,
    ) -> Result<u64, CR::Error> {
        DynamicCodeRead::read(self, reader)
    }

    /// Delegate to the [`DynamicCodeWrite`] implementation.
    ///
    /// This inherent method is provided to reduce ambiguity in method
    /// resolution.
    #[inline(always)]
    pub fn write<E: Endianness, CW: CodesWrite<E> + ?Sized>(
        &self,
        writer: &mut CW,
        value: u64,
    ) -> Result<usize, CW::Error> {
        DynamicCodeWrite::write(self, writer, value)
    }

    /// Convert a code to the constant enum [`code_consts`] used for [`ConstCode`].
    /// This is mostly used to verify that the code is supported by
    /// [`ConstCode`].
    pub fn to_code_const(&self) -> Result<usize> {
        Ok(match self {
            Self::Unary => code_consts::UNARY,
            Self::Gamma => code_consts::GAMMA,
            Self::Delta => code_consts::DELTA,
            Self::Omega => code_consts::OMEGA,
            Self::VByteLe => code_consts::VBYTE_LE,
            Self::VByteBe => code_consts::VBYTE_BE,
            Self::Zeta { k: 1 } => code_consts::ZETA1,
            Self::Zeta { k: 2 } => code_consts::ZETA2,
            Self::Zeta { k: 3 } => code_consts::ZETA3,
            Self::Zeta { k: 4 } => code_consts::ZETA4,
            Self::Zeta { k: 5 } => code_consts::ZETA5,
            Self::Zeta { k: 6 } => code_consts::ZETA6,
            Self::Zeta { k: 7 } => code_consts::ZETA7,
            Self::Zeta { k: 8 } => code_consts::ZETA8,
            Self::Zeta { k: 9 } => code_consts::ZETA9,
            Self::Zeta { k: 10 } => code_consts::ZETA10,
            Self::Rice { log2_b: 0 } => code_consts::RICE0,
            Self::Rice { log2_b: 1 } => code_consts::RICE1,
            Self::Rice { log2_b: 2 } => code_consts::RICE2,
            Self::Rice { log2_b: 3 } => code_consts::RICE3,
            Self::Rice { log2_b: 4 } => code_consts::RICE4,
            Self::Rice { log2_b: 5 } => code_consts::RICE5,
            Self::Rice { log2_b: 6 } => code_consts::RICE6,
            Self::Rice { log2_b: 7 } => code_consts::RICE7,
            Self::Rice { log2_b: 8 } => code_consts::RICE8,
            Self::Rice { log2_b: 9 } => code_consts::RICE9,
            Self::Rice { log2_b: 10 } => code_consts::RICE10,
            Self::Pi { k: 0 } => code_consts::PI0,
            Self::Pi { k: 1 } => code_consts::PI1,
            Self::Pi { k: 2 } => code_consts::PI2,
            Self::Pi { k: 3 } => code_consts::PI3,
            Self::Pi { k: 4 } => code_consts::PI4,
            Self::Pi { k: 5 } => code_consts::PI5,
            Self::Pi { k: 6 } => code_consts::PI6,
            Self::Pi { k: 7 } => code_consts::PI7,
            Self::Pi { k: 8 } => code_consts::PI8,
            Self::Pi { k: 9 } => code_consts::PI9,
            Self::Pi { k: 10 } => code_consts::PI10,
            Self::Golomb { b: 1 } => code_consts::GOLOMB1,
            Self::Golomb { b: 2 } => code_consts::GOLOMB2,
            Self::Golomb { b: 3 } => code_consts::GOLOMB3,
            Self::Golomb { b: 4 } => code_consts::GOLOMB4,
            Self::Golomb { b: 5 } => code_consts::GOLOMB5,
            Self::Golomb { b: 6 } => code_consts::GOLOMB6,
            Self::Golomb { b: 7 } => code_consts::GOLOMB7,
            Self::Golomb { b: 8 } => code_consts::GOLOMB8,
            Self::Golomb { b: 9 } => code_consts::GOLOMB9,
            Self::Golomb { b: 10 } => code_consts::GOLOMB10,
            Self::ExpGolomb { k: 0 } => code_consts::EXP_GOLOMB0,
            Self::ExpGolomb { k: 1 } => code_consts::EXP_GOLOMB1,
            Self::ExpGolomb { k: 2 } => code_consts::EXP_GOLOMB2,
            Self::ExpGolomb { k: 3 } => code_consts::EXP_GOLOMB3,
            Self::ExpGolomb { k: 4 } => code_consts::EXP_GOLOMB4,
            Self::ExpGolomb { k: 5 } => code_consts::EXP_GOLOMB5,
            Self::ExpGolomb { k: 6 } => code_consts::EXP_GOLOMB6,
            Self::ExpGolomb { k: 7 } => code_consts::EXP_GOLOMB7,
            Self::ExpGolomb { k: 8 } => code_consts::EXP_GOLOMB8,
            Self::ExpGolomb { k: 9 } => code_consts::EXP_GOLOMB9,
            Self::ExpGolomb { k: 10 } => code_consts::EXP_GOLOMB10,
            _ => {
                return Err(anyhow::anyhow!(
                    "Code {:?} not supported as const code",
                    self
                ))
            }
        })
    }

    /// Convert a value from [`code_consts`] to a code.
    pub fn from_code_const(const_code: usize) -> Result<Self> {
        Ok(match const_code {
            code_consts::UNARY => Self::Unary,
            code_consts::GAMMA => Self::Gamma,
            code_consts::DELTA => Self::Delta,
            code_consts::OMEGA => Self::Omega,
            code_consts::VBYTE_LE => Self::VByteLe,
            code_consts::VBYTE_BE => Self::VByteBe,
            code_consts::ZETA2 => Self::Zeta { k: 2 },
            code_consts::ZETA3 => Self::Zeta { k: 3 },
            code_consts::ZETA4 => Self::Zeta { k: 4 },
            code_consts::ZETA5 => Self::Zeta { k: 5 },
            code_consts::ZETA6 => Self::Zeta { k: 6 },
            code_consts::ZETA7 => Self::Zeta { k: 7 },
            code_consts::ZETA8 => Self::Zeta { k: 8 },
            code_consts::ZETA9 => Self::Zeta { k: 9 },
            code_consts::ZETA10 => Self::Zeta { k: 10 },
            code_consts::RICE1 => Self::Rice { log2_b: 1 },
            code_consts::RICE2 => Self::Rice { log2_b: 2 },
            code_consts::RICE3 => Self::Rice { log2_b: 3 },
            code_consts::RICE4 => Self::Rice { log2_b: 4 },
            code_consts::RICE5 => Self::Rice { log2_b: 5 },
            code_consts::RICE6 => Self::Rice { log2_b: 6 },
            code_consts::RICE7 => Self::Rice { log2_b: 7 },
            code_consts::RICE8 => Self::Rice { log2_b: 8 },
            code_consts::RICE9 => Self::Rice { log2_b: 9 },
            code_consts::RICE10 => Self::Rice { log2_b: 10 },
            code_consts::PI1 => Self::Pi { k: 1 },
            code_consts::PI2 => Self::Pi { k: 2 },
            code_consts::PI3 => Self::Pi { k: 3 },
            code_consts::PI4 => Self::Pi { k: 4 },
            code_consts::PI5 => Self::Pi { k: 5 },
            code_consts::PI6 => Self::Pi { k: 6 },
            code_consts::PI7 => Self::Pi { k: 7 },
            code_consts::PI8 => Self::Pi { k: 8 },
            code_consts::PI9 => Self::Pi { k: 9 },
            code_consts::PI10 => Self::Pi { k: 10 },
            code_consts::GOLOMB3 => Self::Golomb { b: 3 },
            code_consts::GOLOMB5 => Self::Golomb { b: 5 },
            code_consts::GOLOMB6 => Self::Golomb { b: 6 },
            code_consts::GOLOMB7 => Self::Golomb { b: 7 },
            code_consts::GOLOMB9 => Self::Golomb { b: 9 },
            code_consts::GOLOMB10 => Self::Golomb { b: 10 },
            code_consts::EXP_GOLOMB1 => Self::ExpGolomb { k: 1 },
            code_consts::EXP_GOLOMB2 => Self::ExpGolomb { k: 2 },
            code_consts::EXP_GOLOMB3 => Self::ExpGolomb { k: 3 },
            code_consts::EXP_GOLOMB4 => Self::ExpGolomb { k: 4 },
            code_consts::EXP_GOLOMB5 => Self::ExpGolomb { k: 5 },
            code_consts::EXP_GOLOMB6 => Self::ExpGolomb { k: 6 },
            code_consts::EXP_GOLOMB7 => Self::ExpGolomb { k: 7 },
            code_consts::EXP_GOLOMB8 => Self::ExpGolomb { k: 8 },
            code_consts::EXP_GOLOMB9 => Self::ExpGolomb { k: 9 },
            code_consts::EXP_GOLOMB10 => Self::ExpGolomb { k: 10 },
            _ => return Err(anyhow::anyhow!("Code {} not supported", const_code)),
        })
    }
}

impl DynamicCodeRead for Codes {
    #[inline]
    fn read<E: Endianness, CR: CodesRead<E> + ?Sized>(
        &self,
        reader: &mut CR,
    ) -> Result<u64, CR::Error> {
        Ok(match self {
            Codes::Unary => reader.read_unary()?,
            Codes::Gamma => reader.read_gamma()?,
            Codes::Delta => reader.read_delta()?,
            Codes::Omega => reader.read_omega()?,
            Codes::VByteBe => reader.read_vbyte_be()?,
            Codes::VByteLe => reader.read_vbyte_le()?,
            Codes::Zeta { k: 3 } => reader.read_zeta3()?,
            Codes::Zeta { k } => reader.read_zeta(*k)?,
            Codes::Pi { k } => reader.read_pi(*k)?,
            Codes::Golomb { b } => reader.read_golomb(*b)?,
            Codes::ExpGolomb { k } => reader.read_exp_golomb(*k)?,
            Codes::Rice { log2_b } => reader.read_rice(*log2_b)?,
        })
    }
}

impl DynamicCodeWrite for Codes {
    #[inline]
    fn write<E: Endianness, CW: CodesWrite<E> + ?Sized>(
        &self,
        writer: &mut CW,
        value: u64,
    ) -> Result<usize, CW::Error> {
        Ok(match self {
            Codes::Unary => writer.write_unary(value)?,
            Codes::Gamma => writer.write_gamma(value)?,
            Codes::Delta => writer.write_delta(value)?,
            Codes::Omega => writer.write_omega(value)?,
            Codes::VByteBe => writer.write_vbyte_be(value)?,
            Codes::VByteLe => writer.write_vbyte_le(value)?,
            Codes::Zeta { k: 1 } => writer.write_gamma(value)?,
            Codes::Zeta { k: 3 } => writer.write_zeta3(value)?,
            Codes::Zeta { k } => writer.write_zeta(value, *k)?,
            Codes::Pi { k } => writer.write_pi(value, *k)?,
            Codes::Golomb { b } => writer.write_golomb(value, *b)?,
            Codes::ExpGolomb { k } => writer.write_exp_golomb(value, *k)?,
            Codes::Rice { log2_b } => writer.write_rice(value, *log2_b)?,
        })
    }
}

impl<E: Endianness, CR: CodesRead<E> + ?Sized> StaticCodeRead<E, CR> for Codes {
    #[inline(always)]
    fn read(&self, reader: &mut CR) -> Result<u64, CR::Error> {
        <Self as DynamicCodeRead>::read(self, reader)
    }
}

impl<E: Endianness, CW: CodesWrite<E> + ?Sized> StaticCodeWrite<E, CW> for Codes {
    #[inline(always)]
    fn write(&self, writer: &mut CW, value: u64) -> Result<usize, CW::Error> {
        <Self as DynamicCodeWrite>::write(self, writer, value)
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
            Codes::VByteLe | Codes::VByteBe => bit_len_vbyte(value),
            Codes::Zeta { k: 1 } => len_gamma(value),
            Codes::Zeta { k } => len_zeta(value, *k),
            Codes::Pi { k } => len_pi(value, *k),
            Codes::Golomb { b } => len_golomb(value, *b),
            Codes::ExpGolomb { k } => len_exp_golomb(value, *k),
            Codes::Rice { log2_b } => len_rice(value, *log2_b),
        }
    }
}

#[derive(Debug)]
/// Error type for parsing a code from a string.
pub enum CodeError {
    ParseError(core::num::ParseIntError),
    UnknownCode([u8; 32]),
}
#[cfg(feature = "std")]
impl std::error::Error for CodeError {}
impl core::fmt::Display for CodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            CodeError::ParseError(e) => write!(f, "Parse error: {}", e),
            CodeError::UnknownCode(s) => {
                write!(f, "Unknown code: ")?;
                for c in s {
                    if *c == 0 {
                        break;
                    }
                    write!(f, "{}", *c as char)?;
                }
                Ok(())
            }
        }
    }
}

impl From<core::num::ParseIntError> for CodeError {
    fn from(e: core::num::ParseIntError) -> Self {
        CodeError::ParseError(e)
    }
}

impl core::fmt::Display for Codes {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Codes::Unary => write!(f, "Unary"),
            Codes::Gamma => write!(f, "Gamma"),
            Codes::Delta => write!(f, "Delta"),
            Codes::Omega => write!(f, "Omega"),
            Codes::VByteBe => write!(f, "VByteBe"),
            Codes::VByteLe => write!(f, "VByteLe"),
            Codes::Zeta { k } => write!(f, "Zeta({})", k),
            Codes::Pi { k } => write!(f, "Pi({})", k),
            Codes::Golomb { b } => write!(f, "Golomb({})", b),
            Codes::ExpGolomb { k } => write!(f, "ExpGolomb({})", k),
            Codes::Rice { log2_b } => write!(f, "Rice({})", log2_b),
        }
    }
}

fn array_format_error(s: &str) -> [u8; 32] {
    let mut error_buffer = [0u8; 32];
    const ERROR_PREFIX: &[u8] = b"Could not parse ";
    error_buffer[..ERROR_PREFIX.len()].copy_from_slice(ERROR_PREFIX);
    error_buffer[ERROR_PREFIX.len()..ERROR_PREFIX.len() + s.len().min(32 - ERROR_PREFIX.len())]
        .copy_from_slice(&s.as_bytes()[..s.len().min(32 - ERROR_PREFIX.len())]);
    error_buffer
}

impl core::str::FromStr for Codes {
    type Err = CodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Unary" => Ok(Codes::Unary),
            "Gamma" => Ok(Codes::Gamma),
            "Delta" => Ok(Codes::Delta),
            "Omega" => Ok(Codes::Omega),
            "VByteBe" => Ok(Codes::VByteBe),

            _ => {
                let mut parts = s.split('(');
                let name = parts
                    .next()
                    .ok_or_else(|| CodeError::UnknownCode(array_format_error(s)))?;
                let k = parts
                    .next()
                    .ok_or_else(|| CodeError::UnknownCode(array_format_error(s)))?
                    .split(')')
                    .next()
                    .ok_or_else(|| CodeError::UnknownCode(array_format_error(s)))?;
                match name {
                    "Zeta" => Ok(Codes::Zeta { k: k.parse()? }),
                    "Pi" => Ok(Codes::Pi { k: k.parse()? }),
                    "Golomb" => Ok(Codes::Golomb { b: k.parse()? }),
                    "ExpGolomb" => Ok(Codes::ExpGolomb { k: k.parse()? }),
                    "Rice" => Ok(Codes::Rice { log2_b: k.parse()? }),
                    _ => Err(CodeError::UnknownCode(array_format_error(name))),
                }
            }
        }
    }
}

/// Structure representing minimal binary coding with a fixed length.
///
/// [Minimal binary coding](crate::codes::minimal_binary) does not
/// fit the [`Codes`] enum because it is not defined for all integers.
///
/// Instances of this structure can be used in context in which a
/// [`DynamicCodeRead`], [`DynamicCodeWrite`], [`StaticCodeRead`],
/// [`StaticCodeWrite`] or [`CodeLen`] implementing minimal binary coding
/// is necessary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MinimalBinary(pub u64);

impl DynamicCodeRead for MinimalBinary {
    fn read<E: Endianness, R: CodesRead<E> + ?Sized>(
        &self,
        reader: &mut R,
    ) -> Result<u64, R::Error> {
        reader.read_minimal_binary(self.0)
    }
}

impl DynamicCodeWrite for MinimalBinary {
    fn write<E: Endianness, W: CodesWrite<E> + ?Sized>(
        &self,
        writer: &mut W,
        n: u64,
    ) -> Result<usize, W::Error> {
        writer.write_minimal_binary(n, self.0)
    }
}

impl<E: Endianness, CR: CodesRead<E> + ?Sized> StaticCodeRead<E, CR> for MinimalBinary {
    fn read(&self, reader: &mut CR) -> Result<u64, CR::Error> {
        <Self as DynamicCodeRead>::read(self, reader)
    }
}

impl<E: Endianness, CW: CodesWrite<E> + ?Sized> StaticCodeWrite<E, CW> for MinimalBinary {
    fn write(&self, writer: &mut CW, n: u64) -> Result<usize, CW::Error> {
        <Self as DynamicCodeWrite>::write(self, writer, n)
    }
}

impl CodeLen for MinimalBinary {
    fn len(&self, n: u64) -> usize {
        len_minimal_binary(n, self.0)
    }
}

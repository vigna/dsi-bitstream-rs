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
//! [dynamic](super::dynamic) or [static](super::r#static) dispatch.

use super::*;
#[cfg(feature = "serde")]
use alloc::string::{String, ToString};
#[cfg(feature = "mem_dbg")]
use mem_dbg::{MemDbg, MemSize};

/// An enum whose variants represent all the available codes.
///
/// This enum is kept in sync with implementations in the
/// [`codes`](crate::codes) module.
///
/// Both [`Display`](std::fmt::Display) and [`FromStr`](std::str::FromStr) are
/// implemented for this enum in a compatible way, making it possible to store a
/// code as a string in a configuration file and then parse it back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
#[cfg_attr(feature = "mem_dbg", mem_size_flat)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[non_exhaustive]
pub enum Codes {
    Unary,
    Gamma,
    Delta,
    Omega,
    VByteLe,
    VByteBe,
    Zeta(usize),
    Pi(usize),
    Golomb(u64),
    ExpGolomb(usize),
    Rice(usize),
}

impl Codes {
    /// Returns the canonical form of this code.
    ///
    /// Some codes are equivalent, in the sense that they are defined
    /// differently, but they give rise to the same codewords. Among equivalent
    /// codes, there is usually one that is faster to encode and decode, which
    /// we call the _canonical representative_ of the equivalence class.
    ///
    /// The mapping is:
    ///
    /// - [`Rice(0)`](Codes::Rice),
    ///   [`Golomb(1)`](Codes::Golomb) →
    ///   [`Unary`](Codes::Unary)
    ///
    /// - [`Zeta(1)`](Codes::Zeta),
    ///   [`ExpGolomb(0)`](Codes::ExpGolomb),
    ///   [`Pi(0)`](Codes::Pi) →
    ///   [`Gamma`](Codes::Gamma)
    ///
    /// - [`Golomb(2ⁿ)`](Codes::Golomb) → [`Rice(n)`](Codes::Rice)
    #[must_use]
    pub const fn canonicalize(self) -> Self {
        match self {
            Self::Zeta(1) | Self::ExpGolomb(0) | Self::Pi(0) => Self::Gamma,
            Self::Rice(0) | Self::Golomb(1) => Self::Unary,
            Self::Golomb(b) if b.is_power_of_two() => Self::Rice(b.trailing_zeros() as usize),
            other => other,
        }
    }

    /// Delegates to the [`DynamicCodeRead`] implementation.
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

    /// Delegates to the [`DynamicCodeWrite`] implementation.
    ///
    /// This inherent method is provided to reduce ambiguity in method
    /// resolution.
    #[inline(always)]
    pub fn write<E: Endianness, CW: CodesWrite<E> + ?Sized>(
        &self,
        writer: &mut CW,
        n: u64,
    ) -> Result<usize, CW::Error> {
        DynamicCodeWrite::write(self, writer, n)
    }

    /// Converts a code to the constant enum [`code_consts`]
    /// used for [`ConstCode`]. This is mostly used to verify
    /// that the code is supported by [`ConstCode`].
    ///
    /// The code is [canonicalized](Codes::canonicalize) before
    /// the conversion, so equivalent codes map to the same
    /// constant.
    ///
    /// # Errors
    ///
    /// Returns [`DispatchError::UnsupportedCode`] if the (canonicalized)
    /// code has no corresponding constant in [`code_consts`].
    pub const fn to_code_const(&self) -> Result<usize, DispatchError> {
        Ok(match self.canonicalize() {
            Self::Unary => code_consts::UNARY,
            Self::Gamma => code_consts::GAMMA,
            Self::Delta => code_consts::DELTA,
            Self::Omega => code_consts::OMEGA,
            Self::VByteLe => code_consts::VBYTE_LE,
            Self::VByteBe => code_consts::VBYTE_BE,
            Self::Zeta(2) => code_consts::ZETA2,
            Self::Zeta(3) => code_consts::ZETA3,
            Self::Zeta(4) => code_consts::ZETA4,
            Self::Zeta(5) => code_consts::ZETA5,
            Self::Zeta(6) => code_consts::ZETA6,
            Self::Zeta(7) => code_consts::ZETA7,
            Self::Zeta(8) => code_consts::ZETA8,
            Self::Zeta(9) => code_consts::ZETA9,
            Self::Zeta(10) => code_consts::ZETA10,
            Self::Rice(1) => code_consts::RICE1,
            Self::Rice(2) => code_consts::RICE2,
            Self::Rice(3) => code_consts::RICE3,
            Self::Rice(4) => code_consts::RICE4,
            Self::Rice(5) => code_consts::RICE5,
            Self::Rice(6) => code_consts::RICE6,
            Self::Rice(7) => code_consts::RICE7,
            Self::Rice(8) => code_consts::RICE8,
            Self::Rice(9) => code_consts::RICE9,
            Self::Rice(10) => code_consts::RICE10,
            Self::Pi(1) => code_consts::PI1,
            Self::Pi(2) => code_consts::PI2,
            Self::Pi(3) => code_consts::PI3,
            Self::Pi(4) => code_consts::PI4,
            Self::Pi(5) => code_consts::PI5,
            Self::Pi(6) => code_consts::PI6,
            Self::Pi(7) => code_consts::PI7,
            Self::Pi(8) => code_consts::PI8,
            Self::Pi(9) => code_consts::PI9,
            Self::Pi(10) => code_consts::PI10,
            Self::Golomb(3) => code_consts::GOLOMB3,
            Self::Golomb(5) => code_consts::GOLOMB5,
            Self::Golomb(6) => code_consts::GOLOMB6,
            Self::Golomb(7) => code_consts::GOLOMB7,
            Self::Golomb(9) => code_consts::GOLOMB9,
            Self::Golomb(10) => code_consts::GOLOMB10,
            Self::ExpGolomb(1) => code_consts::EXP_GOLOMB1,
            Self::ExpGolomb(2) => code_consts::EXP_GOLOMB2,
            Self::ExpGolomb(3) => code_consts::EXP_GOLOMB3,
            Self::ExpGolomb(4) => code_consts::EXP_GOLOMB4,
            Self::ExpGolomb(5) => code_consts::EXP_GOLOMB5,
            Self::ExpGolomb(6) => code_consts::EXP_GOLOMB6,
            Self::ExpGolomb(7) => code_consts::EXP_GOLOMB7,
            Self::ExpGolomb(8) => code_consts::EXP_GOLOMB8,
            Self::ExpGolomb(9) => code_consts::EXP_GOLOMB9,
            Self::ExpGolomb(10) => code_consts::EXP_GOLOMB10,
            _ => {
                return Err(DispatchError::UnsupportedCode(*self));
            }
        })
    }

    /// Converts a value from [`code_consts`] to a code.
    ///
    /// # Errors
    ///
    /// Returns [`DispatchError::UnsupportedCodeConst`] if the value
    /// does not correspond to any known code constant.
    pub const fn from_code_const(const_code: usize) -> Result<Self, DispatchError> {
        Ok(match const_code {
            code_consts::UNARY => Self::Unary,
            code_consts::GAMMA => Self::Gamma,
            code_consts::DELTA => Self::Delta,
            code_consts::OMEGA => Self::Omega,
            code_consts::VBYTE_LE => Self::VByteLe,
            code_consts::VBYTE_BE => Self::VByteBe,
            code_consts::ZETA2 => Self::Zeta(2),
            code_consts::ZETA3 => Self::Zeta(3),
            code_consts::ZETA4 => Self::Zeta(4),
            code_consts::ZETA5 => Self::Zeta(5),
            code_consts::ZETA6 => Self::Zeta(6),
            code_consts::ZETA7 => Self::Zeta(7),
            code_consts::ZETA8 => Self::Zeta(8),
            code_consts::ZETA9 => Self::Zeta(9),
            code_consts::ZETA10 => Self::Zeta(10),
            code_consts::RICE1 => Self::Rice(1),
            code_consts::RICE2 => Self::Rice(2),
            code_consts::RICE3 => Self::Rice(3),
            code_consts::RICE4 => Self::Rice(4),
            code_consts::RICE5 => Self::Rice(5),
            code_consts::RICE6 => Self::Rice(6),
            code_consts::RICE7 => Self::Rice(7),
            code_consts::RICE8 => Self::Rice(8),
            code_consts::RICE9 => Self::Rice(9),
            code_consts::RICE10 => Self::Rice(10),
            code_consts::PI1 => Self::Pi(1),
            code_consts::PI2 => Self::Pi(2),
            code_consts::PI3 => Self::Pi(3),
            code_consts::PI4 => Self::Pi(4),
            code_consts::PI5 => Self::Pi(5),
            code_consts::PI6 => Self::Pi(6),
            code_consts::PI7 => Self::Pi(7),
            code_consts::PI8 => Self::Pi(8),
            code_consts::PI9 => Self::Pi(9),
            code_consts::PI10 => Self::Pi(10),
            code_consts::GOLOMB3 => Self::Golomb(3),
            code_consts::GOLOMB5 => Self::Golomb(5),
            code_consts::GOLOMB6 => Self::Golomb(6),
            code_consts::GOLOMB7 => Self::Golomb(7),
            code_consts::GOLOMB9 => Self::Golomb(9),
            code_consts::GOLOMB10 => Self::Golomb(10),
            code_consts::EXP_GOLOMB1 => Self::ExpGolomb(1),
            code_consts::EXP_GOLOMB2 => Self::ExpGolomb(2),
            code_consts::EXP_GOLOMB3 => Self::ExpGolomb(3),
            code_consts::EXP_GOLOMB4 => Self::ExpGolomb(4),
            code_consts::EXP_GOLOMB5 => Self::ExpGolomb(5),
            code_consts::EXP_GOLOMB6 => Self::ExpGolomb(6),
            code_consts::EXP_GOLOMB7 => Self::ExpGolomb(7),
            code_consts::EXP_GOLOMB8 => Self::ExpGolomb(8),
            code_consts::EXP_GOLOMB9 => Self::ExpGolomb(9),
            code_consts::EXP_GOLOMB10 => Self::ExpGolomb(10),
            _ => return Err(DispatchError::UnsupportedCodeConst(const_code)),
        })
    }
}

impl DynamicCodeRead for Codes {
    #[inline]
    fn read<E: Endianness, CR: CodesRead<E> + ?Sized>(
        &self,
        reader: &mut CR,
    ) -> Result<u64, CR::Error> {
        Ok(match self.canonicalize() {
            Codes::Unary => reader.read_unary()?,
            Codes::Gamma => reader.read_gamma()?,
            Codes::Delta => reader.read_delta()?,
            Codes::Omega => reader.read_omega()?,
            Codes::VByteBe => reader.read_vbyte_be()?,
            Codes::VByteLe => reader.read_vbyte_le()?,
            Codes::Zeta(3) => reader.read_zeta3()?,
            Codes::Zeta(k) => reader.read_zeta(k)?,
            Codes::Pi(2) => reader.read_pi2()?,
            Codes::Pi(k) => reader.read_pi(k)?,
            Codes::Golomb(b) => reader.read_golomb(b)?,
            Codes::ExpGolomb(k) => reader.read_exp_golomb(k)?,
            Codes::Rice(log2_b) => reader.read_rice(log2_b)?,
        })
    }
}

impl DynamicCodeWrite for Codes {
    #[inline]
    fn write<E: Endianness, CW: CodesWrite<E> + ?Sized>(
        &self,
        writer: &mut CW,
        n: u64,
    ) -> Result<usize, CW::Error> {
        Ok(match self.canonicalize() {
            Codes::Unary => writer.write_unary(n)?,
            Codes::Gamma => writer.write_gamma(n)?,
            Codes::Delta => writer.write_delta(n)?,
            Codes::Omega => writer.write_omega(n)?,
            Codes::VByteBe => writer.write_vbyte_be(n)?,
            Codes::VByteLe => writer.write_vbyte_le(n)?,
            Codes::Zeta(3) => writer.write_zeta3(n)?,
            Codes::Zeta(k) => writer.write_zeta(n, k)?,
            Codes::Pi(2) => writer.write_pi2(n)?,
            Codes::Pi(k) => writer.write_pi(n, k)?,
            Codes::Golomb(b) => writer.write_golomb(n, b)?,
            Codes::ExpGolomb(k) => writer.write_exp_golomb(n, k)?,
            Codes::Rice(log2_b) => writer.write_rice(n, log2_b)?,
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
    fn write(&self, writer: &mut CW, n: u64) -> Result<usize, CW::Error> {
        <Self as DynamicCodeWrite>::write(self, writer, n)
    }
}

impl CodeLen for Codes {
    #[inline]
    fn len(&self, n: u64) -> usize {
        match self.canonicalize() {
            Codes::Unary => n as usize + 1,
            Codes::Gamma => len_gamma(n),
            Codes::Delta => len_delta(n),
            Codes::Omega => len_omega(n),
            Codes::VByteLe | Codes::VByteBe => bit_len_vbyte(n),
            Codes::Zeta(k) => len_zeta(n, k),
            Codes::Pi(k) => len_pi(n, k),
            Codes::Golomb(b) => len_golomb(n, b),
            Codes::ExpGolomb(k) => len_exp_golomb(n, k),
            Codes::Rice(log2_b) => len_rice(n, log2_b),
        }
    }
}

/// Error type for parsing a code from a string.
#[derive(Debug, Clone)]
pub enum CodeError {
    /// Error parsing an integer parameter.
    ParseError(core::num::ParseIntError),
    /// Unknown code name. Uses a fixed-size array instead of `String` for `no_std` compatibility.
    UnknownCode([u8; 32]),
}
impl core::error::Error for CodeError {}
impl core::fmt::Display for CodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            CodeError::ParseError(e) => write!(f, "parse error: {}", e),
            CodeError::UnknownCode(s) => {
                write!(f, "unknown code: ")?;
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
            Codes::Zeta(k) => write!(f, "Zeta({})", k),
            Codes::Pi(k) => write!(f, "Pi({})", k),
            Codes::Golomb(b) => write!(f, "Golomb({})", b),
            Codes::ExpGolomb(k) => write!(f, "ExpGolomb({})", k),
            Codes::Rice(log2_b) => write!(f, "Rice({})", log2_b),
        }
    }
}

fn array_format_error(s: &str) -> [u8; 32] {
    let mut error_buffer = [0u8; 32];
    let len = s.len().min(32);
    error_buffer[..len].copy_from_slice(&s.as_bytes()[..len]);
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
            "VByteLe" => Ok(Codes::VByteLe),

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
                    "Zeta" => Ok(Codes::Zeta(k.parse()?)),
                    "Pi" => Ok(Codes::Pi(k.parse()?)),
                    "Golomb" => Ok(Codes::Golomb(k.parse()?)),
                    "ExpGolomb" => Ok(Codes::ExpGolomb(k.parse()?)),
                    "Rice" => Ok(Codes::Rice(k.parse()?)),
                    _ => Err(CodeError::UnknownCode(array_format_error(name))),
                }
            }
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Codes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Codes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

/// Structure representing minimal binary coding with a fixed length.
///
/// [Minimal binary coding](crate::codes::minimal_binary) does not
/// fit the [`Codes`] enum because it is not defined for all integers.
///
/// Instances of this structure can be used in contexts in which a
/// [`DynamicCodeRead`], [`DynamicCodeWrite`], [`StaticCodeRead`],
/// [`StaticCodeWrite`] or [`CodeLen`] implementing minimal binary coding
/// is necessary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MinimalBinary(
    /// The upper bound of the minimal binary code.
    pub u64,
);

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

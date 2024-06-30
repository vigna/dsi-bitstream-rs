/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

/*!

Traits for reading and writing instantaneous codes.

This modules contains code for reading and writing instantaneous codes.
Codewords are uniformely indexed from 0 for codes. For example, the
first few words of [unary](crate::traits::BitRead::read_unary),
[γ](gamma), and [δ](delta) codes are:

| Arg |  unary   |    γ    |     δ    |
|-----|---------:|--------:|---------:|
| 0   |        1 |       1 |        1 |
| 1   |       01 |     010 |     0100 |
| 2   |      001 |     011 |     0101 |
| 3   |     0001 |   00100 |    01100 |
| 4   |    00001 |   00101 |    01101 |
| 5   |   000001 |   00110 |    01110 |
| 6   |  0000001 |   00111 |    01111 |
| 7   | 00000001 | 0001000 | 00100000 |

Each code is implemented as a pair of traits for reading and writing
(e.g., [`GammaReadParam`] and [`GammaWriteParam`]). The traits for
reading depend on [`BitRead`], whereas
the traits for writing depend on [`BitWrite`].

The traits ending with `Param` make it possible to specify parameters—for
example, whether to use decoding tables. Usually, one whould instead pull
in scope non-parametric traits such as [`GammaRead`] and [`GammaWrite`],
for which defaults are provided using the mechanism described in the
[`params`] module.

Note that if you are using decoding tables, you must ensure that the
[`peek_bits`](crate::traits::BitRead::peek_bits) method of your
[`BitRead`] implementation returns a sufficient
number of bits: if it does not, an assertion will be triggered in test
mode, but behavior will be unpredictable otherwise. This is unfortunately
difficult to check statically. To stay on the safe side, we recommend
to use a read word of at least 16 bits.

*/
use anyhow::Result;
#[cfg(feature = "mem_dbg")]
use mem_dbg::{MemDbg, MemSize};

use crate::prelude::{BitRead, BitWrite};
pub mod params;

pub mod gamma;
pub use gamma::{
    len_gamma, len_gamma_param, GammaRead, GammaReadParam, GammaWrite, GammaWriteParam,
};

pub mod delta;
pub use delta::{
    len_delta, len_delta_param, DeltaRead, DeltaReadParam, DeltaWrite, DeltaWriteParam,
};

pub mod omega;
pub use omega::{len_omega, OmegaRead, OmegaWrite};

pub mod minimal_binary;
pub use minimal_binary::{len_minimal_binary, MinimalBinaryRead, MinimalBinaryWrite};

pub mod zeta;
pub use zeta::{len_zeta, len_zeta_param, ZetaRead, ZetaReadParam, ZetaWrite, ZetaWriteParam};

pub mod pi;
pub use pi::{len_pi, len_pi_web, PiRead, PiWebRead, PiWebWrite, PiWrite};

pub mod golomb;
pub use golomb::{len_golomb, GolombRead, GolombWrite};

pub mod rice;
pub use rice::{len_rice, RiceRead, RiceWrite};

pub mod exp_golomb;
pub use exp_golomb::{len_exp_golomb, ExpGolombRead, ExpGolombWrite};

pub mod vbyte;
pub use vbyte::{len_vbyte, VByteRead, VByteWrite};

use crate::prelude::Endianness;

pub mod delta_tables;
pub mod gamma_tables;
pub mod zeta_tables;

/// A collection trait for reading all the codes supported by this library.
pub trait ReadCodes<E: Endianness>:
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
    + PiWebRead<E>
    + GolombRead<E>
    + RiceRead<E>
    + ExpGolombRead<E>
    + VByteRead<E>
{
    fn read_code(&mut self, code: Code) -> Result<u64> {
        code.read::<E>(self)
    }
}
impl<E: Endianness, B> ReadCodes<E> for B where
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
        + PiWebRead<E>
        + GolombRead<E>
        + RiceRead<E>
        + ExpGolombRead<E>
        + VByteRead<E>
{
}

/// A collection trait for writing all the codes supported by this library.
pub trait WriteCodes<E: Endianness>:
    BitWrite<E>
    + GammaWrite<E>
    + DeltaWrite<E>
    + ZetaWrite<E>
    + OmegaWrite<E>
    + MinimalBinaryWrite<E>
    + PiWrite<E>
    + PiWebWrite<E>
    + GolombWrite<E>
    + RiceWrite<E>
    + ExpGolombWrite<E>
    + VByteWrite<E>
{
    fn write_code(&mut self, code: Code, value: u64) -> Result<usize> {
        code.write::<E>(self, value)
    }
}
impl<E: Endianness, B> WriteCodes<E> for B where
    B: BitWrite<E>
        + GammaWrite<E>
        + DeltaWrite<E>
        + ZetaWrite<E>
        + OmegaWrite<E>
        + MinimalBinaryWrite<E>
        + PiWrite<E>
        + PiWebWrite<E>
        + GolombWrite<E>
        + RiceWrite<E>
        + ExpGolombWrite<E>
        + VByteWrite<E>
{
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

impl Code {
    /// Read a value with this code from the given bitstream
    pub fn read<E: Endianness>(&self, reader: &mut (impl ReadCodes<E> + ?Sized)) -> Result<u64> {
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

    /// Write a value with this code con the given bitsteam
    pub fn write<E: Endianness>(
        &self,
        writer: &mut (impl WriteCodes<E> + ?Sized),
        value: u64,
    ) -> Result<usize> {
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

    #[inline]
    /// Compute how many bits it takes to encode a value with this code.
    pub fn len(&self, value: u64) -> usize {
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

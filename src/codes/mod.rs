/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

/*!

Traits for reading and writing instantaneous codes.

This modules contains code for reading and writing instantaneous codes. Each
code is implemented as a pair of traits for reading and writing (e.g., [`GammaRead`]
and [`GammaWrite`]). The traits for reading depend on the trait [`BitRead`](crate::traits::BitRead), whereas
the traits for writing depend on the trait [`BitWrite`](crate::traits::BitWrite).

Some codes have associated decoding tables that are generated by a Python script. In this case,
there are parametric traits (e.g., [`GammaReadParam`] and [`GammaWriteParam`]) with methods
that let you specify whether to use a table or not.

All this logic is hidden by the traits [`table_params::ReadParams`] and [`table_params::WriteParams`],
which contains decision for the Boolean parameters.
[`BufBitReader`](crate::impls::BufBitReader) and [`BufBitWriter`](crate::impls::BufBitWriter) have a
parameter `DC` that is a type implementing [`table_params::ReadParams`] or [`table_params::WriteParams`],
and use by default the type [`table_params::DefaultReadParams`] or [`table_params::DefaultWriteParams`],
which contains reasonable default choices.

*/

// Available codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum Code {
    Unary,
    Gamma,
    Delta,
    Zeta { k: u64 },
    Golomb { b: u64 },
    SkewedGolomb { b: u64 },
    MinimalBinary { k: u64 },
    Nibble,
}

pub mod table_params;

mod gamma;

pub use gamma::{
    len_gamma, len_gamma_param, GammaRead, GammaReadParam, GammaWrite, GammaWriteParam,
};

mod delta;
pub use delta::{
    len_delta, len_delta_param, DeltaRead, DeltaReadParam, DeltaWrite, DeltaWriteParam,
};

mod minimal_binary;
pub use minimal_binary::{len_minimal_binary, MinimalBinaryRead, MinimalBinaryWrite};

mod zeta;
pub use zeta::{len_zeta, len_zeta_param, ZetaRead, ZetaReadParam, ZetaWrite, ZetaWriteParam};

pub mod delta_tables;
pub mod gamma_tables;
pub mod unary_tables;
pub mod zeta_tables;

mod stats;
pub use stats::*;

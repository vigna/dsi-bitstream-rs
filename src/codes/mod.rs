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
reading depend on [`BitRead`](crate::traits::BitRead), whereas
the traits for writing depend on [`BitWrite`](crate::traits::BitWrite).

The traits ending with `Param` make it possible to specify parameters—for
example, whether to use decoding tables. Usually, one whould instead pull
in scope non-parametric traits such as [`GammaRead`] and [`GammaWrite`],
for which defaults are provided using the mechanism described in the
[`params`] module.

Note that if you are using decoding tables, you must ensure that the
[`peek_bits`](crate::traits::BitRead::peek_bits) method of your
[`BitRead`](crate::traits::BitRead) implementation returns a sufficient
number of bits: if it does not, an assertion will be triggered in test
mode, but behavior will be unpredictable otherwise. This is unfortunately
difficult to check statically. To stay on the safe side, we recommend
to use a read word of at least 16 bits.

*/

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

pub mod golomb;
pub use golomb::{len_golomb, GolombRead, GolombWrite};

pub mod rice;
pub use rice::{len_rice, RiceRead, RiceWrite};

pub mod exp_golomb;
pub use exp_golomb::{len_exp_golomb, ExpGolombRead, ExpGolombWrite};

pub mod vbyte;
pub use vbyte::{len_vbyte, VByteRead, VByteWrite};

pub mod delta_tables;
pub mod gamma_tables;
pub mod zeta_tables;

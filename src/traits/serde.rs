/*
 * SPDX-FileCopyrightText: 2023 Inria
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 */

use super::Endianness;
use crate::prelude::{ReadCodes, WriteCodes};
use anyhow::Result;

pub trait BitSerializable {
    /// Write self to a bit stream that provides code implementations.
    ///
    /// Return the number of bits written.
    fn serialize<E: Endianness, B: WriteCodes<E>>(&self, bitstream: &mut B) -> Result<usize>;
}

pub trait BitDeserializable {
    /// The type returned by the deserialization.
    type DeserType;
    /// Read a value of type [`BitDeserializable::DeserType`] from a bit stream that provides code implementations.
    fn deserialize<E: Endianness, B: ReadCodes<E>>(bitstream: &mut B) -> Result<Self::DeserType>;
}

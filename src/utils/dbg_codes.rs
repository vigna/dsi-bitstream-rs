/*
 * SPDX-FileCopyrightText: 2023 Inria
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use crate::prelude::*;
use crate::traits::*;
use anyhow::Result;

/// A wrapper over a code reader that prints on stdout all the codes read
pub struct DbgBitReader<E: Endianness, CR: ReadCodes<E>> {
    reader: CR,
    _marker: core::marker::PhantomData<E>,
}

impl<E: Endianness, CR: ReadCodes<E>> DbgBitReader<E, CR> {
    pub fn new(cr: CR) -> Self {
        Self {
            reader: cr,
            _marker: Default::default(),
        }
    }
}

impl<E: Endianness, CR: ReadCodes<E>> BitRead<E> for DbgBitReader<E, CR>
where
    CR::PeekWord: core::fmt::Display,
{
    type PeekWord = CR::PeekWord;

    fn peek_bits(&mut self, n_bits: usize) -> Result<Self::PeekWord> {
        let value = self.reader.peek_bits(n_bits)?;
        println!("peek_bits({}): {}", n_bits, value);
        Ok(value)
    }
    fn skip_bits(&mut self, n_bits: usize) -> Result<()> {
        println!("skip_bits({})", n_bits);
        self.reader.skip_bits(n_bits)
    }
    fn read_bits(&mut self, n_bits: usize) -> Result<u64> {
        let value = self.reader.read_bits(n_bits)?;
        println!("read_bits({}): {}", n_bits, value);
        Ok(value)
    }
    fn read_unary_param<const USE_TABLE: bool>(&mut self) -> Result<u64> {
        let value = self.reader.read_unary_param::<USE_TABLE>()?;
        println!("{{U<{}>:{}}}", USE_TABLE, value);
        Ok(value)
    }
    fn read_unary(&mut self) -> Result<u64> {
        let value = self.reader.read_unary()?;
        println!("{{U:{}}}", value);
        Ok(value)
    }
}

impl<E: Endianness, CR: ReadCodes<E>> GammaRead<E> for DbgBitReader<E, CR>
where
    CR::PeekWord: core::fmt::Display,
{
    fn read_gamma(&mut self) -> Result<u64> {
        let value = self.reader.read_gamma()?;
        println!("{{g:{}}}", value);
        Ok(value)
    }

    fn skip_gamma(&mut self) -> Result<()> {
        self.reader.skip_gamma()?;
        println!("{{skip g}}");
        Ok(())
    }
}

impl<E: Endianness, CR: ReadCodes<E>> DeltaRead<E> for DbgBitReader<E, CR>
where
    CR::PeekWord: core::fmt::Display,
{
    fn read_delta(&mut self) -> Result<u64> {
        let value = self.reader.read_delta()?;
        println!("{{d:{}}}", value);
        Ok(value)
    }

    fn skip_delta(&mut self) -> Result<()> {
        self.reader.skip_delta()?;
        println!("{{skip d}}");
        Ok(())
    }
}

impl<E: Endianness, CR: ReadCodes<E>> ZetaRead<E> for DbgBitReader<E, CR>
where
    CR::PeekWord: core::fmt::Display,
{
    fn read_zeta3(&mut self) -> Result<u64> {
        let value = self.reader.read_zeta3()?;
        println!("{{z3:{}}}", value);
        Ok(value)
    }

    fn skip_zeta3(&mut self) -> Result<()> {
        self.reader.skip_zeta3()?;
        println!("{{skip z3}}");
        Ok(())
    }

    fn read_zeta(&mut self, k: u64) -> Result<u64> {
        let value = self.reader.read_zeta(k)?;
        println!("{{z{}:{}}}", k, value);
        Ok(value)
    }

    fn skip_zeta(&mut self, k: u64) -> Result<()> {
        self.reader.skip_zeta(k)?;
        println!("{{skip z {}}}", k);
        Ok(())
    }
}

/// A wrapper over a code writer that prints on stdout all the codes written
pub struct DbgBitWriter<E: Endianness, CW: WriteCodes<E>> {
    writer: CW,
    _marker: core::marker::PhantomData<E>,
}

impl<E: Endianness, CW: WriteCodes<E>> DbgBitWriter<E, CW> {
    pub fn new(cw: CW) -> Self {
        Self {
            writer: cw,
            _marker: Default::default(),
        }
    }
}

impl<E: Endianness, CW: WriteCodes<E>> BitWrite<E> for DbgBitWriter<E, CW> {
    fn write_bits(&mut self, value: u64, n_bits: usize) -> Result<usize> {
        println!("write_bits({}, {})", value, n_bits);
        self.writer.write_bits(value, n_bits)
    }
    fn write_unary_param<const USE_TABLE: bool>(&mut self, value: u64) -> Result<usize> {
        println!("{{U<{}>:{}}}", USE_TABLE, value);
        self.writer.write_unary(value)
    }
    fn write_unary(&mut self, value: u64) -> Result<usize> {
        println!("{{U:{}}}", value);
        self.writer.write_unary(value)
    }
    fn flush(self) -> Result<()> {
        self.writer.flush()
    }
}

impl<E: Endianness, CW: WriteCodes<E>> GammaWrite<E> for DbgBitWriter<E, CW> {
    fn write_gamma(&mut self, value: u64) -> Result<usize> {
        println!("{{g:{}}}", value);
        self.writer.write_gamma(value)
    }
}

impl<E: Endianness, CW: WriteCodes<E>> DeltaWrite<E> for DbgBitWriter<E, CW> {
    fn write_delta(&mut self, value: u64) -> Result<usize> {
        println!("{{d:{}}}", value);
        self.writer.write_delta(value)
    }
}

impl<E: Endianness, CW: WriteCodes<E>> ZetaWrite<E> for DbgBitWriter<E, CW> {
    fn write_zeta(&mut self, value: u64, k: u64) -> Result<usize> {
        println!("{{z{}:{}}}", value, k);
        self.writer.write_zeta(value, k)
    }
    fn write_zeta3(&mut self, value: u64) -> Result<usize> {
        println!("{{z3:{}}}", value);
        self.writer.write_zeta3(value)
    }
}

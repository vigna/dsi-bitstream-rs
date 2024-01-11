/*
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use crate::prelude::*;
use crate::traits::*;

/// A wrapper over a [`CodeRead`] that report on standard error all codes read.
pub struct DbgCodeReader<E: Endianness, CR: CodeRead<E>> {
    reader: CR,
    _marker: core::marker::PhantomData<E>,
}

impl<E: Endianness, CR: CodeRead<E>> DbgCodeReader<E, CR> {
    pub fn new(cr: CR) -> Self {
        Self {
            reader: cr,
            _marker: Default::default(),
        }
    }
}

impl<E: Endianness, CR: CodeRead<E>> BitRead<E> for DbgCodeReader<E, CR>
where
    CR::PeekWord: core::fmt::Display,
{
    type Error = CR::Error;
    type PeekWord = CR::PeekWord;

    fn peek_bits(&mut self, n_bits: usize) -> Result<Self::PeekWord, Self::Error> {
        let value = self.reader.peek_bits(n_bits)?;
        eprintln!("peek_bits({}): {}", n_bits, value);
        Ok(value)
    }
    fn skip_bits(&mut self, n_bits: usize) -> Result<(), Self::Error> {
        eprintln!("skip_bits({})", n_bits);
        self.reader.skip_bits(n_bits)
    }
    fn read_bits(&mut self, n_bits: usize) -> Result<u64, Self::Error> {
        let value = self.reader.read_bits(n_bits)?;
        eprintln!("read_bits({}): {}", n_bits, value);
        Ok(value)
    }
    fn read_unary_param<const USE_TABLE: bool>(&mut self) -> Result<u64, Self::Error> {
        let value = self.reader.read_unary_param::<USE_TABLE>()?;
        eprintln!("{{U<{}>:{}}}", USE_TABLE, value);
        Ok(value)
    }
    fn read_unary(&mut self) -> Result<u64, Self::Error> {
        let value = self.reader.read_unary()?;
        eprintln!("{{U:{}}}", value);
        Ok(value)
    }
}

impl<E: Endianness, CR: CodeRead<E>> GammaRead<E> for DbgCodeReader<E, CR>
where
    CR::PeekWord: core::fmt::Display,
{
    fn read_gamma(&mut self) -> Result<u64, CR::Error> {
        let value = self.reader.read_gamma()?;
        eprintln!("{{g:{}}}", value);
        Ok(value)
    }

    fn skip_gamma(&mut self) -> Result<(), CR::Error> {
        self.reader.skip_gamma()?;
        eprintln!("{{skip g}}");
        Ok(())
    }
}

impl<E: Endianness, CR: CodeRead<E>> DeltaRead<E> for DbgCodeReader<E, CR>
where
    CR::PeekWord: core::fmt::Display,
{
    fn read_delta(&mut self) -> Result<u64, CR::Error> {
        let value = self.reader.read_delta()?;
        eprintln!("{{d:{}}}", value);
        Ok(value)
    }

    fn skip_delta(&mut self) -> Result<(), CR::Error> {
        self.reader.skip_delta()?;
        eprintln!("{{skip d}}");
        Ok(())
    }
}

impl<E: Endianness, CR: CodeRead<E>> ZetaRead<E> for DbgCodeReader<E, CR>
where
    CR::PeekWord: core::fmt::Display,
{
    fn read_zeta3(&mut self) -> Result<u64, CR::Error> {
        let value = self.reader.read_zeta3()?;
        eprintln!("{{z3:{}}}", value);
        Ok(value)
    }

    fn skip_zeta3(&mut self) -> Result<(), CR::Error> {
        self.reader.skip_zeta3()?;
        eprintln!("{{skip z3}}");
        Ok(())
    }

    fn read_zeta(&mut self, k: u64) -> Result<u64, CR::Error> {
        let value = self.reader.read_zeta(k)?;
        eprintln!("{{z{}:{}}}", k, value);
        Ok(value)
    }

    fn skip_zeta(&mut self, k: u64) -> Result<(), CR::Error> {
        self.reader.skip_zeta(k)?;
        eprintln!("{{skip z {}}}", k);
        Ok(())
    }
}

/// A wrapper over a [`CodeWrite`] that report on standard error all codes written.
pub struct DbgCodeWriter<E: Endianness, CW: CodeWrite<E>> {
    writer: CW,
    _marker: core::marker::PhantomData<E>,
}

impl<E: Endianness, CW: CodeWrite<E>> DbgCodeWriter<E, CW> {
    pub fn new(cw: CW) -> Self {
        Self {
            writer: cw,
            _marker: Default::default(),
        }
    }
}

impl<E: Endianness, CW: CodeWrite<E>> BitWrite<E> for DbgCodeWriter<E, CW> {
    type Error = CW::Error;

    fn write_bits(&mut self, value: u64, n_bits: usize) -> Result<usize, Self::Error> {
        eprintln!("write_bits({}, {})", value, n_bits);
        self.writer.write_bits(value, n_bits)
    }
    fn write_unary_param<const USE_TABLE: bool>(
        &mut self,
        value: u64,
    ) -> Result<usize, Self::Error> {
        eprintln!("{{U<{}>:{}}}", USE_TABLE, value);
        self.writer.write_unary(value)
    }
    fn write_unary(&mut self, value: u64) -> Result<usize, Self::Error> {
        eprintln!("{{U:{}}}", value);
        self.writer.write_unary(value)
    }
    fn flush(self) -> Result<(), Self::Error> {
        self.writer.flush()
    }
}

impl<E: Endianness, CW: CodeWrite<E>> GammaWrite<E> for DbgCodeWriter<E, CW> {
    fn write_gamma(&mut self, value: u64) -> Result<usize, CW::Error> {
        eprintln!("{{g:{}}}", value);
        self.writer.write_gamma(value)
    }
}

impl<E: Endianness, CW: CodeWrite<E>> DeltaWrite<E> for DbgCodeWriter<E, CW> {
    fn write_delta(&mut self, value: u64) -> Result<usize, CW::Error> {
        eprintln!("{{d:{}}}", value);
        self.writer.write_delta(value)
    }
}

impl<E: Endianness, CW: CodeWrite<E>> ZetaWrite<E> for DbgCodeWriter<E, CW> {
    fn write_zeta(&mut self, value: u64, k: u64) -> Result<usize, CW::Error> {
        eprintln!("{{z{}:{}}}", value, k);
        self.writer.write_zeta(value, k)
    }
    fn write_zeta3(&mut self, value: u64) -> Result<usize, CW::Error> {
        eprintln!("{{z3:{}}}", value);
        self.writer.write_zeta3(value)
    }
}

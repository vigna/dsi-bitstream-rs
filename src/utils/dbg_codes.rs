/*
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use crate::prelude::*;
use crate::traits::*;

/// A wrapper over a [`BitRead`] that report on standard error all operations performed,
/// including all code reads.
pub struct DbgBitReader<E: Endianness, R> {
    reader: R,
    _marker: core::marker::PhantomData<E>,
}

impl<E: Endianness, R> DbgBitReader<E, R> {
    pub fn new(cr: R) -> Self {
        Self {
            reader: cr,
            _marker: Default::default(),
        }
    }
}

impl<E: Endianness, R: BitRead<E>> BitRead<E> for DbgBitReader<E, R>
where
    R::PeekWord: core::fmt::Display,
{
    type Error = R::Error;
    type PeekWord = R::PeekWord;

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

impl<E: Endianness, R: GammaRead<E>> GammaRead<E> for DbgBitReader<E, R>
where
    R::PeekWord: core::fmt::Display,
{
    fn read_gamma(&mut self) -> Result<u64, R::Error> {
        let value = self.reader.read_gamma()?;
        eprintln!("{{g:{}}}", value);
        Ok(value)
    }

    fn skip_gamma(&mut self) -> Result<(), R::Error> {
        self.reader.skip_gamma()?;
        eprintln!("{{skip g}}");
        Ok(())
    }
}

impl<E: Endianness, R: DeltaRead<E>> DeltaRead<E> for DbgBitReader<E, R>
where
    R::PeekWord: core::fmt::Display,
{
    fn read_delta(&mut self) -> Result<u64, R::Error> {
        let value = self.reader.read_delta()?;
        eprintln!("{{d:{}}}", value);
        Ok(value)
    }

    fn skip_delta(&mut self) -> Result<(), R::Error> {
        self.reader.skip_delta()?;
        eprintln!("{{skip d}}");
        Ok(())
    }
}

impl<E: Endianness, R: ZetaRead<E>> ZetaRead<E> for DbgBitReader<E, R>
where
    R::PeekWord: core::fmt::Display,
{
    fn read_zeta3(&mut self) -> Result<u64, R::Error> {
        let value = self.reader.read_zeta3()?;
        eprintln!("{{z3:{}}}", value);
        Ok(value)
    }

    fn skip_zeta3(&mut self) -> Result<(), R::Error> {
        self.reader.skip_zeta3()?;
        eprintln!("{{skip z3}}");
        Ok(())
    }

    fn read_zeta(&mut self, k: u64) -> Result<u64, R::Error> {
        let value = self.reader.read_zeta(k)?;
        eprintln!("{{z{}:{}}}", k, value);
        Ok(value)
    }

    fn skip_zeta(&mut self, k: u64) -> Result<(), R::Error> {
        self.reader.skip_zeta(k)?;
        eprintln!("{{skip z {}}}", k);
        Ok(())
    }
}

/// A wrapper over a [`BitWrite`] that report on standard error all operations performed,
/// including all code writes.
pub struct DbgBitWriter<E: Endianness, W> {
    writer: W,
    _marker: core::marker::PhantomData<E>,
}

impl<E: Endianness, W> DbgBitWriter<E, W> {
    pub fn new(cw: W) -> Self {
        Self {
            writer: cw,
            _marker: Default::default(),
        }
    }
}

impl<E: Endianness, W: BitWrite<E>> BitWrite<E> for DbgBitWriter<E, W> {
    type Error = W::Error;

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

impl<E: Endianness, W: GammaWrite<E>> GammaWrite<E> for DbgBitWriter<E, W> {
    fn write_gamma(&mut self, value: u64) -> Result<usize, W::Error> {
        eprintln!("{{g:{}}}", value);
        self.writer.write_gamma(value)
    }
}

impl<E: Endianness, W: DeltaWrite<E>> DeltaWrite<E> for DbgBitWriter<E, W> {
    fn write_delta(&mut self, value: u64) -> Result<usize, W::Error> {
        eprintln!("{{d:{}}}", value);
        self.writer.write_delta(value)
    }
}

impl<E: Endianness, W: ZetaWrite<E>> ZetaWrite<E> for DbgBitWriter<E, W> {
    fn write_zeta(&mut self, value: u64, k: u64) -> Result<usize, W::Error> {
        eprintln!("{{z{}:{}}}", value, k);
        self.writer.write_zeta(value, k)
    }
    fn write_zeta3(&mut self, value: u64) -> Result<usize, W::Error> {
        eprintln!("{{z3:{}}}", value);
        self.writer.write_zeta3(value)
    }
}

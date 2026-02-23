/*
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use crate::prelude::*;
use crate::traits::*;
#[cfg(feature = "mem_dbg")]
use mem_dbg::{MemDbg, MemSize};

/// A wrapper over a [`BitRead`] that reports on standard error all
/// operations performed, including all code reads.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
pub struct DbgBitReader<E: Endianness, R> {
    reader: R,
    _marker: core::marker::PhantomData<E>,
}

impl<E: Endianness, R> DbgBitReader<E, R> {
    #[must_use]
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
    const PEEK_BITS: usize = R::PEEK_BITS;

    fn peek_bits(&mut self, n_bits: usize) -> Result<Self::PeekWord, Self::Error> {
        let value = self.reader.peek_bits(n_bits)?;
        #[cfg(feature = "std")]
        eprintln!("peek_bits({}): {}", n_bits, value);
        Ok(value)
    }

    fn skip_bits(&mut self, n_bits: usize) -> Result<(), Self::Error> {
        #[cfg(feature = "std")]
        eprintln!("skip_bits({})", n_bits);
        self.reader.skip_bits(n_bits)
    }

    fn read_bits(&mut self, num_bits: usize) -> Result<u64, Self::Error> {
        let value = self.reader.read_bits(num_bits)?;
        #[cfg(feature = "std")]
        eprintln!("read_bits({}): {}", num_bits, value);
        Ok(value)
    }

    fn read_unary(&mut self) -> Result<u64, Self::Error> {
        let value = self.reader.read_unary()?;
        #[cfg(feature = "std")]
        eprintln!("{{U:{}}}", value);
        Ok(value)
    }

    fn skip_bits_after_peek(&mut self, n: usize) {
        self.reader.skip_bits_after_peek(n)
    }
}

impl<E: Endianness, R: GammaRead<E>> GammaRead<E> for DbgBitReader<E, R>
where
    R::PeekWord: core::fmt::Display,
{
    fn read_gamma(&mut self) -> Result<u64, R::Error> {
        let value = self.reader.read_gamma()?;
        #[cfg(feature = "std")]
        eprintln!("{{g:{}}}", value);
        Ok(value)
    }
}

impl<E: Endianness, R: DeltaRead<E>> DeltaRead<E> for DbgBitReader<E, R>
where
    R::PeekWord: core::fmt::Display,
{
    fn read_delta(&mut self) -> Result<u64, R::Error> {
        let value = self.reader.read_delta()?;
        #[cfg(feature = "std")]
        eprintln!("{{d:{}}}", value);
        Ok(value)
    }
}

impl<E: Endianness, R: ZetaRead<E>> ZetaRead<E> for DbgBitReader<E, R>
where
    R::PeekWord: core::fmt::Display,
{
    fn read_zeta3(&mut self) -> Result<u64, R::Error> {
        let value = self.reader.read_zeta3()?;
        #[cfg(feature = "std")]
        eprintln!("{{z3:{}}}", value);
        Ok(value)
    }

    fn read_zeta(&mut self, k: usize) -> Result<u64, R::Error> {
        let value = self.reader.read_zeta(k)?;
        #[cfg(feature = "std")]
        eprintln!("{{z{}:{}}}", k, value);
        Ok(value)
    }
}

impl<E: Endianness, R: OmegaRead<E>> OmegaRead<E> for DbgBitReader<E, R>
where
    R::PeekWord: core::fmt::Display,
{
    fn read_omega(&mut self) -> Result<u64, R::Error> {
        let value = self.reader.read_omega()?;
        #[cfg(feature = "std")]
        eprintln!("{{o:{}}}", value);
        Ok(value)
    }
}

impl<E: Endianness, R: PiRead<E>> PiRead<E> for DbgBitReader<E, R>
where
    R::PeekWord: core::fmt::Display,
{
    fn read_pi(&mut self, k: usize) -> Result<u64, R::Error> {
        let value = self.reader.read_pi(k)?;
        #[cfg(feature = "std")]
        eprintln!("{{p{}:{}}}", k, value);
        Ok(value)
    }

    fn read_pi2(&mut self) -> Result<u64, R::Error> {
        let value = self.reader.read_pi2()?;
        #[cfg(feature = "std")]
        eprintln!("{{p2:{}}}", value);
        Ok(value)
    }
}

/// A wrapper over a [`BitWrite`] that reports on standard error all operations performed,
/// including all code writes.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
pub struct DbgBitWriter<E: Endianness, W> {
    writer: W,
    _marker: core::marker::PhantomData<E>,
}

impl<E: Endianness, W> DbgBitWriter<E, W> {
    #[must_use]
    pub fn new(cw: W) -> Self {
        Self {
            writer: cw,
            _marker: Default::default(),
        }
    }
}

impl<E: Endianness, W: BitWrite<E>> BitWrite<E> for DbgBitWriter<E, W> {
    type Error = W::Error;

    fn write_bits(&mut self, value: u64, num_bits: usize) -> Result<usize, Self::Error> {
        #[cfg(feature = "std")]
        eprintln!("write_bits({}, {})", value, num_bits);
        self.writer.write_bits(value, num_bits)
    }

    fn write_unary(&mut self, n: u64) -> Result<usize, Self::Error> {
        #[cfg(feature = "std")]
        eprintln!("{{U:{}}}", n);
        self.writer.write_unary(n)
    }

    fn flush(&mut self) -> Result<usize, Self::Error> {
        self.writer.flush()
    }
}

impl<E: Endianness, W: GammaWrite<E>> GammaWrite<E> for DbgBitWriter<E, W> {
    fn write_gamma(&mut self, n: u64) -> Result<usize, W::Error> {
        #[cfg(feature = "std")]
        eprintln!("{{g:{}}}", n);
        self.writer.write_gamma(n)
    }
}

impl<E: Endianness, W: DeltaWrite<E>> DeltaWrite<E> for DbgBitWriter<E, W> {
    fn write_delta(&mut self, n: u64) -> Result<usize, W::Error> {
        #[cfg(feature = "std")]
        eprintln!("{{d:{}}}", n);
        self.writer.write_delta(n)
    }
}

impl<E: Endianness, W: ZetaWrite<E>> ZetaWrite<E> for DbgBitWriter<E, W> {
    fn write_zeta(&mut self, n: u64, k: usize) -> Result<usize, W::Error> {
        #[cfg(feature = "std")]
        eprintln!("{{z{}:{}}}", k, n);
        self.writer.write_zeta(n, k)
    }
    fn write_zeta3(&mut self, n: u64) -> Result<usize, W::Error> {
        #[cfg(feature = "std")]
        eprintln!("{{z3:{}}}", n);
        self.writer.write_zeta3(n)
    }
}

impl<E: Endianness, W: OmegaWrite<E>> OmegaWrite<E> for DbgBitWriter<E, W> {
    fn write_omega(&mut self, n: u64) -> Result<usize, W::Error> {
        #[cfg(feature = "std")]
        eprintln!("{{o:{}}}", n);
        self.writer.write_omega(n)
    }
}

impl<E: Endianness, W: PiWrite<E>> PiWrite<E> for DbgBitWriter<E, W> {
    fn write_pi(&mut self, n: u64, k: usize) -> Result<usize, W::Error> {
        #[cfg(feature = "std")]
        eprintln!("{{p{}:{}}}", k, n);
        self.writer.write_pi(n, k)
    }

    fn write_pi2(&mut self, n: u64) -> Result<usize, W::Error> {
        #[cfg(feature = "std")]
        eprintln!("{{p2:{}}}", n);
        self.writer.write_pi2(n)
    }
}

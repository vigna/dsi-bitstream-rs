/*
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use crate::{
    prelude::{
        DeltaRead, DeltaWrite, GammaRead, GammaWrite, OmegaRead, OmegaWrite, PiRead, PiWrite,
        ZetaRead, ZetaWrite, len_delta, len_gamma, len_omega, len_pi, len_zeta,
    },
    traits::*,
};
#[cfg(feature = "mem_dbg")]
use mem_dbg::{MemDbg, MemSize};

/// A wrapper around a [`BitWrite`] that keeps track of the number
/// of bits written and optionally prints on standard error the
/// operations performed on the stream.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
pub struct CountBitWriter<E: Endianness, BW: BitWrite<E>, const PRINT: bool = false> {
    bit_write: BW,
    /// The number of bits written so far on the underlying [`BitWrite`].
    pub bits_written: usize,
    _marker: core::marker::PhantomData<E>,
}

impl<E: Endianness, BW: BitWrite<E>, const PRINT: bool> CountBitWriter<E, BW, PRINT> {
    #[must_use]
    pub const fn new(bit_write: BW) -> Self {
        Self {
            bit_write,
            bits_written: 0,
            _marker: core::marker::PhantomData,
        }
    }

    /// Consumes this writer and returns the underlying [`BitWrite`].
    #[must_use]
    pub fn into_inner(self) -> BW {
        self.bit_write
    }
}

impl<E: Endianness, BW: BitWrite<E>, const PRINT: bool> BitWrite<E>
    for CountBitWriter<E, BW, PRINT>
{
    type Error = <BW as BitWrite<E>>::Error;

    fn write_bits(&mut self, value: u64, num_bits: usize) -> Result<usize, Self::Error> {
        self.bit_write.write_bits(value, num_bits).inspect(|x| {
            self.bits_written += *x;
            if PRINT {
                #[cfg(feature = "std")]
                eprintln!(
                    "write_bits({:#016x}, {}) = {} (total = {})",
                    value, num_bits, x, self.bits_written
                );
            }
        })
    }

    fn write_unary(&mut self, n: u64) -> Result<usize, Self::Error> {
        self.bit_write.write_unary(n).inspect(|x| {
            self.bits_written += *x;
            if PRINT {
                #[cfg(feature = "std")]
                eprintln!("write_unary({}) = {} (total = {})", n, x, self.bits_written);
            }
        })
    }

    fn flush(&mut self) -> Result<usize, Self::Error> {
        self.bit_write.flush().inspect(|x| {
            self.bits_written += *x;
            if PRINT {
                #[cfg(feature = "std")]
                eprintln!("flush() = {} (total = {})", x, self.bits_written);
            }
        })
    }
}

impl<E: Endianness, BW: BitWrite<E> + GammaWrite<E>, const PRINT: bool> GammaWrite<E>
    for CountBitWriter<E, BW, PRINT>
{
    fn write_gamma(&mut self, n: u64) -> Result<usize, BW::Error> {
        self.bit_write.write_gamma(n).inspect(|x| {
            self.bits_written += *x;
            if PRINT {
                #[cfg(feature = "std")]
                eprintln!("write_gamma({}) = {} (total = {})", n, x, self.bits_written);
            }
        })
    }
}

impl<E: Endianness, BW: BitWrite<E> + DeltaWrite<E>, const PRINT: bool> DeltaWrite<E>
    for CountBitWriter<E, BW, PRINT>
{
    fn write_delta(&mut self, n: u64) -> Result<usize, BW::Error> {
        self.bit_write.write_delta(n).inspect(|x| {
            self.bits_written += *x;
            if PRINT {
                #[cfg(feature = "std")]
                eprintln!("write_delta({}) = {} (total = {})", n, x, self.bits_written);
            }
        })
    }
}

impl<E: Endianness, BW: BitWrite<E> + ZetaWrite<E>, const PRINT: bool> ZetaWrite<E>
    for CountBitWriter<E, BW, PRINT>
{
    fn write_zeta(&mut self, n: u64, k: usize) -> Result<usize, BW::Error> {
        self.bit_write.write_zeta(n, k).inspect(|x| {
            self.bits_written += *x;
            if PRINT {
                #[cfg(feature = "std")]
                eprintln!(
                    "write_zeta({}, {}) = {} (total = {})",
                    n, k, x, self.bits_written
                );
            }
        })
    }

    fn write_zeta3(&mut self, n: u64) -> Result<usize, BW::Error> {
        self.bit_write.write_zeta3(n).inspect(|x| {
            self.bits_written += *x;
            if PRINT {
                #[cfg(feature = "std")]
                eprintln!("write_zeta3({}) = {} (total = {})", n, x, self.bits_written);
            }
        })
    }
}

impl<E: Endianness, BW: BitWrite<E> + OmegaWrite<E>, const PRINT: bool> OmegaWrite<E>
    for CountBitWriter<E, BW, PRINT>
{
    fn write_omega(&mut self, n: u64) -> Result<usize, BW::Error> {
        self.bit_write.write_omega(n).inspect(|x| {
            self.bits_written += *x;
            if PRINT {
                #[cfg(feature = "std")]
                eprintln!("write_omega({}) = {} (total = {})", n, x, self.bits_written);
            }
        })
    }
}

impl<E: Endianness, BW: BitWrite<E> + PiWrite<E>, const PRINT: bool> PiWrite<E>
    for CountBitWriter<E, BW, PRINT>
{
    fn write_pi(&mut self, n: u64, k: usize) -> Result<usize, BW::Error> {
        self.bit_write.write_pi(n, k).inspect(|x| {
            self.bits_written += *x;
            if PRINT {
                #[cfg(feature = "std")]
                eprintln!(
                    "write_pi({}, {}) = {} (total = {})",
                    n, k, x, self.bits_written
                );
            }
        })
    }

    fn write_pi2(&mut self, n: u64) -> Result<usize, BW::Error> {
        self.bit_write.write_pi2(n).inspect(|x| {
            self.bits_written += *x;
            if PRINT {
                #[cfg(feature = "std")]
                eprintln!("write_pi2({}) = {} (total = {})", n, x, self.bits_written);
            }
        })
    }
}

impl<E: Endianness, BW: BitWrite<E> + BitSeek, const PRINT: bool> BitSeek
    for CountBitWriter<E, BW, PRINT>
{
    type Error = <BW as BitSeek>::Error;

    fn bit_pos(&mut self) -> Result<u64, Self::Error> {
        self.bit_write.bit_pos()
    }

    fn set_bit_pos(&mut self, bit_pos: u64) -> Result<(), Self::Error> {
        self.bit_write.set_bit_pos(bit_pos)
    }
}

/// A wrapper around a [`BitRead`] that keeps track of the number
/// of bits read and optionally prints on standard error the
/// operations performed on the stream.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
pub struct CountBitReader<E: Endianness, BR: BitRead<E>, const PRINT: bool = false> {
    bit_read: BR,
    /// The number of bits read (or skipped) so far from the underlying [`BitRead`].
    pub bits_read: usize,
    _marker: core::marker::PhantomData<E>,
}

impl<E: Endianness, BR: BitRead<E>, const PRINT: bool> CountBitReader<E, BR, PRINT> {
    #[must_use]
    pub const fn new(bit_read: BR) -> Self {
        Self {
            bit_read,
            bits_read: 0,
            _marker: core::marker::PhantomData,
        }
    }

    /// Consumes this reader and returns the underlying [`BitRead`].
    #[must_use]
    pub fn into_inner(self) -> BR {
        self.bit_read
    }
}

impl<E: Endianness, BR: BitRead<E>, const PRINT: bool> BitRead<E> for CountBitReader<E, BR, PRINT> {
    type Error = <BR as BitRead<E>>::Error;
    type PeekWord = BR::PeekWord;
    const PEEK_BITS: usize = BR::PEEK_BITS;

    fn read_bits(&mut self, num_bits: usize) -> Result<u64, Self::Error> {
        self.bit_read.read_bits(num_bits).inspect(|x| {
            let _ = x;
            self.bits_read += num_bits;
            if PRINT {
                #[cfg(feature = "std")]
                eprintln!(
                    "read_bits({}) = {:#016x} (total = {})",
                    num_bits, x, self.bits_read
                );
            }
        })
    }

    fn read_unary(&mut self) -> Result<u64, Self::Error> {
        self.bit_read.read_unary().inspect(|x| {
            self.bits_read += *x as usize + 1;
            if PRINT {
                #[cfg(feature = "std")]
                eprintln!("read_unary() = {} (total = {})", x, self.bits_read);
            }
        })
    }

    fn peek_bits(&mut self, n_bits: usize) -> Result<Self::PeekWord, Self::Error> {
        self.bit_read.peek_bits(n_bits)
    }

    fn skip_bits(&mut self, n_bits: usize) -> Result<(), Self::Error> {
        self.bits_read += n_bits;
        if PRINT {
            #[cfg(feature = "std")]
            eprintln!("skip_bits({}) (total = {})", n_bits, self.bits_read);
        }
        self.bit_read.skip_bits(n_bits)
    }

    fn skip_bits_after_peek(&mut self, n: usize) {
        self.bit_read.skip_bits_after_peek(n)
    }
}

impl<E: Endianness, BR: BitRead<E> + GammaRead<E>, const PRINT: bool> GammaRead<E>
    for CountBitReader<E, BR, PRINT>
{
    fn read_gamma(&mut self) -> Result<u64, BR::Error> {
        self.bit_read.read_gamma().inspect(|x| {
            self.bits_read += len_gamma(*x);
            if PRINT {
                #[cfg(feature = "std")]
                eprintln!("read_gamma() = {} (total = {})", x, self.bits_read);
            }
        })
    }
}

impl<E: Endianness, BR: BitRead<E> + DeltaRead<E>, const PRINT: bool> DeltaRead<E>
    for CountBitReader<E, BR, PRINT>
{
    fn read_delta(&mut self) -> Result<u64, BR::Error> {
        self.bit_read.read_delta().inspect(|x| {
            self.bits_read += len_delta(*x);
            if PRINT {
                #[cfg(feature = "std")]
                eprintln!("read_delta() = {} (total = {})", x, self.bits_read);
            }
        })
    }
}

impl<E: Endianness, BR: BitRead<E> + ZetaRead<E>, const PRINT: bool> ZetaRead<E>
    for CountBitReader<E, BR, PRINT>
{
    fn read_zeta(&mut self, k: usize) -> Result<u64, BR::Error> {
        self.bit_read.read_zeta(k).inspect(|x| {
            self.bits_read += len_zeta(*x, k);
            if PRINT {
                #[cfg(feature = "std")]
                eprintln!("read_zeta({}) = {} (total = {})", k, x, self.bits_read);
            }
        })
    }

    fn read_zeta3(&mut self) -> Result<u64, BR::Error> {
        self.bit_read.read_zeta3().inspect(|x| {
            self.bits_read += len_zeta(*x, 3);
            if PRINT {
                #[cfg(feature = "std")]
                eprintln!("read_zeta3() = {} (total = {})", x, self.bits_read);
            }
        })
    }
}

impl<E: Endianness, BR: BitRead<E> + OmegaRead<E>, const PRINT: bool> OmegaRead<E>
    for CountBitReader<E, BR, PRINT>
{
    fn read_omega(&mut self) -> Result<u64, BR::Error> {
        self.bit_read.read_omega().inspect(|x| {
            self.bits_read += len_omega(*x);
            if PRINT {
                #[cfg(feature = "std")]
                eprintln!("read_omega() = {} (total = {})", x, self.bits_read);
            }
        })
    }
}

impl<E: Endianness, BR: BitRead<E> + PiRead<E>, const PRINT: bool> PiRead<E>
    for CountBitReader<E, BR, PRINT>
{
    fn read_pi(&mut self, k: usize) -> Result<u64, BR::Error> {
        self.bit_read.read_pi(k).inspect(|x| {
            self.bits_read += len_pi(*x, k);
            if PRINT {
                #[cfg(feature = "std")]
                eprintln!("read_pi({}) = {} (total = {})", k, x, self.bits_read);
            }
        })
    }

    fn read_pi2(&mut self) -> Result<u64, BR::Error> {
        self.bit_read.read_pi2().inspect(|x| {
            self.bits_read += len_pi(*x, 2);
            if PRINT {
                #[cfg(feature = "std")]
                eprintln!("read_pi2() = {} (total = {})", x, self.bits_read);
            }
        })
    }
}

impl<E: Endianness, BR: BitRead<E> + BitSeek, const PRINT: bool> BitSeek
    for CountBitReader<E, BR, PRINT>
{
    type Error = <BR as BitSeek>::Error;

    fn bit_pos(&mut self) -> Result<u64, Self::Error> {
        self.bit_read.bit_pos()
    }

    fn set_bit_pos(&mut self, bit_pos: u64) -> Result<(), Self::Error> {
        self.bit_read.set_bit_pos(bit_pos)
    }
}

#[cfg(test)]
#[cfg(feature = "std")]
mod tests {
    use super::*;
    use crate::prelude::*;

    #[test]
    fn test_count() -> Result<(), Box<dyn core::error::Error + Send + Sync + 'static>> {
        let mut buffer = <Vec<u64>>::new();
        let bit_write = <BufBitWriter<LE, _>>::new(MemWordWriterVec::new(&mut buffer));
        let mut count_bit_write = CountBitWriter::<_, _, true>::new(bit_write);

        count_bit_write.write_unary(5)?;
        assert_eq!(count_bit_write.bits_written, 6);
        count_bit_write.write_unary(100)?;
        assert_eq!(count_bit_write.bits_written, 107);
        count_bit_write.write_bits(1, 20)?;
        assert_eq!(count_bit_write.bits_written, 127);
        count_bit_write.write_bits(1, 33)?;
        assert_eq!(count_bit_write.bits_written, 160);
        count_bit_write.write_gamma(2)?;
        assert_eq!(count_bit_write.bits_written, 163);
        count_bit_write.write_delta(1)?;
        assert_eq!(count_bit_write.bits_written, 167);
        count_bit_write.write_zeta(0, 4)?;
        assert_eq!(count_bit_write.bits_written, 171);
        count_bit_write.write_zeta3(0)?;
        assert_eq!(count_bit_write.bits_written, 174);
        count_bit_write.write_omega(3)?;
        assert_eq!(count_bit_write.bits_written, 174 + len_omega(3));
        let after_omega = count_bit_write.bits_written;
        count_bit_write.write_pi(5, 3)?;
        assert_eq!(count_bit_write.bits_written, after_omega + len_pi(5, 3));
        let after_pi = count_bit_write.bits_written;
        count_bit_write.write_pi2(7)?;
        assert_eq!(count_bit_write.bits_written, after_pi + len_pi(7, 2));
        let after_pi2 = count_bit_write.bits_written;
        count_bit_write.flush()?;
        drop(count_bit_write);

        let bit_read = <BufBitReader<LE, _>>::new(MemWordReader::<u64, _>::new(&buffer));
        let mut count_bit_read = CountBitReader::<_, _, true>::new(bit_read);

        assert_eq!(count_bit_read.peek_bits(5)?, 0);
        assert_eq!(count_bit_read.read_unary()?, 5);
        assert_eq!(count_bit_read.bits_read, 6);
        assert_eq!(count_bit_read.read_unary()?, 100);
        assert_eq!(count_bit_read.bits_read, 107);
        assert_eq!(count_bit_read.read_bits(20)?, 1);
        assert_eq!(count_bit_read.bits_read, 127);
        count_bit_read.skip_bits(33)?;
        assert_eq!(count_bit_read.bits_read, 160);
        assert_eq!(count_bit_read.read_gamma()?, 2);
        assert_eq!(count_bit_read.bits_read, 163);
        assert_eq!(count_bit_read.read_delta()?, 1);
        assert_eq!(count_bit_read.bits_read, 167);
        assert_eq!(count_bit_read.read_zeta(4)?, 0);
        assert_eq!(count_bit_read.bits_read, 171);
        assert_eq!(count_bit_read.read_zeta3()?, 0);
        assert_eq!(count_bit_read.bits_read, 174);
        assert_eq!(count_bit_read.read_omega()?, 3);
        assert_eq!(count_bit_read.bits_read, after_omega);
        assert_eq!(count_bit_read.read_pi(3)?, 5);
        assert_eq!(count_bit_read.bits_read, after_pi);
        assert_eq!(count_bit_read.read_pi2()?, 7);
        assert_eq!(count_bit_read.bits_read, after_pi2);

        Ok(())
    }
}

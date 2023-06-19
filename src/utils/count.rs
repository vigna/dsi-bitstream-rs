/*
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use crate::{
    prelude::{
        len_delta, len_gamma, len_zeta, DeltaRead, DeltaWrite, GammaRead, GammaWrite, ZetaRead,
        ZetaWrite,
    },
    traits::*,
};
use anyhow::Result;

/// Wrapping struct that keep tracks of written bits. Optionally,
/// prints to standard error information about methods called.
#[derive(Debug, Clone)]
pub struct CountBitWrite<E: Endianness, BW: BitWrite<E>, const PRINT: bool = false> {
    bit_write: BW,
    /// The number of bits written so far on the underlying [`BitWrite`].
    pub bits_written: usize,
    _marker: std::marker::PhantomData<E>,
}

impl<E: Endianness, BW: BitWrite<E>, const PRINT: bool> CountBitWrite<E, BW, PRINT> {
    pub fn new(bit_write: BW) -> Self {
        Self {
            bit_write,
            bits_written: 0,
            _marker: std::marker::PhantomData::default(),
        }
    }
}

impl<E: Endianness, BW: BitWrite<E>, const PRINT: bool> BitWrite<E>
    for CountBitWrite<E, BW, PRINT>
{
    fn write_bits(&mut self, value: u64, n_bits: usize) -> Result<usize> {
        self.bit_write.write_bits(value, n_bits).map(|x| {
            self.bits_written += x;
            if PRINT {
                eprintln!(
                    "write_bits({:#016x}, {}) = {} (total = {})",
                    value, n_bits, x, self.bits_written
                );
            }
            x
        })
    }

    fn write_unary_param<const USE_TABLE: bool>(&mut self, value: u64) -> Result<usize> {
        self.bit_write
            .write_unary_param::<USE_TABLE>(value)
            .map(|x| {
                self.bits_written += x;
                if PRINT {
                    eprintln!(
                        "write_unary_param<{}>({}) = {} (total = {})",
                        USE_TABLE, value, x, self.bits_written
                    );
                }
                x
            })
    }

    fn write_unary(&mut self, value: u64) -> Result<usize> {
        self.bit_write.write_unary(value).map(|x| {
            self.bits_written += x;
            if PRINT {
                eprintln!(
                    "write_unary({}) = {} (total = {})",
                    value, x, self.bits_written
                );
            }
            x
        })
    }

    fn flush(self) -> Result<()> {
        self.bit_write.flush()
    }
}

impl<E: Endianness, BW: BitWrite<E> + GammaWrite<E>, const PRINT: bool> GammaWrite<E>
    for CountBitWrite<E, BW, PRINT>
{
    fn write_gamma(&mut self, value: u64) -> Result<usize> {
        self.bit_write.write_gamma(value).map(|x| {
            self.bits_written += x;
            if PRINT {
                eprintln!(
                    "write_gamma({}) = {} (total = {})",
                    value, x, self.bits_written
                );
            }
            x
        })
    }
}

impl<E: Endianness, BW: BitWrite<E> + DeltaWrite<E>, const PRINT: bool> DeltaWrite<E>
    for CountBitWrite<E, BW, PRINT>
{
    fn write_delta(&mut self, value: u64) -> Result<usize> {
        self.bit_write.write_delta(value).map(|x| {
            self.bits_written += x;
            if PRINT {
                eprintln!(
                    "write_delta({}) = {} (total = {})",
                    value, x, self.bits_written
                );
            }
            x
        })
    }
}

impl<E: Endianness, BW: BitWrite<E> + ZetaWrite<E>, const PRINT: bool> ZetaWrite<E>
    for CountBitWrite<E, BW, PRINT>
{
    fn write_zeta(&mut self, value: u64, k: u64) -> Result<usize> {
        self.bit_write.write_zeta(value, k).map(|x| {
            self.bits_written += x;
            if PRINT {
                eprintln!(
                    "write_zeta({}, {}) = {} (total = {})",
                    value, x, k, self.bits_written
                );
            }
            x
        })
    }

    fn write_zeta3(&mut self, value: u64) -> Result<usize> {
        self.bit_write.write_zeta3(value).map(|x| {
            self.bits_written += x;
            if PRINT {
                eprintln!(
                    "write_zeta({}) = {} (total = {})",
                    value, x, self.bits_written
                );
            }
            x
        })
    }
}

impl<E: Endianness, BR: BitWrite<E> + BitSeek, const PRINT: bool> BitSeek
    for CountBitWrite<E, BR, PRINT>
{
    fn set_pos(&mut self, bit_pos: usize) -> Result<()> {
        self.bit_write.set_pos(bit_pos)
    }

    fn get_pos(&self) -> usize {
        self.bit_write.get_pos()
    }
}

/// Wrapping struct that keep tracks of read bits. Optionally,
/// prints to standard error information about methods called.
#[derive(Debug, Clone)]
pub struct CountBitRead<E: Endianness, BR: BitRead<E>, const PRINT: bool = false> {
    bit_read: BR,
    /// The number of bits read (or skipped) so far from the underlying [`BitRead`].
    pub bits_read: usize,
    _marker: std::marker::PhantomData<E>,
}

impl<E: Endianness, BR: BitRead<E>, const PRINT: bool> CountBitRead<E, BR, PRINT> {
    pub fn new(bit_read: BR) -> Self {
        Self {
            bit_read,
            bits_read: 0,
            _marker: std::marker::PhantomData::default(),
        }
    }
}

impl<E: Endianness, BR: BitRead<E>, const PRINT: bool> BitRead<E> for CountBitRead<E, BR, PRINT> {
    type PeekType = BR::PeekType;
    fn read_bits(&mut self, n_bits: usize) -> Result<u64> {
        self.bit_read.read_bits(n_bits).map(|x| {
            self.bits_read += n_bits;
            if PRINT {
                eprintln!(
                    "read_bits({}) = {:#016x} (total = {})",
                    n_bits, x, self.bits_read
                );
            }
            x
        })
    }

    fn read_unary_param<const USE_TABLE: bool>(&mut self) -> Result<u64> {
        self.bit_read.read_unary_param::<USE_TABLE>().map(|x| {
            self.bits_read += x as usize + 1;
            if PRINT {
                eprintln!(
                    "read_unary_param<{}>() = {} (total = {})",
                    USE_TABLE, x, self.bits_read
                );
            }
            x
        })
    }

    fn read_unary(&mut self) -> Result<u64> {
        self.bit_read.read_unary().map(|x| {
            self.bits_read += x as usize + 1;
            if PRINT {
                eprintln!("read_unary() = {} (total = {})", x, self.bits_read);
            }
            x
        })
    }

    fn peek_bits(&mut self, n_bits: usize) -> Result<Self::PeekType> {
        self.bit_read.peek_bits(n_bits)
    }

    fn skip_bits(&mut self, n_bits: usize) -> Result<()> {
        self.bits_read += n_bits;
        if PRINT {
            eprintln!("skip_bits({}) (total = {})", n_bits, self.bits_read);
        }
        self.bit_read.skip_bits(n_bits)
    }
}

impl<E: Endianness, BR: BitRead<E> + GammaRead<E>, const PRINT: bool> GammaRead<E>
    for CountBitRead<E, BR, PRINT>
{
    fn read_gamma(&mut self) -> Result<u64> {
        self.bit_read.read_gamma().map(|x| {
            self.bits_read += len_gamma(x);
            if PRINT {
                eprintln!("read_gamma() = {} (total = {})", x, self.bits_read);
            }
            x
        })
    }

    fn skip_gamma(&mut self, n: usize) -> Result<usize> {
        self.bit_read.skip_gamma(n).map(|x| {
            self.bits_read += x;
            if PRINT {
                eprintln!("skip_gammas({}) = {} (total = {})", n, x, self.bits_read);
            }
            x
        })
    }
}

impl<E: Endianness, BR: BitRead<E> + DeltaRead<E>, const PRINT: bool> DeltaRead<E>
    for CountBitRead<E, BR, PRINT>
{
    fn read_delta(&mut self) -> Result<u64> {
        self.bit_read.read_delta().map(|x| {
            self.bits_read += len_delta(x);
            if PRINT {
                eprintln!("read_delta() = {} (total = {})", x, self.bits_read);
            }
            x
        })
    }
    fn skip_delta(&mut self, n: usize) -> Result<usize> {
        self.bit_read.skip_delta(n).map(|x| {
            self.bits_read += x;
            if PRINT {
                eprintln!("skip_deltas({}) = {} (total = {})", n, x, self.bits_read);
            }
            x
        })
    }
}

impl<E: Endianness, BR: BitRead<E> + ZetaRead<E>, const PRINT: bool> ZetaRead<E>
    for CountBitRead<E, BR, PRINT>
{
    fn read_zeta(&mut self, k: u64) -> Result<u64> {
        self.bit_read.read_zeta(k).map(|x| {
            self.bits_read += len_zeta(x, k);
            if PRINT {
                eprintln!("read_zeta({}) = {} (total = {})", k, x, self.bits_read);
            }
            x
        })
    }

    fn skip_zeta(&mut self, k: u64, n: usize) -> Result<usize> {
        self.bit_read.skip_zeta(k, n).map(|x| {
            self.bits_read += x;
            if PRINT {
                eprintln!(
                    "skip_zeta({}, {}) = {} (total = {})",
                    k, n, x, self.bits_read
                );
            }
            x
        })
    }

    fn read_zeta3(&mut self) -> Result<u64> {
        self.bit_read.read_zeta3().map(|x| {
            self.bits_read += len_zeta(x, 3);
            if PRINT {
                eprintln!("read_zeta3() = {} (total = {})", x, self.bits_read);
            }
            x
        })
    }

    fn skip_zeta3(&mut self, n: usize) -> Result<usize> {
        self.bit_read.skip_zeta3(n).map(|x| {
            self.bits_read += x;
            if PRINT {
                eprintln!("skip_deltas({}) = {} (total = {})", n, x, self.bits_read);
            }
            x
        })
    }
}

impl<E: Endianness, BR: BitRead<E> + BitSeek, const PRINT: bool> BitSeek
    for CountBitRead<E, BR, PRINT>
{
    fn set_pos(&mut self, bit_pos: usize) -> Result<()> {
        self.bit_read.set_pos(bit_pos)
    }

    fn get_pos(&self) -> usize {
        self.bit_read.get_pos()
    }
}

#[cfg(test)]
#[test]
fn test_count() -> Result<()> {
    use crate::prelude::*;
    let mut buffer = <Vec<u64>>::new();
    let bit_write = <BufferedBitStreamWrite<LE, _>>::new(MemWordWriteVec::new(&mut buffer));
    let mut count_bit_write = CountBitWrite::<_, _, true>::new(bit_write);

    count_bit_write.write_unary(5)?;
    assert_eq!(count_bit_write.bits_written, 6);
    count_bit_write.write_unary_param::<true>(100)?;
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
    count_bit_write.flush()?;

    let bit_read =
        <BufferedBitStreamRead<LE, u64, _>>::new(MemWordReadInfinite::<u64, _>::new(&buffer));
    let mut count_bit_read = CountBitRead::<_, _, true>::new(bit_read);

    assert_eq!(count_bit_read.peek_bits(5)?, 0);
    assert_eq!(count_bit_read.read_unary()?, 5);
    assert_eq!(count_bit_read.bits_read, 6);
    assert_eq!(count_bit_read.read_unary_param::<true>()?, 100);
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

    Ok(())
}

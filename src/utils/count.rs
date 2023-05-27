/*
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use crate::traits::*;
use anyhow::Result;

/// Wrapping struct that keep tracks of written bits. Optionally,
/// prints to standard error information about methods called.
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
            _marker: std::marker::PhantomData,
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

/// Wrapping struct that keep tracks of read bits. Optionally,
/// prints to standard error information about methods called.
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
            _marker: std::marker::PhantomData,
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

    Ok(())
}

/*
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use crate::{
    prelude::{
        len_delta, len_exp_golomb, len_gamma, len_golomb, len_omega, len_pi, len_pi_web, len_rice,
        len_vbyte, len_zeta, DeltaRead, DeltaWrite, ExpGolombRead, ExpGolombWrite, GammaRead,
        GammaWrite, GolombRead, GolombWrite, OmegaRead, OmegaWrite, PiRead, PiWebRead, PiWebWrite,
        PiWrite, RiceRead, RiceWrite, VByteRead, VByteWrite, ZetaRead, ZetaWrite,
    },
    traits::*,
};
#[cfg(feature = "mem_dbg")]
use mem_dbg::{MemDbg, MemSize};

/// A wrapper around a [`BitWrite`] that keeps track of the number of
/// bits written and optionally prints on standard error the operations performed on the stream.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
pub struct CountBitWriter<E: Endianness, BW: BitWrite<E>, const PRINT: bool = false> {
    bit_write: BW,
    /// The number of bits written so far on the underlying [`BitWrite`].
    pub bits_written: usize,
    _marker: core::marker::PhantomData<E>,
}

impl<E: Endianness, BW: BitWrite<E>, const PRINT: bool> CountBitWriter<E, BW, PRINT> {
    pub fn new(bit_write: BW) -> Self {
        Self {
            bit_write,
            bits_written: 0,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<E: Endianness, BW: BitWrite<E>, const PRINT: bool> BitWrite<E>
    for CountBitWriter<E, BW, PRINT>
{
    type Error = <BW as BitWrite<E>>::Error;

    fn write_bits(&mut self, value: u64, n_bits: usize) -> Result<usize, Self::Error> {
        self.bit_write.write_bits(value, n_bits).inspect(|x| {
            self.bits_written += *x;
            if PRINT {
                eprintln!(
                    "write_bits({:#016x}, {}) = {} (total = {})",
                    value, n_bits, x, self.bits_written
                );
            }
        })
    }

    fn write_unary(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.bit_write.write_unary(value).inspect(|x| {
            self.bits_written += *x;
            if PRINT {
                eprintln!(
                    "write_unary({}) = {} (total = {})",
                    value, x, self.bits_written
                );
            }
        })
    }

    fn flush(&mut self) -> Result<usize, Self::Error> {
        self.bit_write.flush().inspect(|x| {
            self.bits_written += *x;
            if PRINT {
                eprintln!("flush() = {} (total = {})", x, self.bits_written);
            }
        })
    }
}

impl<E: Endianness, BW: BitWrite<E> + GammaWrite<E>, const PRINT: bool> GammaWrite<E>
    for CountBitWriter<E, BW, PRINT>
{
    fn write_gamma(&mut self, value: u64) -> Result<usize, BW::Error> {
        self.bit_write.write_gamma(value).inspect(|x| {
            self.bits_written += *x;
            if PRINT {
                eprintln!(
                    "write_gamma({}) = {} (total = {})",
                    value, x, self.bits_written
                );
            }
        })
    }
}

impl<E: Endianness, BW: BitWrite<E> + DeltaWrite<E>, const PRINT: bool> DeltaWrite<E>
    for CountBitWriter<E, BW, PRINT>
{
    fn write_delta(&mut self, value: u64) -> Result<usize, BW::Error> {
        self.bit_write.write_delta(value).inspect(|x| {
            self.bits_written += *x;
            if PRINT {
                eprintln!(
                    "write_delta({}) = {} (total = {})",
                    value, x, self.bits_written
                );
            }
        })
    }
}

impl<E: Endianness, BW: BitWrite<E> + PiWebWrite<E>, const PRINT: bool> PiWebWrite<E>
    for CountBitWriter<E, BW, PRINT>
{
    fn write_pi_web(&mut self, value: u64, k: u64) -> Result<usize, BW::Error> {
        self.bit_write.write_pi_web(value, k).inspect(|x| {
            self.bits_written += *x;
            if PRINT {
                eprintln!(
                    "write_pi_web({}, {}) = {} (total = {})",
                    value, k, x, self.bits_written
                );
            }
        })
    }
}

impl<E: Endianness, BW: BitWrite<E> + PiWrite<E>, const PRINT: bool> PiWrite<E>
    for CountBitWriter<E, BW, PRINT>
{
    fn write_pi(&mut self, value: u64, k: u64) -> Result<usize, BW::Error> {
        self.bit_write.write_pi(value, k).inspect(|x| {
            self.bits_written += *x;
            if PRINT {
                eprintln!(
                    "write_pi({}, {}) = {} (total = {})",
                    value, k, x, self.bits_written
                );
            }
        })
    }

    fn write_pi2(&mut self, value: u64) -> Result<usize, BW::Error> {
        self.bit_write.write_pi2(value).inspect(|x| {
            self.bits_written += *x;
            if PRINT {
                eprintln!(
                    "write_pi2({}) = {} (total = {})",
                    value, x, self.bits_written
                );
            }
        })
    }
}

impl<E: Endianness, BW: BitWrite<E> + ZetaWrite<E>, const PRINT: bool> ZetaWrite<E>
    for CountBitWriter<E, BW, PRINT>
{
    fn write_zeta(&mut self, value: u64, k: u64) -> Result<usize, BW::Error> {
        self.bit_write.write_zeta(value, k).inspect(|x| {
            self.bits_written += *x;
            if PRINT {
                eprintln!(
                    "write_zeta({}, {}) = {} (total = {})",
                    value, x, k, self.bits_written
                );
            }
        })
    }

    fn write_zeta3(&mut self, value: u64) -> Result<usize, BW::Error> {
        self.bit_write.write_zeta3(value).inspect(|x| {
            self.bits_written += *x;
            if PRINT {
                eprintln!(
                    "write_zeta({}) = {} (total = {})",
                    value, x, self.bits_written
                );
            }
        })
    }
}

impl<E: Endianness, BW: BitWrite<E> + OmegaWrite<E>, const PRINT: bool> OmegaWrite<E>
    for CountBitWriter<E, BW, PRINT>
{
    fn write_omega(&mut self, value: u64) -> Result<usize, BW::Error> {
        self.bit_write.write_omega(value).inspect(|x| {
            self.bits_written += *x;
            if PRINT {
                eprintln!(
                    "write_omega({}) = {} (total = {})",
                    value, x, self.bits_written
                );
            }
        })
    }
}

impl<E: Endianness, BW: BitWrite<E> + VByteWrite<E>, const PRINT: bool> VByteWrite<E>
    for CountBitWriter<E, BW, PRINT>
{
    fn write_vbyte(&mut self, value: u64) -> Result<usize, BW::Error> {
        self.bit_write.write_vbyte(value).inspect(|x| {
            self.bits_written += *x;
            if PRINT {
                eprintln!(
                    "write_vbyte({}) = {} (total = {})",
                    value, x, self.bits_written
                );
            }
        })
    }
}

impl<E: Endianness, BW: BitWrite<E> + GolombWrite<E>, const PRINT: bool> GolombWrite<E>
    for CountBitWriter<E, BW, PRINT>
{
    fn write_golomb(&mut self, value: u64, k: u64) -> Result<usize, BW::Error> {
        self.bit_write.write_golomb(value, k).inspect(|x| {
            self.bits_written += *x;
            if PRINT {
                eprintln!(
                    "write_golomb({}, {}) = {} (total = {})",
                    value, k, x, self.bits_written
                );
            }
        })
    }
}

impl<E: Endianness, BW: BitWrite<E> + ExpGolombWrite<E>, const PRINT: bool> ExpGolombWrite<E>
    for CountBitWriter<E, BW, PRINT>
{
    fn write_exp_golomb(&mut self, value: u64, k: usize) -> Result<usize, BW::Error> {
        self.bit_write.write_exp_golomb(value, k).inspect(|x| {
            self.bits_written += *x;
            if PRINT {
                eprintln!(
                    "write_exp_golomb({}, {}) = {} (total = {})",
                    value, k, x, self.bits_written
                );
            }
        })
    }
}

impl<E: Endianness, BW: BitWrite<E> + RiceWrite<E>, const PRINT: bool> RiceWrite<E>
    for CountBitWriter<E, BW, PRINT>
{
    fn write_rice(&mut self, value: u64, log2_b: usize) -> Result<usize, BW::Error> {
        self.bit_write.write_rice(value, log2_b).inspect(|x| {
            self.bits_written += *x;
            if PRINT {
                eprintln!(
                    "write_rice({}, {}) = {} (total = {})",
                    value, log2_b, x, self.bits_written
                );
            }
        })
    }
}

impl<E: Endianness, BR: BitWrite<E> + BitSeek, const PRINT: bool> BitSeek
    for CountBitWriter<E, BR, PRINT>
{
    type Error = <BR as BitSeek>::Error;

    fn bit_pos(&mut self) -> Result<u64, Self::Error> {
        self.bit_write.bit_pos()
    }

    fn set_bit_pos(&mut self, bit_pos: u64) -> Result<(), Self::Error> {
        self.bit_write.set_bit_pos(bit_pos)
    }
}

/// A wrapper around a [`BitRead`] that keeps track of the number of
/// bits read and optionally prints on standard error the operations performed on the stream.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
pub struct CountBitReader<E: Endianness, BR: BitRead<E>, const PRINT: bool = false> {
    bit_read: BR,
    /// The number of bits read (or skipped) so far from the underlying [`BitRead`].
    pub bits_read: usize,
    _marker: core::marker::PhantomData<E>,
}

impl<E: Endianness, BR: BitRead<E>, const PRINT: bool> CountBitReader<E, BR, PRINT> {
    pub fn new(bit_read: BR) -> Self {
        Self {
            bit_read,
            bits_read: 0,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<E: Endianness, BR: BitRead<E>, const PRINT: bool> BitRead<E> for CountBitReader<E, BR, PRINT> {
    type Error = <BR as BitRead<E>>::Error;
    type PeekWord = BR::PeekWord;

    fn read_bits(&mut self, n_bits: usize) -> Result<u64, Self::Error> {
        self.bit_read.read_bits(n_bits).inspect(|x| {
            self.bits_read += n_bits;
            if PRINT {
                eprintln!(
                    "read_bits({}) = {:#016x} (total = {})",
                    n_bits, x, self.bits_read
                );
            }
        })
    }

    fn read_unary(&mut self) -> Result<u64, Self::Error> {
        self.bit_read.read_unary().inspect(|x| {
            self.bits_read += *x as usize + 1;
            if PRINT {
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
            eprintln!("skip_bits({}) (total = {})", n_bits, self.bits_read);
        }
        self.bit_read.skip_bits(n_bits)
    }

    fn skip_bits_after_table_lookup(&mut self, n: usize) {
        self.bit_read.skip_bits_after_table_lookup(n)
    }
}

impl<E: Endianness, BR: BitRead<E> + GammaRead<E>, const PRINT: bool> GammaRead<E>
    for CountBitReader<E, BR, PRINT>
{
    fn read_gamma(&mut self) -> Result<u64, BR::Error> {
        self.bit_read.read_gamma().inspect(|x| {
            self.bits_read += len_gamma(*x);
            if PRINT {
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
                eprintln!("read_delta() = {} (total = {})", x, self.bits_read);
            }
        })
    }
}

impl<E: Endianness, BR: BitRead<E> + PiWebRead<E>, const PRINT: bool> PiWebRead<E>
    for CountBitReader<E, BR, PRINT>
{
    fn read_pi_web(&mut self, k: u64) -> Result<u64, BR::Error> {
        self.bit_read.read_pi_web(k).inspect(|x| {
            self.bits_read += len_pi_web(*x, k);
            if PRINT {
                eprintln!("len_pi_web({}) = {} (total = {})", k, x, self.bits_read);
            }
        })
    }
}

impl<E: Endianness, BR: BitRead<E> + PiRead<E>, const PRINT: bool> PiRead<E>
    for CountBitReader<E, BR, PRINT>
{
    fn read_pi(&mut self, k: u64) -> Result<u64, BR::Error> {
        self.bit_read.read_pi(k).inspect(|x| {
            self.bits_read += len_pi(*x, k);
            if PRINT {
                eprintln!("read_pi({}) = {} (total = {})", k, x, self.bits_read);
            }
        })
    }

    fn read_pi2(&mut self) -> Result<u64, BR::Error> {
        self.bit_read.read_pi2().inspect(|x| {
            self.bits_read += len_pi(*x, 2);
            if PRINT {
                eprintln!("read_pi2() = {} (total = {})", x, self.bits_read);
            }
        })
    }
}

impl<E: Endianness, BR: BitRead<E> + ZetaRead<E>, const PRINT: bool> ZetaRead<E>
    for CountBitReader<E, BR, PRINT>
{
    fn read_zeta(&mut self, k: u64) -> Result<u64, BR::Error> {
        self.bit_read.read_zeta(k).inspect(|x| {
            self.bits_read += len_zeta(*x, k);
            if PRINT {
                eprintln!("read_zeta({}) = {} (total = {})", k, x, self.bits_read);
            }
        })
    }

    fn read_zeta3(&mut self) -> Result<u64, BR::Error> {
        self.bit_read.read_zeta3().inspect(|x| {
            self.bits_read += len_zeta(*x, 3);
            if PRINT {
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
                eprintln!("read_omega() = {} (total = {})", x, self.bits_read);
            }
        })
    }
}

impl<E: Endianness, BR: BitRead<E> + VByteRead<E>, const PRINT: bool> VByteRead<E>
    for CountBitReader<E, BR, PRINT>
{
    fn read_vbyte(&mut self) -> Result<u64, BR::Error> {
        self.bit_read.read_vbyte().inspect(|x| {
            self.bits_read += len_vbyte(*x);
            if PRINT {
                eprintln!("read_vbyte() = {} (total = {})", x, self.bits_read);
            }
        })
    }
}

impl<E: Endianness, BR: BitRead<E> + GolombRead<E>, const PRINT: bool> GolombRead<E>
    for CountBitReader<E, BR, PRINT>
{
    fn read_golomb(&mut self, b: u64) -> Result<u64, BR::Error> {
        self.bit_read.read_golomb(b).inspect(|x| {
            self.bits_read += len_golomb(*x, b);
            if PRINT {
                eprintln!("read_golomb({}) = {} (total = {})", b, x, self.bits_read);
            }
        })
    }
}

impl<E: Endianness, BR: BitRead<E> + ExpGolombRead<E>, const PRINT: bool> ExpGolombRead<E>
    for CountBitReader<E, BR, PRINT>
{
    fn read_exp_golomb(&mut self, k: usize) -> Result<u64, BR::Error> {
        self.bit_read.read_exp_golomb(k).inspect(|x| {
            self.bits_read += len_exp_golomb(*x, k);
            if PRINT {
                eprintln!(
                    "read_exp_golomb({}) = {} (total = {})",
                    k, x, self.bits_read
                );
            }
        })
    }
}

impl<E: Endianness, BR: BitRead<E> + RiceRead<E>, const PRINT: bool> RiceRead<E>
    for CountBitReader<E, BR, PRINT>
{
    fn read_rice(&mut self, log2_b: usize) -> Result<u64, BR::Error> {
        self.bit_read.read_rice(log2_b).inspect(|x| {
            self.bits_read += len_rice(*x, log2_b);
            if PRINT {
                eprintln!("read_rice({}) = {} (total = {})", log2_b, x, self.bits_read);
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
#[test]
fn test_count() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    use crate::prelude::*;
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

    Ok(())
}

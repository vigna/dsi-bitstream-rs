/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use crate::traits::*;
use anyhow::Result;

/// Trait providing bit-based positional methods.
pub trait BitSeek {
    /// Move the stream cursor so that if we call `read_bits(1)` we will read
    /// the `bit_pos`-th bit in the stream
    ///
    /// # Errors
    /// This function return an error if the specified bit position
    /// is not available
    fn set_pos(&mut self, bit_pos: usize) -> Result<()>;

    #[must_use]
    /// Return the current bit position in the stream
    fn get_pos(&self) -> usize;
}

/// Objects that can read a fixed number of bits and unary codes from a stream
/// of bits. The endianess of the returned bytes HAS TO BE THE NATIVE ONE.
pub trait BitRead<BO: Endianness> {
    /// The type we can read form the stream without advancing.
    /// On buffered readers this is usually half the buffer size.
    type PeekType: UpcastableInto<u64>;
    /// Read `n_bits` bits from the stream and return them in the lowest bits
    ///
    /// # Errors
    /// This function return an error if we cannot read `n_bits`, this usually
    /// happens if we finished the stream.
    fn read_bits(&mut self, n_bits: usize) -> Result<u64>;

    /// Like read_bits but it doesn't seek forward
    ///
    /// # Errors
    /// This function return an error if we cannot read `n_bits`, this usually
    /// happens if we finished the stream.
    fn peek_bits(&mut self, n_bits: usize) -> Result<Self::PeekType>;

    /// Skip n_bits from the stream
    ///
    /// # Errors
    /// Thi function errors if skipping n_bits the underlying streams ends.
    fn skip_bits(&mut self, n_bits: usize) -> Result<()>;

    /// Skip n_bits from the stream after reading from a table.
    /// For unbuffered reads this is just `skip_bits` while
    /// for buffereds reads we know that the bits are already in the
    /// buffer.
    ///
    /// # Errors
    /// This is never supposed to happen.
    #[inline(always)]
    fn skip_bits_after_table_lookup(&mut self, n_bits: usize) -> Result<()> {
        self.skip_bits(n_bits)
    }

    /// Read an unary code
    ///
    /// # Errors
    /// This function return an error if we cannot read the unary code, this
    /// usually happens if we finished the stream.
    fn read_unary_param<const USE_TABLE: bool>(&mut self) -> Result<u64> {
        let mut count = 0;
        loop {
            let bit = self.read_bits(1)?;
            if bit != 0 {
                return Ok(count);
            }
            count += 1;
        }
    }
    /// Read an unary code
    ///
    /// # Errors
    /// This function return an error if we cannot read the unary code, this
    /// usually happens if we finished the stream.
    #[inline(always)]
    fn read_unary(&mut self) -> Result<u64> {
        self.read_unary_param::<false>()
    }

    #[inline(always)]
    fn skip_unary(&mut self) -> Result<()> {
        self.read_unary()?;
        Ok(())
    }
}

/// Objects that can read a fixed number of bits and unary codes from a stream
/// of bits. The endianess of the returned bytes HAS TO BE THE NATIVE ONE.
/// [`BitWrite`] does not depends on [`BitRead`] because on most implementation
/// we will have to write on bytes or words. Thus to be able to write the bits
/// we would have to be able to read them back, thus impling implementing
/// [`BitRead`]. Nothing stops someone to implement both [`BitRead`] and
/// [`BitWrite`] for the same structure
pub trait BitWrite<BO: Endianness> {
    /// Write the lowest `n_bits` of value to the steam and return the number of
    /// bits written.
    ///
    /// # Errors
    /// This function return an error if we cannot write `n_bits`, this usually
    /// happens if we finished the stream.
    fn write_bits(&mut self, value: u64, n_bits: usize) -> Result<usize>;

    /// Write `value` as an unary code to the stream and return the number of
    /// bits written.
    ///
    /// # Errors
    /// This function return an error if we cannot write the unary code, this
    /// usually happens if we finished the stream.
    fn write_unary_param<const USE_TABLE: bool>(&mut self, mut value: u64) -> Result<usize> {
        while value > 0 {
            self.write_bits(0, 1)?;
            value -= 1;
        }
        self.write_bits(1, 1)?;
        Ok((value + 1) as usize)
    }

    /// Write `value` as an unary code to the stream
    ///
    /// # Errors
    /// This function return an error if we cannot write the unary code, this
    /// usually happens if we finished the stream.
    #[inline(always)]
    fn write_unary(&mut self, value: u64) -> Result<usize> {
        self.write_unary_param::<false>(value)
    }

    /// Flushes the buffer, making the bit stream no longer writable.
    fn flush(self) -> Result<()>;
}

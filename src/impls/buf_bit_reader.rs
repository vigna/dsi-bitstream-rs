/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use num_primitive::{PrimitiveInteger, PrimitiveNumber};

use crate::codes::params::{DefaultReadParams, ReadParams};
use crate::traits::*;
#[cfg(feature = "mem_dbg")]
use mem_dbg::{MemDbg, MemSize};

/// An internal shortcut to the double type of the word of a
/// [`WordRead`].
type BB<WR> = <<WR as WordRead>::Word as DoubleType>::DoubleType;

/// An implementation of [`BitRead`] and [`BitSeek`] for a [`WordRead`] and a
/// [`WordSeek`].
///
/// This implementation uses a bit buffer to store bits that are not yet read.
/// The buffer is sized as twice the word size of the underlying [`WordRead`].
/// Typically, the best choice is to have a buffer that is sized as `usize`,
/// which means that the word of the underlying [`WordRead`] should be half of
/// that (i.e., `u32` for a 64-bit architecture). However, results will vary
/// depending on the CPU.
///
/// The peek word is equal to the bit buffer. The value returned
/// by [`peek_bits`](crate::traits::BitRead::peek_bits) contains at least as
/// many bits as the word size plus one (extended with zeros beyond end of
/// stream).
///
/// The convenience functions [`from_path`] and [`from_file`] (requiring the
/// `std` feature) create a [`BufBitReader`] around a buffered file reader.
///
/// This implementation is usually faster than
/// [`BitReader`](crate::impls::BitReader).
///
/// The additional type parameter `RP` is used to select the parameters for the
/// instantaneous codes, but the casual user should be happy with the default
/// value. See [`ReadParams`] for more details.
///
/// For additional flexibility, when the `std` feature is enabled, this
/// structure implements [`std::io::Read`]. Note that because of coherence
/// rules it is not possible to implement [`std::io::Read`] for a generic
/// [`BitRead`].

#[derive(Debug)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
pub struct BufBitReader<E: Endianness, WR: WordRead, RP: ReadParams = DefaultReadParams>
where
    WR::Word: DoubleType,
{
    /// The [`WordRead`] used to fill the buffer.
    backend: WR,
    /// The 2-word bit buffer that is used to read the codes. It is never full,
    /// but it may be empty. Only the upper (BE) or lower (LE)
    /// `bits_in_buffer` bits are valid; the other bits are always zeroes.
    buffer: BB<WR>,
    /// Number of valid upper (BE) or lower (LE) bits in the buffer.
    /// It is always smaller than `BB::<WR>::BITS`.
    bits_in_buffer: usize,
    _marker: core::marker::PhantomData<(E, RP)>,
}

/// Creates a new [`BufBitReader`] with [default read
/// parameters](`DefaultReadParams`) from a file path using the provided
/// endianness and read word.
///
/// # Examples
///
/// ```no_run
/// use dsi_bitstream::prelude::*;
/// let mut reader = buf_bit_reader::from_path::<LE, u32>("data.bin")?;
/// # Ok::<(), Box<dyn core::error::Error>>(())
/// ```
#[cfg(feature = "std")]
pub fn from_path<E: Endianness, W: Word + DoubleType>(
    path: impl AsRef<std::path::Path>,
) -> std::io::Result<
    BufBitReader<E, super::WordAdapter<W, std::io::BufReader<std::fs::File>>, DefaultReadParams>,
>
where
    W::Bytes: Default + AsMut<[u8]>,
{
    Ok(from_file::<E, W>(std::fs::File::open(path)?))
}

/// Creates a new [`BufBitReader`] with [default read
/// parameters](`DefaultReadParams`) from a file using the provided
/// endianness and read word.
///
/// See also [`from_path`] for a version that takes a path.
#[must_use]
#[cfg(feature = "std")]
pub fn from_file<E: Endianness, W: Word + DoubleType>(
    file: std::fs::File,
) -> BufBitReader<E, super::WordAdapter<W, std::io::BufReader<std::fs::File>>, DefaultReadParams>
where
    W::Bytes: Default + AsMut<[u8]>,
{
    BufBitReader::new(super::WordAdapter::new(std::io::BufReader::new(file)))
}

impl<E: Endianness, WR: WordRead + Clone, RP: ReadParams> core::clone::Clone
    for BufBitReader<E, WR, RP>
where
    WR::Word: DoubleType,
{
    fn clone(&self) -> Self {
        Self {
            backend: self.backend.clone(),
            buffer: self.buffer,
            bits_in_buffer: self.bits_in_buffer,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<E: Endianness, WR: WordRead, RP: ReadParams> BufBitReader<E, WR, RP>
where
    WR::Word: DoubleType,
{
    const WORD_BITS: usize = WR::Word::BITS as usize;
    const BUFFER_BITS: usize = BB::<WR>::BITS as usize;

    /// Creates a new [`BufBitReader`] around a [`WordRead`].
    ///
    /// # Examples
    /// ```
    /// use dsi_bitstream::prelude::*;
    /// let words: [u32; 2] = [0x0043b59f, 0xccf16077];
    /// let word_reader = MemWordReader::new_inf(&words);
    /// let mut buf_bit_reader = <BufBitReader<BE, _>>::new(word_reader);
    /// ```
    #[must_use]
    pub const fn new(backend: WR) -> Self {
        Self {
            backend,
            buffer: BB::<WR>::ZERO,
            bits_in_buffer: 0,
            _marker: core::marker::PhantomData,
        }
    }

    /// Consumes this reader and returns the underlying [`WordRead`].
    #[must_use]
    pub fn into_inner(self) -> WR {
        self.backend
    }
}

//
// Big-endian implementation
//

impl<WR: WordRead, RP: ReadParams> BufBitReader<BE, WR, RP>
where
    WR::Word: DoubleType,
{
    /// Ensures that in the buffer there are at least `Self::WORD_BITS` bits to read.
    /// This method can be called only if there are at least
    /// `Self::WORD_BITS` free bits in the buffer.
    #[inline(always)]
    fn refill(&mut self) -> Result<(), <WR as WordRead>::Error> {
        debug_assert!(Self::BUFFER_BITS - self.bits_in_buffer >= Self::WORD_BITS);

        let new_word: BB<WR> = self.backend.read_word()?.to_be().as_double();
        self.bits_in_buffer += Self::WORD_BITS;
        self.buffer |= new_word << (Self::BUFFER_BITS - self.bits_in_buffer);
        Ok(())
    }
    /// Single-word refill for 64-bit words (128-bit buffer), specialized so
    /// that the result and the pre-top-up buffer, which both live entirely
    /// in the high 64 bits of the buffer, are computed in u64 arithmetic
    /// (no cross-half 128-bit shifts).
    ///
    /// Must be called only when `WORD_BITS == 64`, with `w` the word read
    /// for this request and `bits_in_buffer < num_bits <= WORD_BITS`.
    #[inline(always)]
    fn read_bits_refill_word64(
        &mut self,
        num_bits: usize,
        w: WR::Word,
    ) -> Result<u64, <WR as WordRead>::Error> {
        debug_assert!(Self::WORD_BITS == 64);
        let bits = self.bits_in_buffer;
        // High half of buffer | word placed right below the buffered bits.
        // Shifts valid: bits < num_bits <= WORD_BITS = 64, and num_bits >= 1
        // because the in-buffer fast path handled num_bits <= bits.
        let virt_hi: u64 = (self.buffer >> Self::WORD_BITS).as_to::<u64>() | (w.as_u64() >> bits);
        let result = virt_hi >> (Self::WORD_BITS - num_bits);
        // The remaining low WORD_BITS - (num_bits - bits) bits of the word,
        // placed at the top of the high half; double shift as
        // num_bits - bits may equal WORD_BITS.
        let hi = (w << (num_bits - bits - 1)) << 1_u32;
        let mut buffer = hi.as_double() << Self::WORD_BITS;
        let mut new_bits = bits + Self::WORD_BITS - num_bits;
        // Top up with a second word if available: new_bits < WORD_BITS here,
        // so there is always room and the buffer stays short of full. The
        // atomic optional read never consumes past the end of the stream.
        if let Some(w2) = self.backend.read_word_opt() {
            // Shifts valid: WORD_BITS and new_bits are < BUFFER_BITS.
            buffer |= (w2.to_be().as_double() << Self::WORD_BITS) >> new_bits;
            new_bits += Self::WORD_BITS;
        }
        self.buffer = buffer;
        self.bits_in_buffer = new_bits;
        Ok(result)
    }
}

impl<WR: WordRead, RP: ReadParams> BitRead<BE> for BufBitReader<BE, WR, RP>
where
    WR::Word: DoubleType,
{
    type Error = <WR as WordRead>::Error;
    type PeekWord = BB<WR>;
    // We guarantee only half a buffer (one word) of peekable bits, so that a
    // peek needs at most one refill and the buffer is never completely full;
    // this keeps the read/skip/unary hot paths free of full-buffer handling.
    const PEEK_BITS: usize = <WR as WordRead>::Word::BITS as usize;

    #[inline(always)]
    fn peek_bits(&mut self, n_bits: usize) -> Result<Self::PeekWord, Self::Error> {
        debug_assert!(n_bits > 0);
        debug_assert!(n_bits <= Self::PeekWord::BITS as usize);

        // A peek can do at most one refill, otherwise we might lose data
        if n_bits > self.bits_in_buffer {
            self.refill()?;
        }

        debug_assert!(n_bits <= self.bits_in_buffer);

        // Move the n_bits highest bits of the buffer to the lowest
        Ok(self.buffer >> (Self::BUFFER_BITS - n_bits))
    }

    #[inline(always)]
    fn skip_bits_after_peek(&mut self, n_bits: usize) {
        self.bits_in_buffer -= n_bits;
        self.buffer <<= n_bits;
    }

    #[inline]
    fn read_bits(&mut self, mut num_bits: usize) -> Result<u64, Self::Error> {
        debug_assert!(num_bits <= 64);
        debug_assert!(self.bits_in_buffer < Self::BUFFER_BITS);

        // most common path, we just read the buffer
        if num_bits <= self.bits_in_buffer {
            // Valid right shift of BB::<WR>::BITS - num_bits, even when num_bits is zero
            let result: u64 = (self.buffer >> (Self::BUFFER_BITS - num_bits - 1) >> 1_u32).as_to();
            self.bits_in_buffer -= num_bits;
            self.buffer <<= num_bits;
            return Ok(result);
        }
        // Single-word refill path: past the test above, a request of at most
        // WORD_BITS bits always consumes exactly one word, which we can
        // compose with the buffer without the loop and the branches of the
        // general path below.
        if num_bits <= Self::WORD_BITS {
            let bits = self.bits_in_buffer;
            // The word is required in any case, so no peek is needed.
            let w = self.backend.read_word()?.to_be();
            // For 64-bit words (128-bit buffer), the result and the
            // pre-top-up buffer both live entirely in the high 64 bits of
            // the buffer, so the specialized helper uses u64 arithmetic,
            // avoiding cross-half 128-bit shifts. The branch is resolved at
            // compile time; the helper keeps the dead branch's MIR footprint
            // to one statement so that inlining decisions for narrow-word
            // readers are unaffected.
            if Self::WORD_BITS == 64 {
                return self.read_bits_refill_word64(num_bits, w);
            }
            // Place the word right below the buffered bits; the word fits
            // entirely because bits < num_bits <= WORD_BITS, and both
            // shifts are valid as WORD_BITS and bits are < BUFFER_BITS.
            let placed = (w.as_double() << Self::WORD_BITS) >> bits;
            let virt = self.buffer | placed;
            // Valid right shift, even when num_bits is zero
            let result: u64 = (virt >> (Self::BUFFER_BITS - num_bits - 1) >> 1_u32).as_to();
            // The new buffer comes from the placed word alone: bits <
            // num_bits, so all buffered bits are consumed and
            // `self.buffer << num_bits` would be zero; dropping the `|` from
            // this computation shortens the loop-carried dependency chain.
            let mut buffer = placed << num_bits;
            let mut new_bits = bits + Self::WORD_BITS - num_bits;
            // Top up with a second word if available: new_bits < WORD_BITS
            // here, so there is always room and the buffer stays short of
            // full. This halves the number of refills, making the in-buffer
            // test above more predictable. The atomic optional read never
            // consumes past the end of the stream.
            if let Some(w2) = self.backend.read_word_opt() {
                // Shifts valid: WORD_BITS and new_bits are < BUFFER_BITS.
                buffer |= (w2.to_be().as_double() << Self::WORD_BITS) >> new_bits;
                new_bits += Self::WORD_BITS;
            }
            self.buffer = buffer;
            self.bits_in_buffer = new_bits;
            return Ok(result);
        }

        let mut result: u64 =
            (self.buffer >> (Self::BUFFER_BITS - 1 - self.bits_in_buffer) >> 1_u8).as_to();
        num_bits -= self.bits_in_buffer;

        // Directly read to the result without updating the buffer
        while num_bits > Self::WORD_BITS {
            let new_word: u64 = self.backend.read_word()?.to_be().as_u64();
            result = (result << Self::WORD_BITS) | new_word;
            num_bits -= Self::WORD_BITS;
        }

        debug_assert!(num_bits > 0);
        debug_assert!(num_bits <= Self::WORD_BITS);

        // get the final word
        let new_word = self.backend.read_word()?.to_be();
        self.bits_in_buffer = Self::WORD_BITS - num_bits;
        // compose the remaining bits
        let upcast: u64 = new_word.as_u64();
        let final_bits: u64 = upcast >> self.bits_in_buffer;
        result = (result << (num_bits - 1) << 1) | final_bits;
        // and put the rest in the buffer
        self.buffer = (new_word.as_double() << (Self::BUFFER_BITS - self.bits_in_buffer - 1)) << 1;

        Ok(result)
    }

    #[inline]
    fn read_unary(&mut self) -> Result<u64, Self::Error> {
        debug_assert!(self.bits_in_buffer < Self::BUFFER_BITS);

        // count the zeros from the left
        let zeros: usize = self.buffer.leading_zeros() as _;

        // if we encountered a 1 in the bits_in_buffer we can return
        if zeros < self.bits_in_buffer {
            // zeros + 1 <= bits_in_buffer < BUFFER_BITS, so both forms are
            // always valid. On 128-bit buffers a single merged shift avoids a
            // second synthesized wide shift (measured -13% on u64-word
            // read_unary); on 64-bit buffers the two-step shift measured
            // slightly faster, so each width keeps its best form (const-folded).
            if Self::BUFFER_BITS > 64 {
                self.buffer <<= zeros + 1;
            } else {
                self.buffer = self.buffer << zeros << 1;
            }
            self.bits_in_buffer -= zeros + 1;
            return Ok(zeros as u64);
        }

        let mut result: u64 = self.bits_in_buffer as _;

        loop {
            let new_word = self.backend.read_word()?.to_be();

            if new_word != WR::Word::ZERO {
                let zeros: usize = new_word.leading_zeros() as _;
                let mut buffer = new_word.as_double() << (Self::WORD_BITS + zeros) << 1;
                let mut new_bits = Self::WORD_BITS - zeros - 1;
                // Top up with a second word if available: new_bits <
                // WORD_BITS here, so there is always room and the buffer
                // stays short of full. The atomic optional read never
                // consumes past the end of the stream (see read_bits).
                if let Some(w2) = self.backend.read_word_opt() {
                    // Shifts valid: WORD_BITS and new_bits are < BUFFER_BITS.
                    buffer |= (w2.to_be().as_double() << Self::WORD_BITS) >> new_bits;
                    new_bits += Self::WORD_BITS;
                }
                self.buffer = buffer;
                self.bits_in_buffer = new_bits;
                return Ok(result + zeros as u64);
            }
            result += Self::WORD_BITS as u64;
        }
    }

    #[inline]
    fn skip_bits(&mut self, mut n_bits: usize) -> Result<(), Self::Error> {
        debug_assert!(self.bits_in_buffer < Self::BUFFER_BITS);
        // happy case, just shift the buffer
        if n_bits <= self.bits_in_buffer {
            self.bits_in_buffer -= n_bits;
            self.buffer <<= n_bits;
            return Ok(());
        }

        n_bits -= self.bits_in_buffer;

        // skip words as needed
        while n_bits > Self::WORD_BITS {
            let _ = self.backend.read_word()?;
            n_bits -= Self::WORD_BITS;
        }

        // get the final word
        let new_word = self.backend.read_word()?.to_be();
        self.bits_in_buffer = Self::WORD_BITS - n_bits;

        self.buffer = new_word.as_double() << (Self::BUFFER_BITS - 1 - self.bits_in_buffer) << 1;

        Ok(())
    }

    #[cfg(not(feature = "no_copy_impls"))]
    fn copy_to<F: Endianness, W: BitWrite<F>>(
        &mut self,
        bit_write: &mut W,
        mut n: u64,
    ) -> Result<(), CopyError<Self::Error, W::Error>> {
        // Copy from the buffer at most 64 bits at a time, as the buffer
        // can hold more than 64 bits, but write_bits accepts at most 64
        while n > 0 && self.bits_in_buffer > 0 {
            let m = Ord::min(Ord::min(n, 64), self.bits_in_buffer as u64) as usize;
            // The m highest bits of the buffer; m >= 1, so the shift is valid
            let value: u64 = (self.buffer >> (Self::BUFFER_BITS - m)).as_to();
            bit_write
                .write_bits(value, m)
                .map_err(CopyError::WriteError)?;
            // m >= 1, so the two-step shift is valid even when m == BUFFER_BITS
            self.buffer = self.buffer << (m - 1) << 1;
            self.bits_in_buffer -= m;
            n -= m as u64;
        }

        if n == 0 {
            return Ok(());
        }

        // The buffer is empty: copy whole words
        while n > Self::WORD_BITS as u64 {
            bit_write
                .write_bits(
                    self.backend
                        .read_word()
                        .map_err(CopyError::ReadError)?
                        .to_be()
                        .as_u64(),
                    Self::WORD_BITS,
                )
                .map_err(CopyError::WriteError)?;
            n -= Self::WORD_BITS as u64;
        }

        debug_assert!(n > 0);
        // Copy the n highest bits of a final word, and store the remaining
        // bits at the top of the buffer, with zeros below
        let new_word = self
            .backend
            .read_word()
            .map_err(CopyError::ReadError)?
            .to_be();
        self.bits_in_buffer = Self::WORD_BITS - n as usize;
        bit_write
            .write_bits((new_word >> self.bits_in_buffer).as_u64(), n as usize)
            .map_err(CopyError::WriteError)?;
        // n >= 1, so the two-step shift is valid
        self.buffer = (new_word.as_double() << (Self::WORD_BITS + n as usize - 1)) << 1;

        Ok(())
    }
}

impl<WR: WordRead + WordSeek<Error = <WR as WordRead>::Error>, RP: ReadParams> BitSeek
    for BufBitReader<BE, WR, RP>
where
    WR::Word: DoubleType,
{
    type Error = <WR as WordSeek>::Error;

    #[inline]
    fn bit_pos(&mut self) -> Result<u64, Self::Error> {
        Ok(self.backend.word_pos()? * Self::WORD_BITS as u64 - self.bits_in_buffer as u64)
    }

    #[inline]
    fn set_bit_pos(&mut self, bit_index: u64) -> Result<(), Self::Error> {
        self.backend
            .set_word_pos(bit_index / Self::WORD_BITS as u64)?;
        let bit_offset = (bit_index % Self::WORD_BITS as u64) as usize;
        self.buffer = BB::<WR>::ZERO;
        self.bits_in_buffer = 0;
        if bit_offset != 0 {
            let new_word: BB<WR> = self.backend.read_word()?.to_be().as_double();
            self.bits_in_buffer = Self::WORD_BITS - bit_offset;
            self.buffer = new_word << (Self::BUFFER_BITS - self.bits_in_buffer);
        }
        Ok(())
    }
}

//
// Little-endian implementation
//

impl<WR: WordRead, RP: ReadParams> BufBitReader<LE, WR, RP>
where
    WR::Word: DoubleType,
{
    /// Ensures that in the buffer there are at least `Self::WORD_BITS` bits to read.
    /// This method can be called only if there are at least
    /// `Self::WORD_BITS` free bits in the buffer.
    #[inline(always)]
    fn refill(&mut self) -> Result<(), <WR as WordRead>::Error> {
        debug_assert!(Self::BUFFER_BITS - self.bits_in_buffer >= Self::WORD_BITS);

        let new_word: BB<WR> = self.backend.read_word()?.to_le().as_double();
        self.buffer |= new_word << self.bits_in_buffer;
        self.bits_in_buffer += Self::WORD_BITS;
        Ok(())
    }
}

impl<WR: WordRead, RP: ReadParams> BitRead<LE> for BufBitReader<LE, WR, RP>
where
    WR::Word: DoubleType,
{
    type Error = <WR as WordRead>::Error;
    type PeekWord = BB<WR>;
    // We guarantee only half a buffer (one word) of peekable bits, so that a
    // peek needs at most one refill and the buffer is never completely full;
    // this keeps the read/skip/unary hot paths free of full-buffer handling.
    const PEEK_BITS: usize = <WR as WordRead>::Word::BITS as usize;

    #[inline(always)]
    fn peek_bits(&mut self, n_bits: usize) -> Result<Self::PeekWord, Self::Error> {
        debug_assert!(n_bits > 0);
        debug_assert!(n_bits <= Self::PeekWord::BITS as usize);

        // A peek can do at most one refill, otherwise we might lose data
        if n_bits > self.bits_in_buffer {
            self.refill()?;
        }

        debug_assert!(n_bits <= self.bits_in_buffer);

        // Keep the n_bits lowest bits of the buffer
        let shamt = Self::BUFFER_BITS - n_bits;
        Ok((self.buffer << shamt) >> shamt)
    }

    #[inline(always)]
    fn skip_bits_after_peek(&mut self, n_bits: usize) {
        self.bits_in_buffer -= n_bits;
        self.buffer >>= n_bits;
    }

    #[inline]
    fn read_bits(&mut self, mut num_bits: usize) -> Result<u64, Self::Error> {
        debug_assert!(num_bits <= 64);
        debug_assert!(self.bits_in_buffer < Self::BUFFER_BITS);

        // most common path, we just read the buffer
        if num_bits <= self.bits_in_buffer {
            let result: u64 = (self.buffer & ((BB::<WR>::ONE << num_bits) - BB::<WR>::ONE)).as_to();
            self.bits_in_buffer -= num_bits;
            self.buffer >>= num_bits;
            return Ok(result);
        }

        // Single-word refill path: see the big-endian implementation for the
        // invariants.
        if num_bits <= Self::WORD_BITS {
            let bits = self.bits_in_buffer;
            // The word is required in any case, so no peek is needed.
            let w = self.backend.read_word()?.to_le();
            // Shift valid: bits < num_bits <= WORD_BITS < BUFFER_BITS,
            // and the word fits entirely above the buffered bits.
            let virt = self.buffer | (w.as_double() << bits);
            // Extract the low num_bits (num_bits <= WORD_BITS < BUFFER_BITS)
            let result: u64 = (virt & ((BB::<WR>::ONE << num_bits) - BB::<WR>::ONE)).as_to();
            // The new buffer comes from the word alone: bits < num_bits, so
            // all buffered bits are consumed and `buffer >> num_bits` would
            // be zero. Keeping `virt` off this computation shortens the
            // loop-carried dependency chain. Shift valid: 1 <= num_bits -
            // bits <= WORD_BITS < BUFFER_BITS.
            let mut buffer = w.as_double() >> (num_bits - bits);
            let mut new_bits = bits + Self::WORD_BITS - num_bits;
            // Top up with a second word if available: new_bits < WORD_BITS
            // here, so there is always room and the buffer stays short of
            // full. This halves the number of refills, making the in-buffer
            // test above more predictable. The atomic optional read never
            // consumes past the end of the stream.
            if let Some(w2) = self.backend.read_word_opt() {
                // Shift valid: new_bits < WORD_BITS <= BUFFER_BITS - WORD_BITS.
                buffer |= w2.to_le().as_double() << new_bits;
                new_bits += Self::WORD_BITS;
            }
            self.buffer = buffer;
            self.bits_in_buffer = new_bits;
            return Ok(result);
        }

        let mut result: u64 = self.buffer.as_to();
        let mut bits_in_res = self.bits_in_buffer;

        // Directly read to the result without updating the buffer
        while num_bits > Self::WORD_BITS + bits_in_res {
            let new_word: u64 = self.backend.read_word()?.to_le().as_u64();
            result |= new_word << bits_in_res;
            bits_in_res += Self::WORD_BITS;
        }

        num_bits -= bits_in_res;

        debug_assert!(num_bits > 0);
        debug_assert!(num_bits <= Self::WORD_BITS);

        // get the final word
        let new_word = self.backend.read_word()?.to_le();
        self.bits_in_buffer = Self::WORD_BITS - num_bits;
        // compose the remaining bits
        let shamt = 64 - num_bits;
        let upcast: u64 = new_word.as_u64();
        let final_bits: u64 = (upcast << shamt) >> shamt;
        result |= final_bits << bits_in_res;
        // and put the rest in the buffer
        self.buffer = new_word.as_double() >> num_bits;

        Ok(result)
    }

    #[inline]
    fn read_unary(&mut self) -> Result<u64, Self::Error> {
        debug_assert!(self.bits_in_buffer < Self::BUFFER_BITS);

        // count the zeros from the right
        let zeros: usize = self.buffer.trailing_zeros() as usize;

        // if we encountered a 1 in the bits_in_buffer we can return
        if zeros < self.bits_in_buffer {
            // See the big-endian implementation for the shift-form choice.
            if Self::BUFFER_BITS > 64 {
                self.buffer >>= zeros + 1;
            } else {
                self.buffer = self.buffer >> zeros >> 1;
            }
            self.bits_in_buffer -= zeros + 1;
            return Ok(zeros as u64);
        }

        let mut result: u64 = self.bits_in_buffer as _;

        loop {
            let new_word = self.backend.read_word()?.to_le();

            if new_word != WR::Word::ZERO {
                let zeros: usize = new_word.trailing_zeros() as _;
                let mut buffer = new_word.as_double() >> zeros >> 1;
                let mut new_bits = Self::WORD_BITS - zeros - 1;
                // Top up with a second word if available: new_bits <
                // WORD_BITS here, so there is always room and the buffer
                // stays short of full. The atomic optional read never
                // consumes past the end of the stream (see read_bits).
                if let Some(w2) = self.backend.read_word_opt() {
                    // Shift valid: new_bits < WORD_BITS <= BUFFER_BITS - WORD_BITS.
                    buffer |= w2.to_le().as_double() << new_bits;
                    new_bits += Self::WORD_BITS;
                }
                self.buffer = buffer;
                self.bits_in_buffer = new_bits;
                return Ok(result + zeros as u64);
            }
            result += Self::WORD_BITS as u64;
        }
    }

    #[inline]
    fn skip_bits(&mut self, mut n_bits: usize) -> Result<(), Self::Error> {
        debug_assert!(self.bits_in_buffer < Self::BUFFER_BITS);
        // happy case, just shift the buffer
        if n_bits <= self.bits_in_buffer {
            self.bits_in_buffer -= n_bits;
            self.buffer >>= n_bits;
            return Ok(());
        }

        n_bits -= self.bits_in_buffer;

        // skip words as needed
        while n_bits > Self::WORD_BITS {
            let _ = self.backend.read_word()?;
            n_bits -= Self::WORD_BITS;
        }

        // get the final word
        let new_word = self.backend.read_word()?.to_le();
        self.bits_in_buffer = Self::WORD_BITS - n_bits;
        self.buffer = new_word.as_double() >> n_bits;

        Ok(())
    }

    #[cfg(not(feature = "no_copy_impls"))]
    fn copy_to<F: Endianness, W: BitWrite<F>>(
        &mut self,
        bit_write: &mut W,
        mut n: u64,
    ) -> Result<(), CopyError<Self::Error, W::Error>> {
        // Copy from the buffer at most 64 bits at a time, as the buffer
        // can hold more than 64 bits, but write_bits accepts at most 64
        while n > 0 && self.bits_in_buffer > 0 {
            let m = Ord::min(Ord::min(n, 64), self.bits_in_buffer as u64) as usize;
            // The m lowest bits of the buffer; m >= 1, so the mask shift is valid
            let value = self.buffer.as_to::<u64>() & (u64::MAX >> (64 - m));
            bit_write
                .write_bits(value, m)
                .map_err(CopyError::WriteError)?;
            // m >= 1, so the two-step shift is valid even when m == BUFFER_BITS
            self.buffer = self.buffer >> (m - 1) >> 1;
            self.bits_in_buffer -= m;
            n -= m as u64;
        }

        if n == 0 {
            return Ok(());
        }

        // The buffer is empty: copy whole words
        while n > Self::WORD_BITS as u64 {
            bit_write
                .write_bits(
                    self.backend
                        .read_word()
                        .map_err(CopyError::ReadError)?
                        .to_le()
                        .as_u64(),
                    Self::WORD_BITS,
                )
                .map_err(CopyError::WriteError)?;
            n -= Self::WORD_BITS as u64;
        }

        debug_assert!(n > 0);
        // Copy the n lowest bits of a final word, and store the remaining
        // bits at the bottom of the buffer, with zeros above
        let new_word = self
            .backend
            .read_word()
            .map_err(CopyError::ReadError)?
            .to_le();
        self.bits_in_buffer = Self::WORD_BITS - n as usize;
        // n >= 1, so the mask shift is valid
        let value = new_word.as_u64() & (u64::MAX >> (64 - n as usize));
        bit_write
            .write_bits(value, n as usize)
            .map_err(CopyError::WriteError)?;
        self.buffer = new_word.as_double() >> n;
        Ok(())
    }
}

impl<WR: WordRead + WordSeek<Error = <WR as WordRead>::Error>, RP: ReadParams> BitSeek
    for BufBitReader<LE, WR, RP>
where
    WR::Word: DoubleType,
{
    type Error = <WR as WordSeek>::Error;

    #[inline]
    fn bit_pos(&mut self) -> Result<u64, Self::Error> {
        Ok(self.backend.word_pos()? * Self::WORD_BITS as u64 - self.bits_in_buffer as u64)
    }

    #[inline]
    fn set_bit_pos(&mut self, bit_index: u64) -> Result<(), Self::Error> {
        self.backend
            .set_word_pos(bit_index / Self::WORD_BITS as u64)?;

        let bit_offset = (bit_index % Self::WORD_BITS as u64) as usize;
        self.buffer = BB::<WR>::ZERO;
        self.bits_in_buffer = 0;
        if bit_offset != 0 {
            let new_word: BB<WR> = self.backend.read_word()?.to_le().as_double();
            self.bits_in_buffer = Self::WORD_BITS - bit_offset;
            self.buffer = new_word >> bit_offset;
        }
        Ok(())
    }
}

#[cfg(feature = "std")]
impl<WR: WordRead, RP: ReadParams> std::io::Read for BufBitReader<LE, WR, RP>
where
    WR::Word: DoubleType,
{
    /// Note that this implementation transfers data in 8-byte chunks, and a
    /// [`WordRead`] backend error is not atomic with respect to the chunk:
    /// near the end of the stream a partial chunk may be consumed and then
    /// discarded, so up to 7 trailing bytes can be unreachable through this
    /// interface when the destination buffer length is a multiple of 8.
    /// Moreover, the backend error type cannot distinguish end of stream
    /// from a backend failure, so reading past the last available byte fails
    /// with [`std::io::ErrorKind::UnexpectedEof`] instead of returning
    /// `Ok(0)`.
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut read = 0;
        let mut iter = buf.chunks_exact_mut(8);

        for chunk in &mut iter {
            match self.read_bits(64) {
                Ok(word) => {
                    chunk.copy_from_slice(&word.to_le_bytes());
                    read += 8;
                }
                // If we read some bytes, return them; the error will
                // resurface at the next call
                Err(_) if read > 0 => return Ok(read),
                Err(e) => {
                    return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, e));
                }
            }
        }

        let rem = iter.into_remainder();
        if !rem.is_empty() {
            match self.read_bits(rem.len() * 8) {
                Ok(word) => {
                    rem.copy_from_slice(&word.to_le_bytes()[..rem.len()]);
                    read += rem.len();
                }
                Err(_) if read > 0 => return Ok(read),
                Err(e) => {
                    return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, e));
                }
            }
        }

        Ok(read)
    }
}

#[cfg(feature = "std")]
impl<WR: WordRead, RP: ReadParams> std::io::Read for BufBitReader<BE, WR, RP>
where
    WR::Word: DoubleType,
{
    /// Note that this implementation transfers data in 8-byte chunks, and a
    /// [`WordRead`] backend error is not atomic with respect to the chunk:
    /// near the end of the stream a partial chunk may be consumed and then
    /// discarded, so up to 7 trailing bytes can be unreachable through this
    /// interface when the destination buffer length is a multiple of 8.
    /// Moreover, the backend error type cannot distinguish end of stream
    /// from a backend failure, so reading past the last available byte fails
    /// with [`std::io::ErrorKind::UnexpectedEof`] instead of returning
    /// `Ok(0)`.
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut read = 0;
        let mut iter = buf.chunks_exact_mut(8);

        for chunk in &mut iter {
            match self.read_bits(64) {
                Ok(word) => {
                    chunk.copy_from_slice(&word.to_be_bytes());
                    read += 8;
                }
                // If we read some bytes, return them; the error will
                // resurface at the next call
                Err(_) if read > 0 => return Ok(read),
                Err(e) => {
                    return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, e));
                }
            }
        }

        let rem = iter.into_remainder();
        if !rem.is_empty() {
            match self.read_bits(rem.len() * 8) {
                Ok(word) => {
                    rem.copy_from_slice(&word.to_be_bytes()[8 - rem.len()..]);
                    read += rem.len();
                }
                Err(_) if read > 0 => return Ok(read),
                Err(e) => {
                    return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, e));
                }
            }
        }

        Ok(read)
    }
}

#[cfg(test)]
#[cfg(feature = "std")]
mod tests {
    use super::*;
    use crate::prelude::{MemWordReader, MemWordWriterVec};
    use core::error::Error;
    use std::io::Read;
    /// On a strict (finite) backend, the two-word top-up must consume a word
    /// only when one is available, and the read-and-consume must be atomic:
    /// every bit of the stream is read back exactly once, the reader works
    /// through the exact end of the data, and only then errors.
    #[test]
    fn test_topup_at_end_of_stream() {
        macro_rules! check {
            ($E:ty, $to:ident) => {{
                let words: [u32; 3] = [0xA1B2_C3D4_u32.$to(), 0x1596_37D8_u32.$to(), !0];
                let mut r = BufBitReader::<$E, _>::new(MemWordReader::new(&words));
                let mut all: Vec<u64> = Vec::new();
                // 20 + 40 + 20 + 16 = 96 bits = exactly three words; the
                // first and third reads cross a word boundary and top up.
                for n in [20, 40, 20, 16] {
                    all.push(r.read_bits(n).unwrap());
                }
                // The stream is exhausted: no word was consumed early, none
                // was lost, and the next read fails.
                assert!(r.read_bits(1).is_err());
                // The same bits read in one 64-bit and one 32-bit piece must
                // reassemble identically.
                let mut r2 = BufBitReader::<$E, _>::new(MemWordReader::new(&words));
                let a = r2.read_bits(64).unwrap();
                let b = r2.read_bits(32).unwrap();
                assert!(r2.read_bits(1).is_err());
                if TypeId::of::<$E>() == TypeId::of::<BE>() {
                    assert_eq!(all[0], a >> 44);
                    assert_eq!(all[1], (a >> 4) & ((1 << 40) - 1));
                    assert_eq!(all[2], ((a & 0xF) << 16) | (b >> 16));
                    assert_eq!(all[3], b & 0xFFFF);
                } else {
                    assert_eq!(all[0], a & ((1 << 20) - 1));
                    assert_eq!(all[1], (a >> 20) & ((1 << 40) - 1));
                    assert_eq!(all[2], (a >> 60) | ((b & 0xFFFF) << 4));
                    assert_eq!(all[3], b >> 16);
                }
            }};
        }
        use core::any::TypeId;
        check!(BE, to_be);
        check!(LE, to_le);
    }

    #[test]
    fn test_read() -> std::io::Result<()> {
        let data = [
            0x90, 0x2d, 0xd0, 0x26, 0xdf, 0x89, 0xbb, 0x7e, 0x3a, 0xd6, 0xc6, 0x96, 0x73, 0xe9,
            0x9d, 0xc9, 0x2a, 0x77, 0x82, 0xa9, 0xe6, 0x4b, 0x53, 0xcc, 0x83, 0x80, 0x4a, 0xf3,
            0xcd, 0xe3, 0x50, 0x4e, 0x45, 0x4a, 0x3a, 0x42, 0x00, 0x4b, 0x4d, 0xbe, 0x4c, 0x88,
            0x24, 0xf2, 0x4b, 0x6b, 0xbd, 0x79, 0xeb, 0x74, 0xbc, 0xe8, 0x7d, 0xff, 0x4b, 0x3d,
            0xa7, 0xd6, 0x0d, 0xef, 0x9c, 0x5b, 0xb3, 0xec, 0x94, 0x97, 0xcc, 0x8b, 0x41, 0xe1,
            0x9c, 0xcc, 0x1a, 0x03, 0x58, 0xc4, 0xfb, 0xd0, 0xc0, 0x10, 0xe2, 0xa0, 0xc9, 0xac,
            0xa7, 0xbb, 0x50, 0xf6, 0x5c, 0x87, 0x68, 0x0f, 0x42, 0x93, 0x3f, 0x2e, 0x28, 0x28,
            0x76, 0x83, 0x9b, 0xeb, 0x12, 0xe0, 0x4f, 0xc5, 0xb0, 0x8d, 0x14, 0xda, 0x3b, 0xdf,
            0xd3, 0x4b, 0x80, 0xd1, 0xfc, 0x87, 0x85, 0xae, 0x54, 0xc7, 0x45, 0xc9, 0x38, 0x43,
            0xa7, 0x9f, 0xdd, 0xa9, 0x71, 0xa7, 0x52, 0x36, 0x82, 0xff, 0x49, 0x55, 0xdb, 0x84,
            0xc2, 0x95, 0xad, 0x45, 0x80, 0xc6, 0x02, 0x80, 0xf8, 0xfc, 0x86, 0x79, 0xae, 0xb9,
            0x57, 0xe7, 0x3b, 0x33, 0x64, 0xa8,
        ];
        let data_u32 = unsafe { data.align_to::<u32>().1 };

        for i in 0..data.len() {
            let mut reader = BufBitReader::<LE, _>::new(MemWordReader::new_inf(&data_u32));
            let mut buffer = vec![0; i];
            assert_eq!(reader.read(&mut buffer)?, i);
            assert_eq!(&buffer, &data[..i]);

            let mut reader = BufBitReader::<BE, _>::new(MemWordReader::new_inf(&data_u32));
            let mut buffer = vec![0; i];
            assert_eq!(reader.read(&mut buffer)?, i);
            assert_eq!(&buffer, &data[..i]);
        }
        Ok(())
    }

    #[test]
    fn test_copy_to_then_decode() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        use crate::prelude::BufBitWriter;
        // Regression test: the big-endian copy_to used to leave garbage in
        // the low bits of the buffer, corrupting reads after a refill
        // caused by a peek
        let data: Vec<u32> = vec![u32::from_be(0x0000_FFFF), 0, 0, 0, 0, 0, 0, 0];
        let mut r = BufBitReader::<BE, _>::new(MemWordReader::new(&data));
        assert_eq!(r.read_bits(16)?, 0);
        let mut sink: Vec<u64> = vec![];
        let mut w = BufBitWriter::<BE, _>::new(MemWordWriterVec::new(&mut sink));
        r.copy_to(&mut w, 10)?;
        assert_eq!(r.read_bits(2)?, 0b11);
        let _ = r.peek_bits(32)?;
        assert_eq!(r.read_bits(30)?, 0b1111 << 26);
        let _ = r.peek_bits(32)?;
        assert_eq!(r.read_bits(33)?, 0);
        Ok(())
    }

    #[test]
    fn test_copy_to_large_buffer() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        use crate::prelude::BufBitWriter;
        // Regression test: copy_to used to pass more than 64 bits to a
        // single write_bits call when the buffer held more than 64 bits
        let data: Vec<u64> = vec![
            u64::from_be(0x0123_4567_89AB_CDEF),
            u64::from_be(0xFEDC_BA98_7654_3210),
            u64::from_be(0xAAAA_5555_AAAA_5555),
            0,
            0,
        ];
        let mut r = BufBitReader::<BE, _>::new(MemWordReader::new(&data));
        let _ = r.read_bits(10)?;
        // A peek of a full word grows the buffer past 64 bits (54 + 64 = 118)
        let _ = r.peek_bits(64)?; // now the buffer holds more than 64 bits
        let mut sink: Vec<u64> = vec![];
        {
            let mut w = BufBitWriter::<BE, _>::new(MemWordWriterVec::new(&mut sink));
            r.copy_to(&mut w, 100)?;
            w.flush()?;
        }
        let mut r2 = BufBitReader::<BE, _>::new(MemWordReader::new(&data));
        let _ = r2.read_bits(10)?;
        let hi = r2.read_bits(50)?;
        let lo = r2.read_bits(50)?;
        let mut check = BufBitReader::<BE, _>::new(MemWordReader::new(&sink));
        assert_eq!(check.read_bits(50)?, hi);
        assert_eq!(check.read_bits(50)?, lo);
        Ok(())
    }

    macro_rules! test_buf_bit_reader {
        ($f: ident, $word:ty) => {
            #[test]
            fn $f() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
                #[allow(unused_imports)]
                use crate::{
                    codes::{GammaRead, GammaWrite},
                    prelude::{
                        BufBitWriter, DeltaRead, DeltaWrite, MemWordReader, len_delta, len_gamma,
                    },
                };
                use rand::{RngExt, SeedableRng, rngs::SmallRng};

                let mut buffer_be: Vec<$word> = vec![];
                let mut buffer_le: Vec<$word> = vec![];
                let mut big = BufBitWriter::<BE, _>::new(MemWordWriterVec::new(&mut buffer_be));
                let mut little = BufBitWriter::<LE, _>::new(MemWordWriterVec::new(&mut buffer_le));

                let mut r = SmallRng::seed_from_u64(0);
                const ITER: usize = 1_000_000;

                for _ in 0..ITER {
                    let value = r.random_range(0..128);
                    assert_eq!(big.write_gamma(value)?, len_gamma(value));
                    let value = r.random_range(0..128);
                    assert_eq!(little.write_gamma(value)?, len_gamma(value));
                    let value = r.random_range(0..128);
                    assert_eq!(big.write_gamma(value)?, len_gamma(value));
                    let value = r.random_range(0..128);
                    assert_eq!(little.write_gamma(value)?, len_gamma(value));
                    let value = r.random_range(0..128);
                    assert_eq!(big.write_delta(value)?, len_delta(value));
                    let value = r.random_range(0..128);
                    assert_eq!(little.write_delta(value)?, len_delta(value));
                    let value = r.random_range(0..128);
                    assert_eq!(big.write_delta(value)?, len_delta(value));
                    let value = r.random_range(0..128);
                    assert_eq!(little.write_delta(value)?, len_delta(value));
                    let n_bits = r.random_range(0..=64);
                    if n_bits == 0 {
                        big.write_bits(0, 0)?;
                    } else {
                        big.write_bits(1, n_bits)?;
                    }
                    let n_bits = r.random_range(0..=64);
                    if n_bits == 0 {
                        little.write_bits(0, 0)?;
                    } else {
                        little.write_bits(1, n_bits)?;
                    }
                    let value = r.random_range(0..128);
                    assert_eq!(big.write_unary(value)?, value as usize + 1);
                    let value = r.random_range(0..128);
                    assert_eq!(little.write_unary(value)?, value as usize + 1);
                }

                drop(big);
                drop(little);

                type ReadWord = $word;

                #[allow(clippy::size_of_in_element_count)] // false positive
                let be_trans: &[ReadWord] = unsafe {
                    core::slice::from_raw_parts(
                        buffer_be.as_ptr() as *const ReadWord,
                        buffer_be.len()
                            * (core::mem::size_of::<$word>() / core::mem::size_of::<ReadWord>()),
                    )
                };

                #[allow(clippy::size_of_in_element_count)] // false positive
                let le_trans: &[ReadWord] = unsafe {
                    core::slice::from_raw_parts(
                        buffer_le.as_ptr() as *const ReadWord,
                        buffer_le.len()
                            * (core::mem::size_of::<$word>() / core::mem::size_of::<ReadWord>()),
                    )
                };

                let mut big_buff = BufBitReader::<BE, _>::new(MemWordReader::new_inf(be_trans));
                let mut little_buff = BufBitReader::<LE, _>::new(MemWordReader::new_inf(le_trans));

                let mut r = SmallRng::seed_from_u64(0);

                for _ in 0..ITER {
                    assert_eq!(big_buff.read_gamma()?, r.random_range(0..128));
                    assert_eq!(little_buff.read_gamma()?, r.random_range(0..128));
                    assert_eq!(big_buff.read_gamma()?, r.random_range(0..128));
                    assert_eq!(little_buff.read_gamma()?, r.random_range(0..128));
                    assert_eq!(big_buff.read_delta()?, r.random_range(0..128));
                    assert_eq!(little_buff.read_delta()?, r.random_range(0..128));
                    assert_eq!(big_buff.read_delta()?, r.random_range(0..128));
                    assert_eq!(little_buff.read_delta()?, r.random_range(0..128));
                    let n_bits = r.random_range(0..=64);
                    if n_bits == 0 {
                        assert_eq!(big_buff.read_bits(0)?, 0);
                    } else {
                        assert_eq!(big_buff.read_bits(n_bits)?, 1);
                    }
                    let n_bits = r.random_range(0..=64);
                    if n_bits == 0 {
                        assert_eq!(little_buff.read_bits(0)?, 0);
                    } else {
                        assert_eq!(little_buff.read_bits(n_bits)?, 1);
                    }

                    assert_eq!(big_buff.read_unary()?, r.random_range(0..128));
                    assert_eq!(little_buff.read_unary()?, r.random_range(0..128));
                }

                Ok(())
            }
        };
    }

    test_buf_bit_reader!(test_u64, u64);
    test_buf_bit_reader!(test_u32, u32);

    test_buf_bit_reader!(test_u16, u16);
}

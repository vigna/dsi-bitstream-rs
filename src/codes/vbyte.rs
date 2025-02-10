/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Variable-length byte codes.
//!
//! These codes represent a natural number as a sequence of bytes, the length of
//! the sequence depends on the magnitude of the number. They are used in many
//! contexts, and they are known under a plethora of different names such
//! “vbyte”, “varint”, “[variable-length
//! quantity](https://en.wikipedia.org/wiki/Variable-length_quantity)”, “LEB”,
//! and so on.
//!
//! There are several variants of their definition, but their implied
//! distribution is always ≈ 1/*x*<sup>8/7</sup>
//!
//! # Definition
//!
//! Since there are a few slightly different variants used in production code
//! and in the literature, before going into the details of this implementation
//! we will try to define a clear taxonomy by explaining in detail the four
//! three properties that define such variants.
//!
//! The simplest variable-length byte code encodes a number with a binary
//! representation of *k* bits using ⌈*k* / 7⌉ bytes. The binary representation
//! is left-padded with zeros so to obtain exactly ⌈*k* / 7⌉ blocks of 7 bits.
//! Each block is stored in a byte, prefixed with a continuation bit which is
//! one for all blocks except for the last one.
//!
//! ## Endianness
//!
//! The first property is the endianness of the bytes: in big-endian codes, the
//! first byte contains the highest (most significant) bits, whereas in
//! little-endian codes, the first byte contains the lowest (less significant)
//! bits.
//!
//! The advantage of the big-endian variant is that is lexicographical, that is,
//! comparing lexicographically a stream of encoded natural numbers will give
//! the same results as comparing lexicographically the encoded numbers, much
//! like it happens for [UTF-8 encoding](https://en.wikipedia.org/wiki/UTF-8).
//!
//! ## Completeness
//!
//! This basic representation discussed above is not *complete*, as there are
//! sequences that are not used. For example, zero can be written in many ways
//! (e.g., `0x00` or `0x80 0x00` ), but we are using only the single-byte
//! representation. Uncompleteness leads to a (small) loss in compression.
//!
//! To have completeness, one can offset the representation in *k* bits by the
//! maximum number representable using *k* – 1 bits. That is, we represent the
//! interval [0..2⁷) with one byte, then the interval [2⁷..2⁷ + 2¹⁴] with two
//! bytes, the interval [2⁷ + 2¹⁴..2⁷ + 2¹⁴ + 2²¹] with three bytes, and so on.
//!
//! ## Grouping
//!
//! In the basic representation, the continuation bit is the most significant
//! bit of each byte. However, one can gather all continuation bits in the first
//! byte ([as UTF-8 does](https://en.wikipedia.org/wiki/UTF-8)). This approach
//! makes it possible to compute the length of the code using a call to
//! [`usize::leading_ones`] on the first negated byte, which usually maps to a
//! negation and a call to a fast instruction for the detection of the most
//! significant bit, improving branch prediction.
//!
//! Note that if the code is grouped, choosing a code with the same endianness
//! as your hardare can lead to a performance improvement, as after the first
//! byte the rest of the code can be read with a
//![`read_exact`](std::io::Read::read_exact). This is indeed the only reason why
//! we provide both big-endian and little-endian codes.
//!
//! ## Sign
//!
//! It is possible to extend the codes to represent signed integers. Two
//! possible approaches are using a [bijective mapping](crate::utils::ToInt)
//! between the integers and the natural numbers, or defining a specialized
//! code.
//!
//! # Implementations
//!
//! We provide two unsigned, grouped, complete representations, one big-endian
//! and one little-endian. We recommend using the big-endian code if you need
//! lexicographical comparisons. Otherwise, you might choose an endianness
//! matching that of your hardware, which might increase performance.
//!
//! Since this code is byte-aligned, we provide also convenient, fast methods
//! [`vbyte_encode`] and [`vbyte_decode`] that can be used on types implementing
//! [`std::io::Read`] and [`std::io::Write`].
//!
//! [`LEB128`]: https://en.wikipedia.org/wiki/LEB128
//! [`VLQ`]: https://en.wikipedia.org/wiki/Variable-length_quantity
//!
//! # Examples
//!
//! - The [LEB128](https://en.wikipedia.org/wiki/LEB128) code used by LLVM is a
//!   little-endian incomplete ungrouped representation. There is both a signed
//!   and an unsigned version; the signed version represent negative numbers
//!   using two's complement.
//!
//! - The [code used by
//!   git](https://github.com/git/git/blob/7fb6aefd2aaffe66e614f7f7b83e5b7ab16d4806/varint.c)
//!   is a big-endian complete ungrouped representation.
//!
//! - [This implementation in
//!   folly](https://github.com/facebook/folly/blob/dd4a5eb057afbc3c7c7da71801df2ee3c61c47d1/folly/Varint.h#L109)
//!   is a little-endian incomplete ungrouped representation.

use crate::traits::*;
use common_traits::CastableInto;

const UPPER_BOUND_1: u64 = 128;
const UPPER_BOUND_2: u64 = 128_u64.pow(2) + UPPER_BOUND_1;
const UPPER_BOUND_3: u64 = 128_u64.pow(3) + UPPER_BOUND_2;
const UPPER_BOUND_4: u64 = 128_u64.pow(4) + UPPER_BOUND_3;
const UPPER_BOUND_5: u64 = 128_u64.pow(5) + UPPER_BOUND_4;
const UPPER_BOUND_6: u64 = 128_u64.pow(6) + UPPER_BOUND_5;
const UPPER_BOUND_7: u64 = 128_u64.pow(7) + UPPER_BOUND_6;
const UPPER_BOUND_8: u64 = 128_u64.pow(8) + UPPER_BOUND_7;

/// Return the length of the variable-length byte code for `value` in bytes.
#[must_use]
#[inline]
pub fn byte_len_vbyte(value: u64) -> usize {
    if value < UPPER_BOUND_1 {
        return 1;
    }
    if value < UPPER_BOUND_2 {
        return 2;
    }
    if value < UPPER_BOUND_3 {
        return 3;
    }
    if value < UPPER_BOUND_4 {
        return 4;
    }
    if value < UPPER_BOUND_5 {
        return 5;
    }
    if value < UPPER_BOUND_6 {
        return 6;
    }
    if value < UPPER_BOUND_7 {
        return 7;
    }
    if value < UPPER_BOUND_8 {
        return 8;
    }
    9
}

/// Return the length of the variable-length byte code for `value` in bits.
#[must_use]
#[inline]
pub fn bit_len_vbyte(value: u64) -> usize {
    8 * byte_len_vbyte(value)
}

/// Trait for reading variable-length byte codes.
pub trait VByteRead<E: Endianness>: BitRead<E> {
    #[inline(always)]
    fn read_vbyte(&mut self) -> Result<u64, Self::Error> {
        let len = self.peek_bits(8)?.cast() as u8;
        let len = if core::any::TypeId::of::<E>() == core::any::TypeId::of::<BigEndian>() {
            len.leading_ones() as u8
        } else {
            len.trailing_ones() as u8
        }
        .min(8);
        self.skip_bits((1 + len as usize).min(8))?;

        match len {
            0 => self.read_bits(8 - 1),
            1 => self.read_bits(16 - 2).map(|x| x + UPPER_BOUND_1),
            2 => self.read_bits(24 - 3).map(|x| x + UPPER_BOUND_2),
            3 => self.read_bits(32 - 4).map(|x| x + UPPER_BOUND_3),
            4 => self.read_bits(40 - 5).map(|x| x + UPPER_BOUND_4),
            5 => self.read_bits(48 - 6).map(|x| x + UPPER_BOUND_5),
            6 => self.read_bits(56 - 7).map(|x| x + UPPER_BOUND_6),
            7 => self.read_bits(64 - 8).map(|x| x + UPPER_BOUND_7),
            8.. => self.read_bits(64).map(|x| x + UPPER_BOUND_8),
        }
    }
}

/// Trait for writing variable-length byte codes.
pub trait VByteWrite<E: Endianness>: BitWrite<E> {
    #[inline]
    fn write_vbyte(&mut self, mut value: u64) -> Result<usize, Self::Error> {
        // endianness dependant constant
        macro_rules! edc {
            ($be:literal, $le:literal) => {
                if core::any::TypeId::of::<E>() == core::any::TypeId::of::<LittleEndian>() {
                    $le
                } else {
                    $be
                }
            };
        }

        if value < UPPER_BOUND_1 {
            self.write_bits(0, 1)?;
            return self.write_bits(value, 8 - 1);
        }
        if value < UPPER_BOUND_2 {
            value -= UPPER_BOUND_1;
            debug_assert!((value >> 8) < (1 << 6));
            self.write_bits(edc!(0b10, 0b01), 2)?;
            return self.write_bits(value, 16 - 2);
        }
        if value < UPPER_BOUND_3 {
            value -= UPPER_BOUND_2;
            debug_assert!((value >> 16) < (1 << 5));
            self.write_bits(edc!(0b110, 0b011), 3)?;
            return self.write_bits(value, 24 - 3);
        }
        if value < UPPER_BOUND_4 {
            value -= UPPER_BOUND_3;
            debug_assert!((value >> 24) < (1 << 4));
            self.write_bits(edc!(0b1110, 0b0111), 4)?;
            return self.write_bits(value, 32 - 4);
        }
        if value < UPPER_BOUND_5 {
            value -= UPPER_BOUND_4;
            debug_assert!((value >> 32) < (1 << 3));
            self.write_bits(edc!(0b11110, 0b01111), 5)?;
            return self.write_bits(value, 40 - 5);
        }
        if value < UPPER_BOUND_6 {
            value -= UPPER_BOUND_5;
            debug_assert!((value >> 40) < (1 << 2));
            self.write_bits(edc!(0b111110, 0b011111), 6)?;
            return self.write_bits(value, 48 - 6);
        }
        if value < UPPER_BOUND_7 {
            value -= UPPER_BOUND_6;
            debug_assert!((value >> 48) < (1 << 1));
            self.write_bits(edc!(0b1111110, 0b0111111), 7)?;
            return self.write_bits(value, 56 - 7);
        }
        if value < UPPER_BOUND_8 {
            value -= UPPER_BOUND_7;
            self.write_bits(edc!(0b11111110, 0b01111111), 8)?;
            return self.write_bits(value, 64 - 8);
        }
        // Note that we save one byte by using 0xFF as the first byte.
        // TODO: sometimes we subtract UPPER_BOUND_8, sometimes not.
        self.write_bits(0b11111111, 8)?;
        self.write_bits(value - UPPER_BOUND_8, 64)
    }
}

impl<E: Endianness, B: BitRead<E>> VByteRead<E> for B {}
impl<E: Endianness, B: BitWrite<E>> VByteWrite<E> for B {}

/// Encode an integer to a byte stream using variable-length byte codes and
/// return the number of bytes written.
///
/// This method just delegates to the correct endianness-specific method.
#[inline(always)]
pub fn vbyte_encode<E: Endianness, W: std::io::Write>(
    value: u64,
    writer: &mut W,
) -> std::io::Result<usize> {
    if core::any::TypeId::of::<E>() == core::any::TypeId::of::<BigEndian>() {
        vbyte_encode_be(value, writer)
    } else {
        vbyte_encode_le(value, writer)
    }
}

/// Encode an integer to a little-endian byte stream using variable-length byte
/// codes and return the number of bytes written.
pub fn vbyte_encode_le<W: std::io::Write>(
    mut value: u64,
    writer: &mut W,
) -> std::io::Result<usize> {
    if value < UPPER_BOUND_1 {
        writer.write_all(&[value as u8])?;
        return Ok(1);
    }
    if value < UPPER_BOUND_2 {
        value -= UPPER_BOUND_1;
        debug_assert!((value >> 8) < (1 << 6));
        writer.write_all(&[0x80 | (value & 0b11_1111) as u8, (value >> 6) as u8])?;
        return Ok(2);
    }
    if value < UPPER_BOUND_3 {
        value -= UPPER_BOUND_2;
        debug_assert!((value >> 16) < (1 << 5));
        writer.write_all(&[
            0xC0 | (value & 0b1_1111) as u8,
            (value >> 5) as u8,
            (value >> 13) as u8,
        ])?;
        return Ok(3);
    }
    if value < UPPER_BOUND_4 {
        value -= UPPER_BOUND_3;
        debug_assert!((value >> 24) < (1 << 4));
        writer.write_all(&[
            0xE0 | (value & 0b1111) as u8,
            (value >> 4) as u8,
            (value >> 12) as u8,
            (value >> 20) as u8,
        ])?;
        return Ok(4);
    }
    if value < UPPER_BOUND_5 {
        value -= UPPER_BOUND_4;
        debug_assert!((value >> 32) < (1 << 3));
        writer.write_all(&[
            0xF0 | (value & 0b111) as u8,
            (value >> 3) as u8,
            (value >> 11) as u8,
            (value >> 19) as u8,
            (value >> 27) as u8,
        ])?;
        return Ok(5);
    }
    if value < UPPER_BOUND_6 {
        value -= UPPER_BOUND_5;
        debug_assert!((value >> 40) < (1 << 2));
        writer.write_all(&[
            0xF8 | (value & 0b11) as u8,
            (value >> 2) as u8,
            (value >> 10) as u8,
            (value >> 18) as u8,
            (value >> 26) as u8,
            (value >> 34) as u8,
        ])?;
        return Ok(6);
    }
    if value < UPPER_BOUND_7 {
        value -= UPPER_BOUND_6;
        debug_assert!((value >> 48) < (1 << 1));
        writer.write_all(&[
            0xFC | (value & 0b1) as u8,
            (value >> 1) as u8,
            (value >> 9) as u8,
            (value >> 17) as u8,
            (value >> 25) as u8,
            (value >> 33) as u8,
            (value >> 41) as u8,
        ])?;
        return Ok(7);
    }
    if value < UPPER_BOUND_8 {
        value -= UPPER_BOUND_7;
        writer.write_all(&[
            0xFE,
            value as u8,
            (value >> 8) as u8,
            (value >> 16) as u8,
            (value >> 24) as u8,
            (value >> 32) as u8,
            (value >> 40) as u8,
            (value >> 48) as u8,
        ])?;
        return Ok(8);
    }

    writer.write_all(&[
        0xFF,
        value as u8,
        (value >> 8) as u8,
        (value >> 16) as u8,
        (value >> 24) as u8,
        (value >> 32) as u8,
        (value >> 40) as u8,
        (value >> 48) as u8,
        (value >> 56) as u8,
    ])?;
    Ok(9)
}

/// Encode an integer to a big-endian byte stream using variable-length byte
/// codes and return the number of bytes written.
pub fn vbyte_encode_be<W: std::io::Write>(
    mut value: u64,
    writer: &mut W,
) -> std::io::Result<usize> {
    if value < UPPER_BOUND_1 {
        writer.write_all(&[value as u8])?;
        return Ok(1);
    }
    if value < UPPER_BOUND_2 {
        value -= UPPER_BOUND_1;
        debug_assert!((value >> 8) < (1 << 6));
        writer.write_all(&[0x80 | (value >> 8) as u8, value as u8])?;
        return Ok(2);
    }
    if value < UPPER_BOUND_3 {
        value -= UPPER_BOUND_2;
        debug_assert!((value >> 16) < (1 << 5));
        writer.write_all(&[0xC0 | (value >> 16) as u8, (value >> 8) as u8, value as u8])?;
        return Ok(3);
    }
    if value < UPPER_BOUND_4 {
        value -= UPPER_BOUND_3;
        debug_assert!((value >> 24) < (1 << 4));
        writer.write_all(&[
            0xE0 | (value >> 24) as u8,
            (value >> 16) as u8,
            (value >> 8) as u8,
            value as u8,
        ])?;
        return Ok(4);
    }
    if value < UPPER_BOUND_5 {
        value -= UPPER_BOUND_4;
        debug_assert!((value >> 32) < (1 << 3));
        writer.write_all(&[
            0xF0 | (value >> 32) as u8,
            (value >> 24) as u8,
            (value >> 16) as u8,
            (value >> 8) as u8,
            value as u8,
        ])?;
        return Ok(5);
    }
    if value < UPPER_BOUND_6 {
        value -= UPPER_BOUND_5;
        debug_assert!((value >> 40) < (1 << 2));
        writer.write_all(&[
            0xF8 | (value >> 40) as u8,
            (value >> 32) as u8,
            (value >> 24) as u8,
            (value >> 16) as u8,
            (value >> 8) as u8,
            value as u8,
        ])?;
        return Ok(6);
    }
    if value < UPPER_BOUND_7 {
        value -= UPPER_BOUND_6;
        debug_assert!((value >> 48) < (1 << 1));
        writer.write_all(&[
            0xFC | (value >> 48) as u8,
            (value >> 40) as u8,
            (value >> 32) as u8,
            (value >> 24) as u8,
            (value >> 16) as u8,
            (value >> 8) as u8,
            value as u8,
        ])?;
        return Ok(7);
    }
    if value < UPPER_BOUND_8 {
        value -= UPPER_BOUND_7;
        writer.write_all(&[
            0xFE,
            (value >> 48) as u8,
            (value >> 40) as u8,
            (value >> 32) as u8,
            (value >> 24) as u8,
            (value >> 16) as u8,
            (value >> 8) as u8,
            value as u8,
        ])?;
        return Ok(8);
    }

    writer.write_all(&[
        0xFF,
        (value >> 56) as u8,
        (value >> 48) as u8,
        (value >> 40) as u8,
        (value >> 32) as u8,
        (value >> 24) as u8,
        (value >> 16) as u8,
        (value >> 8) as u8,
        value as u8,
    ])?;
    Ok(9)
}

#[inline(always)]
/// Decode an integer from a byte stream using variable-length byte codes.
///
/// This method just delegates to the correct endianness-specific method.
pub fn vbyte_decode<E: Endianness, R: std::io::Read>(reader: &mut R) -> std::io::Result<u64> {
    if core::any::TypeId::of::<E>() == core::any::TypeId::of::<BigEndian>() {
        vbyte_decode_be(reader)
    } else {
        vbyte_decode_le(reader)
    }
}

/// Decode an integer from a little-endian byte stream using variable-length
/// byte codes.
pub fn vbyte_decode_le<R: std::io::Read>(reader: &mut R) -> std::io::Result<u64> {
    let mut data = [0; 9];
    reader.read_exact(&mut data[..1])?;
    let x = data[0];
    if x < 0x80 {
        return Ok(x as u64);
    }
    if x < 0xC0 {
        reader.read_exact(&mut data[1..2])?;
        let x = (((x & !0xC0) as u64) | (data[1] as u64) << 6) + UPPER_BOUND_1;
        return Ok(x);
    }
    if x < 0xE0 {
        reader.read_exact(&mut data[1..3])?;
        let x =
            (((x & !0xE0) as u64) | (data[1] as u64) << 5 | (data[2] as u64) << 13) + UPPER_BOUND_2;
        return Ok(x);
    }
    if x < 0xF0 {
        reader.read_exact(&mut data[1..4])?;
        let x = (((x & !0xF0) as u64)
            | (data[1] as u64) << 4
            | (data[2] as u64) << 12
            | (data[3] as u64) << 20)
            + UPPER_BOUND_3;
        return Ok(x);
    }
    if x < 0xF8 {
        reader.read_exact(&mut data[1..5])?;
        let x = (((x & !0xF8) as u64)
            | (data[1] as u64) << 3
            | (data[2] as u64) << 11
            | (data[3] as u64) << 19
            | (data[4] as u64) << 27)
            + UPPER_BOUND_4;
        return Ok(x);
    }
    if x < 0xFC {
        reader.read_exact(&mut data[1..6])?;
        let x = (((x & !0xFC) as u64)
            | (data[1] as u64) << 2
            | (data[2] as u64) << 10
            | (data[3] as u64) << 18
            | (data[4] as u64) << 26
            | (data[5] as u64) << 34)
            + UPPER_BOUND_5;
        return Ok(x);
    }
    if x < 0xFE {
        reader.read_exact(&mut data[1..7])?;
        let x = (((x & !0xFE) as u64)
            | (data[1] as u64) << 1
            | (data[2] as u64) << 9
            | (data[3] as u64) << 17
            | (data[4] as u64) << 25
            | (data[5] as u64) << 33
            | (data[6] as u64) << 41)
            + UPPER_BOUND_6;
        return Ok(x);
    }
    if x < 0xFF {
        reader.read_exact(&mut data[1..8])?;
        let x = ((data[1] as u64)
            | (data[2] as u64) << 8
            | (data[3] as u64) << 16
            | (data[4] as u64) << 24
            | (data[5] as u64) << 32
            | (data[6] as u64) << 40
            | (data[7] as u64) << 48)
            + UPPER_BOUND_7;
        return Ok(x);
    }

    reader.read_exact(&mut data[1..9])?;
    let x = u64::from_le_bytes(data[1..].try_into().unwrap());

    Ok(x)
}

/// Decode an integer from a big-endian byte stream using variable-length byte
/// codes.
pub fn vbyte_decode_be<R: std::io::Read>(reader: &mut R) -> std::io::Result<u64> {
    let mut data = [0; 9];
    reader.read_exact(&mut data[..1])?;
    let x = data[0];
    if x < 0x80 {
        return Ok(x as u64);
    }
    if x < 0xC0 {
        reader.read_exact(&mut data[1..2])?;
        let x = (((x & !0xC0) as u64) << 8 | data[1] as u64) + UPPER_BOUND_1;
        return Ok(x);
    }
    if x < 0xE0 {
        reader.read_exact(&mut data[1..3])?;
        let x =
            (((x & !0xE0) as u64) << 16 | (data[1] as u64) << 8 | data[2] as u64) + UPPER_BOUND_2;
        return Ok(x);
    }
    if x < 0xF0 {
        reader.read_exact(&mut data[1..4])?;
        let x = (((x & !0xF0) as u64) << 24
            | (data[1] as u64) << 16
            | (data[2] as u64) << 8
            | data[3] as u64)
            + UPPER_BOUND_3;
        return Ok(x);
    }
    if x < 0xF8 {
        reader.read_exact(&mut data[1..5])?;
        let x = (((x & !0xF8) as u64) << 32
            | (data[1] as u64) << 24
            | (data[2] as u64) << 16
            | (data[3] as u64) << 8
            | data[4] as u64)
            + UPPER_BOUND_4;
        return Ok(x);
    }
    if x < 0xFC {
        reader.read_exact(&mut data[1..6])?;
        let x = (((x & !0xFC) as u64) << 40
            | (data[1] as u64) << 32
            | (data[2] as u64) << 24
            | (data[3] as u64) << 16
            | (data[4] as u64) << 8
            | data[5] as u64)
            + UPPER_BOUND_5;
        return Ok(x);
    }
    if x < 0xFE {
        reader.read_exact(&mut data[1..7])?;
        let x = (((x & !0xFE) as u64) << 48
            | (data[1] as u64) << 40
            | (data[2] as u64) << 32
            | (data[3] as u64) << 24
            | (data[4] as u64) << 16
            | (data[5] as u64) << 8
            | data[6] as u64)
            + UPPER_BOUND_6;
        return Ok(x);
    }
    if x < 0xFF {
        reader.read_exact(&mut data[1..8])?;
        let x = ((data[1] as u64) << 48
            | (data[2] as u64) << 40
            | (data[3] as u64) << 32
            | (data[4] as u64) << 24
            | (data[5] as u64) << 16
            | (data[6] as u64) << 8
            | data[7] as u64)
            + UPPER_BOUND_7;
        return Ok(x);
    }

    reader.read_exact(&mut data[1..9])?;
    let x = u64::from_be_bytes(data[1..].try_into().unwrap());

    Ok(x)
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! impl_tests {
        ($test_name:ident, $E:ty) => {
            #[test]
            fn $test_name() {
                const MAX: usize = 1 << 20;
                const MIN: usize = 0;
                let mut buffer = std::io::Cursor::new(Vec::with_capacity(128));
                let mut lens = Vec::new();

                for i in MIN..MAX {
                    lens.push(vbyte_encode::<$E, _>(i as _, &mut buffer).unwrap());
                }
                buffer.set_position(0);
                for (i, l) in (MIN..MAX).zip(lens.iter()) {
                    let j = vbyte_decode::<$E, _>(&mut buffer).unwrap();
                    assert_eq!(*l, byte_len_vbyte(i as _));
                    assert_eq!(i as u64, j);
                }

                let values = [
                    0,
                    UPPER_BOUND_1 - 1,
                    UPPER_BOUND_1 + 1,
                    UPPER_BOUND_2 - 1,
                    UPPER_BOUND_2 + 1,
                    UPPER_BOUND_3 - 1,
                    UPPER_BOUND_3 + 1,
                    UPPER_BOUND_4 - 1,
                    UPPER_BOUND_4 + 1,
                    UPPER_BOUND_5 - 1,
                    UPPER_BOUND_5 + 1,
                    UPPER_BOUND_6 - 1,
                    UPPER_BOUND_6 + 1,
                    UPPER_BOUND_7 - 1,
                    UPPER_BOUND_7 + 1,
                    UPPER_BOUND_8 - 1,
                    UPPER_BOUND_8 + 1,
                    u64::MAX,
                ];

                let tell: u64 = buffer.position();
                for &i in values.iter() {
                    vbyte_encode::<$E, _>(i, &mut buffer).unwrap();
                }
                buffer.set_position(tell);
                for &i in values.iter() {
                    assert_eq!(i, vbyte_decode::<$E, _>(&mut buffer).unwrap());
                }
            }
        };
    }

    impl_tests!(test_vbytes_be, BE);
    impl_tests!(test_vbytes_le, LE);
}

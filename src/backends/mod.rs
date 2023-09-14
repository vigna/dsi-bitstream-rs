/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Implementations of Word readers and writers and Bit readers and writers.

mod mem_word_reader;
pub use mem_word_reader::*;

mod mem_word_writer;
pub use mem_word_writer::*;

#[cfg(feature = "std")]
mod file_backend;
#[cfg(feature = "std")]
pub use file_backend::*;

mod bit_reader;
pub use bit_reader::BitReader;

mod buf_bit_reader;
pub use buf_bit_reader::BufBitReader;

mod buf_bit_writer;
pub use buf_bit_writer::BufBitWriter;

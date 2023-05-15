/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Implementations of Word readers and writers and Bit readers and writers.

mod word_stream;
pub use word_stream::*;

#[cfg(feature="std")]
mod file_backend;
#[cfg(feature="std")]
pub use file_backend::*;

mod unbuffered_bit_stream_reader;
pub use unbuffered_bit_stream_reader::UnbufferedBitStreamRead;

mod buffered_bit_stream_reader;
pub use buffered_bit_stream_reader::BufferedBitStreamRead;

mod buffered_bit_stream_writer;
pub use buffered_bit_stream_writer::BufferedBitStreamWrite;

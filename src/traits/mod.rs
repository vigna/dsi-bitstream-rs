/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! # Traits
//! This modules contains the traits that are used throughout the crate.
//! They are collected into a module so you can do `use dsi_bitstream::traits::*;`
//! for ease of use.

mod castable;
pub use castable::*;

mod downcastable;
pub use downcastable::*;

mod upcastable;
pub use upcastable::*;

mod count;
pub use count::*;

mod word;
pub use word::*;

mod bit_stream;
pub use bit_stream::*;

mod word_stream;
pub use word_stream::*;

mod endianness;
pub use endianness::*;

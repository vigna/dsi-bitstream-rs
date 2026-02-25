/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Shared library for benchmark data generation and utilities.

#![allow(dead_code)]

pub mod data;
pub mod utils;

/// Number of read/write operations tested for each combination of parameters.
pub const N: usize = 1_000_000;

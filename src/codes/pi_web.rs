/*
 * SPDX-FileCopyrightText: 2024 Tommaso Fontana
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! π web codes
//!
//! π web codes are a modified version of π-codes in which 0 is encoded
//! by 1 and any other positive integer n is encoded with a 0 followed by
//! the π-code of n.
//!
//! ## Reference
//! Alberto Apostolico and Guido Drovandi.
//! "Graph Compression by BFS,"
//! Algorithms 2009, 2, 1031-1044; <https://doi.org/10.3390/a2031031>.

use crate::codes::pi::{len_pi, PiRead, PiWrite};
use crate::traits::*;

/// Returns the length of the π web code for `n`.
#[must_use]
#[inline(always)]
pub fn len_pi_web(n: u64, k: u64) -> usize {
    1 + if n == 0 { 0 } else { len_pi(n - 1, k) }
}

/// Trait for reading π web codes.
///
/// This is the trait you should usually pull in scope to read π web codes.
pub trait PiWebRead<E: Endianness>: BitRead<E> + PiRead<E> {
    fn read_pi_web(&mut self, k: u64) -> Result<u64, Self::Error> {
        if self.read_bits(1)? == 1 {
            Ok(0)
        } else {
            Ok(self.read_pi(k)? + 1)
        }
    }
}

/// Trait for writing π web codes.
///
/// This is the trait you should usually pull in scope to write π web codes.
pub trait PiWebWrite<E: Endianness>: BitWrite<E> + PiWrite<E> {
    #[inline(always)]
    fn write_pi_web(&mut self, n: u64, k: u64) -> Result<usize, Self::Error> {
        if n == 0 {
            self.write_bits(1, 1)
        } else {
            Ok(self.write_bits(0, 1)? + self.write_pi(n - 1, k)?)
        }
    }
}

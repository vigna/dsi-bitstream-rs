/*
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 * SPDX-FileCopyrightText: 2024 Tommaso Fontana
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */
#[cfg(feature = "mem_dbg")]
use mem_dbg::{MemDbg, MemSize};

use crate::prelude::{
    len_delta, len_exp_golomb, len_gamma, len_golomb, len_omega, len_pi, len_pi_web, len_rice,
    len_vbyte, len_zeta, Code,
};

/// Keeps track of the space needed to store a stream of integers using
/// different codes.
///
/// This structure can be used to determine empirically which code provides the
/// best compression for a given stream. You have to [update the
/// structure](Self::update) with the integers in the stream; at any time, you
/// can examine the statistics or call [`best_code`](Self::best_code) to get the
/// best code.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
pub struct CodesStats<
    // How many ζ codes to consider.
    const ZETA: usize = 10,
    // How many Golomb codes to consider.
    const GOLOMB: usize = 20,
    // How many Exponential Golomb codes to consider.
    const EXP_GOLOMB: usize = 10,
    // How many Rice codes to consider.
    const RICE: usize = 10,
    // How many Pi and Pi web codes to consider.
    const PI: usize = 10,
> {
    pub unary: u64,
    pub gamma: u64,
    pub delta: u64,
    pub omega: u64,
    pub vbyte: u64,
    pub zeta: [u64; ZETA],
    pub golomb: [u64; GOLOMB],
    pub exp_golomb: [u64; EXP_GOLOMB],
    pub rice: [u64; RICE],
    pub pi: [u64; PI],
    pub pi_web: [u64; PI],
}

impl<
        const ZETA: usize,
        const GOLOMB: usize,
        const EXP_GOLOMB: usize,
        const RICE: usize,
        const PI: usize,
    > core::default::Default for CodesStats<ZETA, GOLOMB, EXP_GOLOMB, RICE, PI>
{
    fn default() -> Self {
        Self {
            unary: 0,
            gamma: 0,
            delta: 0,
            omega: 0,
            vbyte: 0,
            zeta: [0; ZETA],
            golomb: [0; GOLOMB],
            exp_golomb: [0; EXP_GOLOMB],
            rice: [0; RICE],
            pi: [0; PI],
            pi_web: [0; PI],
        }
    }
}

impl<
        const ZETA: usize,
        const GOLOMB: usize,
        const EXP_GOLOMB: usize,
        const RICE: usize,
        const PI: usize,
    > CodesStats<ZETA, GOLOMB, EXP_GOLOMB, RICE, PI>
{
    /// Update the stats with the lengths of the codes for `n` and return
    /// `n` for convenience.
    pub fn update(&mut self, n: u64) -> u64 {
        self.update_many(n, 1)
    }

    #[inline]
    pub fn update_many(&mut self, n: u64, count: u64) -> u64 {
        self.unary += (n + 1) * count;
        self.gamma += len_gamma(n) as u64 * count;
        self.delta += len_delta(n) as u64 * count;
        self.omega += len_omega(n) as u64 * count;
        self.vbyte += len_vbyte(n) as u64 * count;

        for (k, val) in self.zeta.iter_mut().enumerate() {
            *val += (len_zeta(n, (k + 1) as _) as u64) * count;
        }
        for (b, val) in self.golomb.iter_mut().enumerate() {
            *val += (len_golomb(n, (b + 1) as _) as u64) * count;
        }
        for (k, val) in self.exp_golomb.iter_mut().enumerate() {
            *val += (len_exp_golomb(n, k as _) as u64) * count;
        }
        for (log2_b, val) in self.rice.iter_mut().enumerate() {
            *val += (len_rice(n, log2_b as _) as u64) * count;
        }
        // +2 because π0 = gamma and π1 = zeta_2
        for (k, val) in self.pi.iter_mut().enumerate() {
            *val += (len_pi(n, (k + 2) as _) as u64) * count;
        }
        for (k, val) in self.pi_web.iter_mut().enumerate() {
            *val += (len_pi_web(n, k as _) as u64) * count;
        }
        n
    }

    // Combines additively this stats with another one.
    pub fn add(&mut self, rhs: &Self) {
        self.unary += rhs.unary;
        self.gamma += rhs.gamma;
        self.delta += rhs.delta;
        self.omega += rhs.omega;
        self.vbyte += rhs.vbyte;
        for (a, b) in self.zeta.iter_mut().zip(rhs.zeta.iter()) {
            *a += *b;
        }
        for (a, b) in self.golomb.iter_mut().zip(rhs.golomb.iter()) {
            *a += *b;
        }
        for (a, b) in self.exp_golomb.iter_mut().zip(rhs.exp_golomb.iter()) {
            *a += *b;
        }
        for (a, b) in self.rice.iter_mut().zip(rhs.rice.iter()) {
            *a += *b;
        }
        for (a, b) in self.pi.iter_mut().zip(rhs.pi.iter()) {
            *a += *b;
        }
        for (a, b) in self.pi_web.iter_mut().zip(rhs.pi_web.iter()) {
            *a += *b;
        }
    }

    /// Return the best code for the stream and its space usage.
    pub fn best_code(&self) -> (Code, u64) {
        let mut best = self.unary;
        let mut best_code = Code::Unary;

        macro_rules! check {
            ($code:expr, $len:expr) => {
                if $len < best {
                    best = $len;
                    best_code = $code;
                }
            };
        }

        check!(Code::Gamma, self.gamma);
        check!(Code::Delta, self.delta);
        check!(Code::Omega, self.omega);
        check!(Code::VByte, self.vbyte);

        for (k, val) in self.zeta.iter().enumerate() {
            check!(Code::Zeta { k: (k + 1) as _ }, *val);
        }
        for (b, val) in self.golomb.iter().enumerate() {
            check!(Code::Golomb { b: (b + 1) as _ }, *val);
        }
        for (k, val) in self.exp_golomb.iter().enumerate() {
            check!(Code::ExpGolomb { k: k as _ }, *val);
        }
        for (log2_b, val) in self.rice.iter().enumerate() {
            check!(
                Code::Rice {
                    log2_b: log2_b as _
                },
                *val
            );
        }
        for (k, val) in self.pi.iter().enumerate() {
            check!(Code::Pi { k: (k + 2) as _ }, *val);
        }
        for (k, val) in self.pi_web.iter().enumerate() {
            check!(Code::PiWeb { k: k as _ }, *val);
        }

        (best_code, best)
    }
}

/// Combines additively this stats with another one.
impl<
        const ZETA: usize,
        const GOLOMB: usize,
        const EXP_GOLOMB: usize,
        const RICE: usize,
        const PI: usize,
    > core::ops::AddAssign for CodesStats<ZETA, GOLOMB, EXP_GOLOMB, RICE, PI>
{
    fn add_assign(&mut self, rhs: Self) {
        self.add(&rhs);
    }
}

/// Combines additively this stats with another one creating a new one.
impl<
        const ZETA: usize,
        const GOLOMB: usize,
        const EXP_GOLOMB: usize,
        const RICE: usize,
        const PI: usize,
    > core::ops::Add for CodesStats<ZETA, GOLOMB, EXP_GOLOMB, RICE, PI>
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let mut res = self;
        res += rhs;
        res
    }
}

/// Allow to call .sum() on an iterator of CodesStats.
impl<
        const ZETA: usize,
        const GOLOMB: usize,
        const EXP_GOLOMB: usize,
        const RICE: usize,
        const PI: usize,
    > core::iter::Sum for CodesStats<ZETA, GOLOMB, EXP_GOLOMB, RICE, PI>
{
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::default(), |a, b| a + b)
    }
}

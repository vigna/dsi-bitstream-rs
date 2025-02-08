/*
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 * SPDX-FileCopyrightText: 2024 Tommaso Fontana
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */
#[cfg(feature = "mem_dbg")]
use mem_dbg::{MemDbg, MemSize};

use crate::prelude::code::{CodeRead, CodeReadDispatch, CodeWrite, CodeWriteDispatch};
use crate::prelude::Endianness;
use crate::prelude::{
    bit_len_vbyte, len_delta, len_exp_golomb, len_gamma, len_golomb, len_omega, len_pi, len_rice,
    len_zeta, Code, CodesRead, CodesWrite,
};
use anyhow::Result;
use core::fmt::Debug;
use std::sync::Mutex;

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
    /// The total number of elements observed.
    pub total: u64,
    /// The total space used to store the elements if
    /// they were stored using the unary code.
    pub unary: u64,
    /// The total space used to store the elements if
    /// they were stored using the gamma code.
    pub gamma: u64,
    /// The total space used to store the elements if
    /// they were stored using the delta code.
    pub delta: u64,
    /// The total space used to store the elements if
    /// they were stored using the omega code.
    pub omega: u64,
    /// The total space used to store the elements if
    /// they were stored using the variable byte code.
    pub vbyte: u64,
    /// The total space used to store the elements if
    /// they were stored using the zeta code.
    pub zeta: [u64; ZETA],
    /// The total space used to store the elements if
    /// they were stored using the Golomb code.
    pub golomb: [u64; GOLOMB],
    /// The total space used to store the elements if
    /// they were stored using the exponential Golomb code.
    pub exp_golomb: [u64; EXP_GOLOMB],
    /// The total space used to store the elements if
    /// they were stored using the Rice code.
    pub rice: [u64; RICE],
    /// The total space used to store the elements if
    /// they were stored using the Pi code.
    pub pi: [u64; PI],
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
            total: 0,
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
        self.total += count;
        self.unary += (n + 1) * count;
        self.gamma += len_gamma(n) as u64 * count;
        self.delta += len_delta(n) as u64 * count;
        self.omega += len_omega(n) as u64 * count;
        self.vbyte += bit_len_vbyte(n) as u64 * count;

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
        n
    }

    // Combines additively this stats with another one.
    pub fn add(&mut self, rhs: &Self) {
        self.total += rhs.total;
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

#[derive(Debug)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
/// A struct that can wrap `Code` and compute `CodesStats` for a given stream.
pub struct CodesStatsWrapper<
    W,
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
    // TODO!: figure out how we can do this without a lock.
    // This is needed because the `CodeRead` and `CodeWrite` traits must have
    // &self and not &mut self.
    stats: Mutex<CodesStats<ZETA, GOLOMB, EXP_GOLOMB, RICE, PI>>,
    wrapped: W,
}

impl<
        W,
        const ZETA: usize,
        const GOLOMB: usize,
        const EXP_GOLOMB: usize,
        const RICE: usize,
        const PI: usize,
    > CodesStatsWrapper<W, ZETA, GOLOMB, EXP_GOLOMB, RICE, PI>
{
    /// Create a new `CodesStatsWrapper` with the given wrapped value.
    pub fn new(wrapped: W) -> Self {
        Self {
            stats: Mutex::new(CodesStats::default()),
            wrapped,
        }
    }

    /// Returns a reference to the stats.
    pub fn stats(&self) -> &Mutex<CodesStats<ZETA, GOLOMB, EXP_GOLOMB, RICE, PI>> {
        &self.stats
    }

    /// Consumes the wrapper and returns the inner wrapped value and the stats.
    pub fn into_inner(self) -> (W, CodesStats<ZETA, GOLOMB, EXP_GOLOMB, RICE, PI>) {
        (self.wrapped, self.stats.into_inner().unwrap())
    }
}

impl<
        W,
        const ZETA: usize,
        const GOLOMB: usize,
        const EXP_GOLOMB: usize,
        const RICE: usize,
        const PI: usize,
    > CodeRead for CodesStatsWrapper<W, ZETA, GOLOMB, EXP_GOLOMB, RICE, PI>
where
    W: CodeRead,
{
    type Error<CRE>
        = W::Error<CRE>
    where
        CRE: Debug + Send + Sync + 'static;
    #[inline]
    fn read<E: Endianness, CR: CodesRead<E> + ?Sized>(
        &self,
        reader: &mut CR,
    ) -> Result<u64, Self::Error<CR::Error>> {
        let res = self.wrapped.read(reader)?;
        self.stats.lock().unwrap().update(res);
        Ok(res)
    }
}

impl<
        W,
        const ZETA: usize,
        const GOLOMB: usize,
        const EXP_GOLOMB: usize,
        const RICE: usize,
        const PI: usize,
        E: Endianness,
        CR: CodesRead<E> + ?Sized,
    > CodeReadDispatch<E, CR> for CodesStatsWrapper<W, ZETA, GOLOMB, EXP_GOLOMB, RICE, PI>
where
    W: CodeReadDispatch<E, CR>,
{
    type Error<CRE>
        = W::Error<CRE>
    where
        CRE: Debug + Send + Sync + 'static;
    #[inline]
    fn read_dispatch(&self, reader: &mut CR) -> Result<u64, Self::Error<CR::Error>> {
        let res = self.wrapped.read_dispatch(reader)?;
        self.stats.lock().unwrap().update(res);
        Ok(res)
    }
}

impl<
        W,
        const ZETA: usize,
        const GOLOMB: usize,
        const EXP_GOLOMB: usize,
        const RICE: usize,
        const PI: usize,
    > CodeWrite for CodesStatsWrapper<W, ZETA, GOLOMB, EXP_GOLOMB, RICE, PI>
where
    W: CodeWrite,
{
    type Error<CWE>
        = W::Error<CWE>
    where
        CWE: Debug + Send + Sync + 'static;
    #[inline]
    fn write<E: Endianness, CW: CodesWrite<E> + ?Sized>(
        &self,
        writer: &mut CW,
        value: u64,
    ) -> Result<usize, Self::Error<CW::Error>> {
        let res = self.wrapped.write(writer, value)?;
        self.stats.lock().unwrap().update(value);
        Ok(res)
    }
}

impl<
        W,
        const ZETA: usize,
        const GOLOMB: usize,
        const EXP_GOLOMB: usize,
        const RICE: usize,
        const PI: usize,
        E: Endianness,
        CW: CodesWrite<E> + ?Sized,
    > CodeWriteDispatch<E, CW> for CodesStatsWrapper<W, ZETA, GOLOMB, EXP_GOLOMB, RICE, PI>
where
    W: CodeWriteDispatch<E, CW>,
{
    type Error<CWE>
        = W::Error<CWE>
    where
        CWE: Debug + Send + Sync + 'static;
    #[inline]
    fn write_dispatch(&self, writer: &mut CW, value: u64) -> Result<usize, Self::Error<CW::Error>> {
        let res = self.wrapped.write_dispatch(writer, value)?;
        self.stats.lock().unwrap().update(value);
        Ok(res)
    }
}

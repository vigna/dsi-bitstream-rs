/*
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 * SPDX-FileCopyrightText: 2024 Tommaso Fontana
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

#[cfg(feature = "mem_dbg")]
use mem_dbg::{MemDbg, MemSize};

use crate::prelude::{
    Codes, bit_len_vbyte, len_delta, len_exp_golomb, len_gamma, len_minimal_binary, len_omega,
    len_pi, len_zeta,
};
use core::fmt::Debug;

#[cfg(feature = "std")]
use crate::dispatch::{CodesRead, CodesWrite};
#[cfg(feature = "std")]
use crate::prelude::Endianness;
#[cfg(feature = "std")]
use crate::prelude::{DynamicCodeRead, DynamicCodeWrite, StaticCodeRead, StaticCodeWrite};
#[cfg(feature = "std")]
use std::sync::Mutex;

#[cfg(feature = "serde")]
use alloc::string::ToString;
#[cfg(feature = "alloc")]
use alloc::{vec, vec::Vec};

/// Keeps track of the space needed to store a stream of integers using
/// different codes.
///
/// This structure can be used to determine empirically which code provides the
/// best compression for a given stream. You have to [update the structure] with
/// the integers in the stream; at any time, you can examine the statistics or
/// call [`best_code`] to get the best code.
///
/// The structure keeps tracks of the codes for which the module [`code_consts`]
/// provide constants.
///
/// [update the structure]: Self::update
/// [`best_code`]: Self::best_code
/// [`code_consts`]: crate::dispatch::code_consts
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
pub struct CodesStats<
    // How many ζ codes to consider.
    const ZETA: usize = 10,
    // How many Golomb codes to consider.
    const GOLOMB: usize = 10,
    // How many Exponential Golomb codes to consider.
    const EXP_GOLOMB: usize = 10,
    // How many Rice codes to consider.
    const RICE: usize = 10,
    // How many streamlined π codes to consider.
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
    /// The total space used to store the elements if they were stored using a
    /// zeta code. `zeta[0]` represents ζ₁, `zeta[1]` represents ζ₂, and so on.
    pub zeta: [u64; ZETA],
    /// The total space used to store the elements if they were stored using a
    /// Golomb code. `golomb[0]` represents the Golomb code with modulus 1,
    /// `golomb[1]` represents the Golomb code with modulus 2, and so on.
    pub golomb: [u64; GOLOMB],
    /// The total space used to store the elements if they were stored using an
    /// exponential Golomb code. `exp_golomb[0]` represents the exponential
    /// Golomb code with parameter 0, `exp_golomb[1]` with parameter 1, and
    /// so on.
    pub exp_golomb: [u64; EXP_GOLOMB],
    /// The total space used to store the elements if they were stored using a
    /// Rice code. `rice[0]` represents the Rice code with log₂(*b*) = 0,
    /// `rice[1]` with log₂(*b*) = 1, and so on.
    pub rice: [u64; RICE],
    /// The total space used to store the elements if they were stored using a
    /// pi code. `pi[0]` represents π₂, `pi[1]` represents π₃, and so on.
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

    /// Update the stats with `count` occurrences of `n` and return `n` for convenience.
    ///
    /// # Panics
    ///
    /// If `n` is [`u64::MAX`]: the totals for every code are updated on each
    /// call, and some code-length helpers (e.g. `len_gamma`) reject
    /// [`u64::MAX`] because they compute `n + 1` -- even though other codes
    /// (VByte, Golomb with b > 1, Rice/exp-Golomb with k > 0) could
    /// represent it.
    #[inline]
    pub fn update_many(&mut self, n: u64, count: u64) -> u64 {
        // No occurrences: record nothing.
        if count == 0 {
            return n;
        }
        self.total = self.total.saturating_add(count);

        // Saturating arithmetic: an overflowing total means the code is
        // impractically large for this data, so it saturates (and never wins
        // best_code) instead of panicking (debug) or wrapping to a small,
        // wrongly-winning value (release).
        self.unary = self.unary.saturating_add((n + 1).saturating_mul(count));
        self.gamma = self
            .gamma
            .saturating_add((len_gamma(n) as u64).saturating_mul(count));
        self.delta = self
            .delta
            .saturating_add((len_delta(n) as u64).saturating_mul(count));
        self.omega = self
            .omega
            .saturating_add((len_omega(n) as u64).saturating_mul(count));
        self.vbyte = self
            .vbyte
            .saturating_add((bit_len_vbyte(n) as u64).saturating_mul(count));
        for (k, val) in self.zeta.iter_mut().enumerate() {
            *val = val.saturating_add((len_zeta(n, (k + 1) as _) as u64).saturating_mul(count));
        }
        for (b, val) in self.golomb.iter_mut().enumerate() {
            // Length in u64 to avoid truncating n / b on 32-bit targets, where
            // len_golomb's usize return can drop high bits.
            let b = (b + 1) as u64;
            let len = (n / b) + 1 + len_minimal_binary(n % b, b) as u64;
            *val = val.saturating_add(len.saturating_mul(count));
        }
        for (k, val) in self.exp_golomb.iter_mut().enumerate() {
            *val = val.saturating_add((len_exp_golomb(n, k as _) as u64).saturating_mul(count));
        }
        for (log2_b, val) in self.rice.iter_mut().enumerate() {
            // Length in u64 to avoid truncating n >> log2_b on 32-bit targets.
            let len = (n >> log2_b) + 1 + log2_b as u64;
            *val = val.saturating_add(len.saturating_mul(count));
        }
        for (k, val) in self.pi.iter_mut().enumerate() {
            *val = val.saturating_add((len_pi(n, (k + 2) as _) as u64).saturating_mul(count));
        }
        n
    }

    /// Combines additively this stats with another one.
    pub fn add(&mut self, rhs: &Self) {
        self.total = self.total.saturating_add(rhs.total);
        self.unary = self.unary.saturating_add(rhs.unary);
        self.gamma = self.gamma.saturating_add(rhs.gamma);
        self.delta = self.delta.saturating_add(rhs.delta);
        self.omega = self.omega.saturating_add(rhs.omega);
        self.vbyte = self.vbyte.saturating_add(rhs.vbyte);
        for (a, b) in self.zeta.iter_mut().zip(rhs.zeta.iter()) {
            *a = a.saturating_add(*b);
        }
        for (a, b) in self.golomb.iter_mut().zip(rhs.golomb.iter()) {
            *a = a.saturating_add(*b);
        }
        for (a, b) in self.exp_golomb.iter_mut().zip(rhs.exp_golomb.iter()) {
            *a = a.saturating_add(*b);
        }
        for (a, b) in self.rice.iter_mut().zip(rhs.rice.iter()) {
            *a = a.saturating_add(*b);
        }
        for (a, b) in self.pi.iter_mut().zip(rhs.pi.iter()) {
            *a = a.saturating_add(*b);
        }
    }

    /// Returns the best code for the stream and its space usage.
    ///
    /// When VByte is the best code, [`Codes::VByteBe`] is returned as the
    /// canonical representative (both variants have the same bit length).
    #[must_use]
    pub fn best_code(&self) -> (Codes, u64) {
        let mut best = (Codes::Unary, self.unary);
        if self.gamma < best.1 {
            best = (Codes::Gamma, self.gamma);
        }
        if self.delta < best.1 {
            best = (Codes::Delta, self.delta);
        }
        if self.omega < best.1 {
            best = (Codes::Omega, self.omega);
        }
        if self.vbyte < best.1 {
            best = (Codes::VByteBe, self.vbyte);
        }
        for (k, val) in self.zeta.iter().enumerate() {
            if *val < best.1 {
                best = (Codes::Zeta((k + 1) as _), *val);
            }
        }
        for (b, val) in self.golomb.iter().enumerate() {
            if *val < best.1 {
                best = (Codes::Golomb((b + 1) as _), *val);
            }
        }
        for (k, val) in self.exp_golomb.iter().enumerate() {
            if *val < best.1 {
                best = (Codes::ExpGolomb(k as _), *val);
            }
        }
        for (log2_b, val) in self.rice.iter().enumerate() {
            if *val < best.1 {
                best = (Codes::Rice(log2_b as _), *val);
            }
        }
        for (k, val) in self.pi.iter().enumerate() {
            if *val < best.1 {
                best = (Codes::Pi((k + 2) as _), *val);
            }
        }
        best
    }

    /// Returns a vector of all codes and their space usage, in ascending order by space usage.
    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn get_codes(&self) -> Vec<(Codes, u64)> {
        let mut codes = vec![
            (Codes::Unary, self.unary),
            (Codes::Gamma, self.gamma),
            (Codes::Delta, self.delta),
            (Codes::Omega, self.omega),
            (Codes::VByteBe, self.vbyte),
        ];
        for (k, val) in self.zeta.iter().enumerate() {
            codes.push((Codes::Zeta((k + 1) as _), *val));
        }
        for (b, val) in self.golomb.iter().enumerate() {
            codes.push((Codes::Golomb((b + 1) as _), *val));
        }
        for (k, val) in self.exp_golomb.iter().enumerate() {
            codes.push((Codes::ExpGolomb(k as _), *val));
        }
        for (log2_b, val) in self.rice.iter().enumerate() {
            codes.push((Codes::Rice(log2_b as _), *val));
        }
        for (k, val) in self.pi.iter().enumerate() {
            codes.push((Codes::Pi((k + 2) as _), *val));
        }
        // sort them by length
        codes.sort_by_key(|&(_, len)| len);
        codes
    }

    /// Returns the number of bits used by the given code.
    #[must_use]
    pub fn bits_for(&self, code: Codes) -> Option<u64> {
        match code {
            Codes::Unary => Some(self.unary),
            Codes::Gamma => Some(self.gamma),
            Codes::Delta => Some(self.delta),
            Codes::Omega => Some(self.omega),
            Codes::VByteBe | Codes::VByteLe => Some(self.vbyte),
            Codes::Zeta(k) => self.zeta.get(k.checked_sub(1)?).copied(),
            Codes::Golomb(b) => self.golomb.get(b.checked_sub(1)? as usize).copied(),
            Codes::ExpGolomb(k) => self.exp_golomb.get(k).copied(),
            Codes::Rice(log2_b) => self.rice.get(log2_b).copied(),
            Codes::Pi(k) => self.pi.get(k.checked_sub(2)?).copied(),
        }
    }
}

impl<
    const ZETA: usize,
    const GOLOMB: usize,
    const EXP_GOLOMB: usize,
    const RICE: usize,
    const PI: usize,
> core::ops::AddAssign for CodesStats<ZETA, GOLOMB, EXP_GOLOMB, RICE, PI>
{
    /// Combines additively this stats with another one.
    fn add_assign(&mut self, rhs: Self) {
        self.add(&rhs);
    }
}

impl<
    const ZETA: usize,
    const GOLOMB: usize,
    const EXP_GOLOMB: usize,
    const RICE: usize,
    const PI: usize,
> core::ops::Add for CodesStats<ZETA, GOLOMB, EXP_GOLOMB, RICE, PI>
{
    type Output = Self;

    /// Combines additively this stats with another one, creating a new one.
    fn add(self, rhs: Self) -> Self {
        let mut res = self;
        res += rhs;
        res
    }
}

/// Allows calling `.sum()` on an iterator of [`CodesStats`].
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

#[cfg(feature = "serde")]
impl<
    const ZETA: usize,
    const GOLOMB: usize,
    const EXP_GOLOMB: usize,
    const RICE: usize,
    const PI: usize,
> serde::Serialize for CodesStats<ZETA, GOLOMB, EXP_GOLOMB, RICE, PI>
{
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("CodesStats", 11)?;
        state.serialize_field("total", &self.total)?;
        state.serialize_field("unary", &self.unary)?;
        state.serialize_field("gamma", &self.gamma)?;
        state.serialize_field("delta", &self.delta)?;
        state.serialize_field("omega", &self.omega)?;
        state.serialize_field("vbyte", &self.vbyte)?;
        // these are array which don't play well with serde, so we convert them to slices
        state.serialize_field("zeta", &self.zeta.as_slice())?;
        state.serialize_field("golomb", &self.golomb.as_slice())?;
        state.serialize_field("exp_golomb", &self.exp_golomb.as_slice())?;
        state.serialize_field("rice", &self.rice.as_slice())?;
        state.serialize_field("pi", &self.pi.as_slice())?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<
    'de,
    const ZETA: usize,
    const GOLOMB: usize,
    const EXP_GOLOMB: usize,
    const RICE: usize,
    const PI: usize,
> serde::Deserialize<'de> for CodesStats<ZETA, GOLOMB, EXP_GOLOMB, RICE, PI>
{
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::{MapAccess, Visitor};

        struct CodesStatsVisitor<
            const ZETA: usize,
            const GOLOMB: usize,
            const EXP_GOLOMB: usize,
            const RICE: usize,
            const PI: usize,
        >;

        impl<
            'de,
            const ZETA: usize,
            const GOLOMB: usize,
            const EXP_GOLOMB: usize,
            const RICE: usize,
            const PI: usize,
        > Visitor<'de> for CodesStatsVisitor<ZETA, GOLOMB, EXP_GOLOMB, RICE, PI>
        {
            type Value = CodesStats<ZETA, GOLOMB, EXP_GOLOMB, RICE, PI>;

            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                formatter.write_str("struct CodesStats")
            }

            fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<Self::Value, V::Error> {
                let mut total = None;
                let mut unary = None;
                let mut gamma = None;
                let mut delta = None;
                let mut omega = None;
                let mut vbyte = None;
                let mut zeta: Option<[u64; ZETA]> = None;
                let mut golomb: Option<[u64; GOLOMB]> = None;
                let mut exp_golomb: Option<[u64; EXP_GOLOMB]> = None;
                let mut rice: Option<[u64; RICE]> = None;
                let mut pi: Option<[u64; PI]> = None;

                // Helper to deserialize a Vec<u64> into a fixed-size array
                fn vec_to_array<E: serde::de::Error, const N: usize>(
                    v: Vec<u64>,
                ) -> Result<[u64; N], E> {
                    v.try_into().map_err(|v: Vec<u64>| {
                        serde::de::Error::invalid_length(v.len(), &N.to_string().as_str())
                    })
                }

                while let Some(key) = map.next_key::<&str>()? {
                    match key {
                        "total" => total = Some(map.next_value()?),
                        "unary" => unary = Some(map.next_value()?),
                        "gamma" => gamma = Some(map.next_value()?),
                        "delta" => delta = Some(map.next_value()?),
                        "omega" => omega = Some(map.next_value()?),
                        "vbyte" => vbyte = Some(map.next_value()?),
                        "zeta" => zeta = Some(vec_to_array(map.next_value()?)?),
                        "golomb" => golomb = Some(vec_to_array(map.next_value()?)?),
                        "exp_golomb" => exp_golomb = Some(vec_to_array(map.next_value()?)?),
                        "rice" => rice = Some(vec_to_array(map.next_value()?)?),
                        "pi" => pi = Some(vec_to_array(map.next_value()?)?),
                        _ => {
                            let _ = map.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }

                Ok(CodesStats {
                    total: total.unwrap_or_default(),
                    unary: unary.unwrap_or_default(),
                    gamma: gamma.unwrap_or_default(),
                    delta: delta.unwrap_or_default(),
                    omega: omega.unwrap_or_default(),
                    vbyte: vbyte.unwrap_or_default(),
                    zeta: zeta.unwrap_or([0; ZETA]),
                    golomb: golomb.unwrap_or([0; GOLOMB]),
                    exp_golomb: exp_golomb.unwrap_or([0; EXP_GOLOMB]),
                    rice: rice.unwrap_or([0; RICE]),
                    pi: pi.unwrap_or([0; PI]),
                })
            }
        }

        deserializer.deserialize_struct(
            "CodesStats",
            &[
                "total",
                "unary",
                "gamma",
                "delta",
                "omega",
                "vbyte",
                "zeta",
                "golomb",
                "exp_golomb",
                "rice",
                "pi",
            ],
            CodesStatsVisitor::<ZETA, GOLOMB, EXP_GOLOMB, RICE, PI>,
        )
    }
}

/// A struct that can wrap a [`DynamicCodeRead`], [`DynamicCodeWrite`],
/// [`StaticCodeRead`], or [`StaticCodeWrite`] and compute [`CodesStats`]
/// for a given stream.
#[derive(Debug)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
#[cfg(feature = "std")]
pub struct CodesStatsWrapper<
    W,
    // How many ζ codes to consider.
    const ZETA: usize = 10,
    // How many Golomb codes to consider.
    const GOLOMB: usize = 10,
    // How many Exponential Golomb codes to consider.
    const EXP_GOLOMB: usize = 10,
    // How many Rice codes to consider.
    const RICE: usize = 10,
    // How many streamlined π codes to consider.
    const PI: usize = 10,
> {
    // TODO: figure out how we can do this without a lock.
    // This is needed because the [`DynamicCodeRead`] and [`DynamicCodeWrite`] traits must have
    // &self and not &mut self.
    stats: Mutex<CodesStats<ZETA, GOLOMB, EXP_GOLOMB, RICE, PI>>,
    wrapped: W,
}

#[cfg(feature = "std")]
impl<
    W,
    const ZETA: usize,
    const GOLOMB: usize,
    const EXP_GOLOMB: usize,
    const RICE: usize,
    const PI: usize,
> CodesStatsWrapper<W, ZETA, GOLOMB, EXP_GOLOMB, RICE, PI>
{
    /// Creates a new `CodesStatsWrapper` with the given wrapped value.
    #[must_use]
    pub fn new(wrapped: W) -> Self {
        Self {
            stats: Mutex::new(CodesStats::default()),
            wrapped,
        }
    }

    /// Returns a reference to the stats.
    pub const fn stats(&self) -> &Mutex<CodesStats<ZETA, GOLOMB, EXP_GOLOMB, RICE, PI>> {
        &self.stats
    }

    /// Consumes the wrapper and returns the inner wrapped value and the stats.
    #[must_use]
    pub fn into_inner(self) -> (W, CodesStats<ZETA, GOLOMB, EXP_GOLOMB, RICE, PI>) {
        (
            self.wrapped,
            self.stats.into_inner().expect("mutex is not poisoned"),
        )
    }
}

#[cfg(feature = "std")]
impl<
    W: DynamicCodeRead,
    const ZETA: usize,
    const GOLOMB: usize,
    const EXP_GOLOMB: usize,
    const RICE: usize,
    const PI: usize,
> DynamicCodeRead for CodesStatsWrapper<W, ZETA, GOLOMB, EXP_GOLOMB, RICE, PI>
{
    #[inline]
    fn read<E: Endianness, CR: CodesRead<E> + ?Sized>(
        &self,
        reader: &mut CR,
    ) -> Result<u64, CR::Error> {
        let res = self.wrapped.read(reader)?;
        self.stats.lock().unwrap().update(res);
        Ok(res)
    }
}

#[cfg(feature = "std")]
impl<
    W: StaticCodeRead<E, CR>,
    const ZETA: usize,
    const GOLOMB: usize,
    const EXP_GOLOMB: usize,
    const RICE: usize,
    const PI: usize,
    E: Endianness,
    CR: CodesRead<E> + ?Sized,
> StaticCodeRead<E, CR> for CodesStatsWrapper<W, ZETA, GOLOMB, EXP_GOLOMB, RICE, PI>
{
    #[inline]
    fn read(&self, reader: &mut CR) -> Result<u64, CR::Error> {
        let res = self.wrapped.read(reader)?;
        self.stats.lock().unwrap().update(res);
        Ok(res)
    }
}

#[cfg(feature = "std")]
impl<
    W: DynamicCodeWrite,
    const ZETA: usize,
    const GOLOMB: usize,
    const EXP_GOLOMB: usize,
    const RICE: usize,
    const PI: usize,
> DynamicCodeWrite for CodesStatsWrapper<W, ZETA, GOLOMB, EXP_GOLOMB, RICE, PI>
{
    #[inline]
    fn write<E: Endianness, CW: CodesWrite<E> + ?Sized>(
        &self,
        writer: &mut CW,
        n: u64,
    ) -> Result<usize, CW::Error> {
        let res = self.wrapped.write(writer, n)?;
        self.stats.lock().unwrap().update(n);
        Ok(res)
    }
}

#[cfg(feature = "std")]
impl<
    W: StaticCodeWrite<E, CW>,
    const ZETA: usize,
    const GOLOMB: usize,
    const EXP_GOLOMB: usize,
    const RICE: usize,
    const PI: usize,
    E: Endianness,
    CW: CodesWrite<E> + ?Sized,
> StaticCodeWrite<E, CW> for CodesStatsWrapper<W, ZETA, GOLOMB, EXP_GOLOMB, RICE, PI>
{
    #[inline]
    fn write(&self, writer: &mut CW, n: u64) -> Result<usize, CW::Error> {
        let res = self.wrapped.write(writer, n)?;
        self.stats.lock().unwrap().update(n);
        Ok(res)
    }
}

#[cfg(test)]
#[cfg(feature = "serde")]
mod serde_tests {
    use super::*;

    #[test]
    fn test_serde_code_stats() -> serde_json::Result<()> {
        let mut stats: CodesStats = CodesStats::default();
        for i in 0..100 {
            stats.update(i);
        }
        let json = serde_json::to_string(&stats)?;
        let deserialized: CodesStats = serde_json::from_str(&json)?;
        assert_eq!(stats, deserialized);
        Ok(())
    }

    #[test]
    fn test_roundtrip_different_sizes() -> serde_json::Result<()> {
        let mut stats: CodesStats<10, 20, 5, 8, 6> = CodesStats::default();
        for i in 0..1000 {
            stats.update(i);
        }
        let json = serde_json::to_string_pretty(&stats)?;
        let deserialized: CodesStats<10, 20, 5, 8, 6> = serde_json::from_str(&json)?;
        assert_eq!(stats, deserialized);
        Ok(())
    }

    #[test]
    #[should_panic]
    fn test_mismatched_sizes() {
        let mut stats: CodesStats<10, 20, 5, 8, 6> = CodesStats::default();
        for i in 0..1000 {
            stats.update(i);
        }
        let json = serde_json::to_string_pretty(&stats).unwrap();
        // This should panic because the JSON has 20 golomb values but we expect 21
        let _deserialized: CodesStats<10, 21, 5, 8, 6> = serde_json::from_str(&json).unwrap();
    }
}

#[cfg(test)]
mod robustness_tests {
    use super::*;

    #[test]
    #[should_panic]
    fn u64_max_panics_as_documented() {
        let mut stats: CodesStats = CodesStats::default();
        // update evaluates every code's length helper, and helpers such as
        // len_gamma reject u64::MAX (they compute n + 1); the documented
        // contract is a panic.
        stats.update(u64::MAX);
    }

    #[test]
    fn test_count_zero_is_a_noop() {
        let mut stats: CodesStats = CodesStats::default();
        let before = stats;
        stats.update_many(u64::MAX, 0);
        assert_eq!(
            stats, before,
            "update_many with count 0 must change nothing"
        );
    }

    #[test]
    fn test_large_count_saturates_without_panic() {
        let mut stats: CodesStats = CodesStats::default();
        // (n + 1) * count overflows u64; it must saturate rather than panic
        // (debug) or wrap to a small, wrongly-winning value (release).
        stats.update_many(1 << 40, 1 << 30);
        let (_, bits) = stats.best_code();
        // A finite-length code (e.g. gamma) still wins; unary saturated.
        assert!(bits < u64::MAX);
    }
}

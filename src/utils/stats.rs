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
    Codes, bit_len_vbyte, len_delta, len_exp_golomb, len_gamma, len_golomb, len_omega, len_pi,
    len_rice, len_zeta,
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

#[cfg(feature = "alloc")]
use alloc::vec;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Keeps track of the space needed to store a stream of integers using
/// different codes.
///
/// This structure can be used to determine empirically which code provides the
/// best compression for a given stream. You have to [update the
/// structure](Self::update) with the integers in the stream; at any time, you
/// can examine the statistics or call [`best_code`](Self::best_code) to get the
/// best code.
///
/// The structure keeps tracks of the codes for which the module
/// [`code_consts`](crate::dispatch::code_consts) provide constants.
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

    /// Combines additively this stats with another one.
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

    /// Returns the best code for the stream and its space usage.
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

/// A struct that can wrap `Codes` and compute `CodesStats` for a given stream.
#[derive(Debug)]
#[cfg_attr(feature = "mem_dbg", derive(MemDbg, MemSize))]
#[cfg(feature = "std")]
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
        value: u64,
    ) -> Result<usize, CW::Error> {
        let res = self.wrapped.write(writer, value)?;
        self.stats.lock().unwrap().update(value);
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
    fn write(&self, writer: &mut CW, value: u64) -> Result<usize, CW::Error> {
        let res = self.wrapped.write(writer, value)?;
        self.stats.lock().unwrap().update(value);
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

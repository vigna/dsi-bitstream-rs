/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Utility functions for benchmarks.

/// Pins the process to one core to avoid context switching and cache flushes
/// which would result in noise in the measurement.
#[cfg(target_os = "linux")]
pub fn pin_to_core(core_id: usize) {
    unsafe {
        let mut cpu_set = core::mem::MaybeUninit::zeroed().assume_init();
        libc::CPU_ZERO(&mut cpu_set);
        libc::CPU_SET(core_id, &mut cpu_set);
        let res = libc::sched_setaffinity(
            libc::getpid(),
            core::mem::size_of::<libc::cpu_set_t>(),
            &cpu_set as *const libc::cpu_set_t,
        );
        assert_ne!(res, -1);
    }
}

/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use dsi_bitstream::utils::{ToInt, ToNat};

#[test]
fn test_toint_tonat_8() {
    for i in -128_i8..=127 {
        assert_eq!(i.to_nat().to_int(), i);
    }

    for i in 0_u8..=255 {
        assert_eq!(i.to_int().to_nat(), i);
    }
}

#[test]
fn test_toint_tonat_16() {
    for i in -32768_i16..=32767 {
        assert_eq!(i.to_nat().to_int(), i);
    }

    for i in 0_u16..=65535 {
        assert_eq!(i.to_int().to_nat(), i);
    }
}

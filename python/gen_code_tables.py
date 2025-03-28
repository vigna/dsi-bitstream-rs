#!/usr/bin/env python3

#
# SPDX-FileCopyrightText: 2023 Tommaso Fontana
# SPDX-FileCopyrightText: 2023 Inria
# SPDX-FileCopyrightText: 2023 Sebastiano Vigna
#
# SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
#

"""
To run just execute `$ python ./gen_code_tables.py`
To provide a build folder, pass it as the first positional argument.

This script is not implemented using the `build.rs` mechanism because 
it would significantly slow down the build process. Moreover, the tables
will be generated very rarely.
"""
import os
import subprocess
from math import log2, ceil, floor

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ROOT = os.path.join(ROOT, "src", "codes")

def get_best_fitting_type(n_bits):
    """Find the smallest Rust type that can fit n_bits"""
    if n_bits <= 8:
        return "u8"
    if n_bits <= 16:
        return "u16"
    if n_bits <= 32:
        return "u32"
    if n_bits <= 64:
        return "u64"
    if n_bits <= 128:
        return "u128"
    raise ValueError(n_bits)


read_func_merged_table = """
#[inline(always)]
/// Read a value using a decoding table.
///
/// If the result is `Some` the decoding was successful, and
/// the decoded value and the length of the code are returned.
pub fn read_table_%(bo)s<B: BitRead<%(BO)s>>(backend: &mut B) -> Option<(u64, usize)> {
    if let Ok(idx) = backend.peek_bits(READ_BITS) {
        let idx: u64 = idx.cast();
        let (value, len) = READ_%(BO)s[idx as usize];
        if len != MISSING_VALUE_LEN_%(BO)s {
            backend.skip_bits_after_peek(len as usize);
            return Some((value as u64, len as usize));
        }
    }
    None
}
"""

read_func_two_table = """
#[inline(always)]
/// Read a value using a decoding table.
///
/// If the result is `Some` the decoding was successful, and
/// the decoded value and the length of the code are returned.
pub fn read_table_%(bo)s<B: BitRead<%(BO)s>>(backend: &mut B) -> Option<(u64, usize)> {
    if let Ok(idx) = backend.peek_bits(READ_BITS) {
        let idx: u64 = idx.cast();
        let len = READ_LEN_%(BO)s[idx as usize];
        if len != MISSING_VALUE_LEN_%(BO)s {
            backend.skip_bits_after_peek(len as usize);
            return Some((READ_%(BO)s[idx as usize] as u64, len as usize));
        }
    }
    None
}
#[inline(always)]
/// Compute the length of the code representing a value using a decoding table.
///
/// If the result is `Some` the lookup was successful, and
/// the length of the code is returned.
pub fn len_table_%(bo)s<B: BitRead<%(BO)s>>(backend: &mut B) -> Option<usize> {
    if let Ok(idx) = backend.peek_bits(READ_BITS) {
        let idx: u64 = idx.cast();
        let len = READ_LEN_%(BO)s[idx as usize];
        if len != MISSING_VALUE_LEN_%(BO)s {
            backend.skip_bits_after_peek(len as usize);
            return Some(len as usize);
        }
    }
    None
}
"""

write_func_merged_table = """
#[inline(always)]
#[allow(clippy::unnecessary_cast)]  // rationale: "*bits as u64" is flaky redundant
/// Write a value using an encoding table.
///
/// If the result is `Some` the encoding was successful, and
/// length of the code is returned.
pub fn write_table_%(bo)s<B: BitWrite<%(BO)s>>(backend: &mut B, value: u64) -> Result<Option<usize>, B::Error> {
    Ok(if let Some((bits, len)) = WRITE_%(BO)s.get(value as usize) {
        backend.write_bits(*bits as u64, *len as usize)?;
        Some(*len as usize)        
    } else {
        None
    })
}
"""

write_func_two_table = """
#[inline(always)]
/// Write a value using an encoding table.
///
/// If the result is `Some` the encoding was successful, and
/// length of the code is returned.
pub fn write_table_%(bo)s<B: BitWrite<%(BO)s>>(backend: &mut B, value: u64) -> Result<Option<usize>, B::Error> {
    Ok(if let Some(bits) = WRITE_%(BO)s.get(value as usize) {
        let len = WRITE_LEN_%(BO)s[value as usize] as usize;
        backend.write_bits(*bits as u64, len)?;
        Some(len)
    } else {
        None
    })
}
"""

def gen_table(
    read_bits,
    write_max_val,
    len_max_val,
    code_name,
    len_func,
    read_func,
    write_func,
    merged_table,
):
    """Main routine that generates the tables for a given code."""

    with open(os.path.join(ROOT, "{}_tables.rs".format(code_name)), "w") as f:
        f.write(
            "#![doc(hidden)]\n// THIS FILE HAS BEEN GENERATED BY THE SCRIPT {}\n".format(
                os.path.basename(__file__)
            )
        )
        f.write("// ~~~~~~~~~~~~~~~~~~~ DO NOT MODIFY ~~~~~~~~~~~~~~~~~~~~~~\n")
        f.write(
            "// Methods for reading and writing values using precomputed tables for {} codes\n".format(  # NoQA
                code_name
            )
        )
        f.write("use crate::traits::{BitRead, BitWrite, BE, LE};\n")
        f.write("use common_traits::*;\n")

        f.write("/// How many bits are needed to read the tables in this\n")
        f.write("pub const READ_BITS: usize = {};\n".format(read_bits))
        f.write(
            "/// Maximum value writable using the table(s)\n"
        )
        f.write(
            "pub const WRITE_MAX: u64 = {};\n".format(write_max_val)
        )

        if merged_table:
            read_func_template = read_func_merged_table
            write_func_template = write_func_merged_table
        else:
            read_func_template = read_func_two_table
            write_func_template = write_func_two_table

        for bo in ["le", "be"]:
            f.write(read_func_template % {"bo": bo, "BO": bo.upper()})
            f.write(write_func_template % {"bo": bo, "BO": bo.upper()})

        # Write the read tables
        for BO in ["BE", "LE"]:
            codes = []
            for value in range(0, 2**read_bits):
                bits = ("{:0%sb}" % read_bits).format(value)
                try:
                    value, bits_left = read_func(bits, BO == "BE")
                    codes.append((value, read_bits - len(bits_left)))
                except ValueError:
                    codes.append((None, None))
            read_max_val = max(x[0] or 0 for x in codes)
            read_max_len = max(x[1] or 0 for x in codes)
            len_ty = "u8"
            f.write(
                "/// The len we assign to a code that cannot be decoded through the table\n"
            )
            f.write(
                "pub const MISSING_VALUE_LEN_{}: {} = {};\n".format(BO, len_ty, read_max_len + 1)
            )


            if merged_table:
                f.write(
                    "/// Precomputed table for reading {} codes\n".format(
                        code_name
                    )
                )
                f.write(
                    "pub const READ_%s: &[(%s, %s)] = &["
                    % (
                        BO,
                        get_best_fitting_type(log2(read_max_val + 1)),
                        get_best_fitting_type(log2(read_max_len + 2)),  
                    )
                )
                for value, l in codes:
                    f.write("({}, {}), ".format(value or 0, l or (read_max_len + 1)))
                f.write("];\n")
            else:
                f.write(
                    "/// Precomputed table for writing {} codes\n".format(  # NoQA
                        code_name
                    )
                )
                f.write(
                    "pub const READ_%s: &[%s] = &["
                    % (
                        BO,
                        get_best_fitting_type(log2(read_max_val + 1)),
                    )
                )
                for value, l in codes:
                    f.write("{}, ".format(value or 0))
                f.write("];\n")

                f.write(
                    "/// Precomputed lengths table for reading {} codes\n".format(  # NoQA
                        code_name
                    )
                )
                f.write(
                    "pub const READ_LEN_%s: &[%s] = &["
                    % (
                        BO,
                        get_best_fitting_type(log2(read_max_len + 2)),
                    )
                )
                for value, l in codes:
                    f.write("{}, ".format(l or (read_max_len + 1)))
                f.write("];\n")

        # Write the write tables
        for bo in ["BE", "LE"]:
            if merged_table:
                f.write(
                    "/// Precomputed lengths table for writing {} codes\n".format(code_name)
                )
                f.write(
                    "pub const WRITE_%s: &[(%s, u8)] = &["
                    % (bo, get_best_fitting_type(len_func(write_max_val)))
                )
                for value in range(write_max_val + 1):
                    bits = write_func(value, "", bo == "BE")
                    f.write("({}, {}),".format(int(bits, 2), len(bits)))
                f.write("];\n")
            else:
                f.write(
                    "///Table used to speed up the writing of {} codes\n".format(code_name)
                )
                f.write(
                    "pub const WRITE_%s: &[%s] = &["
                    % (bo, get_best_fitting_type(len_func(write_max_val)))
                )
                len_bits = []
                for value in range(write_max_val + 1):
                    bits = write_func(value, "", bo == "BE")
                    len_bits.append(len(bits))
                    f.write("{},".format(int(bits, 2)))

                f.write("];\n")

                f.write(
                    "///Table used to speed up the writing of {} codes\n".format(code_name)
                )
                f.write(
                    "pub const WRITE_LEN_%s: &[%s] = &["
                    % (bo, get_best_fitting_type(len_func(write_max_val)))
                )
                for l in len_bits:
                    f.write("{}, ".format(l))
                f.write("];\n")

        # Write the len table
        f.write(
            "///Table used to speed up the skipping of {} codes\n".format(code_name)
        )
        f.write(
            "pub const LEN: &[%s] = &["
            % (get_best_fitting_type(ceil(log2(len_func(len_max_val)))))
        )
        for value in range(write_max_val + 1):
            f.write("{}, ".format(len_func(value)))
        f.write("];\n")


################################################################################


def read_fixed(n_bits, bitstream, be):
    """Read a fixed number of bits"""
    if len(bitstream) < n_bits:
        raise ValueError()

    if be:
        return int(bitstream[:n_bits], 2), bitstream[n_bits:]
    else:
        return int(bitstream[-n_bits:], 2), bitstream[:-n_bits]


def write_fixed(value, n_bits, bitstream, be):
    """Write a fixed number of bits"""
    if be:
        return bitstream + ("{:0%sb}" % n_bits).format(value)
    else:
        return ("{:0%sb}" % n_bits).format(value) + bitstream


################################################################################


def read_unary(bitstream, be):
    """Read an unary code"""
    if be:
        l = len(bitstream) - len(bitstream.lstrip("0"))  # NoQA: E741
        if l == len(bitstream):
            raise ValueError()
        return l, bitstream[l + 1 :]
    else:
        l = len(bitstream) - len(bitstream.rstrip("0"))  # NoQA: E741
        if l == len(bitstream):
            raise ValueError()
        return l, bitstream[: -l - 1]


def write_unary(value, bitstream, be):
    """Write an unary code"""
    if be:
        return bitstream + "0" * value + "1"
    else:
        return "1" + "0" * value + bitstream


def len_unary(value):
    """The len of an unary code for value"""
    return value + 1


# Test that the impl is reasonable
assert write_unary(0, "", True) == "1"
assert write_unary(0, "", False) == "1"
assert write_unary(1, "", True) == "01"
assert write_unary(1, "", False) == "10"
assert write_unary(2, "", True) == "001"
assert write_unary(2, "", False) == "100"
assert write_unary(3, "", True) == "0001"
assert write_unary(3, "", False) == "1000"

# Little consistency check
for i in range(256):
    wbe = write_unary(i, "", True)
    rbe = read_unary(wbe, True)[0]
    wle = write_unary(i, "", False)
    rle = read_unary(wle, False)[0]
    l = len_unary(i)  # NoQA: E741
    assert i == rbe
    assert i == rle
    assert len(wbe) == l
    assert len(wle) == l


def gen_unary(read_bits, write_max_val, len_max_val=None, merged_table=False):
    """Configuration of `gen_table` for unary"""
    len_max_val = len_max_val or write_max_val
    return gen_table(
        read_bits,
        min(write_max_val, 63),
        len_max_val,
        "unary",
        len_unary,
        read_unary,
        write_unary,
        merged_table,
    )


################################################################################


def read_gamma(bitstream, be):
    """Read a gamma code"""
    l, bitstream = read_unary(bitstream, be)
    if l == 0:
        return 0, bitstream
    f, bitstream = read_fixed(l, bitstream, be)
    v = f + (1 << l) - 1
    return v, bitstream


def write_gamma(value, bitstream, be):
    """Write a gamma code"""
    value += 1
    l = floor(log2(value))  # NoQA: E741
    s = value - (1 << l)
    bitstream = write_unary(l, bitstream, be)
    if l != 0:
        bitstream = write_fixed(s, l, bitstream, be)
    return bitstream


def len_gamma(value):
    """Length of the gamma code of `value`"""
    value += 1
    l = floor(log2(value))  # NoQA: E741
    return 2 * l + 1


# Test that the impl is reasonable
assert write_gamma(0, "", True) == "1"
assert write_gamma(0, "", False) == "1"
assert write_gamma(1, "", True) == "010"
assert write_gamma(1, "", False) == "010"
assert write_gamma(2, "", True) == "011"
assert write_gamma(2, "", False) == "110"
assert write_gamma(3, "", True) == "00100"
assert write_gamma(3, "", False) == "00100"
assert write_gamma(4, "", True) == "00101"
assert write_gamma(4, "", False) == "01100"
assert write_gamma(5, "", True) == "00110"
assert write_gamma(5, "", False) == "10100"

# Little consistency check
for i in range(256):
    wbe = write_gamma(i, "", True)
    rbe = read_gamma(wbe, True)[0]
    wle = write_gamma(i, "", False)
    rle = read_gamma(wle, False)[0]
    l = len_gamma(i)  # NoQA: E741
    assert i == rbe
    assert i == rle
    assert len(wbe) == l
    assert len(wle) == l


def gen_gamma(read_bits, write_max_val, len_max_val=None, merged_table=False):
    """Configuration of `gen_table` for gamma"""
    assert read_bits > 0
    len_max_val = len_max_val or write_max_val
    return gen_table(
        read_bits,
        write_max_val,
        len_max_val,
        "gamma",
        len_gamma,
        read_gamma,
        write_gamma,
        merged_table,
    )


################################################################################


def read_delta(bitstream, be):
    """Read a delta code"""
    l, bitstream = read_gamma(bitstream, be)
    if l == 0:
        return 0, bitstream
    f, bitstream = read_fixed(l, bitstream, be)
    v = f + (1 << l) - 1
    return v, bitstream


def write_delta(value, bitstream, be):
    """Write a delta code"""
    value += 1
    l = floor(log2(value))  # NoQA: E741
    s = value - (1 << l)
    bitstream = write_gamma(l, bitstream, be)
    if l != 0:
        bitstream = write_fixed(s, l, bitstream, be)
    return bitstream


def len_delta(value):
    """Length of the delta code of `value`"""
    value += 1
    l = floor(log2(value))  # NoQA: E741
    return l + len_gamma(l)


# Test that the impl is reasonable
assert write_delta(0, "", True) == "1"
assert write_delta(0, "", False) == "1"
assert write_delta(1, "", True) == "0100"
assert write_delta(1, "", False) == "0010"
assert write_delta(2, "", True) == "0101"
assert write_delta(2, "", False) == "1010"
assert write_delta(3, "", True) == "01100"
assert write_delta(3, "", False) == "00110"
assert write_delta(4, "", True) == "01101"
assert write_delta(4, "", False) == "01110"
assert write_delta(5, "", True) == "01110"
assert write_delta(5, "", False) == "10110"

# Little consistency check
for i in range(256):
    wbe = write_delta(i, "", True)
    rbe = read_delta(wbe, True)[0]
    wle = write_delta(i, "", False)
    rle = read_delta(wle, False)[0]
    l = len_delta(i)  # NoQA: E741
    assert i == rbe
    assert i == rle
    assert len(wbe) == l
    assert len(wle) == l


def gen_delta(read_bits, write_max_val, len_max_val=None, merged_table=False):
    """Configuration of `gen_table` for delta"""
    assert read_bits > 0
    len_max_val = len_max_val or write_max_val
    return gen_table(
        read_bits,
        write_max_val,
        len_max_val,
        "delta",
        len_delta,
        read_delta,
        write_delta,
        merged_table,
    )


################################################################################


def read_minimal_binary(max, bitstream, be):
    """Read a minimal binary code code with max `max`"""
    l = int(floor(log2(max)))  # NoQA: E741
    v, bitstream = read_fixed(l, bitstream, be)
    limit = (1 << (l + 1)) - max

    if v < limit:
        return v, bitstream
    else:
        b, bitstream = read_fixed(1, bitstream, be)
        v = (v << 1) | b
        return v - limit, bitstream


def write_minimal_binary(value, max, bitstream, be):
    """Write a minimal binary code with max `max`"""
    l = int(floor(log2(max)))  # NoQA: E741
    limit = (1 << (l + 1)) - max

    if value < limit:
        return write_fixed(value, l, bitstream, be)
    else:
        to_write = value + limit
        bitstream = write_fixed(to_write >> 1, l, bitstream, be)
        return write_fixed(to_write & 1, 1, bitstream, be)


def len_minimal_binary(value, max):
    """Length of the minimal binary code of `value` with max `max`"""
    l = int(floor(log2(max)))  # NoQA: E741
    limit = (1 << (l + 1)) - max
    if value >= limit:
        return l + 1
    else:
        return l


# Test that the impl is reasonable
assert write_minimal_binary(0, 10, "", True) == "000"
assert write_minimal_binary(0, 10, "", False) == "000"
assert write_minimal_binary(1, 10, "", True) == "001"
assert write_minimal_binary(1, 10, "", False) == "001"
assert write_minimal_binary(2, 10, "", True) == "010"
assert write_minimal_binary(2, 10, "", False) == "010"
assert write_minimal_binary(3, 10, "", True) == "011"
assert write_minimal_binary(3, 10, "", False) == "011"
assert write_minimal_binary(4, 10, "", True) == "100"
assert write_minimal_binary(4, 10, "", False) == "100"
assert write_minimal_binary(5, 10, "", True) == "101"
assert write_minimal_binary(5, 10, "", False) == "101"

assert write_minimal_binary(6, 10, "", True) == "1100"
assert write_minimal_binary(6, 10, "", False) == "0110"
assert write_minimal_binary(7, 10, "", True) == "1101"
assert write_minimal_binary(7, 10, "", False) == "1110"
assert write_minimal_binary(8, 10, "", True) == "1110"
assert write_minimal_binary(8, 10, "", False) == "0111"
assert write_minimal_binary(9, 10, "", True) == "1111"
assert write_minimal_binary(9, 10, "", False) == "1111"

# Little consistency check
_max = 200
for i in range(_max):
    wbe = write_minimal_binary(i, _max, "", True)
    rbe = read_minimal_binary(_max, wbe, True)[0]
    wle = write_minimal_binary(i, _max, "", False)
    rle = read_minimal_binary(_max, wle, False)[0]
    l = len_minimal_binary(i, _max)  # NoQA: E741
    assert i == rbe
    assert i == rle
    assert len(wbe) == l
    assert len(wle) == l

################################################################################


def read_zeta(bitstream, k, be):
    """Read a zeta code"""
    h, bitstream = read_unary(bitstream, be)
    u = 2 ** ((h + 1) * k)
    l = 2 ** (h * k)  # NoQA: E741
    r, bitstream = read_minimal_binary(u - l, bitstream, be)
    return l + r - 1, bitstream


def write_zeta(value, k, bitstream, be):
    """Write a zeta code"""
    value += 1
    h = int(floor(log2(value)) / k)
    u = 2 ** ((h + 1) * k)
    l = 2 ** (h * k)  # NoQA: E741

    bitstream = write_unary(h, bitstream, be)
    bitstream = write_minimal_binary(value - l, u - l, bitstream, be)
    return bitstream


def len_zeta(value, k):
    """Length of the zeta code of `value`"""
    value += 1
    h = int(floor(log2(value)) / k)
    u = 2 ** ((h + 1) * k)
    l = 2 ** (h * k)  # NoQA: E741
    return len_unary(h) + len_minimal_binary(value - l, u - l)


# Test that the impl is reasonable
assert write_zeta(0, 3, "", True) == "100"
assert write_zeta(1, 3, "", True) == "1010"
assert write_zeta(2, 3, "", True) == "1011"
assert write_zeta(3, 3, "", True) == "1100"
assert write_zeta(4, 3, "", True) == "1101"
assert write_zeta(5, 3, "", True) == "1110"
assert write_zeta(6, 3, "", True) == "1111"
assert write_zeta(7, 3, "", True) == "0100000"
assert write_zeta(8, 3, "", True) == "0100001"

assert write_zeta(0, 3, "", False) == "001"
assert write_zeta(1, 3, "", False) == "0011"
assert write_zeta(2, 3, "", False) == "1011"
assert write_zeta(3, 3, "", False) == "0101"
assert write_zeta(4, 3, "", False) == "1101"
assert write_zeta(5, 3, "", False) == "0111"
assert write_zeta(6, 3, "", False) == "1111"
assert write_zeta(7, 3, "", False) == "0000010"
assert write_zeta(8, 3, "", False) == "0000110"

# Little consistency check
for i in range(256):
    l = len_zeta(i, 3)  # NoQA: E741

    wbe = write_zeta(i, 3, "", True)
    rbe = read_zeta(wbe, 3, True)[0]

    assert i == rbe, "%s %s %s" % (i, rbe, wbe)
    assert len(wbe) == l

    wle = write_zeta(i, 3, "", False)
    rle = read_zeta(wle, 3, False)[0]

    assert i == rle, "%s %s %s" % (i, rle, wle)
    assert len(wle) == l


def gen_zeta(read_bits, write_max_val, len_max_val=None, k=3, merged_table=False):
    """Configuration of `gen_table` for zeta"""
    assert read_bits > 0
    len_max_val = len_max_val or write_max_val
    gen_table(
        read_bits,
        write_max_val,
        len_max_val,
        "zeta",
        lambda value: len_zeta(value, k),
        lambda bitstream, be: read_zeta(bitstream, k, be),
        lambda value, bitstream, be: write_zeta(value, k, bitstream, be),
        merged_table,
    )
    with open(os.path.join(ROOT, "zeta_tables.rs"), "a") as f:
        f.write("/// The K of the zeta codes for these tables\n")
        f.write("pub const K: usize = {};".format(k))

################################################################################

def generate_default_tables():
    # Generate the default tables
    gen_gamma(
        read_bits=9, # No use on Xeon/ARM, little useful on i7
        write_max_val=63,
        merged_table=False, # Irrelevant for speed, a bit smaller
    )
    gen_delta(
        read_bits=11, # No use on any architecture if 9-bit gamma tables are available, but just in case someone selects it
        write_max_val=1023, # Very useful, both tables (delta and gamma)
        merged_table=False,
    )
    gen_zeta(
        read_bits=12, # Necessary for all architectures
        write_max_val=1023, # Very useful   
        k=3,
        merged_table=False, # A bit better on ARM, very slightly worse on i7, same on Xeon
    )
    subprocess.check_call(
        "cargo fmt", shell=True,
    )

if __name__ == "__main__":
    generate_default_tables()

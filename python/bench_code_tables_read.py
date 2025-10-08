#!/usr/bin/env python3

#
# SPDX-FileCopyrightText: 2023 Tommaso Fontana
# SPDX-FileCopyrightText: 2023 Inria
# SPDX-FileCopyrightText: 2023 Sebastiano Vigna
#
# SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
#

"""Benchmarks codes with different number of bits for the codes tables
and writes results on standard output in CSV format.
"""

import os
import sys
import subprocess
from gen_code_tables import *

if not os.path.exists("benchmarks") or not os.path.exists("python"):
    sys.exit("You must run this script in the main project directory.")

if len(sys.argv) != 2 or sys.argv[1] not in {"u16", "u32", "u64"}:
    sys.exit("Usage: %s [u16 | u32 | u64]" % sys.argv[0])

read_word = sys.argv[1]
first_time = True

for bits in range(1, 17):
    print(
        "\nBenchmarking with read word = %s, table bits = %d\n" % (read_word, bits),
        file=sys.stderr,
    )
    for tables_num in [1, 2]:
        # Clean the target to force the recreation of the tables
        subprocess.check_call(
            "cargo clean",
            shell=True,
            cwd="benchmarks",
        )

        merged_table = tables_num == 1
        gen_gamma(
            read_bits=bits,
            write_max_val=255,  # unused
            merged_table=merged_table,
        )
        gen_delta(
            read_bits=bits,
            write_max_val=255,  # unused
            merged_table=merged_table,
        )
        gen_zeta(
            read_bits=bits,
            write_max_val=255,  # unused
            k=3,
            merged_table=merged_table,
        )
        # Kludge: this will leave the original tables intact,
        # but avoids failing the static assert for READ_LEN >= 2
        if bits > 1:
            gen_omega(
                read_bits=bits,
                write_max_val=255,  # unused
                merged_table=merged_table,
            )

        # Run the benchmark with native cpu optimizations
        stdout = subprocess.check_output(
            "cargo run --release --no-default-features --features univ,reads,%s"
            % read_word,
            shell=True,
            env={
                **os.environ,
            },
            cwd="benchmarks",
        ).decode()

        for i in range(4, 5):
            gamma_bits = 2 * i + 1
            gen_gamma(
                read_bits=gamma_bits,
                write_max_val=255,  # unused
                merged_table=merged_table,
            )

            # Run the benchmark with native cpu optimizations
            stdout += subprocess.check_output(
                "cargo run --release --no-default-features --features univ,reads,%s,delta_gamma"
                % read_word,
                shell=True,
                env={
                    **os.environ,
                },
                cwd="benchmarks",
            ).decode()

        # Dump the header only the first time
        if first_time:
            print("n_bits,tables_num," + stdout.split("\n")[0])
            first_time = False
        # Dump all lines and add the `n_bits` column
        for line in stdout.split("\n")[1:]:
            if bits == 1 and "omega" in line:
                # Omega tables require at least 2 bits
                continue
            if len(line.strip()) != 0:
                print("{},{},{}".format(bits, tables_num, line))

        sys.stdout.flush()

# Reset the tables to the original state
generate_default_tables()

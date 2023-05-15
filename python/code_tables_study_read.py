#!/usr/bin/python3

#
# SPDX-FileCopyrightText: 2023 Tommaso Fontana
# SPDX-FileCopyrightText: 2023 Inria
#
# SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
#

"""Benchmark the code with different number of bits for the codes tables and
create a `table.csv` file with all the results
"""
import os
import sys
import subprocess
from code_tables_generator import generate_default_tables

if not os.path.exists("benchmarks") or not os.path.exists("python"):
    sys.exit("You must run this script in the main project directory.")

for bits in range(1, 18):
    print("Table bits:", bits, file=sys.stderr)
    for tables_num in [1, 2]:
        # Clean the target to force the recreation of the tables
        subprocess.check_call(
            "cargo clean", shell=True,
            cwd="benchmarks",
        )
        # Generate tables with the desired number of bits 
        stdout = subprocess.check_output(
            "python python/code_tables_generator.py",
            shell=True,
            env={
                **os.environ,
                "UNARY_CODE_TABLE_BITS":str(bits),
                "GAMMA_CODE_TABLE_BITS":str(bits),
                "DELTA_CODE_TABLE_BITS":str(bits),
                "ZETA_CODE_TABLE_BITS":str(bits),
                "MERGED_TABLES":str(2 - tables_num),
            },
        ).decode()

        # Run the benchmark with native cpu optimizations
        stdout = subprocess.check_output(
            "cargo run --release --features \"reads\"",
            shell=True,
            env={
                **os.environ,
                "RUSTFLAGS":"-C target-cpu=native",
            },
            cwd="benchmarks",
        ).decode()

        # Dump the header only the first time
        if bits == 1 and tables_num == 1:
            print("n_bits,tables_num," + stdout.split("\n")[0])
        # Dump all lines and add the `n_bits` column
        for line in stdout.split("\n")[1:]:
            if len(line.strip()) != 0:
                print("{},{},{}".format(bits, tables_num, line))

        sys.stdout.flush()

# Reset the tables to the original state
generate_default_tables()
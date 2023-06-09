#!/usr/bin/env python3

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
from gen_code_tables import *

if not os.path.exists("benchmarks") or not os.path.exists("python"):
    sys.exit("You must run this script in the main project directory.")

first_time = True

for bits in range(1, 18):
    value_max = 2**bits - 1
    print("Table bits:", bits, file=sys.stderr)
    for tables_num in [1, 2]:
        # Clean the target to force the recreation of the tables
        subprocess.check_call(
            "cargo clean", shell=True,
            cwd="benchmarks",
        )
        
        merged_table = tables_num == 1
        gen_unary(
            read_bits=0, 
            write_max_val=min(value_max, 64),
            merged_table=merged_table,
        )
        gen_gamma(
            read_bits=11, 
            write_max_val=value_max,
            merged_table=merged_table,
        )
        gen_delta(
            read_bits=11, 
            write_max_val=value_max,
            merged_table=merged_table,
        )
        gen_zeta(
            read_bits=12, 
            write_max_val=value_max,
            k=3,
            merged_table=merged_table,
        )

        # Run the benchmark with native cpu optimizations
        stdout = subprocess.check_output(
            "cargo run --release",
            shell=True,
            env={
                **os.environ,
                "RUSTFLAGS":"-C target-cpu=native",
            },
            cwd="benchmarks",
        ).decode()

        # Dump the header only the first time
        if first_time:
            print("max,tables_num," + stdout.split("\n")[0])
            first_time = False
        # Dump all lines and add the `max` column
        for line in stdout.split("\n")[1:]:
            if len(line.strip()) != 0:
                print("{},{},{}".format(value_max + 1, tables_num, line))

        sys.stdout.flush()

# Reset the tables to the original state
generate_default_tables()

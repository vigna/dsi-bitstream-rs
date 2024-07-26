#!/usr/bin/env python3

#
# SPDX-FileCopyrightText: 2024 Tommaso Fontana
#
# SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
#

"""Benchmark the codes that don't have tables."""
import os
import sys
import subprocess
from python.gen_default_code_tables import *

if not os.path.exists("benchmarks") or not os.path.exists("python"):
    sys.exit("You must run this script in the main project directory.")

env = os.environ
env.setdefault("RUSTFLAGS", "-C target-cpu=native")

first_time = True

for word in ["u16", "u32", "u64"]:
    # Clean the target to force the recreation of the tables
    subprocess.check_call(
        "cargo clean", shell=True,
        cwd="benchmarks",
    )

    generate_default_tables()
    
    header = ""
    if first_time:
        header = ",print_header"
        first_time = False

    print("Benchmarking all codes reads", file=sys.stderr)
    stdout = subprocess.check_output(
        "cargo run --release --features " + word + ",all,reads" + header,
        shell=True,
        env=env,
        cwd="benchmarks",
    ).decode()
    print(stdout)
    
    print("Benchmarking all codes writes", file=sys.stderr)
    stdout = subprocess.check_output(
        "cargo run --release --features " + word + ",all",
        shell=True,
        env=env,
        cwd="benchmarks",
    ).decode()
    print(stdout)

    print("Benchmarking tabulated codes read without tables", file=sys.stderr)
    stdout = subprocess.check_output(
        "cargo run --release --features reads,no_tables," + word,
        shell=True,
        env=env,
        cwd="benchmarks",
    ).decode()
    print(stdout)
    stdout = subprocess.check_output(
        "cargo run --release --features no_tables," + word,
        shell=True,
        env=env,
        cwd="benchmarks",
    ).decode()
    print(stdout)
    
    for bits in range(1, 18):
        for table_type in ["merged", "separated", "packed_be", "packed_le"]:
            print("Benchmarking tabulated codes with tables of {} bits merged? {}".format(bits, table_type), file=sys.stderr)
            gen_gamma(
                read_bits=bits, 
                write_max_val=2**bits - 1, # unused
                table_type=table_type,
            )
            gen_delta(
                read_bits=bits, 
                write_max_val=2**bits - 1, # unused
                table_type=table_type,
            )
            gen_zeta(
                read_bits=bits, 
                write_max_val=2**bits - 1, # unused
                k=3,
                table_type=table_type,
            )
            
            # Bench tabled codes with their tables
            stdout = subprocess.check_output(
                "cargo run --release --features reads,tables," + word,
                shell=True,
                env=env,
                cwd="benchmarks",
            ).decode()
            print(stdout)
            stdout = subprocess.check_output(
                "cargo run --release --features tables," + word,
                shell=True,
                env=env,
                cwd="benchmarks",
            ).decode()
            print(stdout)

            for i in range(4, 5):
                print("testing gamma with %d bits" % i, file=sys.stderr)
                gamma_bits = 2*i + 1
                gen_gamma(
                    read_bits=gamma_bits, 
                    write_max_val=2**gamma_bits - 1, # unused
                    merged_table=merged_table,
                )

                # Run the benchmark with native cpu optimizations
                stdout += subprocess.check_output(
                    "cargo run --release --features reads,delta_gamma," + word,
                    shell=True,
                    env=env,
                    cwd="benchmarks",
                ).decode()
                print(stdout)

                # Write benchmarks
                stdout += subprocess.check_output(
                    "cargo run --release --features delta_gamma," + word,
                    shell=True,
                    env=env,
                    cwd="benchmarks",
                ).decode()
                print(stdout)


sys.stdout.flush()
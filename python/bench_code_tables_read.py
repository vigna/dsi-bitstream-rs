#!/usr/bin/env python3

#
# SPDX-FileCopyrightText: 2023 Tommaso Fontana
# SPDX-FileCopyrightText: 2023 Inria
# SPDX-FileCopyrightText: 2023 Sebastiano Vigna
#
# SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
#

"""Benchmarks codes with different number of bits for the read tables
and writes results on standard output in TSV format.

CHANGED: Now drives Criterion benchmarks instead of custom timing.
Each table size triggers a recompilation and a Criterion benchmark run.
Results are extracted from Criterion's JSON output.
"""

import os
import sys
import subprocess
from gen_code_tables import *
from extract_criterion import get_table_bench_results, parse_ratios_from_stderr

if not os.path.exists("benchmarks") or not os.path.exists("python"):
    sys.exit("You must run this script in the main project directory.")

if len(sys.argv) < 2 or len(sys.argv) > 3 or sys.argv[1] not in {"u16", "u32", "u64"}:
    sys.exit("Usage: %s [u16 | u32 | u64] [implied | univ]" % sys.argv[0])

read_word = sys.argv[1]
dist = sys.argv[2] if len(sys.argv) == 3 else "univ"

if dist not in {"implied", "univ"}:
    sys.exit("Distribution must be 'implied' or 'univ'")

# TSV header: t_bits is 0 for no table, >0 for table (= number of lookup bits)
print("code\tendian\tt_bits\ttype\top\tratio\tmean\tmin\tmax")

for bits in range(1, 17):
    print(
        "\nBenchmarking with read word = %s, table bits = %d\n" % (read_word, bits),
        file=sys.stderr,
    )
    for tables_num, type_name in [(1, "merged"), (2, "sep")]:
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
        gen_pi(
            read_bits=bits,
            write_max_val=255,  # unused
            k=2,
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

        # CHANGED: Run Criterion benchmarks for the "tables" group,
        # filtered to read operations only.
        features = "reads,%s" % read_word
        if dist == "univ":
            features = "univ," + features

        result = subprocess.run(
            "cargo bench --bench tables --no-default-features --features %s -- tables" % features,
            shell=True,
            cwd="benchmarks",
            capture_output=True,
            text=True,
        )

        if result.returncode != 0:
            print("cargo bench failed:", file=sys.stderr)
            print(result.stderr, file=sys.stderr)
            sys.exit(1)

        # Parse hit ratios from stderr
        ratios = parse_ratios_from_stderr(result.stderr)

        # Extract Criterion results
        bench_results = get_table_bench_results(
            os.path.join("benchmarks", "target", "criterion")
        )

        for r in bench_results:
            code = r["code"]
            endian = r["endian"]
            use_table = r["use_table"]
            if bits < 2 and "omega" in code:
                continue
            ratio = ratios.get((code, endian, use_table), 0.0)
            t_bits = bits if use_table else 0
            # Criterion measures the entire iteration (N operations),
            # so we divide by N to get per-operation nanoseconds.
            n = 1_000_000  # matches benchmarks::N
            print(
                "{}\t{}\t{}\t{}\t{}\t{:.5f}\t{:.6f}\t{:.6f}\t{:.6f}".format(
                    code,
                    endian,
                    t_bits,
                    type_name,
                    r["op"],
                    ratio,
                    r["mean_ns"] / n,
                    r["ci_lower"] / n,
                    r["ci_upper"] / n,
                )
            )

        # Now run delta_gamma variant
        for i in range(4, 5):
            gamma_bits = 2 * i + 1
            gen_gamma(
                read_bits=gamma_bits,
                write_max_val=255,  # unused
                merged_table=merged_table,
            )

            # Clean to force recompilation with new gamma tables
            subprocess.check_call(
                "cargo clean",
                shell=True,
                cwd="benchmarks",
            )

            features = "reads,%s,delta_gamma" % read_word
            if dist == "univ":
                features = "univ," + features

            result = subprocess.run(
                "cargo bench --bench tables --no-default-features --features %s -- tables"
                % features,
                shell=True,
                cwd="benchmarks",
                capture_output=True,
                text=True,
            )

            if result.returncode != 0:
                print("cargo bench (delta_gamma) failed:", file=sys.stderr)
                print(result.stderr, file=sys.stderr)
                sys.exit(1)

            ratios_dg = parse_ratios_from_stderr(result.stderr)
            bench_results_dg = get_table_bench_results(
                os.path.join("benchmarks", "target", "criterion")
            )

            for r in bench_results_dg:
                code = r["code"]
                endian = r["endian"]
                use_table = r["use_table"]
                ratio = ratios_dg.get((code, endian, use_table), 0.0)
                t_bits = bits if use_table else 0
                n = 1_000_000
                print(
                    "{}\t{}\t{}\t{}\t{}\t{:.5f}\t{:.6f}\t{:.6f}\t{:.6f}".format(
                        code,
                        endian,
                        t_bits,
                        type_name,
                        r["op"],
                        ratio,
                        r["mean_ns"] / n,
                        r["ci_lower"] / n,
                        r["ci_upper"] / n,
                    )
                )

        sys.stdout.flush()

# Reset the tables to the original state
generate_default_tables()

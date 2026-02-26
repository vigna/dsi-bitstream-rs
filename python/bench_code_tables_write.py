#!/usr/bin/env python3

#
# SPDX-FileCopyrightText: 2023 Tommaso Fontana
# SPDX-FileCopyrightText: 2023 Inria
# SPDX-FileCopyrightText: 2023 Sebastiano Vigna
#
# SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
#

"""Benchmarks codes with different number of bits for the write tables
and writes results on standard output in TSV format.

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

write_word = sys.argv[1]
dist = sys.argv[2] if len(sys.argv) == 3 else "univ"

if dist not in {"implied", "univ"}:
    sys.exit("Distribution must be 'implied' or 'univ'")

# TSV header: t_bits is 0 for no table, >0 for table (= log2 of table size)
print("code\tendian\tt_bits\ttype\top\tratio\tmean\tmin\tmax")

for bits in range(1, 17):
    value_max = 2**bits - 1
    print(
        "\nBenchmarking with write word = %s, table bits = %d\n" % (write_word, bits),
        file=sys.stderr,
    )
    for tables_num, type_name in [(1, "merged"), (2, "sep")]:
        merged_table = tables_num == 1
        gen_gamma(
            read_bits=11,  # unused
            write_max_val=value_max,
            merged_table=merged_table,
        )
        gen_delta(
            read_bits=11,  # unused
            write_max_val=value_max,
            merged_table=merged_table,
        )
        gen_zeta(
            read_bits=12,  # unused
            write_max_val=value_max,
            k=3,
            merged_table=merged_table,
        )
        gen_pi(
            read_bits=12,  # unused
            write_max_val=value_max,
            k=2,
            merged_table=merged_table,
        )
        # Kludge: this will leave the original tables intact,
        # but avoids failing the static assert for
        if value_max >= 62:
            gen_omega(
                read_bits=11,  # unused
                write_max_val=value_max,
                merged_table=merged_table,
            )

        features = write_word
        if dist == "univ":
            features = "univ," + features

        result = subprocess.run(
            "cargo bench --bench tables --no-default-features --features %s"
            % features,
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

        for r in sorted(bench_results, key=lambda r: (r["code"], r["endian"], r["use_table"], r["op"])):
            code = r["code"]
            endian = r["endian"]
            use_table = r["use_table"]
            if value_max < 62 and "omega" in code:
                continue
            ratio = ratios.get((code, endian, use_table), 0.0)
            t_bits = bits if use_table else 0
            # Criterion measures the entire iteration (N operations),
            # so we divide by N to get per-operation nanoseconds.
            n = 1_000_000  # matches benchmarks::N
            print(
                "{}\t{}\t{}\t{}\t{}\t{:.5f}\t{:7.4f}\t{:7.4f}\t{:7.4f}".format(
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

        # Now run delta_g variant (delta with gamma tables)
        for i in range(4, 5):
            gamma_bits = 2 * i + 1
            gamma_value_max = 2**gamma_bits - 1
            gen_gamma(
                read_bits=gamma_bits,  # unused
                write_max_val=gamma_value_max,
                merged_table=merged_table,
            )

            features = "delta_gamma,%s" % write_word
            if dist == "univ":
                features = "univ," + features

            result = subprocess.run(
                "cargo bench --bench tables --no-default-features --features %s"
                % features,
                shell=True,
                cwd="benchmarks",
                capture_output=True,
                text=True,
            )

            if result.returncode != 0:
                print("cargo bench (delta_g) failed:", file=sys.stderr)
                print(result.stderr, file=sys.stderr)
                sys.exit(1)

            ratios_dg = parse_ratios_from_stderr(result.stderr)
            bench_results_dg = get_table_bench_results(
                os.path.join("benchmarks", "target", "criterion")
            )

            for r in sorted(bench_results_dg, key=lambda r: (r["code"], r["endian"], r["use_table"], r["op"])):
                code = r["code"]
                endian = r["endian"]
                use_table = r["use_table"]
                ratio = ratios_dg.get((code, endian, use_table), 0.0)
                t_bits = bits if use_table else 0
                n = 1_000_000
                print(
                    "{}\t{}\t{}\t{}\t{}\t{:.5f}\t{:7.4f}\t{:7.4f}\t{:7.4f}".format(
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

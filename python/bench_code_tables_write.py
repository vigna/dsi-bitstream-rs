#!/usr/bin/env python3

#
# SPDX-FileCopyrightText: 2023 Tommaso Fontana
# SPDX-FileCopyrightText: 2023 Inria
# SPDX-FileCopyrightText: 2023 Sebastiano Vigna
#
# SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
#

"""Benchmarks codes with different number of bits for the write tables
and writes results in TSV format.

Each table size triggers a recompilation and a Criterion benchmark run.
Results are extracted from Criterion's JSON output.

No-table baselines are run once at the start, then table-only benchmarks
are run for each table size (1–16 bits) and table type (merged, sep).

Criterion output goes to stdout (visible); compiler output goes to stderr.
TSV is written to the output file (3rd argument, or stdout if omitted).
"""

import os
import shutil
import subprocess
import sys

from extract_criterion import get_table_bench_results, parse_ratios_from_stderr
from gen_code_tables import *

if not os.path.exists("benches") or not os.path.exists("python"):
    sys.exit("You must run this script in the main project directory.")

# Separate positional args from Criterion options (--warm-up-time, --measurement-time, etc.)
positional = []
criterion_opts = []
i = 1
while i < len(sys.argv):
    if sys.argv[i].startswith("--"):
        criterion_opts.append(sys.argv[i])
        if i + 1 < len(sys.argv) and not sys.argv[i + 1].startswith("--"):
            criterion_opts.append(sys.argv[i + 1])
            i += 1
    else:
        positional.append(sys.argv[i])
    i += 1

if (
    len(positional) < 1
    or len(positional) > 3
    or positional[0] not in {"u16", "u32", "u64"}
):
    sys.exit(
        "Usage: %s [u16|u32|u64] [implied|univ] [output.tsv] [--warm-up-time S] [--measurement-time S]"
        % sys.argv[0]
    )

write_word = positional[0]
dist = positional[1] if len(positional) >= 2 else "univ"
out_path = positional[2] if len(positional) >= 3 else None

if dist not in {"implied", "univ"}:
    sys.exit("Distribution must be 'implied' or 'univ'")

# Build Criterion CLI options (without leading --, combined with regex after single --)
criterion_opts_str = " ".join(criterion_opts)

# Open output: file or stdout
out = open(out_path, "w") if out_path else sys.stdout


def run_cargo_bench(cmd):
    """Run cargo bench, letting Criterion output go to stdout and forwarding
    compiler output to stderr.  Returns captured RATIO lines as a string."""
    process = subprocess.Popen(
        cmd,
        shell=True,
        stdout=None,  # Criterion output → script's stdout
        stderr=subprocess.PIPE,  # capture for RATIO parsing + forwarding
        text=True,
    )
    ratio_lines = []
    for line in process.stderr:
        if line.startswith("RATIO:"):
            ratio_lines.append(line)
        else:
            sys.stderr.write(line)
    process.wait()
    if process.returncode != 0:
        print("cargo bench failed (exit %d)" % process.returncode, file=sys.stderr)
        sys.exit(1)
    return "\n".join(ratio_lines)


criterion_base = os.path.join("target", "criterion")

# TSV header: t_bits is 0 for no table, >0 for table (= log2 of table size)
print("code\tendian\tt_bits\ttype\top\tratio\tcilower\tmean\tciupper", file=out)

# ── Step 1: No-table baselines (run once) ──────────────────────────────

print(
    "\n===== Benchmarking with write word = %s, table bits = 0\n" % write_word,
    file=sys.stderr,
)

features = "implied,bench-%s" % write_word
if dist == "univ":
    features += ",bench-univ"

# Remove stale results
no_table_dir = os.path.join(criterion_base, "no_table")
if os.path.isdir(no_table_dir):
    shutil.rmtree(no_table_dir)

ratio_text = run_cargo_bench(
    "cargo bench --bench tables --features %s -- %s '^no_table/'"
    % (features, criterion_opts_str)
)

bench_results = get_table_bench_results(criterion_base, group="no_table")

n = 1_000_000  # matches common::N
for r in sorted(bench_results, key=lambda r: (r["code"], r["endian"], r["op"])):
    print(
        "{}\t{}\t{}\t{}\t{}\t{:.4f}\t{:7.4f}\t{:7.4f}\t{:7.4f}".format(
            r["code"],
            r["endian"],
            0,
            "-",
            r["op"],
            0.0,
            r["cilower"] / n,
            r["mean_ns"] / n,
            r["ciupper"] / n,
        ),
        file=out,
    )

# Also run delta_g no-table baseline
features_dg = "implied,bench-delta-gamma,bench-%s" % write_word
if dist == "univ":
    features_dg += ",bench-univ"

if os.path.isdir(no_table_dir):
    shutil.rmtree(no_table_dir)

ratio_text_dg = run_cargo_bench(
    "cargo bench --bench tables --features %s -- %s '^no_table/'"
    % (features_dg, criterion_opts_str)
)

bench_results_dg = get_table_bench_results(criterion_base, group="no_table")
for r in sorted(bench_results_dg, key=lambda r: (r["code"], r["endian"], r["op"])):
    print(
        "{}\t{}\t{}\t{}\t{}\t{:.4f}\t{:7.4f}\t{:7.4f}\t{:7.4f}".format(
            r["code"],
            r["endian"],
            0,
            "-",
            r["op"],
            0.0,
            r["cilower"] / n,
            r["mean_ns"] / n,
            r["ciupper"] / n,
        ),
        file=out,
    )

out.flush()

# ── Step 2: Table sweep (bits 1–16, merged/sep) ────────────────────────

for bits in range(1, 17):
    value_max = 2**bits - 1
    print(
        "\n===== Benchmarking with write word = %s, table bits = %d\n"
        % (write_word, bits),
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
        # but avoids failing the static assert for WRITE_LEN >= 2
        if value_max >= 62:
            gen_omega(
                read_bits=11,  # unused
                write_max_val=value_max,
                merged_table=merged_table,
            )

        # Remove stale Criterion results to avoid picking up old entries
        table_dir = os.path.join(criterion_base, "table")
        if os.path.isdir(table_dir):
            shutil.rmtree(table_dir)

        features = "implied,bench-%s" % write_word
        if dist == "univ":
            features += ",bench-univ"

        ratio_text = run_cargo_bench(
            "cargo bench --bench tables --features %s -- %s '^table/'"
            % (features, criterion_opts_str)
        )

        # Parse hit ratios
        ratios = parse_ratios_from_stderr(ratio_text)

        # Extract Criterion results (table group only)
        bench_results = get_table_bench_results(criterion_base, group="table")

        for r in sorted(bench_results, key=lambda r: (r["code"], r["endian"], r["op"])):
            code = r["code"]
            endian = r["endian"]
            if value_max < 62 and "omega" in code:
                continue
            ratio = ratios.get((code, endian, True), 0.0)
            print(
                "{}\t{}\t{}\t{}\t{}\t{:.4f}\t{:7.4f}\t{:7.4f}\t{:7.4f}".format(
                    code,
                    endian,
                    bits,
                    type_name,
                    r["op"],
                    ratio,
                    r["cilower"] / n,
                    r["mean_ns"] / n,
                    r["ciupper"] / n,
                ),
                file=out,
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

            # Remove stale Criterion results before delta_g run
            if os.path.isdir(table_dir):
                shutil.rmtree(table_dir)

            features = "implied,bench-delta-gamma,bench-%s" % write_word
            if dist == "univ":
                features += ",bench-univ"

            ratio_text_dg = run_cargo_bench(
                "cargo bench --bench tables --features %s -- %s '^table/'"
                % (features, criterion_opts_str)
            )

            ratios_dg = parse_ratios_from_stderr(ratio_text_dg)
            bench_results_dg = get_table_bench_results(criterion_base, group="table")

            for r in sorted(
                bench_results_dg, key=lambda r: (r["code"], r["endian"], r["op"])
            ):
                code = r["code"]
                endian = r["endian"]
                ratio = ratios_dg.get((code, endian, True), 0.0)
                print(
                    "{}\t{}\t{}\t{}\t{}\t{:.4f}\t{:7.4f}\t{:7.4f}\t{:7.4f}".format(
                        code,
                        endian,
                        bits,
                        type_name,
                        r["op"],
                        ratio,
                        r["cilower"] / n,
                        r["mean_ns"] / n,
                        r["ciupper"] / n,
                    ),
                    file=out,
                )

        out.flush()

if out is not sys.stdout:
    out.close()

# Reset the tables to the original state
generate_default_tables()

#!/usr/bin/env python3

#
# SPDX-FileCopyrightText: 2025 Sebastiano Vigna
#
# SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
#

"""Extracts comparative benchmark results from Criterion's JSON output.

Reads the benchmarks/target/criterion/ directory and produces TSV output
compatible with the plot_comp.py script.

Usage:
    python3 extract_comp_results.py [--target-dir DIR]

CHANGED: Output uses mean + confidence interval instead of median + percentiles.
The columns are: code, rw, endianness, mean, ci_lower, ci_upper
"""

import argparse
import sys
from extract_criterion import get_comp_bench_results


def main():
    parser = argparse.ArgumentParser(
        description="Extract Criterion comparative results into TSV for plot_comp.py"
    )
    parser.add_argument(
        "--target-dir",
        default="benchmarks/target/criterion",
        help="Path to the Criterion output directory (default: benchmarks/target/criterion)",
    )
    args = parser.parse_args()

    results = get_comp_bench_results(args.target_dir)
    if not results:
        sys.exit(f"No comparative benchmark results found in: {args.target_dir}")

    # CHANGED: header uses "mean", "ci_lower", "ci_upper" instead of
    # "avg", "std", "25%", "median", "75%"
    print("code\trw\tendianness\tmean\tci_lower\tci_upper")

    # Divide by N to get per-operation nanoseconds
    n = 1_000_000  # matches benchmarks::N

    for r in sorted(results, key=lambda x: (x["code"], x["rw"], x["endianness"])):
        print(
            "{}\t{}\t{}\t{:.3f}\t{:.3f}\t{:.3f}".format(
                r["code"],
                r["rw"],
                r["endianness"],
                r["mean_ns"] / n,
                r["ci_lower"] / n,
                r["ci_upper"] / n,
            )
        )


if __name__ == "__main__":
    main()

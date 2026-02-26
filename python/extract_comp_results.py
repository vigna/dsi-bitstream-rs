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

Output columns: code, op, dist, endian, mean, min, max
(mean and confidence interval bounds, in ns/op).
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
        default="target/criterion",
        help="Path to the Criterion output directory (default: target/criterion)",
    )
    args = parser.parse_args()

    results = get_comp_bench_results(args.target_dir)
    if not results:
        sys.exit(f"No comparative benchmark results found in: {args.target_dir}")

    print("code\top\tdist\tendian\tmean\tmin\tmax")

    # Divide by N to get per-operation nanoseconds
    n = 1_000_000  # matches benchmarks::N

    for r in sorted(results, key=lambda x: (x["code"], x["op"], x["dist"], x["endian"])):
        print(
            "{}\t{}\t{}\t{}\t{:7.4f}\t{:7.4f}\t{:7.4f}".format(
                r["code"],
                r["op"],
                r["dist"],
                r["endian"],
                r["mean_ns"] / n,
                r["ci_lower"] / n,
                r["ci_upper"] / n,
            )
        )


if __name__ == "__main__":
    main()

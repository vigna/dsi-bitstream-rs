#!/usr/bin/env python3

#
# SPDX-FileCopyrightText: 2025 Sebastiano Vigna
#
# SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
#

"""Utility functions for extracting Criterion benchmark results.

Provides functions to parse Criterion's JSON output directory structure
and extract mean estimates with confidence intervals.

Criterion flattens benchmark IDs with "/" separators into directory names
with "_" separators. For example, "gamma::BE::Table/read_buff" becomes
the directory "gamma::BE::Table_read_buff".
"""

import json
import os
import sys


def get_criterion_results(target_dir="benchmarks/target/criterion"):
    """Parse all Criterion benchmark results from the target directory.

    Returns a dict mapping benchmark ID to a dict with keys:
        mean_ns: mean estimate in nanoseconds
        ci_lower: confidence interval lower bound in ns
        ci_upper: confidence interval upper bound in ns
    """
    results = {}
    if not os.path.isdir(target_dir):
        return results

    for entry in os.listdir(target_dir):
        entry_dir = os.path.join(target_dir, entry)
        if not os.path.isdir(entry_dir) or entry == "report":
            continue

        estimates_path = os.path.join(entry_dir, "new", "estimates.json")
        if not os.path.exists(estimates_path):
            continue

        with open(estimates_path) as f:
            estimates = json.load(f)

        mean = estimates["mean"]
        results[entry] = {
            "mean_ns": mean["point_estimate"],
            "ci_lower": mean["confidence_interval"]["lower_bound"],
            "ci_upper": mean["confidence_interval"]["upper_bound"],
        }

    return results


def get_table_bench_results(target_dir="benchmarks/target/criterion"):
    """Parse table-sweep benchmark results.

    Criterion flattens "gamma::BE::Table/read_buff" to directory name
    "gamma::BE::Table_read_buff". We split on the last "_" that matches
    a known operation type to recover the config and op, then further
    split the config on "::" into code, endian, and tables.

    Returns a list of dicts, each with keys:
        code: code name (e.g., "gamma")
        endian: "BE" or "LE"
        tables: "true" or "false"
        op: operation type (e.g., "read_buff", "write", "read_unbuff")
        mean_ns: mean estimate in nanoseconds (for the whole iteration)
        ci_lower: confidence interval lower bound
        ci_upper: confidence interval upper bound
    """
    results = []
    all_results = get_criterion_results(target_dir)
    op_types = ["read_buff", "read_unbuff", "write"]

    def _parse_config(config_str):
        """Split 'gamma::BE::Table' into (code, endian, use_table)."""
        parts = config_str.split("::")
        if len(parts) == 3:
            use_table = parts[2] == "Table"
            return parts[0], parts[1], use_table
        return None

    for bench_id, stats in all_results.items():
        # Try to split the flattened benchmark ID back into config and op
        config_str = None
        op = None
        for op_type in op_types:
            suffix = "_" + op_type
            if bench_id.endswith(suffix):
                config_str = bench_id[: -len(suffix)]
                op = op_type
                break

        if config_str is None:
            # Try splitting on "/" in case Criterion preserves it
            parts = bench_id.rsplit("/", 1)
            if len(parts) == 2:
                config_str, op = parts[0], parts[1]

        if config_str is None:
            continue

        parsed = _parse_config(config_str)
        if parsed is None:
            continue
        code, endian, use_table = parsed

        results.append(
            {
                "code": code,
                "endian": endian,
                "use_table": use_table,
                "op": op,
                "mean_ns": stats["mean_ns"],
                "ci_lower": stats["ci_lower"],
                "ci_upper": stats["ci_upper"],
            }
        )

    return results


def get_comp_bench_results(target_dir="benchmarks/target/criterion"):
    """Parse comparative benchmark results.

    Criterion flattens "gamma/BE/implied/read" to directory name
    "gamma_BE_implied_read". We parse this knowing the structure:
    {code}_{endianness}_{dist}_{op}.

    Returns a list of dicts with keys:
        code, op, dist, endian, mean_ns, ci_lower, ci_upper
    """
    results = []
    all_results = get_criterion_results(target_dir)

    for bench_id, stats in all_results.items():
        # Try "/" first (in case Criterion preserves it)
        parts = bench_id.split("/")
        if len(parts) == 4:
            code, endian, dist, op = parts
        else:
            # Flattened with "_": need to parse carefully
            # Known ops: "read", "write"
            # Known dists: "implied", "univ"
            # Known endianness: "BE", "LE"
            parts = bench_id.split("_")
            if len(parts) < 4:
                continue

            # The structure is {code}_{endian}_{dist}_{op}
            # Code can contain underscores (e.g., "delta_gamma")
            # but endian, dist, op are single tokens.
            op = parts[-1]
            dist = parts[-2]
            endian = parts[-3]
            code = "_".join(parts[:-3])

            if op not in ("read", "write"):
                continue
            if dist not in ("implied", "univ"):
                continue
            if endian not in ("BE", "LE"):
                continue

        results.append(
            {
                "code": code,
                "op": op,
                "dist": dist,
                "endian": endian,
                "mean_ns": stats["mean_ns"],
                "ci_lower": stats["ci_lower"],
                "ci_upper": stats["ci_upper"],
            }
        )

    return results


def parse_ratios_from_stderr(stderr_text):
    """Parse hit ratios from the RATIO: lines printed to stderr.

    Returns a dict mapping (code, endian, use_table) to ratio (float),
    where use_table is True or False.
    """
    ratios = {}
    for line in stderr_text.splitlines():
        if line.startswith("RATIO:"):
            parts = line[6:].split(",")
            if len(parts) == 2:
                key_parts = parts[0].split("::")
                if len(key_parts) == 3:
                    use_table = key_parts[2] == "Table"
                    ratios[(key_parts[0], key_parts[1], use_table)] = float(parts[1])
    return ratios

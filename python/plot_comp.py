#!/usr/bin/env python3

#
# SPDX-FileCopyrightText: 2025 Tommaso Fontana
#
# SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
#

"""
Plots comparative benchmark data.

CHANGED: Now reads TSV with mean + confidence interval columns instead of
median + percentile-based statistics.

Usage:
    python3 plot_comp.py <file.tsv>
    python3 extract_comp_results.py | python3 plot_comp.py /dev/stdin
"""

import argparse
import numpy as np
import matplotlib.pyplot as plt

parser = argparse.ArgumentParser()
parser.add_argument("file")
args = parser.parse_args()

with open(args.file) as f:
    data = f.read().splitlines()
header = [x.strip() for x in data[0].split("\t")]

data = [dict(zip(header, [x.strip() for x in line.split("\t")])) for line in data[1:]]


def create_plot(operations, title):
    # Get unique codes and their best performance
    codes = list(set(d["code"] for d in operations))

    # For each code, find the best (minimum) mean between BE and LE
    code_performance = {}
    for code in codes:
        code_data = [d for d in operations if d["code"] == code]
        best_mean = min(float(d["mean"]) for d in code_data)
        code_performance[code] = best_mean

    # Sort codes by best performance
    codes = sorted(codes, key=lambda x: code_performance[x])

    # Set up the plot
    fig, ax = plt.subplots(figsize=(15, 8))

    # Calculate positions for bars
    x = np.arange(len(codes))
    width = 0.35

    # Plot little endian data
    little_endian = [
        (d["mean"], d["min"], d["max"])
        for code in codes
        for d in operations
        if d["code"] == code and d["endian"] == "LE"
    ]
    means_le, min_le, max_le = zip(*little_endian)
    means_le = [float(x) for x in means_le]
    min_le = [float(x) for x in min_le]
    max_le = [float(x) for x in max_le]
    yerr_le = np.array(
        [
            np.array(means_le) - np.array(min_le),
            np.array(max_le) - np.array(means_le),
        ]
    )

    # Plot big endian data
    big_endian = [
        (d["mean"], d["min"], d["max"])
        for code in codes
        for d in operations
        if d["code"] == code and d["endian"] == "BE"
    ]
    means_be, min_be, max_be = zip(*big_endian)
    means_be = [float(x) for x in means_be]
    min_be = [float(x) for x in min_be]
    max_be = [float(x) for x in max_be]
    yerr_be = np.array(
        [
            np.array(means_be) - np.array(min_be),
            np.array(max_be) - np.array(means_be),
        ]
    )

    # Create the scatter plots with error bars
    ax.errorbar(
        x - width / 2,
        means_le,
        yerr=yerr_le,
        fmt=".",
        label="Little Endian",
        capsize=5,
        capthick=1,
        markersize=8,
    )
    ax.errorbar(
        x + width / 2,
        means_be,
        yerr=yerr_be,
        fmt=".",
        label="Big Endian",
        capsize=5,
        capthick=1,
        markersize=8,
    )

    # Add rotated text labels for mean values
    for i, (mean_le, mean_be) in enumerate(zip(means_le, means_be)):
        # Add label for little endian
        ax.text(
            i - width / 2,
            mean_le + (max_le[i] - min_le[i]) / 2 + 0.1,
            f"{mean_le:.3f}",
            rotation=90,
            ha="center",
            va="bottom",
        )
        # Add label for big endian
        ax.text(
            i + width / 2,
            mean_be + (max_be[i] - min_be[i]) / 2 + 0.1,
            f"{mean_be:.3f}",
            rotation=90,
            ha="center",
            va="bottom",
        )

    # Customize the plot
    ax.set_ylabel("Time (ns)")
    ax.set_title(f"{title}")
    ax.set_xticks(x)
    ax.set_xticklabels(codes, rotation=45, ha="right")
    ax.legend()

    # Add a light gray background grid
    ax.set_axisbelow(True)
    ax.yaxis.grid(True, color="gray", linestyle="--", alpha=0.2)

    y_min, y_max = ax.get_ylim()
    ax.set_ylim(y_min, y_max * 1.1)  # Make space for the written labels

    # Adjust layout to prevent label cutoff
    plt.tight_layout()

    return fig


for op_val, dist_val, title, filename in [
    (
        "read", "implied",
        "Read (u32 read word) on implied distribution",
        "read_implied_performance.svg",
    ),
    (
        "write", "implied",
        "Write (u64 write word) on implied distribution",
        "write_implied_performance.svg",
    ),
    (
        "read", "univ",
        "Read (u32 read word) on distribution ≈1/x (first billion integers)",
        "read_univ_performance.svg",
    ),
    (
        "write", "univ",
        "Write (u64 write word) on distribution ≈1/x (first billion integers)",
        "write_univ_performance.svg",
    ),
]:
    ops = [d for d in data if d["op"] == op_val and d["dist"] == dist_val]
    fig = create_plot(ops, title)
    fig.savefig(filename, dpi=300, bbox_inches="tight")


plt.close("all")

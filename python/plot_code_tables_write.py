#!/usr/bin/env python3

#
# SPDX-FileCopyrightText: 2023 Tommaso Fontana
# SPDX-FileCopyrightText: 2023 Inria
#
# SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
#

"""Plots data generated from `bench_code_tables_write.py`.

Reads TSV with mean + confidence interval columns.
"""

import sys
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt

# plt.rcParams['text.usetex'] = True

if len(sys.argv) < 2 or len(sys.argv) > 3 or sys.argv[1] not in {"u16", "u32", "u64"}:
    sys.exit("Usage: %s [u16 | u32 | u64] [implied | univ]" % sys.argv[0])

write_word = sys.argv[1]
dist = sys.argv[2] if len(sys.argv) == 3 else "univ"

if dist not in {"implied", "univ"}:
    sys.exit("Distribution must be 'implied' or 'univ'")

dist_label = "(implied distribution)" if dist == "implied" else "(distribution ≈1/x, first billion integers)"

nice = {
    "gamma": "γ",
    "delta": "δ (no γ tables)",
    "delta_g": "δ (γ tables)",
    "zeta3": "ζ₃",
    "pi2": "π₂",
    "omega": "ω",
}

df = pd.read_csv(sys.stdin, index_col=None, header=0, sep="\t")
x_label = "t_bits"

plots = []
for code_name in ["gamma", "delta", "delta_g", "zeta3", "pi2", "omega"]:
    fig, ax = plt.subplots(1, 1, figsize=(10, 8), dpi=200, facecolor="white")
    for table_type in ["merged", "sep"]:
        marker = "o" if table_type == "merged" else "s"

        for endian in ["LE", "BE"]:
            values = df[
                (df.code == code_name) & (df.endian == endian)
                & (df.t_bits > 0) & (df.type == table_type)
            ]
            m = min(values["mean"])
            i = np.argmin(values["mean"].values)
            ax.errorbar(
                values[x_label],
                values["mean"],
                label="{}::{} (min: {:.3f}ns @ {} {})".format(
                    endian,
                    table_type,
                    m,
                    i,
                    "bits",
                ),
                marker=marker,
            )
            ax.fill_between(
                values[x_label],
                values["min"],
                values["max"],
                alpha=0.3,
            )

    for endian in ["LE", "BE"]:
        values = (
            df[(df.code == code_name) & (df.endian == endian) & (df.t_bits == 0)]
            .groupby(x_label)
            .mean(numeric_only=True)
        )
        m = min(values["mean"])
        ax.errorbar(
            values.index,
            values["mean"],
            label="{}::no_table (min: {:.3f}ns)".format(endian, m),
            marker="^",
        )
        ax.fill_between(
            values.index,
            values["min"],
            values["max"],
            alpha=0.3,
        )

    ratios = (
        df[(df.code == code_name) & (df.t_bits > 0)]
        .groupby(x_label)
        .mean(numeric_only=True)
    )
    bars = ax.bar(
        ratios.index,
        ratios.ratio,
        label="table hit ratio",
        fc=(0, 0, 1, 0.3),
        linewidth=1,
        edgecolor="black",
    )
    for ratio, rect in zip(ratios.ratio, bars):
        ax.text(
            rect.get_x() + rect.get_width() / 2.0,
            1.2,
            "{:.2f}%".format(ratio * 100),
            ha="center",
            va="bottom",
            rotation=90,
        )

    left = min(ratios.index) - 1
    right = max(ratios.index) + 1

    ax.plot(
        [left - 1, right + 1],
        [1, 1],
        "--",
        alpha=0.3,
        color="gray",
        label="table hit ratio 100% line",
    )

    ax.legend(loc="center left", bbox_to_anchor=(1, 0.5))
    ax.set_ylim(bottom=0)  # ymin is your value
    ax.set_xlim([left, right])  # ymin is your value
    ax.set_xticks(ratios.index)
    ax.set_title(
        (
            "Performance of writes (%s) in %s code in function of the table size %s\n"
            "Shaded areas are 95%% confidence intervals and the plots "
            "are means"
        )
        % (write_word, nice[code_name], dist_label)
    )
    ax.set_xlabel("table bits")
    ax.set_ylabel("ns")
    plots.append((fig, ax, "%s_write_tables.svg" % code_name))

min_x, max_x = np.inf, -np.inf
min_y, max_y = np.inf, -np.inf

for fig, ax, name in plots:
    min_x = min(min_x, ax.get_xlim()[0])
    max_x = max(max_x, ax.get_xlim()[1])
    min_y = min(min_y, ax.get_ylim()[0])
    max_y = max(max_y, ax.get_ylim()[1])

for fig, ax, name in plots:
    ax.set_xlim([min_x, max_x])
    ax.set_ylim([min_y, max_y])
    fig.savefig(name, bbox_inches="tight")

#!/usr/bin/env python3

#
# SPDX-FileCopyrightText: 2023 Tommaso Fontana
# SPDX-FileCopyrightText: 2023 Inria
#
# SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
#

"""Plots data generated from `bench_code_tables_read.py`.

Reads TSV with mean + confidence interval columns.
"""

import os
import sys

import matplotlib
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd

# plt.rcParams['text.usetex'] = True

if len(sys.argv) < 2 or len(sys.argv) > 4 or sys.argv[1] not in {"u16", "u32", "u64"}:
    sys.exit("Usage: %s [u16 | u32 | u64] [implied | univ] [output_dir]" % sys.argv[0])

colors = matplotlib.cm.tab10.colors

read_word = sys.argv[1]
dist = sys.argv[2] if len(sys.argv) >= 3 else "univ"
outdir = sys.argv[3] if len(sys.argv) == 4 else "."

if dist not in {"implied", "univ"}:
    sys.exit("Distribution must be 'implied' or 'univ'")

dist_label = (
    "(implied distribution)"
    if dist == "implied"
    else "(universal Zipf distribution ≈1/x, first billion integers)"
)

nice = {
    "gamma": "γ",
    "delta": "δ (no γ tables)",
    "delta_g": "δ (γ tables)",
    "zeta3": "ζ₃",
    "pi2": "π₂",
    "omega": "ω",
}

# No-table colors: must be clearly distinct from each other and from tab10[0..3]
NO_TABLE_COLORS = {"LE": "black", "BE": "#cc00cc"}

df = pd.read_csv(sys.stdin, index_col=None, header=0, sep="\t")
df["ratio"] = pd.to_numeric(df["ratio"], errors="coerce")

plots = []

for code_name in ["gamma", "delta", "delta_g", "zeta3", "pi2", "omega"]:
    fig, ax = plt.subplots(1, 1, figsize=(10, 8), dpi=200, facecolor="white")
    handles = []
    labels = []

    for op_name, op_label, ls_table, ls_notab in [
        ("read_b", "read_buff", "solid", "solid"),
        ("read_ub", "read_unbuff", "dotted", "dotted"),
    ]:
        color = 0
        # Table data: merged (circle) then sep (square)
        for table_type in ["merged", "sep"]:
            marker = "o" if table_type == "merged" else "s"
            for endian in ["LE", "BE"]:
                values = df[
                    (df.code == code_name)
                    & (df.endian == endian)
                    & (df.t_bits > 0)
                    & (df.op == op_name)
                    & (df.type == table_type)
                ]
                if values.empty:
                    color += 1
                    continue
                m = min(values["mean"])
                i = values.t_bits.iloc[np.argmin(values["mean"].values)]
                h = ax.errorbar(
                    values.t_bits,
                    values["mean"],
                    marker=marker,
                    linestyle=ls_table,
                    color=colors[color],
                )
                handles.append(h)
                labels.append(
                    "{}::Table::{}::{} (min: {:.3f}ns @ {} bits)".format(
                        endian, table_type, op_label, m, i
                    )
                )
                color += 1
                ax.fill_between(
                    values.t_bits,
                    values["cilower"],
                    values["ciupper"],
                    alpha=0.3,
                )

        # No-table baselines as horizontal lines
        for endian in ["LE", "BE"]:
            no_table = df[
                (df.code == code_name)
                & (df.endian == endian)
                & (df.t_bits == 0)
                & (df.op == op_name)
            ]
            if not no_table.empty:
                m = no_table["mean"].mean()
                lo = no_table["cilower"].mean()
                hi = no_table["ciupper"].mean()
                c = NO_TABLE_COLORS[endian]
                h = ax.axhline(
                    y=m,
                    linestyle=ls_notab,
                    color=c,
                    linewidth=1.5,
                )
                handles.append(h)
                labels.append("{}::NoTable::{} ({:.3f}ns)".format(endian, op_label, m))
                ax.axhspan(lo, hi, alpha=0.08, color=c)

    ratios = (
        df[(df.code == code_name) & (df.t_bits > 0)]
        .groupby("t_bits")
        .mean(numeric_only=True)
    )
    if ratios.empty:
        plt.close(fig)
        continue
    bar_container = ax.bar(
        ratios.index,
        ratios.ratio,
        fc=(0, 0, 1, 0.3),
        linewidth=1,
        edgecolor="black",
    )
    handles.append(bar_container)
    labels.append("table hit ratio")
    for ratio, rect in zip(ratios.ratio, bar_container):
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

    (h,) = ax.plot(
        [left - 1, right + 1],
        [1, 1],
        "--",
        alpha=0.3,
        color="gray",
    )
    handles.append(h)
    labels.append("table hit ratio 100% line")

    ax.legend(handles, labels, loc="center left", bbox_to_anchor=(1, 0.5))
    ax.set_ylim(bottom=0)
    ax.set_xlim([left, right])
    ax.set_xticks(ratios.index)
    ax.set_title(
        (
            "Performance of reads (word = %s) in %s code as a function of the table size %s\n"
            "Shaded areas are 95%% confidence intervals and the plots "
            "are means"
        )
        % (read_word, nice[code_name], dist_label)
    )
    ax.set_xlabel("table bits")
    ax.set_ylabel("ns")
    plots.append((fig, ax, os.path.join(outdir, "%s_read_tables.svg" % code_name)))

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

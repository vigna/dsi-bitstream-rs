#!/usr/bin/env python3

#
# SPDX-FileCopyrightText: 2023 Tommaso Fontana
# SPDX-FileCopyrightText: 2023 Inria
#
# SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
#

"""Plots data generated from `bench_code_tables_read.py`"""

import sys
import numpy as np
import pandas as pd
import matplotlib
import matplotlib.pyplot as plt


# plt.rcParams['text.usetex'] = True

if len(sys.argv) < 2 or len(sys.argv) > 3 or sys.argv[1] not in {"u16", "u32", "u64"}:
    sys.exit("Usage: %s [u16 | u32 | u64] [implied | univ]" % sys.argv[0])

colors = matplotlib.cm.tab10.colors

read_word = sys.argv[1]
dist = sys.argv[2] if len(sys.argv) == 3 else "univ"

if dist not in {"implied", "univ"}:
    sys.exit("Distribution must be 'implied' or 'univ'")

dist_label = "(implied distribution)" if dist == "implied" else "(distribution ≈1/x, first billion integers)"

nice = {
    "gamma": "γ",
    "delta": "δ (no γ tables)",
    "delta_gamma": "δ (γ tables)",
    "zeta3": "ζ₃",
    "omega": "ω",
}

df = pd.read_csv(sys.stdin, index_col=None, header=0)

plots = []

for code in ["gamma", "delta", "delta_gamma", "zeta3", "omega"]:
    fig, ax = plt.subplots(1, 1, figsize=(10, 8), dpi=200, facecolor="white")
    for ty in ["read_buff", "read_unbuff"]:
        color = 0
        for tables_n in [1, 2]:
            if tables_n == 1:
                table_txt = "merged"
                marker = "o"
            else:
                table_txt = "sep"
                marker = "s"

            for pat in [
                "%s::LE::Table" % code,
                "%s::BE::Table" % code,
            ]:
                values = df[
                    (df.pat == pat) & (df.type == ty) & (df.tables_num == tables_n)
                ]
                m = min(values.ns_median)
                i = np.argmin(values.ns_median)
                ax.errorbar(
                    values.n_bits,
                    values.ns_median,  # values.ns_std,
                    label="{}::{}::{} (min: {:.3f}ns @ {} bits)".format(
                        "::".join(pat.split("::")[1:]), table_txt, ty, m, i
                    ),
                    marker=marker,
                    linestyle="dotted" if ty == "read_unbuff" else "solid",
                    color=colors[color],
                )
                color += 1
                ax.fill_between(
                    values.n_bits,
                    values.ns_perc25,
                    values.ns_perc75,
                    alpha=0.3,
                )

        for pat in [
            "%s::LE::NoTable" % code,
            "%s::BE::NoTable" % code,
        ]:
            values = (
                df[(df.pat == pat) & (df.type == ty)]
                .groupby("n_bits")
                .mean(numeric_only=True)
            )
            m = min(values.ns_median)
            ax.errorbar(
                values.index,
                values.ns_median,  
                #yerr=values.ns_std,
                label="{}::{} (min: {:.3f}ns)".format(
                    "::".join(pat.split("::")[1:]), ty, m
                ),
                marker="^",
                linestyle="dotted" if ty == "read_unbuff" else "solid",
                color=colors[color % 10],
            )
            color += 1
            ax.fill_between(
                values.index,
                values.ns_perc25,
                values.ns_perc75,
                alpha=0.3,
            )

    ratios = (
        df[df.pat.str.contains(code) & (df.tables_num == tables_n)]
        .groupby("n_bits")
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
        height = rect.get_height()
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
            "Performance of reads (buff: %s) in %s code as a function of the table size %s\n"
            "Shaded areas are the 25%% and 75%% percentiles and the plots "
            "are medians"
        )
        % (read_word, nice[code], dist_label)
    )
    ax.set_xlabel("table bits")
    ax.set_ylabel("ns")
    plots.append((fig, ax, "%s_read_tables.svg" % code))

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

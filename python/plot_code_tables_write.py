#!/usr/bin/env python3

#
# SPDX-FileCopyrightText: 2023 Tommaso Fontana
# SPDX-FileCopyrightText: 2023 Inria
#
# SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
#

"""Plots data generated from `bench_code_tables_write.py`"""

import sys
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt

# plt.rcParams['text.usetex'] = True

if len(sys.argv) != 2 or sys.argv[1] not in {"u16", "u32", "u64"}:
    sys.exit("Usage: %s [u16 | u32 | u64]" % sys.argv[0])

write_word = sys.argv[1]

nice = {
    "gamma": "γ",
    "delta": "δ (no γ tables)",
    "delta_gamma": "δ (γ tables)",
    "zeta3": "ζ₃",
    "omega": "ω",
}

df = pd.read_csv(sys.stdin, index_col=None, header=0)
x_label = "max_log2"
df[x_label] = np.log2(df["max"])

plots = []
for code in ["gamma", "delta", "delta_gamma", "zeta3", "omega"]:
    fig, ax = plt.subplots(1, 1, figsize=(10, 8), dpi=200, facecolor="white")
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
            if code == "unary":
                values = df[
                    (df.pat == pat) & (df.tables_num == tables_n) & (df["max"] <= 64)
                ]
            else:
                values = df[(df.pat == pat) & (df.tables_num == tables_n)]
            m = min(values.ns_median)
            i = np.argmin(values.ns_median)
            ax.errorbar(
                values[x_label],
                values.ns_median,  # values.ns_std,
                label="{}::{} (min: {:.3f}ns @ {} {})".format(
                    "::".join(pat.split("::")[1:]),
                    table_txt,
                    m,
                    i,
                    "bits",
                ),
                marker=marker,
            )
            ax.fill_between(
                values[x_label],
                values.ns_perc25,
                values.ns_perc75,
                alpha=0.3,
            )

    for pat in [
        "%s::LE::NoTable" % code,
        "%s::BE::NoTable" % code,
    ]:
        if code == "unary":
            values = (
                df[(df.pat == pat) & (df["max"] <= 64)]
                .groupby(x_label)
                .mean(numeric_only=True)
            )
        else:
            values = df[df.pat == pat].groupby(x_label).mean(numeric_only=True)
        m = min(values.ns_median)
        ax.errorbar(
            values.index,
            values.ns_median,  # values.ns_std,
            label="{} (min: {:.3f}ns)".format("::".join(pat.split("::")[1:]), m),
            marker="^",
        )
        ax.fill_between(
            values.index,
            values.ns_perc25,
            values.ns_perc75,
            alpha=0.3,
        )

    ratios = (
        df[df.pat.str.contains(code) & (df.tables_num == tables_n)]
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
            "Performance of writes (%s) in %s code in function of the table size\n"
            "Shaded areas are the 25%% and 75%% percentiles and the plots "
            "are medians"
        )
        % (write_word, nice[code])
    )
    ax.set_xlabel("log₂(table size)")
    ax.set_ylabel("ns")
    plots.append((fig, ax, "%s_write_tables.svg" % code))

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

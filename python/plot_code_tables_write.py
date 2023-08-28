#!/usr/bin/env python3

#
# SPDX-FileCopyrightText: 2023 Tommaso Fontana
# SPDX-FileCopyrightText: 2023 Inria
#
# SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
#

"""This script takes the data generated from `code_tables_study.py` and plots them"""

import sys
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt

df = pd.read_csv(sys.stdin, index_col=None, header=0)
x_label = "max_log2"
df[x_label] = np.log2(df["max"]) 

for code in ["unary", "gamma", "delta", "delta_gamma", "zeta3"]:
    plt.figure(figsize=(10, 8), dpi=200, facecolor="white")
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
                (df.pat == pat) & (df.tables_num == tables_n)
            ]
            m = min(values.ns_median)
            i = np.argmin(values.ns_median)
            plt.errorbar(
                values[x_label],
                values.ns_median,  # values.ns_std,
                label="{}::{} (min: {:.3f}ns {} {})".format(
                    "::".join(pat.split("::")[1:]), table_txt, m, i,
                    "bits" if x_label == "n_bits" else "max",
                ),
                marker=marker,
            )
            plt.fill_between(
                values[x_label],
                values.ns_perc25,
                values.ns_perc75,
                alpha=0.3,
            )

    for pat in [
        "%s::LE::NoTable" % code,
        "%s::BE::NoTable" % code,
    ]:
        values = df[df.pat == pat].groupby(x_label).mean(numeric_only=True)
        m = min(values.ns_median)
        plt.errorbar(
            values.index,
            values.ns_median,  # values.ns_std,
            label="{} (min: {:.3f}ns)".format(
                "::".join(pat.split("::")[1:]), m
            ),
            marker="^",
        )
        plt.fill_between(
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
    bars = plt.bar(
        ratios.index,
        ratios.ratio,
        label="table hit ratio",
        fc=(0, 0, 1, 0.3),
        linewidth=1,
        edgecolor="black",
    )
    for ratio, rect in zip(ratios.ratio, bars):
        height = rect.get_height()
        plt.text(
            rect.get_x() + rect.get_width() / 2.0,
            1.2,
            "{:.2f}%".format(ratio * 100),
            ha="center",
            va="bottom",
            rotation=90,
        )

    left = min(ratios.index) - 1
    right = max(ratios.index) + 1

    plt.plot(
        [left - 1, right + 1],
        [1, 1],
        "--",
        alpha=0.3,
        color="gray",
        label="table hit ratio 100% line",
    )

    plt.legend(loc="center left", bbox_to_anchor=(1, 0.5))
    plt.ylim(bottom=0)  # ymin is your value
    plt.xlim([left, right])  # ymin is your value
    plt.xticks(ratios.index)
    plt.title(
        (
            "Performances of %s codes writes in function of the table size\n"
            "Shaded areas are the 25%% and 75%% percentiles and the plots "
            "are medians with stds."
        )
        % (code.capitalize())
    )
    plt.xlabel("Table Bits")
    plt.ylabel("ns")
    plt.savefig("%s_write_tables.svg" % code, bbox_inches="tight")

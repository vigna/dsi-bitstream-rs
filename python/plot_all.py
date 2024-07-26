#!/usr/bin/env python3

#
# SPDX-FileCopyrightText: 2024 Tommaso Fontana
#
# SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
#

"""Benchmark the codes that don't have tables."""
import os
import sys
import argparse
from python.gen_default_code_tables import *
import matplotlib.pyplot as plt
import matplotlib.cm as cm

if not os.path.exists("benchmarks") or not os.path.exists("python"):
    sys.exit("You must run this script in the main project directory.")

parser = argparse.ArgumentParser()
parser.add_argument("type", choices=["read_buff", "read_unbuff", "write"])
args = parser.parse_args()

read_defaults = {}
write_defaults = {}

read_tables = {}
write_tables = {}

header = None
# group data so we can plot them in order
for i, line in enumerate(sys.stdin):
    if i == 0:
        header = line.split(",")
        continue
    if not line.strip():
        continue
    
    values = dict(zip(
        header,
        map(str.strip, line.split(",")),
    ))
    
    if 
    
    codes.setdefault(code, {}).setdefault(endianness, {}).setdefault(ty, {})[tuple(metadata)] = {
        "ns_avg": float(ns_avg),
        "ns_std": float(ns_std),
        "ns_perc25": float(ns_perc25),
        "ns_median": float(ns_median),
        "ns_perc75": float(ns_perc75),
    }

i = 0
colors = cm.get_cmap("tab20").colors
fig, ax = plt.subplots(1, 1, figsize=(10, 9), dpi=100, facecolor="white")    
for j, (code_name, val) in enumerate(sorted(codes.items())):
    start_code = i
    for endianness, val2 in sorted(val.items()):
        start_end = i
        for ty, vals in sorted(val2.items()):
            if ty != args.type:
                continue
            
            for metadata, val in sorted(vals.items()):
                ax.errorbar(
                    x=[i],
                    y=[val["ns_median"]],
                    yerr=[
                        [val["ns_median"] - val["ns_perc25"]],
                        [val["ns_perc75"] - val["ns_median"]],
                    ],
                    fmt='o', linewidth=2, capsize=6,
                    color=colors[j],
                )
                
                s = "{:.2f}".format(val["ns_median"])
                s += " " + " ".join(metadata)
                s = s.strip()
                
                ax.text(
                    x=i-0.5,
                    y=val["ns_perc75"] + 0.1,
                    s=s,
                    rotation=90,
                    fontsize=7,
                )
                i += 1
        ax.text(
            start_end + (i - start_end) / 2 - 1, -0.3, endianness, rotation=90
        )
        i += 1
        
    ax.text(
        start_code + (i - start_code) / 2 - 1.5, -1.5, code_name, rotation=90
    )
    i += 1
    
ax.set_title("Default codes speed for {} on their indended distribution".format(args.type))
ax.set_ylabel("ns")
ax.set_xticks([])
ax.set_ylim(bottom=0)
ax.grid(axis='y')
fig.tight_layout()
fig.savefig("default_codes_{}.svg".format(args.type))
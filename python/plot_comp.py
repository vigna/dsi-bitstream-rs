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

    # For each code, find the best (minimum) median between BE and LE
    code_performance = {}
    for code in codes:
        code_data = [d for d in operations if d["code"] == code]
        best_median = min(float(d["median"]) for d in code_data)
        code_performance[code] = best_median

    # Sort codes by best performance
    codes = sorted(codes, key=lambda x: code_performance[x])

    # Set up the plot
    fig, ax = plt.subplots(figsize=(15, 8))

    # Calculate positions for bars
    x = np.arange(len(codes))
    width = 0.35

    # Plot little endian data
    little_endian = [
        (d["median"], d["25%"], d["75%"])
        for code in codes
        for d in operations
        if d["code"] == code and d["endianness"] == "little"
    ]
    medians_le, q25_le, q75_le = zip(*little_endian)
    medians_le = [float(x) for x in medians_le]
    q25_le = [float(x) for x in q25_le]
    q75_le = [float(x) for x in q75_le]
    yerr_le = np.array(
        [
            np.array(medians_le) - np.array(q25_le),
            np.array(q75_le) - np.array(medians_le),
        ]
    )

    # Plot big endian data
    big_endian = [
        (d["median"], d["25%"], d["75%"])
        for code in codes
        for d in operations
        if d["code"] == code and d["endianness"] == "big"
    ]
    medians_be, q25_be, q75_be = zip(*big_endian)
    medians_be = [float(x) for x in medians_be]
    q25_be = [float(x) for x in q25_be]
    q75_be = [float(x) for x in q75_be]
    yerr_be = np.array(
        [
            np.array(medians_be) - np.array(q25_be),
            np.array(q75_be) - np.array(medians_be),
        ]
    )

    # Create the scatter plots with error bars
    ax.errorbar(
        x - width / 2,
        medians_le,
        yerr=yerr_le,
        fmt=".",
        label="Little Endian",
        capsize=5,
        capthick=1,
        markersize=8,
    )
    ax.errorbar(
        x + width / 2,
        medians_be,
        yerr=yerr_be,
        fmt=".",
        label="Big Endian",
        capsize=5,
        capthick=1,
        markersize=8,
    )

    # Add rotated text labels for median values
    for i, (median_le, median_be) in enumerate(zip(medians_le, medians_be)):
        # Add label for little endian
        ax.text(
            i - width / 2,
            median_le + (q75_le[i] - q25_le[i]) / 2 + 0.1,
            f"{median_le:.3f}",
            rotation=90,
            ha="center",
            va="bottom",
        )
        # Add label for big endian
        ax.text(
            i + width / 2,
            median_be + (q75_be[i] - q25_be[i]) / 2 + 0.1,
            f"{median_be:.3f}",
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
    ax.set_ylim(y_min, y_max * 1.1) # Make space for the written labels

    # Adjust layout to prevent label cutoff
    plt.tight_layout()

    return fig

for rw, title, filename in [
    ("read:implied", "Read (u32 read word) on implied distribution", "read_implied_performance.svg"),
    ("write:implied", "Write (u64 write word) on implied distribution", "write_implied_performance.svg"),
    ("read:univ", "Read (u32 read word) on 1/x distribution", "read_univ_performance.svg"),
    ("write:univ", "Write (u64 write word) on 1/x distribution", "write_univ_performance.svg"),
]:
    ops = [d for d in data if d["rw"].startswith(rw)]
    fig = create_plot(ops, title)
    fig.savefig(filename, dpi=300, bbox_inches="tight")


plt.close("all")

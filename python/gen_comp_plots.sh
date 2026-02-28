#!/bin/bash -e

# Run comparative benchmarks, extract results, and generate plots
# (to be run from project root)
# Usage: gen_comp_plots.sh [implied|univ|both] [-- Criterion options]
# Default: both
# Example: gen_comp_plots.sh implied -- --warm-up-time 0.01 --measurement-time 0.01

DIST="both"
CRITERION_OPTS=""
TARGET_DIR="${CARGO_TARGET_DIR:-target}/criterion"

# Parse arguments: optional dist, then optional -- criterion-opts
if [ $# -gt 0 ] && [ "$1" != "--" ]; then
	DIST="$1"
	shift
fi

if [ "$DIST" != "implied" ] && [ "$DIST" != "univ" ] && [ "$DIST" != "both" ]; then
	echo "Usage: $0 [implied|univ|both] [-- Criterion options]" 1>&2
	exit 1
fi

if [ "$1" = "--" ]; then
	shift
	CRITERION_OPTS="$*"
fi

if [ -n "$CRITERION_OPTS" ]; then
	DASH_OPTS="-- $CRITERION_OPTS"
else
	DASH_OPTS=""
fi

if [ "$DIST" = "both" ]; then
	DISTS="implied univ"
else
	DISTS="$DIST"
fi

for d in $DISTS; do
	mkdir -p "$d"

	# Remove stale Criterion results
	rm -rf "$TARGET_DIR/comparative"

	# Run benchmarks for this distribution only
	cargo bench --bench comparative --features implied,bench-reads -- "/$d/" $CRITERION_OPTS

	# Extract results and generate plots directly into the dist directory
	python3 ./python/extract_comp_results.py --target-dir "$TARGET_DIR" > "$d/comp.tsv"
	python3 ./python/plot_comp.py "$d/comp.tsv" --output-dir "$d" --write-word u64
done

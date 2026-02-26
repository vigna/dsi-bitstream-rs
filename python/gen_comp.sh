#!/bin/bash -e

# Run comparative benchmarks, extract results, and generate plots
# (to be run from project root)
# Usage: gen_comp.sh [implied|univ|both] [-- Criterion options]
# Default: both
# Example: gen_comp.sh implied -- --warm-up-time 0.01 --measurement-time 0.01

DIST=${1:-both}

if [ "$DIST" != "implied" ] && [ "$DIST" != "univ" ] && [ "$DIST" != "both" ]; then
	echo "Usage: $0 [implied|univ|both] [-- Criterion options]" 1>&2
	exit 1
fi

# Collect Criterion options (everything after --)
shift
CRITERION_OPTS=""
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
	rm -rf target/criterion/comparative

	# Run benchmarks for this distribution only
	cargo bench --bench comparative --features implied -- "/$d/" $CRITERION_OPTS

	# Extract results and generate plots directly into the dist directory
	python3 ./python/extract_comp_results.py > "$d/comp.tsv"
	python3 ./python/plot_comp.py "$d/comp.tsv" --output-dir "$d"
done

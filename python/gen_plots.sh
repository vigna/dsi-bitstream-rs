#!/bin/bash -e

# Run all benchmarks and generate plots (to be run from project root)
# Usage: gen_plots.sh [implied|univ|both] [-- Criterion options]
# Default: both
# Example: gen_plots.sh implied -- --warm-up-time 0.01 --measurement-time 0.01

DIST="both"
CRITERION_OPTS=""

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

if [ "$DIST" = "both" ]; then
	DISTS="implied univ"
else
	DISTS="$DIST"
fi

# Create all output directories up front so we don't fail mid-run
for d in $DISTS; do
	for u in u16 u32 u64; do
		mkdir -p "$d/$u"
	done
done

for d in $DISTS; do
	for u in u16 u32 u64; do
		# Run read benchmarks (TSV to file, Criterion output on stdout)
		python3 ./python/bench_code_tables_read.py $u $d "$d/$u/read.tsv" $CRITERION_OPTS
		# Generate plots (SVGs saved directly to final location)
		python3 ./python/plot_code_tables_read.py $u $d "$d/$u" < "$d/$u/read.tsv"

		# Run write benchmarks (TSV to file, Criterion output on stdout)
		python3 ./python/bench_code_tables_write.py $u $d "$d/$u/write.tsv" $CRITERION_OPTS
		# Generate plots (SVGs saved directly to final location)
		python3 ./python/plot_code_tables_write.py $u $d "$d/$u" < "$d/$u/write.tsv"
	done
done

echo "Please restore the source code for decoding tables in src/codes" 1>&2

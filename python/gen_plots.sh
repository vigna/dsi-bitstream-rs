#!/bin/bash -e

# Run all benchmarks and generate plots (to be run from project root)
# Usage: gen_plots.sh [implied|univ|both] [-- Criterion options]
# Default: both
# Example: gen_plots.sh implied -- --warm-up-time 0.01 --measurement-time 0.01

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
		# Run read benchmarks
		python3 ./python/bench_code_tables_read.py $u $d $CRITERION_OPTS > read.tsv
		# Generate plots
		cat read.tsv | python3 ./python/plot_code_tables_read.py $u $d

		# Run write benchmarks
		python3 ./python/bench_code_tables_write.py $u $d $CRITERION_OPTS > write.tsv
		# Generate plots
		cat write.tsv | python3 ./python/plot_code_tables_write.py $u $d

		# Move results to separate directory inside the distribution directory
		mv read.tsv write.tsv *_tables.svg "$d/$u"
	done
done

echo "Please restore the source code for decoding tables in ../src/codes" 1>&2

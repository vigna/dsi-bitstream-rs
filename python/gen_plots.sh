#!/bin/sh

# Run all benchmarks and generate plots (to be run from project root)
# Usage: gen_plots.sh [implied|univ]
# Default: implied

DIST=${1:-implied}

if [ "$DIST" != "implied" ] && [ "$DIST" != "univ" ]; then
	echo "Usage: $0 [implied|univ]" 1>&2
	exit 1
fi

for u in u16 u32 u64; do 
	# Run read benchmarks
	python3 ./python/bench_code_tables_read.py $u $DIST > read.csv
	# Generate plots
	cat read.csv | python3 ./python/plot_code_tables_read.py $u

	# Run write benchmarks
	python3 ./python/bench_code_tables_write.py $u $DIST > write.csv
	# Generate plots
	cat write.csv | python3 ./python/plot_code_tables_write.py $u

	# Move results to separate directory
	rm -fr $u
	mkdir $u
	mv *.csv *.svg $u
done

echo "Please restore the source code for decoding tables in ../src/codes" 1>&2

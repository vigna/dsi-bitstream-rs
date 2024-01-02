#!/bin/sh

# Run all benchmarks and generate plots (to be run from project root)

for u in u16 u32 u64; do 
	# Run read benchmarks
	python3 ./python/bench_code_tables_read.py $u > read.csv
	# Generate plots
	cat read.csv | python3 $u ./python/plot_code_tables_read.py

	# Run write benchmarks
	python3 ./python/bench_code_tables_write.py $u > write.csv
	# Generate plots
	cat write.csv | python3 $u ./python/plot_code_tables_write.py

	# Move results to separate directory
	rm -fr $u
	mkdir $u
	mv *.csv *.svg $u
done

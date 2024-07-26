#!/bin/sh

# Run all benchmarks and generate plots (to be run from project root)

for u in u16 u32 u64; do 
	# Run read benchmarks where tables do not change
	python3 ./python/bench_default_codes.py $u read > default_read.csv
	cat default_read.csv | python3 ./python/plot_default_codes.py read_buff
	cat default_read.csv | python3 ./python/plot_default_codes.py read_unbuff

	python3 ./python/bench_default_codes.py $u write > default_write.csv
	cat default_write.csv | python3 ./python/plot_default_codes.py write
	
	# Run read benchmarks
	python3 ./python/bench_code_tables_read.py $u | tee read.csv \
		| python3 ./python/plot_code_tables_read.py $u

	# Run write benchmarks
	python3 ./python/bench_code_tables_write.py $u | tee write.csv \
		| python3 ./python/plot_code_tables_write.py $u

	# Move results to separate directory
	rm -fr $u
	mkdir $u
	mv *.csv *.svg $u
done

echo "Please restore the source code for decoding tables in ../src/codes" 1>&2

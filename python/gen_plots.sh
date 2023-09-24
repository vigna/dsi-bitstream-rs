#!/bin/sh

# Run all benchmarks and generate plots (to be run from project root)

# Run the read benchmarks
python3 ./python/bench_code_tables_read.py > read.csv
# Make the plots
cat read.csv | python3 ./python/plot_code_tables_read.py
# Run the write benchmarks
python3 ./python/bench_code_tables_write.py > write.csv
# Make the plots
cat write.csv | python3 ./python/plot_code_tables_write.py

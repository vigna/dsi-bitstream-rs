#!/bin/bash -e

# Runs all benchmarks described in the README.
# Run as ./benchmarks/run_all.sh from root dir

./python/gen_plots.sh implied
./python/gen_plots.sh univ

pushd benchmarks
cargo bench --bench comparative
python3 ../python/extract_comp_results.py | tee comp.tsv
python3 ../python/plot_comp.py ./comp.tsv
popd

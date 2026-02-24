#!/bin/bash -e

# Runs all bemchmarks described in the README.
# Run as ./benchmarks/run_all.sh from root dir

./python/gen_plots.sh implied
./python/gen_plots.sh univ

pushd benchmarks
cargo run --bin comp --release | tee comp.tsv
python3 ../python/plot_comp.py ./comp.tsv
popd


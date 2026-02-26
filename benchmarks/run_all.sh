#!/bin/bash -e

# Runs all benchmarks described in the README.
# Usage: run_all.sh [-- Criterion options]
# Example: run_all.sh -- --warm-up-time 0.01 --measurement-time 0.01
# Run as ./benchmarks/run_all.sh from root dir

# Pass all arguments through to subscripts
./python/gen_plots.sh both "$@"
./python/gen_comp.sh both "$@"

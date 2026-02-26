#!/bin/bash -e

# Run comparative benchmarks, extract results, and generate plots
# (to be run from project root)
# Usage: gen_comp.sh [-- Criterion options]
# Example: gen_comp.sh -- --warm-up-time 0.01 --measurement-time 0.01

# Collect Criterion options (everything after --)
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

pushd benchmarks

# Remove stale Criterion results
rm -rf target/criterion/comparative

cargo bench --bench comparative $DASH_OPTS

python3 ../python/extract_comp_results.py | tee comp.tsv
python3 ../python/plot_comp.py ./comp.tsv

popd

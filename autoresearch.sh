#!/usr/bin/env bash
#
# Autoresearch harness: focused BufBitReader read benchmarks.
#
# Runs three Criterion legs and prints METRIC lines with the mean ns/op of
# each case plus their geometric mean as the primary metric:
#   1. `bufbitreader` (u32 read word): read_unary and read_bits anchors;
#   2. `bufbitreader_gamma` (u32): composite read_gamma through the public
#      code API, in a separate binary so the anchors keep their code layout;
#   3. `bufbitreader` with `bench-u64` (u64 read word, u128 bit buffer), in
#      its own CARGO_TARGET_DIR so the feature toggle does not thrash the
#      main build cache.
set -euo pipefail
cd "$(dirname "$0")"

echo "# $(uname -srm) | $(rustc -V) | target-cpu=native (.cargo/config.toml)" >&2

TARGET="${CARGO_TARGET_DIR:-target}"
U64_TARGET="$TARGET/u64"

# Remove stale estimates for these groups so a failed bench run cannot leave
# old results behind for the extraction step to pick up.
rm -rf "$TARGET/criterion/bufbitreader"
rm -rf "$TARGET/criterion/bufbitreader_gamma"
rm -rf "$U64_TARGET/criterion/bufbitreader_u64"

# Criterion output goes to stderr; only METRIC lines belong on stdout.
cargo bench --bench bufbitreader --features implied -- --noplot >&2
cargo bench --bench bufbitreader_gamma --features implied -- --noplot >&2
CARGO_TARGET_DIR="$U64_TARGET" cargo bench --bench bufbitreader \
    --features implied,bench-u64 -- --noplot >&2

python3 - <<'EOF'
import math
import os
import sys

sys.path.insert(0, "python")
from extract_criterion import get_criterion_results

results = get_criterion_results()
results.update(
    get_criterion_results(
        os.path.join(os.environ.get("CARGO_TARGET_DIR", "target"), "u64", "criterion")
    )
)
n = 1_000_000  # matches common::N

cases = {
    "bufbitreader_read_unary_BE": "read_unary_be_ns",
    "bufbitreader_read_unary_LE": "read_unary_le_ns",
    "bufbitreader_read_bits_BE": "read_bits_be_ns",
    "bufbitreader_read_bits_LE": "read_bits_le_ns",
    "bufbitreader_gamma_read_gamma_BE": "read_gamma_be_ns",
    "bufbitreader_gamma_read_gamma_LE": "read_gamma_le_ns",
    "bufbitreader_u64_read_unary_BE": "read_unary_u64_be_ns",
    "bufbitreader_u64_read_unary_LE": "read_unary_u64_le_ns",
    "bufbitreader_u64_read_bits_BE": "read_bits_u64_be_ns",
    "bufbitreader_u64_read_bits_LE": "read_bits_u64_le_ns",
}

vals = []
for bench_id, metric in cases.items():
    if bench_id not in results:
        sys.exit(f"missing criterion result: {bench_id}")
    ns_per_op = results[bench_id]["mean_ns"] / n
    vals.append(ns_per_op)
    print(f"METRIC {metric}={ns_per_op:.4f}")

geomean = math.exp(sum(map(math.log, vals)) / len(vals))
print(f"METRIC read_ns_geomean={geomean:.4f}")
EOF

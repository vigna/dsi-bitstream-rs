# Benchmarks for `dsi-bitstream`

This directory contains Criterion-based performance benchmarks for reading and
writing instantaneous codes. In particular, there are table benchmarks and a
global comparative performance test.

## Table Benchmarks

Tests each code with different table sizes. Each code with tables (ɣ, δ, ζ₃, π₂,
ω) is tested in all combinations of:

- Big endian / little endian
- Buffered / unbuffered (reads only)
- Table enabled / disabled

The distribution is controlled by the `bench-univ` feature: without it, each code's
implied distribution is used; with it, a universal distribution ≈1/x on the
first billion integers is used.

Word size is controlled by features: `bench-u16`, `bench-u32`, or `bench-u64`.
The feature `bench-reads` tests reads; without it, writes are tested. The
feature `bench-delta-gamma` tests δ codes with 9-bit tables for ɣ codes.

### Running Table-Sweep Benchmarks

A comprehensive set of tests across all table sizes can be obtained with:

```bash
./python/gen_table_plots.sh [implied|univ|both] [-- Criterion options]
```

The default distribution is `both`, which runs tests for both distributions.
This iterates over word sizes (`u16`, `u32`, `u64`) and table sizes (2¹ to 2¹⁶),
running Criterion benchmarks for each configuration and generating SVG plots.

Results are saved directly into `DIST/WORD/` directories (e.g.,
`implied/u32/read.tsv`). The script overrides previous results, so be careful to
move them if you want to keep them.

For more fine-grained control, run the scripts individually:

```bash
# Read benchmarks with u32 word, implied distribution
python3 ./python/bench_code_tables_read.py u32 implied > implied/u32/read.tsv
python3 ./python/plot_code_tables_read.py u32 implied implied/u32 < implied/u32/read.tsv

# Write benchmarks with u64 word, universal
python3 ./python/bench_code_tables_write.py u64 univ > univ/u64/write.tsv
python3 ./python/plot_code_tables_write.py u64 univ univ/u64 < univ/u64/write.tsv
```

## Comparative Benchmarks

Compares several codes (including variants with and without tables) side by side
using both implied and universal distributions.

```bash
# Run comparative benchmarks and generate plots
./python/gen_comp_plots.sh [implied|univ|both] [-- Criterion options]
```

Results are saved directly into `DIST/` directories (e.g., `implied/comp.tsv`).
The script overrides previous results, so be careful to move them if you want to
keep them.

For more fine-grained control:

```bash
# Run only gamma and delta reads, big-endian, implied distribution
cargo bench --bench comparative --features implied -- 'gamma|delta.*/BE/implied/read'

# Extract results and generate plots
python3 ./python/extract_comp_results.py > comp.tsv
python3 ./python/plot_comp.py comp.tsv --output-dir .
```

## Criterion Options

Criterion timing can be controlled via CLI options passed after `--`:

```bash
# Quick dry run
cargo bench --bench tables --features implied,bench-reads,bench-u32 -- --warm-up-time 0.01 --measurement-time 0.01

# Fine-grained table benchmarks
    ./python/gen_table_plots.sh implied -- --warm-up-time 0.5 --measurement-time 1
```

## Filtering with Criterion Regex

Criterion's built-in regex filter selects benchmarks by ID. Benchmark IDs
have the form `comparative/{code}/{endian}/{dist}/{op}`:

```bash
# Only gamma benchmarks
cargo bench --bench comparative --features implied -- 'gamma'

# Only big-endian reads with implied distribution
cargo bench --bench comparative --features implied -- '/BE/implied/read'

# Gamma and delta writes only
cargo bench --bench comparative --features implied -- '(gamma|delta).*/write'
```

## Output Formats

### Table-sweep TSV (reads and writes)

```
code	endian	t_bits	type	op	ratio	mean	min	max
```

`t_bits` is 0 when tables are not used, otherwise the number of lookup bits.

### Comparative TSV

```
code	op	dist	endian	mean	min	max
```

## Build Options

The cargo options in `Cargo.toml` and the `rustc` options in
`.cargo/config.toml` select aggressive optimizations and `--target-cpu=native`.
You can modify them to run the benchmarks with different options.

## Reference Results

The `svg` directory in the project root contains reference results from
different architectures (ARM, Xeon, i7).

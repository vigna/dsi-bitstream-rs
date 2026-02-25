# Benchmarks for `dsi-bitstream`

This crate provides Criterion-based performance benchmarks for reading and
writing instantaneous codes (gamma, delta, zeta3, pi2, omega, unary).

## Quick Start

```bash
# Run all benchmarks (table-sweep + comparative)
cargo bench

# Run only table-sweep benchmarks
cargo bench --bench tables

# Run only comparative benchmarks
cargo bench --bench comparative
```

## Table Benchmarks

Tests each code with different table sizes. Each code with tables (gamma, delta,
zeta3, pi2, omega) is tested in all combinations of:

- Big endian / little endian
- Buffered / unbuffered (reads only)
- Table enabled / disabled

The distribution is controlled by the `univ` feature: without it, each code's
implied distribution is used; with it, a universal distribution ~1/x on the
first billion integers is used.

Word size is controlled by features: `u16`, `u32`, or `u64` (default: `u32`).
The feature `reads` (default) tests reads; without it, writes are tested. The
feature `delta_gamma` tests delta codes with gamma tables.

### Running Table-Sweep Benchmarks

A comprehensive set of tests across all table sizes can be obtained with:

```bash
./python/gen_plots.sh [implied|univ|both]
```

The default is `both`, which runs tests for both distributions. This iterates
over word sizes (`u16`, `u32`, `u64`) and table sizes (2¹ to 2¹⁶), running
Criterion benchmarks for each configuration and generating SVG plots.

For more fine-grained control, run the scripts individually:

```bash
# Read benchmarks with u32 word, implied distribution
python3 ./python/bench_code_tables_read.py u32 implied > read.tsv
cat read.tsv | python3 ./python/plot_code_tables_read.py u32 implied

# Write benchmarks with u64 word, universal distribution
python3 ./python/bench_code_tables_write.py u64 univ > write.tsv
cat write.tsv | python3 ./python/plot_code_tables_write.py u64 univ
```

## Comparative Benchmarks

Compares all codes side by side using both implied and universal distributions.

```bash
# Run comparative benchmarks
cargo bench --bench comparative

# Extract results and generate plots
cd benchmarks
python3 ../python/extract_comp_results.py | tee comp.tsv
python3 ../python/plot_comp.py comp.tsv
```

## Environment Variables for Filtering

The comparative benchmarks support environment variables for filtering:

- `BENCH_CODES=gamma,delta` — which codes to benchmark (default: all)
- `BENCH_ENDIAN=BE` — which endianness (default: both BE and LE)
- `BENCH_DIST=implied` — which distribution (default: both implied and univ)
- `BENCH_OPS=read,write` — which operations (default: all)

Example:

```bash
# Only benchmark gamma and delta, big endian, reads
BENCH_CODES=gamma,delta BENCH_ENDIAN=BE BENCH_OPS=read cargo bench --bench comparative
```

Criterion's built-in `--bench` regex filter also works for ad-hoc selection:

```bash
# Only gamma benchmarks
cargo bench -- "gamma"
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

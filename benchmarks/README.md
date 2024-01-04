# Benchmarks for `dsi-bitstream`

This crate provides performance benchmarks for several variants of 
`BitRead` and `BitWrite` implementations. The benchmarks measure the
speed of reading and writing instantaneous codes, and in particular
unary, γ, δ and ζ₃.

Each code is tested in all possible combinations of the following parameters:
- Big endian / little endian
- Buffered/unbuffered
- Word: `u16`, `u32`, `u64`
- Table size: 2⁰, 2¹, 2², ..., 2¹⁸, or no table.

Abscissas show table size and ordinates the timing in nanoseconds, so in 
the no-table case you will see a straight horizontal line.

You can run benchmarks and generate SVG plots with
```shell
./python/gen_plots.sh
```
which starts a few Python scripts (you can run selectively the scripts
for a more fine-grained control).

The cargo options con `Cargo.html` and the `rustc` options in `.cargo/config.toml` 
select aggressive optimizations and `--target-cpu=native`.

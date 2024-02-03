# Benchmarks for `dsi-bitstream`

This crate provides performance benchmarks for several variants of
`BitRead` and `BitWrite` implementations. The benchmarks measure the speed
of reading and writing instantaneous codes, and in particular γ, δ
and ζ₃. The `svg` directory contains reference results of these benchmarks of a
few architectures.

Each code is tested in all possible combinations of the following parameters:

- Big endian / little endian
- Buffered/unbuffered
- Word: `u16`, `u32`, `u64`
- Table size: 2⁰, 2¹, 2², . . . , 2¹⁷, or no table.

Abscissas show table size and ordinates the timing in nanoseconds, so in
the no-table case, you will see a straight horizontal line.

Conditional compilation of this crate requires setting a feature for the word size
(`u16`, `u32`, or `u64`) and the feature `reads` to test reads
instead of writes. Table sizes have to be set by modifying the sources of the
`dsi-bitstream` crate in the directory above. A special feature `delta_gamma`
generates data just for the case of δ codes that use tables for the initial
γ code, without the preamble with column names.

You can run benchmarks and generate SVG plots for all the combinations above by

```shell
./python/gen_plots.sh
```

which starts a few Python scripts (you can run selectively the scripts
for a more fine-grained control). Note that the script will modify
the sources of the `dsi-bitstream` crate in the directory above, and
you will have to restore them manually.

The cargo options in `Cargo.html` and the `rustc` options in `.cargo/config.toml`
select aggressive optimizations and `--target-cpu=native`. You can modify
them to run the tests with different options.

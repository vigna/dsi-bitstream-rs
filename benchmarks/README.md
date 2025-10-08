# Benchmarks for `dsi-bitstream`

This crate provides performance benchmarks for several variants of `BitRead` and
`BitWrite` implementations. The benchmarks measure the speed of reading and
writing instantaneous codes, and in particular γ, δ ζ₃, and ω. The `svg`
directory contains reference results of these benchmarks of a few architectures.

Each code is tested in all possible combinations of the following bitstream
parameters:

- Big endian / little endian
- Buffered / unbuffered

Every complete prefix-free code, such as γ, δ, ζ₃, ω, Rice, Golomb, etc., has an
implied distribution where each symbol is assigned a probability proportional to
the reciprocal of 2 raised to the length of the codeword. The benchmarks
are run using the implied distribution of each code, unless you set the
`univ` feature, in which case a Zipf distribution of exponent one is used.

By conditional compilation you can change the word size used to access the
stream (`u16`, `u32`, or `u64`); moreover, the feature `reads` to test reads
instead of writes. Table sizes have to be set by modifying the sources of the
`dsi-bitstream` crate in the directory above. A special feature `delta_gamma`
generates data just for the case of δ codes that use tables for the initial γ
code, without the preamble with column names.

A more comprehensive set of tests, with associated graphs, can be obtained with

```shell
./python/gen_plots.sh
```

which starts a few Python scripts (you can run selectively the scripts
for a more fine-grained control). Note that the script will modify
the sources of the `dsi-bitstream` crate in the directory above, and
you will have to restore them manually.

The script will go through the following combinations (with the `delta_gamma`
feature or not):

- Word: `u16`, `u32`, `u64`
- Table size: 2⁰, 2¹, 2², . . . , 2¹⁶, or no table.

In the generated SVG plots, abscissas show table size and ordinates the timing
in nanoseconds, so in the no-table case, you will see a straight horizontal
line.

The cargo options in `Cargo.html` and the `rustc` options in `.cargo/config.toml`
select aggressive optimizations and `--target-cpu=native`. You can modify
them to run the tests with different options.

# Benchmarks on implied/universal distributions

To generate comparative graphs that illustrates the speed of reading and
writing for each code based on their respective implied distributions, and,
for universal code, on a Zipf distribution of exponent one, run
the following commands:

```bash
cd benchmarks
cargo run --bin comp --release | tee comp.tsv
python3 ../python/plot_comp.py ./comp.tsv
```

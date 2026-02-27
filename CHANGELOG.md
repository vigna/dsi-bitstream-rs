# Change Log

## [0.8.0] - 2026-10-26

### New

- A new method `Codes::canonicalize`
  returns equivalent, canonical, more efficient implementations of a code
  (e.g., π₀ = ɣ).

### Improved

- Replaced `check_tables` with per-code `check_read_table` const functions
  added to each parametric method. Checks are now performed at compile time via
  `const { }` blocks, only for the tables actually used, using a new constant
  `BitRead::PEEK_BITS`. Unless you exposed bit readers in your APIs, you can
  upgrade to this version as if it was a minor release.

- We are no longer dependent on `anyhow`.

- We are no longer dependent on `common_traits`, which has been replaced by
  `num-primitive` and `num-traits`. The trait `DoubleType` now replaces
  `common_traits::DoubleType`.

- `rand` is optional and only necessary for the `implied` module, which
  is now gated by the `implied` feature.

- `BufBitReader::into_inner` does not return a `Result` anymore, as it cannot
  fail.

- The output of all benchmarks is a file of aligned TAB-separated values, with a
  header line.

- Benchmark-running scripts can pass options to Criterion.

### Changed

- The strict and non-strict version of `MemWordWriter` have been
  exchanged, in the sense that the default now is the strict version,
  which is now as fast as the 0-extended version, likely because
  of improvements in codegen.

- Upgraded to `rand` 0.10.0, `rand_distr` 0.6.0, and `mem_dbg` 0.4.0.

- Removed spurious const parameter from `write_pi_param` and
  `write_zeta_param`.

- Read tables for pi codes are not set up by default.

- `from_path` functions in `buf_bit_reader` and `buf_bit_writer` now
  return `std::io::Result` instead of `anyhow::Result`.

- Dispatch `new()` methods (`FuncCodeReader`, `FuncCodeWriter`,
  `FuncCodeLen`, `FactoryFuncCodeReader`) and `Codes::to_code_const`
  / `Codes::from_code_const` now return `Result<_, DispatchError>`
  instead of `anyhow::Result`.

- Removed manual `PartialEq` implementation for `Codes`. It was equating codes
  with the same canonical form, which was confusing.

- We use `core::error::Error` everywhere.

- Parametric-codes traits and functions are no longer exported at the top
  level.

- All benchmarks are now in the `benches` directory, and they are all based on
  Criterion.

### Fixed

- `[Count|Dbg]Bit[Reader|Writer]` were missing recent code implementations.

- Fixed bug in `Write` implementations for `BufBitWriter`, which would
  work only with word `u64`.

- `get_implied_distribution` now also uses the last data point.

- Write benchmarks were using a vector rather than a slice, which was causing
  significant resizing overhead.

## [0.7.0] - 2026-01-27

### New

- `CodesStats` now supports serde via a default feature.

- Tables for pi codes (k=2).

## [0.6.0] - 2025-12-07

### Changed

- Enum variants defining codes are now all tuple types, making
  it possible to write more readable expressions such as `Zeta(2)`.

### Fixed

- Now we compile again correctly without std.

## [0.5.3] - 2025-12-06

### New

- Partials read (gamma prefix) for delta codes.

- Convenience functions `from_file` and `from_path` in `buf_bit_reader` and
  `buf_bit_writer` modules.

## [0.5.2] - 2025-10-10

### Changed

- Regenerated comparative performance graphs.

- We moved to the 2024 edition.

## [0.5.1] - 2025-10-10

### New

- New `MinimalBinary` struct providing static and dynamic dispatching of minimal
  binary codes.

- All codes now declare their range of validity (parameters and inputs), which
  is checked in the tests.

- Elias Omega has tables supporting partial (de)coding.

- New benchmarks using a distribution ≈1/x on the first billion integers.

- Revised table sizes using new benchmarks.

### Changed

- All benchmarks now use either the implied distribution or a distribution ≈1/x
  on the first billion integers. Previously there were a few handcrafted
  distributions.

## [0.5.0] - 2025-03-17

### New

- Added VByteBe, VByteLe, Elias Omega, and streamlined Pi codes.

- Added `dispatch` module to choose / dispatch codes at runtime.

- Implemented `std::io::Read` for `BitReader` and `BufBitReader`.

- Implemented `std::io::Write` for `BufBitWriter`.

- Added `update_many` to `CodesStats` and added more codes.

- Added `core::ops::Add`, `core::ops::AddAssign`, and `core::iter::Sum` to
  CodesStats so they can be merged using iter's `.sum()`.

- New benchmarks on implied distributions.

- `ToInt`/`ToNat` traits for reading and writing integers.

## [0.4.2] - 2024-04-07

### Changed

- Made mem_dbg optional.

## [0.4.1] - 2024-04-07

### Changed

- Added MemSize and MemDbg to most structs.

## [0.4.0] - 2024-03-18

### Changed

- `Peekable` -> `Peek` to follow Rust naming guidelines.

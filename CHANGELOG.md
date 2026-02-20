# Change Log

## [0.8.0]

### Improved

- Replaced `check_tables` with per-code `check_read_table` const functions
  added to each parametric method. Checks are now performed at compile
  time via `const { }` blocks only for the tables actually used using
  a new constant `BitRead::PEEK_BITS`. Unless you exposed bit readers
  in your APIs, you can upgrade to this version as if it was a minor release.

## [0.7.0] - 2026-10-27

### New

- `CodesStats` now support serde via a default feature.

- Tables for pi codes (k=2).

## [0.6.0] - 2025-12-7

### Changed

- Enum variants defining codes are now all tuple types, making
  it possible to write more readable expressions such as `Zeta(2)`.

### Fixed

- Now we compile again correctly without std.

## [0.5.3] - 2025-12-6

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

- Added `update_many` to `CodeStats` and added more codes.

- Added `core::ops::Add`, `core::ops::AddAssign`, and `core::iter::Sum` to
  CodeStats so they can be merged using iter's `.sum()`.

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

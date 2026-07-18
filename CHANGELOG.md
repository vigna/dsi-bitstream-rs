# Change Log

## [Unreleased]

### New

- `WordRead` has a new method with a default implementation:
  `read_word_opt`, atomically reading and consuming the next word if the
  backend can determine cheaply that one is available (`None` otherwise,
  never failing and never consuming past the end). `MemWordReader`
  implements it.

### Improved

- `BufBitReader::read_bits` and `BufBitReader::read_unary` are significantly
  faster: word-crossing reads of at most one word are composed with
  straight-line code instead of the general word loop, and, on peekable
  backends, refills load two words when possible, halving refill events and
  branch mispredictions. The bit buffer still never fills completely, so the
  hot paths carry no full-buffer handling. On Zen 4 with a u32 read word,
  decoder read benchmarks (74 cases: gamma, delta, zeta, pi, rice,
  exp-Golomb, Golomb, omega, unary, vbyte; both endians, implied and
  universal distributions) measure a 17% geometric-mean speedup, up to -33%
  on exp-Golomb and pi reads; u64-read-word unary decoding is ~13% faster.

### Fixed

- The big-endian `BufBitReader::copy_to` no longer leaves already-copied bits
  in the bit buffer; they could corrupt reads performed after the copy.

- `BufBitReader::copy_to` and `BufBitWriter::copy_from` now transfer at most
  64 bits per `read_bits`/`write_bits` call; previously they could exceed the
  trait limits when buffers held more than 64 bits, silently corrupting the
  copy.

- `BufBitReader::peek_bits` previously advertised `PEEK_BITS` = one word + 1
  bits but could only guarantee one word after a single refill, returning an
  incorrect result when peeking more than one word from an almost-empty
  buffer. `PEEK_BITS` is now exactly one word (half the bit buffer), which is
  sufficient for all decoding tables; a single refill always suffices, the bit
  buffer is never completely full, and the read/skip/unary hot paths are
  unchanged from before (no extra full-buffer handling).

- `BufBitWriter::into_inner` now returns the flush error instead of
  panicking in the drop-time flush.

- `BufBitWriter` flushes no longer modify the bit buffer before writing to
  the backend, so a failed flush can be retried without corrupting the
  stream; the slow paths of `write_bits` and `write_unary` similarly keep
  the buffer state consistent on backend errors.

- `CountBitWriter::flush` no longer double-counts the bits remaining in the
  buffer of the underlying writer; `CountBitReader::skip_bits` no longer
  updates the count when the underlying skip fails.

- Parsing `Zeta(0)` or `Golomb(0)` with `Codes::from_str` now returns an
  error instead of producing a code that panics with a division by zero
  when used.

- The length of the unary code computed by `CodeLen`/`FuncCodeLen` is no
  longer silently truncated on 32-bit platforms.

- The `std::io::Read` implementations of `BufBitReader` and `BitReader` now
  return the number of bytes read before an error, and preserve the
  underlying error as source instead of discarding it.

- `WordAdapter::set_word_pos` detects overflow when converting a word
  position to a byte position; `set_word_pos` on memory-based word readers
  and writers now reports the requested (offending) position instead of the
  current one.

## [0.9.2] - 2026-05-11

### Fixed

- `mem_size_flat` has been replaced with `mem_size(flat)`.

## [0.9.1] - 2026-03-17

### Fixed

- Table indexing would fail at write time on 32-bit platform because of an early
  `as usize` cast.

## [0.9.0] - 2026-03-07

### Changed

- Replaced `AsPrimitive` from `num-traits` with `PrimitiveNumberAs` from
  `num-primitive` for more precise semantics.

- Removed dependency on `num-traits`, as we were using only the constants
  `ZERO` and `ONE`.

## [0.8.0] - 2026-03-05

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

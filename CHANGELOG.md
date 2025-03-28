# Change Log

## [0.5.0] - 2025-03-17

### New

* Added VByteBe, VByteLe, Elias Omega, and streamlined Pi codes.

* Added `dispatch` module to choose / dispatch codes at runtime.

* Implemented `std::io::Read` for `BitReader` and `BufBitReader`.

* Implemented `std::io::Write` for `BufBitWriter`.

* Added `update_many` to `CodeStats` and added more codes.

* Added `core::ops::Add`, `core::ops::AddAssign`, and `core::iter::Sum` to
  CodeStats so they can be merged using iter's `.sum()`.

* New benchmarks on implied distributions.

* `ToInt`/`ToNat` traits for reading and writing integers.

## [0.4.2] - 2024-04-07

### Changed

* made mem_dbg optional.

## [0.4.1] - 2024-04-07

### Changed

* Added MemSize and MemDbg to most structs.

## [0.4.0] - 2024-03-18

### Changed

* `Peekable` -> `Peek` to follow Rust naming guidelines.

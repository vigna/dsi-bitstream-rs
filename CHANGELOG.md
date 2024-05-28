# Change Log

## [0.4.3] - 2024-04-07

### Changed

* Added `update_many` to `CodeStats`.
* Added `core::ops::Add`, `core::ops::AddAssign`, and `core::iter::Sum` to CodeStats.
* Implemented `std::io::Read` for `BitReader` and `BufBitReader`.
* Implemented `std::io::Write` for `BufBitWriter`.

### Notes
We tried to implement a blanket implementation of `std::io::Read` for any type
implementing `BitRead` and `std::io::Write` for any type implementing `BitWrite`
but rust complains that at least one type in the implementation has to be local.

## [0.4.2] - 2024-04-07

### Changed

* made mem_dbg optional.


## [0.4.1] - 2024-04-07

### Changed

* Added MemSize and MemDbg to most structs.


## [0.4.0] - 2024-03-18

### Changed

* `Peekable` -> `Peek` to follow Rust naming guidelines.

# Change Log

## [0.4.3] - 2024-04-07

### New

* Added VByte, Elias Omega, Pi, and PiWeb Codes.
* Added enum `Code` which supports read, write, and len which can be used to
  dynamically choose / dispatch codes.
* Added collection traits `ReadCodes` and `WriteCodes` to read and write all
  the codes supported by this library
* Implemented `std::io::Read` for `BitReader` and `BufBitReader`.
* Implemented `std::io::Write` for `BufBitWriter`.
* Added `update_many` to `CodeStats` and added more codes.
* Added `core::ops::Add`, `core::ops::AddAssign`, and `core::iter::Sum` to 
  CodeStats so they can be merged using iter's `.sum()`.

### Changed

* Now Rice and ExpGolomb are not implemented through a blanket because it would
  make it impossible to inspect and forward the call, like it's done in the Count
  and the Debug decorators.
* Fixed blanket impl of non param codes for BufBitWriter where it had a generic
  `WriteParam` instead of the concrete `DefaultWriteParams`

### Removed

* Removed `DbgBitReader` and `DbgBitWriter` as `CountBitReader` and 
  `CountBitWriter` also allow to print all values. This way we don't have to
  mantain two implementations of the same thing.

## [0.4.2] - 2024-04-07

### Changed

* made mem_dbg optional.


## [0.4.1] - 2024-04-07

### Changed

* Added MemSize and MemDbg to most structs.


## [0.4.0] - 2024-03-18

### Changed

* `Peekable` -> `Peek` to follow Rust naming guidelines.

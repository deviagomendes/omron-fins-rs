# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.0] - 2026-03-27

### Added

- **Linux Binary Support**: Pre-built native binaries for Linux on x86_64 (glibc) and aarch64 (glibc)
  - Binary name: `omron-fins-v0.6.0-{arch}-linux-gnu.{node|so}`
  - Supported architectures: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`
  - Full N-API bindings with async/await support

- **Complete Linux Import Guide** (`LINUX_GUIDE.md`): Comprehensive documentation covering:
  - Direct binary download and usage
  - npm package installation (`@omron-fins/native`)
  - FFI loading with `node:ffi-napi` or `@putout/ffi`
  - Example code for Node.js and Bun
  - Platform-specific considerations (glibc vs musl)
  - Error handling patterns
  - Distribution methods (npm bin field, direct binary)

- **TypeScript Definitions Update**: `index.d.ts` now exported for direct FFI usage

### Changed

- Updated documentation with Linux-specific build instructions
- Enhanced error messages in js_bindings for invalid parameters
- Improved type conversions for PlcValue serialization

### Fixed

- Fixed unclosed delimiter error in `format_hex` function
- Fixed move error in `parse_data_type_input` for struct operations
- Removed non-existent `read_status` and `increment` methods from js_bindings

### Dependencies

- Updated `@napi-rs/cli` to `^2`
- Updated N-API dependencies to latest stable versions

## [0.5.0] - 2026-03-27

### Added

- Initial npm package release (`@omron-fins/native`)
- Windows (.node) and macOS (.node) pre-built binaries
- Complete Node.js/Bun bindings with async/await API
- FinsClient class with full FINS protocol support

### Features

- Memory areas: CIO, WR, HR, DM, AR
- Word and bit read/write operations
- Fill and transfer operations
- Run/Stop PLC control
- Forced set/reset operations
- Multiple read in single request
- Typed helpers: f32, f64, i32, strings
- Struct read/write with automatic word swapping
- Utility functions: bit manipulation, formatting

### Documentation

- README.md with comprehensive API reference
- ARCHITECTURE.md with design rules
- Examples: simple_read.rs, simple_write.rs, simple_setup.rs, simple_struct_ops.rs

## [0.4.0] - 2026-03-26

### Added

- Rust crate release on crates.io
- High-level Client API
- Complete FINS protocol implementation
- UDP transport layer
- Comprehensive error handling

### Features

- Memory area support with bit/word access
- PlcMode control (Debug, Monitor, Run)
- ForcedBit and ForceSpec for maintenance operations
- MultiReadSpec for optimized batch reads
- DataType and PlcValue for struct operations
- Criterion benchmarks for performance testing

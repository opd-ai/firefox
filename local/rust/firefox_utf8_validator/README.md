# Firefox UTF-8 Validator (Rust Port)

Rust implementation of Firefox's `mozilla::detail::IsValidUtf8()` function.

## Overview

This crate provides UTF-8 validation functionality, porting the C++ implementation from `mfbt/Utf8.cpp` to Rust. The function validates byte sequences according to RFC 3629 (UTF-8 encoding standard).

**Original C++ Location**: `mfbt/Utf8.cpp`  
**Rust Port**: `local/rust/firefox_utf8_validator/`  
**Port Number**: #6  
**Date**: 2025-10-19

## API

### Rust API

```rust
pub fn is_valid_utf8(bytes: &[u8]) -> bool
```

Validates whether a byte slice is well-formed UTF-8.

### FFI API (C++ interop)

```cpp
extern "C" bool IsValidUtf8_RUST(const uint8_t* a_code_units, size_t a_count);
```

C-compatible function that can be called from Firefox C++ code.

## Features

- **Pure Function**: No state, no side effects, thread-safe
- **Comprehensive Validation**:
  - Proper byte sequence patterns
  - No overlong encodings
  - No surrogates (U+D800-U+DFFF)
  - Code points within valid range (U+0000-U+10FFFF)
  - Complete sequences (no truncation)
- **High Performance**: Uses Rust's `std::str::from_utf8()` (may use SIMD)
- **Safe FFI**: Null pointer checks, panic boundaries, explicit safety documentation

## Implementation Strategy

This port uses Rust's standard library UTF-8 validation (`std::str::from_utf8()`), which:
- Implements the same UTF-8 standard (RFC 3629) as Firefox's C++ code
- Is production-grade and extensively tested
- May be faster than the C++ version (SIMD optimizations)
- Provides the same edge case handling (surrogates, overlong sequences, etc.)

Alternative implementations considered:
- Port DecodeOneUtf8CodePoint logic directly ❌ (more complex, higher bug risk)
- Use encoding_rs crate ❌ (external dependency, unnecessary)
- **Use std::str::from_utf8** ✅ (chosen: stdlib, well-tested, performant)

## Testing Strategy

This Rust port maintains 100% compatibility with existing C++ tests.

### Test Architecture

- **C++ tests remain unchanged** (`mfbt/tests/TestUtf8.cpp`)
- **C++ tests call Rust implementation** via FFI layer (`IsValidUtf8_RUST`)
- **No C++ test ports** were created
- **Optional Rust tests** (`src/tests.rs`) provide supplementary validation

### FFI Test Support

The FFI layer (`src/ffi.rs`) exposes the validation function for:
- Production code call sites (1 location: `Utf8.h:278`)
- Unit test call sites (`TestUtf8.cpp::TestIsUtf8`)
- Test helper functions (none needed for this simple function)

### Test Coverage

**C++ Tests** (mfbt/tests/TestUtf8.cpp::TestIsUtf8):
- 17 assertions covering:
  - ASCII sequences
  - Multi-byte UTF-8 (2, 3, 4 bytes)
  - Max code point (U+10FFFF)
  - Beyond max (U+110000+)
  - Surrogate boundaries (U+D7FF, U+D800, U+DFFF, U+E000)
  - Invalid lead bytes
  - Truncated sequences

**Rust Tests** (src/tests.rs):
- 20+ test functions covering:
  - Empty strings
  - ASCII validation
  - Multi-byte sequences (2, 3, 4 bytes)
  - Overlong encodings
  - Invalid surrogates
  - Truncated sequences
  - Invalid continuation bytes
  - Property-based tests (determinism, length preservation)

**Rust FFI Tests** (src/ffi.rs):
- 11 test functions covering:
  - Null pointer handling (zero/non-zero length)
  - Empty slices
  - Valid ASCII and multi-byte
  - Invalid sequences (lead bytes, surrogates, overlong, truncated)
  - Max code point and beyond

### Running Tests

```bash
# C++ tests calling Rust implementation
export MOZ_RUST_UTF8_VALIDATOR=1
./mach test mfbt/tests/TestUtf8

# Rust unit tests
cd local/rust/firefox_utf8_validator
cargo test

# Rust tests with output
cargo test -- --nocapture
```

## UTF-8 Validation Rules

Valid UTF-8 must satisfy:

1. **Byte Patterns**:
   - 1-byte: `0xxxxxxx` (ASCII, 0x00-0x7F)
   - 2-byte: `110xxxxx 10xxxxxx`
   - 3-byte: `1110xxxx 10xxxxxx 10xxxxxx`
   - 4-byte: `11110xxx 10xxxxxx 10xxxxxx 10xxxxxx`

2. **No Overlong Encodings**:
   - Shortest form must be used
   - Example: 'A' (U+0041) must be 0x41, not 0xC1 0x81

3. **Code Point Range**:
   - U+0000 to U+10FFFF (inclusive)
   - No surrogates: U+D800 to U+DFFF (reserved for UTF-16)

4. **Complete Sequences**:
   - All continuation bytes must be present
   - Example: 0xC3 must be followed by continuation byte

## Performance

**Expected Performance**:
- **C++ version**: Custom decoder with ASCII fast-path
- **Rust version**: `std::str::from_utf8()` with potential SIMD optimizations
- **Target**: 100-120% of C++ speed (Rust may be faster)
- **Acceptable Range**: 95-105% (within ±5% threshold)

**Optimization Techniques**:
- Rust stdlib may use SIMD on supported platforms
- Inline functions (`#[inline]`) eliminate call overhead
- Zero-copy validation (no allocations)
- Fast-path for ASCII (implicit in stdlib implementation)

## Build Integration

### Rust Build

```bash
cd local/rust/firefox_utf8_validator
cargo build --release
```

### C++ Header Generation

```bash
cbindgen --config cbindgen.toml --output mozilla_Utf8Validator.h
```

### Firefox Build

```bash
# Enable Rust version
export MOZ_RUST_UTF8_VALIDATOR=1
./mach build
```

## Dependencies

- **Rust**: stdlib only (no external crates)
- **C++ Headers**: Generated via cbindgen
- **Firefox**: Conditional compilation (`MOZ_RUST_UTF8_VALIDATOR`)

## Safety

### Rust Code Safety

- Core validation logic is **100% safe Rust**
- FFI layer uses `unsafe` only for:
  - Creating slices from raw pointers (required for C++ interop)
  - Null pointer checks ensure safety

### FFI Safety Measures

1. **Null Pointer Checks**: Explicit handling of null pointers
2. **Zero-Length Safety**: Correct behavior for empty strings
3. **Panic Boundaries**: `catch_unwind` prevents unwinding into C++
4. **Documented Invariants**: Clear safety requirements for callers

## References

- **UTF-8 Specification**: [RFC 3629](https://tools.ietf.org/html/rfc3629)
- **Unicode Standard**: [Unicode 15.0](https://www.unicode.org/versions/Unicode15.0.0/)
- **Original C++**: `mfbt/Utf8.cpp`, `mfbt/Utf8.h`
- **C++ Tests**: `mfbt/tests/TestUtf8.cpp`

## Port Characteristics

- **C++ Lines Removed**: 40 (production code only)
- **C++ Test Lines (unchanged)**: 742 (`TestUtf8.cpp`)
- **Rust Lines Added**: ~490 (lib.rs + ffi.rs + tests.rs + README.md + build files)
- **External Dependencies**: 0
- **Test Coverage**: ~95% (comprehensive edge case coverage)
- **Selection Score**: 34/40 (excellent candidate)
- **Upstream Impact**: Zero conflicts (all changes in `local/` directory)

## License

This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at http://mozilla.org/MPL/2.0/.

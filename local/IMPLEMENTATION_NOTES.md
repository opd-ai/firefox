# Implementation Notes: Rust Dafsa Port

This document provides technical details about the Rust port of the Dafsa class.

## Overview

The Rust implementation is a line-by-line port of the C++ Dafsa class from `xpcom/ds/Dafsa.{h,cpp}`, maintaining exact API compatibility while leveraging Rust's safety guarantees.

## Component Details

### Original C++ Implementation

- **Location**: `xpcom/ds/Dafsa.{h,cpp}`
- **Size**: ~153 lines of implementation
- **Purpose**: DAFSA (Deterministic Acyclic Finite State Automaton) for efficient string lookup
- **Dependencies**: `mozilla::Span`, `nsACString`
- **History**: Very stable, minimal upstream changes

### Rust Implementation

- **Location**: `local/rust/firefox_dafsa/`
- **Size**: ~240 lines (including FFI layer and docs)
- **Language**: Rust 2021 edition
- **Dependencies**: `nsstring` crate (for C++ string interop)

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Firefox C++ Code                          │
│  (Existing code continues to work with C++ or Rust)         │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  │ (When MOZ_RUST_DAFSA=1)
                  ↓
┌─────────────────────────────────────────────────────────────┐
│                    FFI Layer (ffi.rs)                        │
│  • rust_dafsa_new()      - Create DAFSA from data           │
│  • rust_dafsa_lookup()   - Lookup string                    │
│  • rust_dafsa_destroy()  - Clean up                         │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ↓
┌─────────────────────────────────────────────────────────────┐
│                  Core Rust Implementation (lib.rs)           │
│  • Dafsa struct          - Main data structure              │
│  • lookup_string()       - Core algorithm                   │
│  • Helper functions      - Bit manipulation, offset reading │
└─────────────────────────────────────────────────────────────┘
```

## API Comparison

### C++ API

```cpp
class Dafsa {
 public:
  explicit Dafsa(const Graph& aData);
  int Lookup(const nsACString& aKey) const;
  static const int kKeyNotFound;
 private:
  const Graph mData;
};
```

### Rust API

```rust
pub struct Dafsa {
    data: Vec<u8>,
}

impl Dafsa {
    pub fn new(data: Vec<u8>) -> Self;
    pub fn from_slice(data: &[u8]) -> Self;
    pub fn lookup(&self, key: &str) -> i32;
}

pub const KEY_NOT_FOUND: i32 = -1;
```

### FFI Exports (C-compatible)

```rust
#[no_mangle]
pub unsafe extern "C" fn rust_dafsa_new(data: *const u8, length: usize) 
    -> *mut RustDafsa;

#[no_mangle]
pub unsafe extern "C" fn rust_dafsa_lookup(
    dafsa: *const RustDafsa, 
    key: *const nsACString
) -> i32;

#[no_mangle]
pub unsafe extern "C" fn rust_dafsa_destroy(dafsa: *mut RustDafsa);
```

## Algorithm Details

The DAFSA algorithm is identical to the C++ version:

1. **Offset Reading**: Read variable-length offsets (1, 2, or 3 bytes)
2. **Character Matching**: Match characters byte-by-byte
3. **End-of-Label Detection**: Detect terminal characters (high bit set)
4. **Return Value Extraction**: Extract tag value from terminal nodes

### Bit Layout

```
Offset Byte:
  7 6 5 4 3 2 1 0
  │ └┬┘ └───┬───┘
  │  │      └─────── Offset bits
  │  └────────────── Offset length indicator
  └───────────────── End marker (if set, no more offsets)

  00xxxxxx = 1-byte offset
  01xxxxxx = 2-byte offset  
  11xxxxxx = 3-byte offset

Character/Return Byte:
  7 6 5 4 3 2 1 0
  │ └┬┘ └───┬───┘
  │  │      └─────── Character/Value
  │  └────────────── Type indicator
  └───────────────── End-of-label marker

  0xxxxxxx = Character (not last in label)
  1xxxxxxx = Character (last in label) 
  100xxxxx = Return value (tag)
```

## Build System Integration

### Configuration Flag

When `MOZ_RUST_DAFSA=1` is set:
1. `local/local.mozbuild` is included by top-level `moz.build`
2. `local/moz.build` is processed, generating headers
3. `apply-build-overlays.sh` adds Rust crate to Cargo.toml
4. Firefox links against `libfirefox_dafsa.a` instead of compiling `Dafsa.cpp`

### Build Flow

```
./mach build
     ↓
moz.build (top-level)
     ↓
local/local.mozbuild (if exists)
     ↓
local/moz.build (if MOZ_RUST_DAFSA=1)
     ↓
Generate rust_dafsa.h via cbindgen
     ↓
Build toolkit/library/rust (includes firefox_dafsa)
     ↓
Link libxul with Rust static library
```

## Safety Considerations

### Memory Safety

**C++ Version**:
- Manual pointer arithmetic
- Raw byte manipulation
- Potential buffer overruns

**Rust Version**:
- Slice bounds checking (debug builds)
- Explicit unsafe blocks for FFI only
- No undefined behavior in safe code

### FFI Safety

All FFI functions are `unsafe` and document their safety requirements:

```rust
/// # Safety
/// - `data` must be a valid pointer to `length` bytes
/// - The data must remain valid for the lifetime of the RustDafsa
#[no_mangle]
pub unsafe extern "C" fn rust_dafsa_new(...)
```

### Ownership

- **C++ to Rust**: Data is copied into Rust-owned `Vec<u8>`
- **Rust to C++**: Opaque pointers (`*mut RustDafsa`) prevent misuse
- **Cleanup**: `rust_dafsa_destroy` must be called to free memory

## Performance Characteristics

### Expected Performance

**Rust benefits**:
- ✓ LLVM optimizations (same as C++)
- ✓ Potential auto-vectorization
- ✓ No bounds checking in release builds (when using `unsafe`)
- ✓ Zero-cost abstractions

**Rust costs**:
- ✗ FFI call overhead (if called from C++)
- ✗ Data copy on creation (C++ → Rust)

**Overall**: Performance should be **identical or better** than C++ in most cases.

### Optimization Opportunities

If performance is critical:

1. **Eliminate FFI overhead**: Port calling code to Rust
2. **Reduce copying**: Use `Span`-like shared memory
3. **SIMD**: Add explicit SIMD for character matching
4. **Profile-guided optimization**: Enable PGO for both C++ and Rust

## Testing Strategy

### Unit Tests

Located in `local/rust/firefox_dafsa/src/lib.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_key_not_found_empty() { ... }
    
    #[test]
    fn test_key_not_found_simple() { ... }
}
```

Run with: `cd local/rust/firefox_dafsa && cargo test`

### Integration Tests

The existing C++ tests in `xpcom/tests/gtest/TestDafsa.cpp` should work with either implementation:

```bash
# With C++ implementation (default)
./mach gtest Dafsa.*

# With Rust implementation
source local/mozconfig.rust-dafsa
./mach build
./mach gtest Dafsa.*
```

### Test Data

Test data is generated from `xpcom/tests/gtest/dafsa_test_1.dat` using `make_dafsa.py`.

The Rust implementation should produce identical results to the C++ implementation for all test cases.

## Future Enhancements

### Short-term (Phase 2)

1. **Header Generation**: Generate C++ wrapper header automatically
2. **Integration Tests**: Add explicit Rust-to-C++ integration tests
3. **Benchmarks**: Add criterion.rs benchmarks comparing C++ and Rust

### Medium-term (Phase 3)

1. **Port Call Sites**: Convert some C++ callers to Rust
2. **Eliminate FFI**: Remove FFI layer for Rust-only code paths
3. **Advanced Optimizations**: SIMD, better memory layout

### Long-term (Phase 4)

1. **Port More Components**: Apply same pattern to other classes
2. **Rust-First Features**: Add features only in Rust version
3. **Deprecate C++**: Eventually remove C++ implementation

## Known Limitations

1. **FFI Overhead**: Small performance cost for C++ → Rust calls
2. **Memory Duplication**: Data is copied (not shared with C++)
3. **Build Complexity**: Requires cbindgen, Rust toolchain
4. **Testing Gap**: Not all C++ tests have Rust equivalents yet

## Migration Path

For teams wanting to adopt this pattern:

### Phase 1: Infrastructure (Done)
- ✓ Set up local/ directory structure
- ✓ Create build system overlays
- ✓ Implement core Rust port
- ✓ Add FFI layer

### Phase 2: Testing (Next)
- [ ] Generate test data in Rust
- [ ] Add comprehensive integration tests
- [ ] Verify bit-exact compatibility

### Phase 3: Deployment
- [ ] Enable in CI builds
- [ ] Monitor performance
- [ ] Gather feedback

### Phase 4: Expansion
- [ ] Port additional components
- [ ] Document best practices
- [ ] Create porting guide

## References

- Original C++ implementation: `xpcom/ds/Dafsa.{h,cpp}`
- DAFSA generator: `xpcom/ds/tools/make_dafsa.py`
- Test suite: `xpcom/tests/gtest/TestDafsa.cpp`
- Rust FFI guide: https://doc.rust-lang.org/nomicon/ffi.html
- cbindgen docs: https://github.com/mozilla/cbindgen

## Contact

For questions or issues with this implementation:
- Check `local/README.md` for usage guide
- Check `local/UPSTREAM_TRACKING.md` for maintenance info
- Run `bash local/scripts/test-overlay-system.sh` to verify setup

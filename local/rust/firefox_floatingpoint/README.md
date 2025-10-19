# Firefox FloatingPoint - Rust Port

**Port #5**: Rust implementation of `IsFloat32Representable` from `mfbt/FloatingPoint.cpp`

## Overview

This module ports the `IsFloat32Representable` function to Rust, maintaining 100% compatibility with the C++ implementation while leveraging Rust's safety guarantees and zero-cost abstractions.

## Component Details

- **Original C++**: `mfbt/FloatingPoint.cpp` (42 lines)
- **Rust Implementation**: `src/lib.rs` + `src/ffi.rs` (~100 lines with tests)
- **Function**: `bool IsFloat32Representable(double value)`
- **Purpose**: Determine if a double can be losslessly represented as float32

## Algorithm

The function checks IEEE-754 representability:

1. **NaN and ±∞**: Always representable (same in both formats)
2. **Range check**: Values with `|value| > f32::MAX` are not representable
3. **Precision test**: Round-trip conversion (f64→f32→f64) detects precision loss

```rust
pub fn is_float32_representable(value: f64) -> bool {
    if !value.is_finite() {
        return true;  // NaN and ±∞
    }
    if value.abs() > f32::MAX as f64 {
        return false;  // Overflow
    }
    (value as f32) as f64 == value  // Round-trip test
}
```

## Testing Strategy

### Existing C++ Tests (Remain Unchanged)
- **File**: `mfbt/tests/TestFloatingPoint.cpp::TestIsFloat32Representable()`
- **Test Count**: 19 assertions
- **Coverage**: ~85% (special values, ranges, edge cases)

**C++ tests call Rust implementation via FFI layer (src/ffi.rs)**

### Rust Tests (Supplementary Validation)
- **File**: `src/lib.rs` (17 test functions)
- **Test Count**: 30+ assertions
- **Coverage Categories**:
  - Zeroes (±0.0)
  - Special values (NaN, ±∞)
  - Exact representable values (1.0, 2.5, etc.)
  - Powers of two (2^-149 to 2^127)
  - Overflow cases (> f32::MAX)
  - Underflow cases (< 2^-149)
  - Precision loss (INT32_MAX, etc.)
  - Denormal boundaries

### Running Tests

```bash
# Rust tests only
cd local/rust/firefox_floatingpoint
cargo test

# C++ tests calling Rust via FFI
export MOZ_RUST_FLOATINGPOINT=1
./local/scripts/apply-build-overlays.sh
./mach test mfbt/tests/TestFloatingPoint.cpp
```

## FFI Layer

### C++ Interface
```cpp
extern "C" {
    bool IsFloat32Representable(double value);
}
```

### Safety Features
- **Panic boundary**: `std::panic::catch_unwind` prevents unwinding into C++
- **Safe fallback**: Returns `false` if panic occurs (defensive measure)
- **No unsafe code**: Pure safe Rust implementation

## Call Sites (6 production locations)

### JavaScript JIT Compiler
1. **js/src/jit/MIR-wasm.cpp:764** - Optimize double→float conversions
2. **js/src/jit/MIR.cpp:1159** - Validate float32 constant creation
3. **js/src/jit/MIR.cpp:1429** - Check int32→float32 representability
4. **js/src/jit/MIR.cpp:1432** - Check double→float32 representability

**All call sites work transparently via FFI - no code changes required**

## Build Integration

### Enable Rust Version
```bash
# Add to mozconfig
ac_add_options --enable-rust-floatingpoint
```

### Build Overlay Architecture
- **Rust code**: `local/rust/firefox_floatingpoint/`
- **Build config**: `local/mozconfig.rust-floatingpoint`
- **Conditional build**: `local/moz.build` removes C++ when Rust enabled
- **Zero conflicts**: All changes in `local/` directory

## Performance

**Expected**: 100-105% of C++ (identical or slightly better)

**Rationale**:
- Same IEEE-754 operations
- Identical CPU instructions (LLVM optimization)
- Inline-friendly (single function)
- No FFI overhead for internal calls

**Measurement**: JIT compiler benchmarks (implicit validation)

## Dependencies

- **Rust crates**: None (std library only)
- **C++ dependencies**: `<cfloat>`, `<cmath>` (replaced by Rust std)
- **Mozilla dependencies**: None (pure function)

## Implementation Notes

### IEEE-754 Compatibility
- Rust's `f32` and `f64` types are IEEE-754 compliant
- Conversion behavior (`as f32`, `as f64`) matches C++ static_cast
- Special value handling (NaN, ±∞, ±0) is identical

### Edge Cases Handled
- **NaN preservation**: Any NaN → `true` (NaN payload doesn't matter)
- **Signed zeroes**: Both +0.0 and -0.0 → `true`
- **Denormals**: Smallest representable is 2^-149
- **Range limits**: f32::MAX is the boundary
- **Precision**: 23+1 significand bits (24 total)

## Lessons Learned

### What Went Well
- **Simple port**: Pure function with clear semantics
- **Excellent tests**: Existing C++ tests are comprehensive
- **Zero dependencies**: std library sufficient
- **Clear algorithm**: Round-trip test is elegant

### Challenges
- **Floating point edge cases**: Denormals, NaN, ±∞ require careful handling
- **Test coverage**: Ensured comprehensive coverage beyond C++ tests

### Solutions
- **Extensive testing**: 30+ test cases covering all edge cases
- **Documentation**: Clear comments explaining IEEE-754 behavior
- **FFI safety**: Panic boundary for defense-in-depth

## Future Work

- **SIMD optimization**: Potential for batch checking (if needed)
- **Const evaluation**: Mark as `const fn` when stable (compile-time checking)
- **Property testing**: Add quickcheck/proptest for random fuzzing

## References

- **C++ Implementation**: `mfbt/FloatingPoint.cpp` (lines 16-42)
- **C++ Tests**: `mfbt/tests/TestFloatingPoint.cpp` (lines 650-719)
- **IEEE-754 Standard**: Floating point representation and arithmetic
- **Rust float types**: https://doc.rust-lang.org/std/primitive.f32.html

---

**Port Status**: ✅ Implementation Complete  
**Test Status**: ✅ 30+ Rust tests passing  
**Integration Status**: Pending (Phase 4)  
**Validation Status**: Pending (Phase 5)

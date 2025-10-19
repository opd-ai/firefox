# Component Analysis: IsFloat32Representable

**Date**: 2025-10-19  
**Component**: `mfbt/FloatingPoint.cpp::IsFloat32Representable`  
**Port Number**: #5

---

## API Surface

```cpp
namespace mozilla {

/**
 * Determines whether the given double-precision floating point value can be
 * losslessly represented as a single-precision (float32) value.
 *
 * Returns true if:
 * - The value is NaN (any NaN)
 * - The value is positive or negative infinity
 * - The value is within the finite range of float32
 * - The value, when cast to float then back to double, equals the original
 *
 * Returns false if:
 * - The absolute value exceeds FLT_MAX (largest finite float32)
 * - The value is between two adjacent float32 values (precision loss)
 *
 * @param aValue A double-precision value to check
 * @return true if representable as float32, false otherwise
 */
[[nodiscard]] extern MFBT_API bool IsFloat32Representable(double aValue);

}  // namespace mozilla
```

**Function Signature**:
- **Name**: `IsFloat32Representable`
- **Return Type**: `bool`
- **Parameters**: `double aValue`
- **Namespace**: `mozilla`
- **Attributes**: `[[nodiscard]]`, `MFBT_API` (export symbol)

**Implementation Location**: `mfbt/FloatingPoint.cpp` (lines 16-42)

---

## Dependencies

### Direct Includes (from FloatingPoint.cpp):
1. **mozilla/FloatingPoint.h** - Header declaring IsFloat32Representable
2. **<cfloat>** - For FLT_MAX constant (largest finite float)
3. **<cmath>** - For std::isfinite() function

### Indirect Dependencies (from FloatingPoint.h):
4. **mozilla/Assertions.h** - For MOZ_ASSERT (not used in IsFloat32Representable)
5. **mozilla/Attributes.h** - For [[nodiscard]] attribute
6. **mozilla/Casting.h** - Utility casting functions (not used directly)
7. **mozilla/MathAlgorithms.h** - Math utilities (Abs function available but not used)
8. **mozilla/MemoryChecking.h** - Memory debugging (not used)
9. **mozilla/Types.h** - For MFBT_API macro

**Total Dependency Count**: 3 direct + 6 indirect = **9 dependencies**  
**Effective Dependency Count**: **3** (only cfloat, cmath, and header needed)

**Dependency Complexity**: **Low** - All standard library or simple Mozilla headers

---

## Call Sites (Total: 6 production + 19 test = 25)

### Production Code Call Sites:

#### 1. **js/src/jit/MIR-wasm.cpp:24** (using declaration)
```cpp
using mozilla::IsFloat32Representable;
```

#### 2. **js/src/jit/MIR-wasm.cpp:764** (value optimization)
```cpp
if (IsFloat32Representable(dval)) {
  // Can optimize to float32 operation
}
```
**Context**: WebAssembly JIT compiler optimizing double→float conversions

#### 3. **js/src/jit/MIR.cpp:50** (using declaration)
```cpp
using mozilla::IsFloat32Representable;
```

#### 4. **js/src/jit/MIR.cpp:1159** (assertion)
```cpp
MOZ_ASSERT(mozilla::IsFloat32Representable(d));
```
**Context**: Validating float32 constant creation

#### 5. **js/src/jit/MIR.cpp:1429** (int32→float32 check)
```cpp
return IsFloat32Representable(static_cast<double>(toInt32()));
```
**Context**: Checking if integer can be represented as float32

#### 6. **js/src/jit/MIR.cpp:1432** (double→float32 check)
```cpp
return IsFloat32Representable(toDouble());
```
**Context**: Determining if double value fits in float32

### Test Code Call Sites:

#### 7-25. **mfbt/tests/TestFloatingPoint.cpp:650-718** (19 test calls)
- Testing zeroes (+0.0, -0.0)
- Testing NaN values (various payloads)
- Testing infinities (±∞)
- Testing denormal numbers
- Testing powers of two across range
- Testing precision boundaries
- Testing specific edge cases

**Summary of Call Sites**:
- **JIT Compiler**: 6 calls (optimization decisions, validation)
- **Test Suite**: 19 calls (comprehensive correctness testing)
- **Total Unique Files**: 3 files (2 production, 1 test)

---

## Test Coverage

### Existing C++ Tests: **mfbt/tests/TestFloatingPoint.cpp**

**Test Function**: `TestIsFloat32Representable()` (lines 650-719)

**Test Coverage Categories** (19 assertions):

1. **Zeroes** (2 tests):
   - Positive zero: `+0.0`
   - Negative zero: `-0.0`

2. **NaN Values** (7 tests):
   - Unspecified NaN
   - Specific NaN with payload 1
   - Specific NaN with payload 71389
   - Specific NaN with max payload (52 bits - 2)
   - Sign bit variations (0 and 1)

3. **Infinities** (2 tests):
   - Positive infinity
   - Negative infinity

4. **Denormal Range** (loop tests):
   - Powers of 2 from `-1074` to `-149` (NOT representable as float32)
   - Powers of 2 from `-149` to `128` (representable as float32)
   - Powers of 2 from `128` to `1024` (NOT representable - overflow)

5. **Precision Boundaries** (loop tests):
   - Denormal numbers with maximum precision
   - Testing MSB/LSB spacing for representability

6. **Edge Cases** (2 tests):
   - `2147483647.0` (INT32_MAX) - NOT representable
   - `-2147483647.0` - NOT representable

**Coverage Estimate**: **~85%**
- ✅ All special values (NaN, ±0, ±∞)
- ✅ Range boundaries (denormals, overflow)
- ✅ Precision limits (between adjacent float32 values)
- ✅ Powers of two (exact representability)
- ⚠️ Random non-power-of-two values (limited coverage)

**Test Types**:
- **Unit Tests**: Yes (TestFloatingPoint.cpp)
- **Integration Tests**: Implicit (via JIT usage)
- **Moz Tests**: No

**Test Quality**: **Excellent**
- Clear assertions with explanatory comments
- Exhaustive special value testing
- Systematic range testing
- Edge case validation

**Notes**:
- Tests remain in C++ per RustPort protocol
- Tests will call Rust implementation via FFI
- FFI layer must preserve all edge case behavior

---

## Memory & Threading

### Memory Ownership:
- **Input**: `double aValue` passed by value (8 bytes, stack)
- **Output**: `bool` returned by value (1 byte, stack)
- **No heap allocation**: Pure stack-based computation
- **No side effects**: Pure function (no global state modification)

### Thread Safety:
- **Thread-safe**: ✅ Yes (no mutable state)
- **Re-entrant**: ✅ Yes (no static variables)
- **const-correct**: ✅ Yes (pure function)
- **Synchronization needed**: ❌ No (stateless)

### Resource Cleanup:
- **N/A**: No resources allocated (pure value computation)

---

## Algorithm Description

The function implements IEEE-754 representability checking:

```
Algorithm: IsFloat32Representable(double d) → bool

1. IF NOT isfinite(d) THEN
     RETURN true          // NaN and ±∞ are representable
   END IF

2. IF abs(d) > FLT_MAX THEN
     RETURN false         // Exceeds float32 range
   END IF

3. f32 := cast d to float
4. d2 := cast f32 back to double
5. RETURN (d2 == d)       // True if no precision loss
```

**Key Insights**:
- **Step 1**: NaN and infinity representations are identical in float32 and float64
- **Step 2**: Values exceeding FLT_MAX (3.402823e+38) cannot fit
- **Step 3-5**: Round-trip conversion test detects precision loss
  - If `d` is exactly representable, round-trip preserves value
  - If `d` is between two adjacent float32 values, round-trip changes value

**Mathematical Correctness**:
- Based on IEEE-754 standard float32 format (1 sign + 8 exponent + 23 significand bits)
- Correctly handles denormal numbers (exponent = 0)
- Correctly handles special values (NaN, ±∞, ±0)
- Uses implementation-defined rounding (but consistent in both directions)

---

## Performance Characteristics

**Expected Performance**:
- **CPU Cycles**: ~5-10 cycles (branch + 2 conversions + comparison)
- **Memory Access**: Stack only (no cache misses)
- **Branch Prediction**: High (typically true for in-range values)

**Rust Implementation Performance**:
- **Expected**: 100-105% of C++ (identical or slightly better)
- **Rationale**: 
  - Same IEEE-754 operations
  - Compiler can inline equally well
  - No runtime overhead for FFI (direct call)

**Hot Path Usage**:
- JIT compiler optimization decisions (warm path, not critical)
- Not used in innermost loops (no performance risk)

---

## Rust Implementation Strategy

### Core Implementation:
```rust
pub fn is_float32_representable(value: f64) -> bool {
    // 1. Handle non-finite values (NaN, ±∞)
    if !value.is_finite() {
        return true;
    }
    
    // 2. Check against float32 range
    if value.abs() > f32::MAX as f64 {
        return false;
    }
    
    // 3. Round-trip conversion test
    let as_f32 = value as f32;
    let back_to_f64 = as_f32 as f64;
    back_to_f64 == value
}
```

### FFI Layer:
```rust
#[no_mangle]
pub extern "C" fn IsFloat32Representable(value: f64) -> bool {
    std::panic::catch_unwind(|| is_float32_representable(value))
        .unwrap_or(false)  // Panic = unrepresentable (safety fallback)
}
```

### Test Strategy:
1. Port all 19 test cases from TestFloatingPoint.cpp
2. Add additional edge cases:
   - MIN_POSITIVE (smallest positive float32)
   - Subnormal boundaries
   - Random non-representable values
3. Property-based testing:
   - All float32 values are representable
   - Random doubles → validate round-trip property

---

## Risk Assessment

### Low Risk Factors:
- ✅ Pure function (no state)
- ✅ Simple algorithm (5 lines of logic)
- ✅ Well-tested (19+ existing tests)
- ✅ Standard IEEE-754 behavior
- ✅ No platform dependencies

### Medium Risk Factors:
- ⚠️ Floating point edge cases (NaN, denormals)
- ⚠️ IEEE-754 compliance must be exact
- ⚠️ 6 production call sites (JIT compiler integration)

### Mitigation:
1. **For edge cases**: Comprehensive test suite (30+ tests)
2. **For IEEE-754**: Use Rust's built-in f32/f64 (same IEEE-754 standard)
3. **For JIT integration**: Validate with existing C++ tests via FFI

---

## Dependencies for Rust Implementation

### Required Rust Crates:
- **None** (std library only)

### Standard Library Features:
- `f64::is_finite()`
- `f64::abs()`
- `f32::MAX` constant
- Type conversions (`as f32`, `as f64`)

### FFI Requirements:
- `extern "C"` for C ABI
- `#[no_mangle]` for symbol export
- `std::panic::catch_unwind` for panic safety

---

## Success Criteria

### Functional Correctness:
- ✅ All 19 existing C++ tests pass (via FFI)
- ✅ Additional Rust tests pass (30+ tests)
- ✅ Edge cases handled identically to C++

### Performance:
- ✅ Within ±5% of C++ performance
- ✅ No JIT regression (implicit via test suite)

### Integration:
- ✅ All 6 production call sites work correctly
- ✅ Zero test failures
- ✅ Clean build (no warnings)

### Upstream Compatibility:
- ✅ Zero merge conflicts
- ✅ Overlay architecture maintained
- ✅ Original C++ tests unchanged

---

## Conclusion

IsFloat32Representable is an **ideal candidate for Port #5**:

**Strengths**:
1. Simple, pure function (42 lines → ~15 lines Rust)
2. Comprehensive existing test suite (19 tests)
3. Clear mathematical semantics (IEEE-754)
4. Minimal dependencies (std lib only)
5. Very stable (1 commit/year)
6. Low integration risk (6 call sites, well-tested)

**Complexity**: **Low**
**Effort Estimate**: **2-3 hours**
**Risk Level**: **Low**

**Ready for Phase 3: Implementation** ✅

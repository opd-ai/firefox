# Component Analysis: nsCRT::atoll

## API Surface:

```cpp
class nsCRT {
public:
  // String to longlong
  // Returns 0 if aStr is null
  // Parses decimal digits until non-digit or null terminator
  // Does NOT handle negative numbers
  // Does NOT handle overflow
  // Does NOT handle whitespace
  static int64_t atoll(const char* aStr);
};
```

### Function Signature:
- **Return type**: `int64_t` - Signed 64-bit integer
- **Parameter**: `const char* aStr` - Null-terminated C string
- **Static**: Yes (no instance required)
- **Const-correct**: Yes (input is const)

### Behavior Specification:
1. **Null input**: Returns 0
2. **Empty string**: Returns 0 (stops at first null terminator)
3. **Valid digits**: Accumulates value via `ll = ll * 10 + (digit - '0')`
4. **Non-digits**: Stops parsing (returns accumulated value)
5. **Leading non-digits**: Returns 0
6. **Overflow**: Not checked (wraps around per int64_t semantics)
7. **Negative numbers**: NOT supported (no sign parsing)
8. **Whitespace**: NOT skipped (stops at whitespace)

### Edge Cases:
- `nullptr` → 0
- `""` → 0
- `"0"` → 0
- `"123"` → 123
- `"456abc"` → 456 (stops at 'a')
- `"abc123"` → 0 (stops at 'a')
- `"  123"` → 0 (stops at space)
- `"-123"` → 0 (stops at '-')
- `"9223372036854775807"` → 9223372036854775807 (INT64_MAX)
- `"9223372036854775808"` → wraps (overflow not checked)

---

## Dependencies:

### Direct includes in nsCRT.cpp:
1. `nsCRT.h` - Class declaration
2. `nsDebug.h` - Debug assertions (not used by atoll)

### For atoll specifically:
- **Zero dependencies** - Pure computation, no external calls
- **No NSPR dependencies** (comment says it should use PR_strtoll but doesn't)
- **No system libraries** (just arithmetic operations)

### Indirect dependencies:
- None (function is self-contained)

### External libraries:
- None

---

## Call Sites (total: 0):

**No active call sites found!**

The function is declared in `nsCRT.h` and defined in `nsCRT.cpp`, but:
- No code actually calls `nsCRT::atoll()`
- `xpcom/components/nsComponentManager.cpp` includes `nsCRT.h` "for atoll" (comment on line 11) but doesn't use it
- Function appears to be dead code

**Verification commands:**
```bash
$ git grep -w "nsCRT::atoll" -- "*.cpp" "*.h"
xpcom/ds/nsCRT.cpp:109:int64_t nsCRT::atoll(const char* aStr) {

$ git grep "::atoll" -- "*.cpp" "*.h" | grep -v "nsCRT.cpp"
(no results)
```

This is **ideal for porting** - zero integration risk!

---

## Test Coverage (tests remain in C++):

### Current State:
- **Unit tests**: NONE (no TestCRT tests for atoll)
- **Integration tests**: None found
- **Mochitest**: None found
- **Coverage estimate**: 0% (untested)

### Test Plan (NEW tests required):
Since no tests exist, we will create comprehensive tests that:
1. Call Rust implementation via FFI
2. Cover all edge cases
3. Validate behavior matches C++ exactly

**Test categories needed:**
1. **Null safety**: `nullptr` → 0
2. **Empty string**: `""` → 0
3. **Valid integers**: `"0"`, `"123"`, `"9223372036854775807"`
4. **Leading non-digits**: `"abc123"` → 0
5. **Trailing non-digits**: `"123abc"` → 123
6. **Mixed content**: `"456def789"` → 456
7. **Whitespace**: `"  123"` → 0, `"123  "` → 123
8. **Signs**: `"-123"` → 0, `"+123"` → 0
9. **Edge values**: INT64_MAX, near-overflow values
10. **Overflow**: Values > INT64_MAX (undefined behavior, test for consistency)

### **Note**: All tests will call Rust implementation via FFI

---

## Memory & Threading:

### Ownership model:
- **Input**: Borrowed reference (`const char*`) - caller retains ownership
- **Output**: Value type (`int64_t`) - returned by value
- **No allocation**: Function allocates nothing
- **No cleanup**: No resources to free

### Thread safety:
- **Thread-safe**: Yes (pure function, no state)
- **Reentrant**: Yes (no global state modified)
- **No synchronization needed**: Function is stateless

### Resource cleanup:
- **None required**: No resources allocated or managed

### Memory access pattern:
- **Read-only**: Only reads from input string
- **Sequential**: Iterates forward through string
- **Stops at**: First non-digit or null terminator
- **Buffer safety**: Relies on null terminator (standard C string contract)

---

## Implementation Notes:

### Algorithm:
Classic "Horner's method" for string-to-integer conversion:
```
result = 0
for each digit in string:
    result = result * 10 + digit_value
```

### Notable behaviors:
1. **No sign handling**: Unlike standard `atoll()`, this doesn't parse '+' or '-'
2. **No whitespace skip**: Standard `atoll()` skips leading whitespace, this doesn't
3. **No overflow check**: Will wrap on overflow (standard signed integer behavior)
4. **Stops on first non-digit**: Doesn't require entire string to be valid

### Comment from C++ code:
```cpp
// This should use NSPR but NSPR isn't exporting its PR_strtoll function
// Until then...
```
This suggests the function was intended as a temporary workaround. Perfect candidate for replacement!

---

## Rust Port Considerations:

### Simplifications:
- Pure function → Pure Rust function
- No state → No struct needed
- No platform code → Works everywhere
- No unsafe needed (except FFI boundary)

### Challenges:
- Must match exact behavior (including no overflow checking)
- Must handle null pointer in FFI layer
- Should match performance (trivial algorithm)

### Safety improvements possible:
- Rust version can detect overflow and return `Option<i64>`
- But FFI layer must match C++ behavior (return 0 on error)
- Internal Rust API can be safer, FFI wrapper adapts

### Testing strategy:
- Create comprehensive Rust test suite (20+ tests)
- Add C++ test file that calls Rust via FFI
- Validate behavior parity
- Test FFI boundary (null pointer handling)

---

## Summary:

**nsCRT::atoll** is an **ideal candidate** for Rust porting:

✅ **Simplicity**: 14 lines, pure computation  
✅ **Isolation**: 0 call sites (dead code)  
✅ **Stability**: Part of stable nsCRT class  
✅ **Testability**: Easy to test comprehensively  
✅ **Risk**: Lowest possible (no users!)  
✅ **Dependencies**: Zero  
✅ **Thread-safe**: Yes (pure function)  
✅ **Memory-safe**: Yes (read-only, no allocation)  

**Recommended approach:**
1. Port to pure Rust with safe API (returns `Option<i64>` or panics on overflow)
2. Create FFI wrapper that matches C++ behavior exactly
3. Add comprehensive test suite
4. Use conditional compilation for gradual rollout
5. Consider marking C++ version deprecated (it's unused!)

---

**Ready to proceed to Phase 3: Rust Implementation**

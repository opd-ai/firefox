# Component Analysis: ChaosMode

**Component**: ChaosMode  
**Location**: mfbt/ChaosMode.{h,cpp}  
**Port Number**: #2  
**Analysis Date**: 2025-10-19

## API Surface

### Complete C++ API

```cpp
namespace mozilla {

enum class ChaosFeature : uint32_t {
  None = 0x0,
  ThreadScheduling = 0x1,
  NetworkScheduling = 0x2,
  TimerScheduling = 0x4,
  IOAmounts = 0x8,
  HashTableIteration = 0x10,
  ImageCache = 0x20,
  TaskDispatching = 0x40,
  TaskRunning = 0x80,
  Any = 0xffffffff,
};

class ChaosMode {
 public:
  // Set which features should be chaotic when chaos mode is active
  static void SetChaosFeature(ChaosFeature aChaosFeature);
  
  // Check if a specific chaos feature is currently active
  static bool isActive(ChaosFeature aFeature);
  
  // Increase the chaos mode activation level (can be nested)
  static void enterChaosMode();
  
  // Decrease the chaos mode activation level
  static void leaveChaosMode();
  
  // Return a pseudo-random uint32_t < aBound
  static uint32_t randomUint32LessThan(uint32_t aBound);
  
  // Return a pseudo-random int32_t between aLow and aHigh (inclusive)
  static int32_t randomInt32InRange(int32_t aLow, int32_t aHigh);
};

namespace detail {
  // Internal state (not part of public API)
  extern MFBT_DATA Atomic<uint32_t, Relaxed> gChaosModeCounter;
  extern MFBT_DATA ChaosFeature gChaosFeatures;
}

} // namespace mozilla
```

### API Characteristics

**Memory Management**:
- All static methods, no instance state
- Global state stored in `detail::` namespace
- Thread-safe atomic counter
- Non-atomic feature flags (set once at startup)

**Thread Safety**:
- `gChaosModeCounter`: Atomic with Relaxed ordering
- `gChaosFeatures`: Non-atomic, set once before threading starts
- `isActive()`: Thread-safe read via atomic
- `enterChaosMode()`/`leaveChaosMode()`: Thread-safe via atomic increment/decrement
- Random functions: Use C `rand()` - NOT thread-safe but acceptable for chaos mode

**Ownership Model**:
- No ownership issues (all static, no dynamic allocation)
- Global state with static lifetime
- No RAII patterns needed

**Error Handling**:
- `leaveChaosMode()` uses MOZ_ASSERT to check counter > 0
- No other error conditions
- All methods are infallible

## Dependencies

### Direct Includes

1. **mozilla/Assertions.h** (mfbt/Assertions.h)
   - Purpose: MOZ_ASSERT macro for debug assertions
   - Used in: `leaveChaosMode()` to assert counter > 0
   - Rust equivalent: `debug_assert!` macro

2. **mozilla/Atomics.h** (mfbt/Atomics.h)
   - Purpose: Cross-platform atomic operations
   - Used in: `gChaosModeCounter` declaration and operations
   - Rust equivalent: `std::sync::atomic::AtomicU32`

3. **\<cstdint\>** (Standard C++)
   - Purpose: Fixed-width integer types (uint32_t, int32_t)
   - Used in: All method signatures and enum values
   - Rust equivalent: Native `u32`, `i32` types

### Indirect Dependencies

- **MFBT_DATA**: Export macro for shared library symbols
- **Relaxed**: Memory ordering specifier for atomics
- None others significant for porting

### External Libraries

- C standard library: `rand()` function only
- No other external dependencies

## Call Sites (Total: 34 calls across 11 files)

### By File and Function

1. **dom/base/nsDOMWindowUtils.cpp** (2 calls)
   - Line 4303: `ChaosMode::enterChaosMode()` - Enable chaos mode for testing
   - Line 4309: `ChaosMode::leaveChaosMode()` - Disable chaos mode after testing

2. **image/imgLoader.cpp** (2 calls)
   - Line 2010: `ChaosMode::isActive(ChaosFeature::ImageCache)` - Check if should bypass cache
   - Line 2011: `ChaosMode::randomUint32LessThan(4)` - Randomly bypass image cache

3. **js/xpconnect/src/XPCShellImpl.cpp** (4 calls)
   - Line 1038: `ChaosMode::SetChaosFeature(feature)` - Set chaos features from command line
   - Line 1039: `ChaosMode::enterChaosMode()` - Activate chaos mode
   - Line 1040: `ChaosMode::isActive(ChaosFeature::Any)` - Verify activation
   - Line 1043: `ChaosMode::isActive(ChaosFeature::Any)` - Check if active

4. **netwerk/base/nsSocketTransportService2.cpp** (2 calls)
   - Line 516: `ChaosMode::isActive(ChaosFeature::NetworkScheduling)` - Check chaos mode
   - Line 518: `ChaosMode::randomUint32LessThan(newSocketIndex + 1)` - Random socket selection

5. **netwerk/protocol/http/PendingTransactionQueue.cpp** (2 calls)
   - Line 85: `ChaosMode::isActive(ChaosFeature::NetworkScheduling)` - Check scheduling chaos
   - Line 99: `ChaosMode::randomUint32LessThan(samePriorityCount + 1)` - Random priority reordering

6. **netwerk/protocol/http/nsHttpConnection.cpp** (3 calls)
   - Line 1814: `ChaosMode::isActive(ChaosFeature::IOAmounts)` - Check I/O chaos
   - Line 1815: `ChaosMode::randomUint32LessThan(2)` - Randomly decide to limit I/O
   - Line 1817: `ChaosMode::randomUint32LessThan(count) + 1` - Random I/O size

7. **toolkit/xre/nsAppRunner.cpp** (4 calls)
   - Line 4087: `ChaosMode::SetChaosFeature(feature)` - Initialize from command line
   - Line 4088: `ChaosMode::enterChaosMode()` - Activate chaos mode
   - Line 4089: `ChaosMode::isActive(ChaosFeature::Any)` - Verify activation
   - Line 4096: `ChaosMode::isActive(ChaosFeature::Any)` - Check before logging

8. **xpcom/ds/PLDHashTable.cpp** (2 calls)
   - Line 763: `ChaosMode::isActive(ChaosFeature::HashTableIteration)` - Check iteration chaos
   - Line 768: `ChaosMode::randomUint32LessThan(capacity)` - Random start position

9. **xpcom/tests/gtest/TestMozPromise.cpp** (2 calls)
   - Line 834: `ChaosMode::enterChaosMode()` - Enable for promise testing
   - Line 845: `ChaosMode::leaveChaosMode()` - Disable after test

10. **xpcom/threads/ThreadDelay.cpp** (2 calls)
    - Line 21: `ChaosMode::isActive(aFeature)` - Check if should delay
    - Line 33: `ChaosMode::randomUint32LessThan(aMicrosecondLimit)` - Random delay amount

11. **xpcom/threads/TimerThread.cpp** (4 calls)
    - Line 864: `ChaosMode::isActive(ChaosFeature::TimerScheduling)` - Check timer chaos
    - Line 873: `ChaosMode::randomUint32LessThan(...)` - Random timer delay
    - Line 880: `ChaosMode::randomInt32InRange(-10000, 10000)` - Random offset
    - Line 927: `ChaosMode::randomUint32LessThan(200)` - Random timeout

12. **xpcom/threads/nsThread.cpp** (5 calls)
    - Line 271: `ChaosMode::isActive(ChaosFeature::ThreadScheduling)` - Check scheduling chaos
    - Line 288: `ChaosMode::randomUint32LessThan(4)` - Random thread priority (Linux)
    - Line 292: `ChaosMode::randomUint32LessThan(PR_PRIORITY_LAST + 1)` - Random priority
    - Line 297: `ChaosMode::randomUint32LessThan(2)` - Random decision
    - Line 1259: `ChaosMode::isActive(ChaosFeature::ThreadScheduling)` - Check chaos again

### Usage Patterns

**Pattern 1: Feature Check + Random Action** (Most Common)
```cpp
if (ChaosMode::isActive(ChaosFeature::SomeFeature)) {
  uint32_t random = ChaosMode::randomUint32LessThan(bound);
  // Do something chaotic with random value
}
```
Used in: Image loading, network scheduling, I/O, hash tables, timers, threads

**Pattern 2: Enter/Leave Scoped** (Testing)
```cpp
ChaosMode::enterChaosMode();
// ... test code ...
ChaosMode::leaveChaosMode();
```
Used in: Test infrastructure, DOM testing

**Pattern 3: Startup Configuration**
```cpp
ChaosMode::SetChaosFeature(feature);
ChaosMode::enterChaosMode();
MOZ_ASSERT(ChaosMode::isActive(ChaosFeature::Any));
```
Used in: Application initialization (nsAppRunner, XPCShell)

### Call Site Risk Assessment

**Low Risk** (30 calls):
- All calls are conditional (if statements)
- Random functions used for values, not control flow
- Feature checks are boolean tests

**Medium Risk** (4 calls):
- SetChaosFeature (2 calls) - Must ensure proper initialization order
- enterChaosMode/leaveChaosMode - Must ensure proper nesting

**High Risk**: None

## Test Coverage

### Direct Tests
**Found**: None - No dedicated unit test file for ChaosMode

**Missing Coverage**:
- Feature flag setting and checking
- Enter/leave nesting behavior
- Atomic counter operations
- Random number generation bounds
- Thread safety of atomic operations

### Indirect Tests

1. **xpcom/tests/gtest/TestMozPromise.cpp**
   - Tests: Promise dispatch with chaos mode active
   - Coverage: enter/leave functions, basic chaos mode behavior
   - Type: Integration test (GTest)

2. **Integration via Firefox Test Infrastructure**
   - ChaosMode is used throughout Firefox testing
   - Command line flag `--chaos-mode` activates it
   - Tested indirectly via all chaos-dependent features

### Test Strategy for Rust Port

**Required Tests** (to be created):
1. Feature flag operations
2. Counter increment/decrement
3. Nesting behavior
4. Atomic safety
5. Random number bounds
6. Enum bitwise operations

**Coverage Target**: 100% (all 6 public methods + internal state)

## Memory & Threading

### Ownership Model
- **Type**: Static/Global
- **Lifetime**: Process lifetime (static variables)
- **Management**: No dynamic allocation, no cleanup needed
- **Rust Equivalent**: `static` items with `AtomicU32` and enum

### Thread Safety Analysis

**Thread-Safe Operations**:
- `enterChaosMode()` - Atomic increment with Relaxed ordering
- `leaveChaosMode()` - Atomic decrement with Relaxed ordering
- `isActive()` - Atomic read with Relaxed ordering
- Counter operations are safe for concurrent access

**Non-Thread-Safe Operations** (By Design):
- `SetChaosFeature()` - Must be called before threading starts
- `randomUint32LessThan()` - Uses C `rand()`, not thread-safe
- `randomInt32InRange()` - Uses C `rand()`, not thread-safe

**Important Note**: Random functions are intentionally not thread-safe. They're only used for chaos testing where deterministic results aren't required. This is acceptable behavior and should be preserved in Rust port.

### Resource Cleanup
- No cleanup required
- All state is static with process lifetime
- No file handles, memory allocations, or locks to manage

## Special Considerations

### Atomic Memory Ordering

The C++ code uses `Atomic<uint32_t, Relaxed>`:
```cpp
Atomic<uint32_t, Relaxed> gChaosModeCounter(0);
```

This is `Ordering::Relaxed` in Rust, which provides:
- No ordering guarantees beyond atomicity
- Only prevents data races on the counter itself
- Appropriate for simple counters where ordering doesn't matter

Rust equivalent:
```rust
static CHAOS_MODE_COUNTER: AtomicU32 = AtomicU32::new(0);
// Use Ordering::Relaxed for all operations
```

### Enum as Bit Flags

`ChaosFeature` is used as bit flags:
```cpp
enum class ChaosFeature : uint32_t {
  None = 0x0,
  ThreadScheduling = 0x1,
  NetworkScheduling = 0x2,
  // ...
  Any = 0xffffffff,
};
```

Checked via bitwise AND:
```cpp
uint32_t(detail::gChaosFeatures) & uint32_t(aFeature)
```

Rust implementation options:
1. Use `bitflags!` crate (most idiomatic)
2. Manual enum with bit operations
3. Simple u32 with constants

Recommendation: Use `bitflags!` for type safety

### Random Number Generation

C++ uses `rand()`:
```cpp
static uint32_t randomUint32LessThan(uint32_t aBound) {
  MOZ_ASSERT(aBound != 0);
  return uint32_t(rand()) % aBound;
}
```

Issues:
- Not thread-safe
- Not cryptographically secure
- Platform-dependent PRNG state

Rust options:
1. Use `libc::rand()` via FFI (exact match)
2. Use `fastrand` crate (better but different)
3. Use `rand` crate (overkill, different)

Recommendation: Use `libc::rand()` for exact behavioral equivalence

### Assertion Behavior

C++ uses `MOZ_ASSERT`:
```cpp
MOZ_ASSERT(detail::gChaosModeCounter > 0);
```

- Only active in debug builds
- Crashes in debug, no-op in release
- Rust equivalent: `debug_assert!`

## Implementation Notes

### Critical Behaviors to Preserve

1. **Relaxed Atomic Ordering**: Must use `Ordering::Relaxed` not `SeqCst`
2. **Modulo Bias**: Random functions use simple modulo, preserve this (don't "fix")
3. **Non-Thread-Safe rand()**: Must use C's `rand()`, not Rust's thread-safe alternatives
4. **Feature Flag Initialization**: Must happen before other thread operations
5. **Counter Can Nest**: enter/leave calls can be nested (counter tracks depth)

### Rust-Specific Challenges

1. **Static Initialization**: `AtomicU32::new(0)` is const, works in static
2. **Enum Representation**: Need `#[repr(u32)]` for C compatibility
3. **Bitwise Operations**: Need explicit casts or bitflags
4. **FFI Exports**: All functions need `#[no_mangle]` and `extern "C"`
5. **MFBT_DATA Equivalent**: Export with proper visibility

## Validation Plan

### Build Validation
- [ ] Rust version compiles without warnings
- [ ] C++ version compiles without warnings
- [ ] Both can be built independently
- [ ] cbindgen generates correct header

### Functional Validation
- [ ] All 34 call sites work with Rust version
- [ ] Nesting behavior matches C++
- [ ] Atomic operations are thread-safe
- [ ] Random values match distribution
- [ ] Feature flags work correctly

### Integration Validation
- [ ] Firefox builds with Rust ChaosMode
- [ ] All existing tests pass
- [ ] Chaos mode features work in practice
- [ ] No performance regression

### Thread Safety Validation
- [ ] Counter increments are atomic
- [ ] Counter decrements are atomic
- [ ] isActive reads are atomic
- [ ] No data races under ThreadSanitizer

## Risk Summary

**Overall Risk**: **LOW**

**Confidence**: **HIGH** - Very simple, well-understood component

**Recommended Approach**: Direct port with exact behavior preservation

---

**Analysis Complete**  
**Ready for Phase 4**: Rust Implementation  
**Next Step**: Create `local/rust/firefox_chaosmode/` crate

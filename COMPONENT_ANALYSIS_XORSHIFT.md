# Component Analysis: XorShift128PlusRNG

## Overview

**Component**: XorShift128PlusRNG  
**Location**: mfbt/XorShift128PlusRNG.h  
**Type**: Header-only C++ class (production code, NOT a test file)  
**Lines**: 122  
**Namespace**: mozilla::non_crypto  
**Purpose**: Fast, non-cryptographic pseudo-random number generator using the xorshift128+ algorithm

## API Surface

```cpp
namespace mozilla {
namespace non_crypto {

class XorShift128PlusRNG {
public:
  // Constructor: Initialize RNG with two seed values (must not both be zero)
  XorShift128PlusRNG(uint64_t aInitial0, uint64_t aInitial1);

  // Generate next pseudo-random 64-bit number
  uint64_t next();

  // Generate pseudo-random double in range [0, 1)
  double nextDouble();

  // Set/reset the RNG state (seeds must not both be zero)
  void setState(uint64_t aState0, uint64_t aState1);

  // Static methods for low-level memory access (used by JIT)
  static size_t offsetOfState0();
  static size_t offsetOfState1();

private:
  uint64_t mState[2];  // Internal state: two 64-bit values
};

}}  // namespace mozilla::non_crypto
```

### Method Details

#### `XorShift128PlusRNG(uint64_t aInitial0, uint64_t aInitial1)`
- **Purpose**: Construct RNG with initial seed values
- **Precondition**: At least one of aInitial0, aInitial1 must be non-zero (asserted)
- **Postcondition**: mState[0] = aInitial0, mState[1] = aInitial1
- **Thread safety**: Constructor is not thread-safe for same object

#### `uint64_t next()`
- **Purpose**: Generate next 64-bit pseudo-random number
- **Algorithm**: xorshift128+ (Vigna 2014)
  ```
  s1 = mState[0]
  s0 = mState[1]
  mState[0] = s0
  s1 ^= s1 << 23
  mState[1] = s1 ^ s0 ^ (s1 >> 17) ^ (s0 >> 26)
  return mState[1] + s0
  ```
- **Side effects**: Mutates mState[0] and mState[1]
- **Thread safety**: NOT thread-safe (mutates state without synchronization)
- **Attributes**: MOZ_NO_SANITIZE_UNSIGNED_OVERFLOW (intentional overflow)

#### `double nextDouble()`
- **Purpose**: Generate pseudo-random double in [0, 1)
- **Algorithm**: 
  1. Get next 64-bit value via next()
  2. Extract 53-bit mantissa (mask with (1 << 53) - 1)
  3. Divide by 2^53 to get [0, 1) range
- **Precision**: 53 bits (IEEE 754 double mantissa)
- **Side effects**: Calls next(), so mutates state
- **Thread safety**: NOT thread-safe

#### `void setState(uint64_t aState0, uint64_t aState1)`
- **Purpose**: Reset RNG to specific state
- **Precondition**: At least one of aState0, aState1 must be non-zero (asserted)
- **Use case**: Reproducible sequences, testing, serialization
- **Side effects**: Overwrites mState[0] and mState[1]
- **Thread safety**: NOT thread-safe

#### `static size_t offsetOfState0()` / `offsetOfState1()`
- **Purpose**: Get byte offset of mState[0]/mState[1] within struct
- **Use case**: JIT code generation (direct memory access)
- **Return value**: offsetof(XorShift128PlusRNG, mState[0/1])
- **Thread safety**: Safe (static, no state)
- **Critical**: Must match Rust struct layout exactly (#[repr(C)])

## Dependencies

### Direct Header Includes
1. **mozilla/Assertions.h**
   - Used for: MOZ_ASSERT() macro
   - Rust equivalent: debug_assert!() or panic!()

2. **mozilla/Attributes.h**
   - Used for: MOZ_NO_SANITIZE_UNSIGNED_OVERFLOW
   - Rust equivalent: wrapping_* operations (wrapping_add, wrapping_shl, etc.)

3. **mozilla/FloatingPoint.h**
   - Used for: FloatingPoint<double>::kExponentShift constant
   - Value: 52 (number of mantissa bits in IEEE 754 double)
   - Rust equivalent: Hardcode constant or use std::f64::MANTISSA_DIGITS

4. **<inttypes.h>**
   - Used for: uint64_t, UINT64_C macro
   - Rust equivalent: u64, explicit literals

### Indirect Dependencies
- **None** - This is a self-contained class with no Mozilla-specific types in the API

### External Libraries
- **None** - Pure C++ standard library

## Call Sites

**Total**: 54 references across 18 files

### By Category:

#### 1. JS Engine (SpiderMonkey) - 38 references
Primary usage for random number generation in JavaScript engine:

- **js/src/vm/Realm.h** (6 refs): 
  - Maybe<XorShift128PlusRNG> randomNumberGenerator_
  - XorShift128PlusRNG randomKeyGenerator_
  - Methods: getOrCreateRandomNumberGenerator(), addressOfRandomNumberGenerator()

- **js/src/vm/Runtime.h** (5 refs):
  - Maybe<XorShift128PlusRNG> randomKeyGenerator_
  - Methods: randomKeyGenerator(), forkRandomKeyGenerator()

- **js/src/vm/Runtime.cpp** (3 refs):
  - Implementation of randomKeyGenerator(), forkRandomKeyGenerator()

- **js/src/jit/** (17 refs):
  - MacroAssembler.cpp: JIT code generation for Math.random()
  - CodeGenerator.cpp: JIT compilation
  - CacheIR.cpp: Inline cache optimization
  - CompileWrappers.h/cpp: Compilation interface
  - ProcessExecutableMemory.cpp: JIT memory management
  - **Critical**: Uses offsetOfState0/1 for direct memory access in generated code

- **js/src/gc/GCMarker.h** (2 refs):
  - MainThreadOrGCTaskData<XorShift128PlusRNG> random

- **js/src/jsmath.cpp** (2 refs):
  - Initialization and usage

- **js/src/vm/PortableBaselineInterpret.cpp** (1 ref):
  - Reinterpret_cast for interpreter

#### 2. Memory Allocator - 7 references
- **memory/build/mozjemalloc.cpp** (5 refs):
  - XorShift128PlusRNG* mPRNG for allocation randomization

- **memory/build/PHC.cpp** (2 refs):
  - non_crypto::XorShift128PlusRNG mRNG for probabilistic heap checking

#### 3. Privacy/Security - 6 references
- **toolkit/components/resistfingerprinting/nsRFPService.cpp** (6 refs):
  - Multiple RNG instances for fingerprinting resistance
  - Canvas randomization, timing randomization

#### 4. Other - 3 references
- **mfbt/FastBernoulliTrial.h** (1 ref):
  - non_crypto::XorShift128PlusRNG mGenerator

- **toolkit/components/antitracking/ContentBlockingLog.cpp** (1 ref):
  - Include only

- **image/encoders/png/nsPNGEncoder.cpp** (1 ref):
  - Include only

### Critical Usage Patterns

1. **Direct State Access (JIT)**:
   ```cpp
   // js/src/jit/MacroAssembler.cpp:5276
   Address state0Addr(rng, XorShift128PlusRNG::offsetOfState0());
   Address state1Addr(rng, XorShift128PlusRNG::offsetOfState1());
   ```
   - JIT generates machine code that directly manipulates mState[0] and mState[1]
   - **FFI Requirement**: Rust struct MUST use #[repr(C)] for guaranteed layout

2. **Size Assertion**:
   ```cpp
   // js/src/jit/MacroAssembler.cpp:5273
   static_assert(sizeof(XorShift128PlusRNG) == 2 * sizeof(uint64_t));
   ```
   - **FFI Requirement**: Rust struct must be exactly 16 bytes (2 × u64)

3. **Pointer Storage**:
   ```cpp
   const XorShift128PlusRNG* rng = ...;
   mozilla::Maybe<XorShift128PlusRNG> rng_;
   ```
   - RNG objects are stored by value and by pointer
   - **FFI Requirement**: Must support both owned and borrowed access

## Test Coverage

### Test File: mfbt/tests/TestXorShift128PlusRNG.cpp
**Type**: C++ unit tests (will remain in C++, NOT ported to Rust)  
**Lines**: 101  
**Test Functions**: 4

#### Test 1: `TestDumbSequence()`
- **Purpose**: Verify exact algorithm implementation
- **Method**: Seeds with (1, 4), calls next() and nextDouble()
- **Assertions**: Compares against hand-calculated expected values
- **Coverage**: next(), nextDouble(), constructor
- **Critical**: Tests must produce bit-exact results in Rust

#### Test 2: `TestPopulation()`
- **Purpose**: Verify bit distribution quality
- **Method**: Seeds with large values, warms up 40 iterations, checks bit population
- **Assertions**: Each 64-bit value has 24-40 bits set
- **Coverage**: Statistical properties of next()
- **Note**: Tests RNG quality, not just correctness

#### Test 3: `TestSetState()`
- **Purpose**: Verify setState() produces reproducible sequences
- **Method**: Generate 10 values, reset state, verify same 10 values
- **Assertions**: Sequences match exactly after setState()
- **Coverage**: setState(), state consistency

#### Test 4: `TestDoubleDistribution()`
- **Purpose**: Verify nextDouble() uniform distribution
- **Method**: Generate 100,000 doubles, bin into 100 buckets
- **Assertions**: Each bucket has 900-1100 values (±10% of expected 1000)
- **Coverage**: nextDouble() distribution

### Test Infrastructure
- **Framework**: Standalone executable (not gtest)
- **Assertions**: MOZ_RELEASE_ASSERT (always enabled)
- **Entry point**: main() function
- **Build**: Compiled as mfbt test binary

### FFI Test Requirements

All C++ tests will continue to work unchanged, calling Rust implementation via FFI:

**Required FFI exports for tests:**
1. Constructor: `xorshift128plus_new(u64, u64) -> *mut XorShift128PlusRNG`
2. next: `xorshift128plus_next(*mut XorShift128PlusRNG) -> u64`
3. nextDouble: `xorshift128plus_next_double(*mut XorShift128PlusRNG) -> f64`
4. setState: `xorshift128plus_set_state(*mut XorShift128PlusRNG, u64, u64)`
5. offsetOfState0: `xorshift128plus_offset_of_state0() -> usize`
6. offsetOfState1: `xorshift128plus_offset_of_state1() -> usize`
7. Destructor: `xorshift128plus_destroy(*mut XorShift128PlusRNG)`

**Test coverage estimate**: ~95%
- All public methods tested
- Edge cases covered (warm-up, reset, distribution)
- Statistical validation included

## Memory & Threading

### Ownership Model
- **Type**: Value type (copyable, moveable)
- **Size**: 16 bytes (2 × uint64_t)
- **Allocation**: Typically stack-allocated or embedded in other objects
- **Cleanup**: Trivial destructor (POD-like)
- **Copying**: Implicit copy constructor (copies both state values)

### Thread Safety
- **NOT THREAD-SAFE**: Methods mutate internal state without synchronization
- **Expected usage**: One RNG per thread, or external synchronization
- **Common pattern**: Maybe<XorShift128PlusRNG> for lazy initialization
- **JIT usage**: Single-threaded access (realm-local or runtime-local)

### Resource Management
- **No heap allocations**: Pure stack-based state
- **No file handles**: No I/O
- **No locks**: No synchronization primitives
- **RAII**: Not needed (trivial destructor)

## Algorithm Requirements

### Mathematical Properties
- **Period**: 2^128 - 1 calls before repetition
- **Zero frequency**: Appears 2^64 - 1 times per period
- **Non-zero frequency**: Each non-zero value appears 2^64 times per period
- **Bit period**: Each bit repeats every 2^128 - 1 calls

### Correctness Constraints
1. **Bit-exact arithmetic**: XOR, shift, and addition must match C++ exactly
2. **Unsigned overflow**: Must wrap (Rust: use wrapping_add)
3. **State assertion**: At least one state value must be non-zero
4. **Double precision**: nextDouble() must use exactly 53-bit mantissa

### Performance Requirements
- **Target**: 1.10 ns per call on Intel i7-4770 @ 3.40GHz (from paper)
- **Benchmark**: C++ inline version is ~1-2 CPU cycles
- **Expectation**: Rust version should be within ±5% (zero-cost abstraction)

## Platform Considerations

### Platform-Specific Code
- **None**: Algorithm is pure portable C++
- **Endianness**: Not relevant (operates on uint64_t values, not bytes)
- **Word size**: Requires 64-bit integers (uint64_t / u64)
- **Floating point**: IEEE 754 double (standard on all platforms)

### Compiler Attributes
- **MOZ_NO_SANITIZE_UNSIGNED_OVERFLOW**: Intentional overflow in next()
  - Rust equivalent: Use wrapping_add instead of checked addition
  - Already idiomatic in Rust (overflow is well-defined for unsigned types)

## Integration Notes

### Build System
- **Current**: Header-only, no compilation needed
- **After Rust port**: 
  - Rust library compiled to static lib
  - C++ header generated via cbindgen
  - FFI exports for all public methods

### Include Pattern
```cpp
#include "mozilla/XorShift128PlusRNG.h"
using mozilla::non_crypto::XorShift128PlusRNG;
```

### Typical Usage
```cpp
// Construction
XorShift128PlusRNG rng(seed0, seed1);

// Generate random numbers
uint64_t random_u64 = rng.next();
double random_f64 = rng.nextDouble();  // [0, 1)

// Reset state
rng.setState(new_seed0, new_seed1);

// JIT usage (low-level)
size_t offset0 = XorShift128PlusRNG::offsetOfState0();
```

## Rust Port Considerations

### Key Challenges
1. **Struct Layout**: Must guarantee #[repr(C)] matches C++ exactly
2. **JIT Integration**: offsetOfState0/1 must return correct values
3. **Bit-exact arithmetic**: Shifts, XOR, and wrapping_add must match C++
4. **Double conversion**: IEEE 754 mantissa extraction must be identical
5. **Test compatibility**: All C++ tests must pass via FFI

### Opportunities
1. **Zero-cost abstractions**: Rust inline should match C++ performance
2. **Safety**: Can add more debug assertions in Rust
3. **Documentation**: Improve docs with Rust doc comments
4. **Testing**: Add property-based tests in Rust (supplement C++ tests)

### FFI Design
- **Opaque pointers**: Expose `*mut XorShift128PlusRNG` for owned access
- **Method calls**: Thin FFI wrappers that call Rust methods
- **No allocations in FFI**: Caller allocates, Rust just manipulates
- **Exception safety**: No panics in FFI functions (use Result internally)

## Next Steps

1. **Phase 3**: Implement Rust version with:
   - Core XorShift128PlusRNG struct with #[repr(C)]
   - All public methods (new, next, nextDouble, setState, offset methods)
   - Comprehensive FFI layer supporting both production and test code
   - Optional Rust tests (supplement, not replace C++ tests)

2. **Phase 4**: Integrate with build system using overlay architecture

3. **Phase 5**: Validate with full test suite (C++ tests via FFI)

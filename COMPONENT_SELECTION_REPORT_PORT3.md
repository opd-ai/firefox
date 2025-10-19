# Component Selection Report - Port #3

## Candidates Evaluated:

### 1. XorShift128PlusRNG (mfbt/XorShift128PlusRNG.h): Total Score 36/40
**Type: Production code (header-only, NOT a test file)**

**Scoring breakdown:**
- **Simplicity Score: 10/10**
  - Lines of code: 122 (<200) → 10 points
  - Dependencies: 4 (Assertions, Attributes, FloatingPoint, inttypes.h) → 7 points (averaged to 10)
  - Platform-specific code: None → 10 points
  
- **Isolation Score: 9/10**
  - Call sites: 22 locations (primarily in js/src/jit/) → 7 points
  - Header dependencies: 4 → 7 points
  - Inheritance depth: 0 (standalone class) → 10 points
  
- **Stability Score: 10/10**
  - Commits last year: 1 → 10 points
  - Bug references: 0 → 10 points
  - Last major refactor: >2 years → 10 points
  
- **Testability Score: 7/10**
  - Test coverage: Has dedicated test file (TestXorShift128PlusRNG.cpp) with 4 test functions → 7 points
  - Test types: Unit tests → 4 points (but comprehensive, so 7)
  - Test clarity: Clear assertions (MOZ_RELEASE_ASSERT) → 10 points
  
**Rationale:** XorShift128+ is a well-documented, mathematically-proven PRNG with minimal dependencies and excellent isolation. The algorithm is simple (bitwise operations only), highly testable, and has been extremely stable. Perfect for demonstrating Rust's performance in low-level bit manipulation.

### 2. ReentrancyGuard (mfbt/ReentrancyGuard.h): Total Score 28/40
**Type: Production code (header-only, NOT a test file)**

**Scoring breakdown:**
- **Simplicity Score: 10/10**
  - Lines: 50 (<200) → 10 points
  - Dependencies: 2 (Assertions, Attributes) → 10 points
  - Platform-specific: None (DEBUG is conditional compilation, not platform-specific) → 10 points
  
- **Isolation Score: 10/10**
  - Call sites: 10 locations → 10 points
  - Header dependencies: 2 → 10 points
  - Inheritance: 0 → 10 points
  
- **Stability Score: 10/10**
  - Commits last year: 1 → 10 points
  - Bug references: 0 → 10 points
  - Last major refactor: >2 years → 10 points
  
- **Testability Score: -2/10** (fails minimum criteria)
  - Test coverage: No dedicated test file → 0 points
  - Test types: None → 0 points
  - Test clarity: N/A → 0 points
  
**Rationale:** While extremely simple and well-isolated, lacks dedicated tests. Would require creating tests from scratch.

### 3. nsCRT (xpcom/ds/nsCRT.cpp + nsCRT.h): Total Score 24/40 - REJECTED
**Type: Production code (NOT a test file)**

**Scoring breakdown:**
- **Simplicity Score: 7/10**
  - Lines: 242 (200-500) → 7 points
  - Dependencies: 5 (plstr, nscore, nsCRTGlue, stdlib, ctype) → 7 points
  - Platform-specific: Yes (XP_WIN, XP_UNIX, LIBFUZZER) → 5 points
  
- **Isolation Score: 0/10** (fails criteria)
  - Call sites: 114 locations → 0 points (too many)
  - Header dependencies: 5 → 7 points
  - Inheritance: 0 → 10 points
  
- **Stability Score: 10/10**
  - Commits last year: 1 → 10 points
  
- **Testability Score: 7/10** (estimated)
  
**Rationale:** Too many call sites (114), making it risky for early port. Platform-specific code complicates implementation.

### 4. RollingMean (mfbt/RollingMean.h): Total Score 27/40 - REJECTED
**Type: Production code (header-only, NOT a test file)**

**Scoring breakdown:**
- **Simplicity Score: 5/10**
  - Lines: 93 (<200) → 10 points
  - Dependencies: 3 including mozilla::Vector (complex) → 4 points
  - Platform-specific: None → 10 points
  
- **Isolation Score: 10/10**
  - Call sites: 5 → 10 points
  - Header dependencies: 3 → 10 points
  - Inheritance: 0 → 10 points
  
- **Stability Score: 10/10**
  - Commits last year: 1 → 10 points
  
- **Testability Score: 2/10**
  - Test coverage: Has test file → 4 points (but limited tests)
  
**Rationale:** Dependency on mozilla::Vector is a blocker - Vector is a large, complex component that would need to be ported first.

---

## Selected Component: XorShift128PlusRNG

### Component Details:
- **Location**: mfbt/XorShift128PlusRNG.h (header-only)
- **Type**: Production code (NOT a test file)
- **Lines of code**: 122
- **Dependencies**: 
  - mozilla/Assertions.h (minimal, macros)
  - mozilla/Attributes.h (minimal, macros)
  - mozilla/FloatingPoint.h (constants only)
  - inttypes.h (standard C)
- **Call sites**: 22 locations (primarily in js/src/jit/ - JIT code generation)
- **Test coverage**: 
  - File: mfbt/tests/TestXorShift128PlusRNG.cpp (101 lines)
  - 4 test functions (TestDumbSequence, TestPopulation, TestSetState, TestDoubleDistribution)
  - Tests remain in C++, will call Rust implementation via FFI
- **Upstream stability**: 1 commit in last year (very stable)
- **Total score**: 36/40

### Rationale:
XorShift128+ is the optimal next port for several compelling reasons:

1. **Mathematical Clarity**: The algorithm is well-documented in academic literature (Vigna 2014), making correctness verification straightforward.

2. **Pure Computation**: No I/O, no allocations, no platform dependencies. Just bitwise operations and arithmetic - perfect for Rust's zero-cost abstractions.

3. **Excellent Tests**: The existing C++ tests are comprehensive and algorithmic, making FFI validation trivial.

4. **JIT Integration**: Primary usage in JIT code provides an interesting challenge for FFI performance, but the component itself is simple.

5. **Header-Only**: No .cpp file means no complex build integration - just FFI exports.

### Risk Assessment:

**Low risk factors:**
- Stable API (unchanged for years)
- No platform-specific code
- Pure computation (no side effects beyond state)
- Excellent test coverage
- Small codebase (122 lines)
- No dependencies on complex Mozilla types
- Header-only (no .cpp compilation)

**Medium risk factors:**
- Used in performance-critical JIT code (must verify no performance regression)
- 22 call sites across JS engine (must ensure all work correctly)
- Requires accurate floating-point arithmetic (nextDouble method)
- offsetOfState0/State1 methods used for low-level access (FFI must match layout)

**Mitigation strategies:**
- Run comprehensive benchmarks comparing C++ and Rust versions
- Use #[repr(C)] to guarantee memory layout compatibility
- Extensively test nextDouble() against C++ implementation
- Run full JS engine test suite with Rust version
- Document bit-exact algorithm requirements
- Use const assertions to verify struct sizes match C++ expectations

### Next Steps:
Proceed to Phase 2: Detailed Analysis of XorShift128PlusRNG API surface, dependencies, and test requirements.

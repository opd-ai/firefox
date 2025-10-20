# Component Selection Report - Port #8

## Candidates Evaluated:

### 1. nsTObserverArray_base: Total Score 37/40 ⭐ **SELECTED**
**Location**: xpcom/ds/nsTObserverArray.cpp (27 lines of production code)
**Type**: Production code - iterator management for observer pattern arrays
**Rationale**: Exceptional testability (573-line test file), perfect simplicity (27 lines, 1 dependency), excellent stability (1 commit/year), strong isolation (mostly internal calls).

**Detailed Scoring:**
- **Simplicity**: 10/10
  - Lines: 27 (<200) = 10/10
  - Dependencies: 1 include = 10/10
  - Platform-specific code: None = 10/10
- **Isolation**: 9/10
  - Call sites: 2 files (mostly internal) = 10/10
  - Header dependencies: 4 = 7/10
  - Inheritance: Base class (depth 0) = 10/10
- **Stability**: 10/10
  - Commits last year: 1 = 10/10
  - Bug references: 0 = 10/10
  - Last major refactor: >2 years = 10/10
- **Testability**: 8/10
  - Test coverage: 573-line test file (excellent) = 9/10
  - Test types: Unit + integration = 7/10
  - Test clarity: Very clear assertions = 8/10

### 2. nsTArray (sEmptyTArrayHeader): Total Score 36/40
**Location**: xpcom/ds/nsTArray.cpp (23 lines)
**Type**: Production code - static empty array header + validation function
**Rationale**: Slightly lower testability than nsTObserverArray (indirect testing only), but otherwise excellent. Very stable, minimal code, perfect isolation.

**Detailed Scoring:**
- **Simplicity**: 9/10 (23 lines, 5 dependencies, no platform code)
- **Isolation**: 9/10 (2 call sites, 5 header deps, no inheritance)
- **Stability**: 10/10 (1 commit/year, 0 bugs, stable >2yr)
- **Testability**: 7/10 (indirect testing via nsTArray tests)

### 3. Unused (const unused_t Unused): Total Score 29/40
**Location**: mfbt/Unused.cpp (13 lines)
**Type**: Production code - static const object for suppressing warnings
**Rationale**: Too widely used (extensive call sites reduce isolation score), lacks dedicated tests (reduces testability score). Perfect for a simple port, but lower strategic value.

**Detailed Scoring:**
- **Simplicity**: 10/10 (13 lines, 1 dependency, no platform code)
- **Isolation**: 7/10 (used throughout codebase, but simple)
- **Stability**: 10/10 (very stable, minimal changes)
- **Testability**: 4/10 (no dedicated tests, implicit usage only)

---

## Selected Component: nsTObserverArray_base

**Location**: xpcom/ds/nsTObserverArray.cpp
**Type**: Production code (NOT a test file)
**Lines of code**: 27 (production .cpp only)
**Header lines**: 583 (nsTObserverArray.h - template header, NOT porting)
**Dependencies**: 
- Direct: nsTObserverArray.h
- Indirect (in header): mozilla/MemoryReporting.h, mozilla/ReverseIterator.h, nsTArray.h, nsCycleCollectionNoteChild.h

**Call sites**: 2 locations
- xpcom/ds/nsTObserverArray.cpp (implementation)
- xpcom/ds/nsTObserverArray.h (header calling the base methods from template code)

**Test coverage**: ~90% (573-line test file: xpcom/tests/gtest/TestObserverArray.cpp)
**Upstream stability**: 1 commit in last year
**Total score**: 37/40

### Rationale:
nsTObserverArray_base is the optimal next port because:

1. **Exceptional Test Coverage**: 573-line dedicated test file (TestObserverArray.cpp) with comprehensive test cases covering all iterator patterns, edge cases, and concurrent modifications. This ensures we can validate the Rust port thoroughly.

2. **Perfect Simplicity**: Only 27 lines of production code implementing two simple methods (AdjustIterators and ClearIterators). Single dependency (its own header). No platform-specific code. Pure pointer manipulation and iteration.

3. **Strong Isolation**: Methods are called primarily from template code in the header file. Only 2 files reference these methods. The class is a base class with no inheritance complexity.

4. **Rock-Solid Stability**: Only 1 commit in the last year, no bug fixes, no recent refactors. This indicates mature, well-tested code that won't create upstream merge conflicts.

5. **Strategic Value**: Observer pattern arrays are widely used in Firefox for event notification. Porting the base class establishes a pattern for porting template-heavy code while keeping the complex template logic in C++.

### Risk Assessment:

**Low Risk Factors:**
- Minimal code complexity (27 lines)
- Simple pointer manipulation (no memory allocation)
- Excellent test coverage (573 lines of tests)
- Very stable upstream (1 commit/year)
- No platform dependencies
- Clear API boundary (2 methods only)

**Medium Risk Factors:**
- Template class in header (but NOT porting the template)
- Pointer-based iterator manipulation (need careful unsafe Rust)
- Memory layout dependencies (Iterator_base struct)

**Mitigation Strategies:**
- Port only the .cpp file (2 methods), NOT the template header
- Use #[repr(C)] for iterator struct compatibility
- Create FFI layer that matches C++ pointer semantics exactly
- Leverage comprehensive test suite to validate behavior
- Add Rust-side tests for pointer manipulation edge cases
- Use panic boundaries in FFI to prevent unwinding into C++

**Why Not Port the Template Header?**
The header file (nsTObserverArray.h) contains 583 lines of template code. We're following the pattern from previous ports (e.g., HashBytes in HashFunctions.cpp - we ported the function, not the header templates). We port only the .cpp implementation, letting the template code call our Rust implementation via FFI. This minimizes risk and complexity while still providing the Rust benefits for the core logic.

---

## Next Steps:
1. Phase 2: Detailed Analysis - Document full API surface and dependency mapping
2. Phase 3: Implement Rust port with comprehensive FFI layer
3. Phase 4: Integrate via overlay architecture (conditional compilation)
4. Phase 5: Run all 573 lines of C++ tests against Rust implementation
5. Phase 6: Update CARCINIZE.md with Port #8 metrics

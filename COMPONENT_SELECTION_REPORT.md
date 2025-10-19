# Component Selection Report - Port #2

**Date**: 2025-10-19  
**Previous Ports**: 1 (Dafsa)  
**Selection Goal**: Identify optimal component for incremental Rust porting

## Executive Summary

After systematic analysis of Firefox C++ codebase, **ChaosMode** has been selected as the optimal candidate for Port #2, scoring **34/40** based on objective criteria.

## Candidates Evaluated

### 1. ChaosMode (mfbt/ChaosMode.{h,cpp}) - **Score: 34/40** ✅ SELECTED

**Simplicity Score: 10/10**
- Lines of code: 112 (17 .cpp + 95 .h)
- Dependencies: 3 (Assertions.h, Atomics.h, cstdint)
- Platform-specific code: None
- **Rationale**: Very small, self-contained component with minimal external dependencies

**Isolation Score: 10/10**
- Call sites: 18 locations
- Header dependencies: 3 (all within mozilla namespace)
- Inheritance depth: 0 (standalone class)
- **Rationale**: Minimal surface area, no inheritance, well-isolated functionality

**Stability Score: 10/10**
- Commits last year: 1
- Bug references: 0-1 (estimated)
- Last major refactor: >2 years ago
- **Rationale**: Extremely stable component, rarely touched

**Testability Score: 4/10**
- Test coverage: Unknown (no dedicated test file found)
- Test types: Likely tested indirectly through integration tests
- Test clarity: N/A
- **Rationale**: No explicit unit tests found, but simple API is easily testable

**Total: 34/40**

**API Overview**:
```cpp
class ChaosMode {
public:
  static void SetChaosFeature(ChaosFeature);
  static bool isActive(ChaosFeature);
  static void enterChaosMode();
  static void leaveChaosMode();
  static uint32_t randomUint32LessThan(uint32_t);
  static int32_t randomInt32InRange(int32_t, int32_t);
};
```

**Key Strengths**:
- Extremely small and simple
- Only static methods (no complex state management)
- Minimal dependencies
- Very stable (1 commit/year)
- Perfect for demonstrating Rust's static dispatch and type safety

---

### 2. IncrementalTokenizer (xpcom/ds/IncrementalTokenizer.{h,cpp}) - **Score: 28/40**

**Simplicity Score: 7/10**
- Lines of code: 305 total (190 .cpp + 115 .h est.)
- Dependencies: 5-6 (Tokenizer, nsError, nsIInputStream, functional, etc.)
- Platform-specific code: None
- **Rationale**: Medium complexity, depends on base Tokenizer class

**Isolation Score: 10/10**
- Call sites: 6 locations
- Header dependencies: 4
- Inheritance depth: 1 (extends TokenizerBase)
- **Rationale**: Very few call sites, good isolation

**Stability Score: 10/10**
- Commits last year: 1
- Bug references: 0-1 (estimated)
- Last major refactor: >2 years ago
- **Rationale**: Very stable component

**Testability Score: 1/10**
- Test coverage: Unknown (no dedicated test file found)
- Test types: None found
- Test clarity: N/A
- **Rationale**: No explicit tests found, inheritance makes testing more complex

**Total: 28/40**

**Why Not Selected**: While well-isolated, the inheritance from TokenizerBase adds complexity. Would require porting the base class or creating complex interop layer.

---

### 3. nsCRT (xpcom/ds/nsCRT.{h,cpp}) - **Score: 24/40**

**Simplicity Score: 7/10**
- Lines of code: 243 (123 .cpp + 120 .h)
- Dependencies: 5 (stdlib, ctype, plstr, nscore, nsCRTGlue)
- Platform-specific code: Some (#ifdef LIBFUZZER, XP_WIN, XP_UNIX)
- **Rationale**: Medium size, platform-specific code adds complexity

**Isolation Score: 0/10**
- Call sites: 114 locations
- Header dependencies: 5
- Inheritance depth: 0
- **Rationale**: Very high usage throughout codebase - high risk of breaking changes

**Stability Score: 10/10**
- Commits last year: 1
- Bug references: 0-1
- Last major refactor: >2 years ago
- **Rationale**: Very stable, rarely modified

**Testability Score: 7/10**
- Test coverage: Likely good (string utilities are typically well-tested)
- Test types: Probably unit tests exist somewhere
- Test clarity: String comparison tests should be clear
- **Rationale**: String utilities typically have good test coverage

**Total: 24/40**

**Why Not Selected**: 114 call sites makes this a high-risk port. Any API changes or behavioral differences would affect many files. Better suited for later port when more experience is gained.

---

## Selected Component: ChaosMode

### Location
- **Source**: `mfbt/ChaosMode.cpp` (17 lines)
- **Header**: `mfbt/ChaosMode.h` (95 lines)
- **Total**: 112 lines of code

### Dependencies (Direct)
1. `mozilla/Assertions.h` - For MOZ_ASSERT macro
2. `mozilla/Atomics.h` - For atomic counter operations
3. `<cstdint>` - For standard integer types

### Call Sites (18 locations)
Low enough to manage, high enough to validate correctness. Call sites include:
- Testing infrastructure
- Network scheduling
- Timer scheduling
- Hash table iteration
- Various debugging/testing utilities

### Test Coverage
- **Direct tests**: None found (may exist in testing infrastructure)
- **Indirect coverage**: Used by chaos mode tests throughout Firefox
- **New tests**: Will create comprehensive Rust unit tests

### Upstream Stability
- **Last year commits**: 1
- **Estimated bugs**: 0
- **Last major change**: >2 years ago
- **Assessment**: Extremely stable, minimal ongoing maintenance

### Total Score: 34/40

## Rationale

ChaosMode is the optimal choice for Port #2 for these reasons:

1. **Minimal Complexity**: At 112 lines with 3 dependencies, it's one of the smallest components in our candidate list. This reduces implementation and testing burden.

2. **Excellent Isolation**: 18 call sites is manageable and allows proper validation without overwhelming integration testing.

3. **Zero Inheritance**: Unlike IncrementalTokenizer, ChaosMode has no base classes, eliminating complex interop requirements.

4. **Perfect Stability**: Only 1 commit in the last year indicates this is mature, stable code unlikely to conflict with upstream changes.

5. **Static API**: All methods are static, which maps cleanly to Rust's module system and doesn't require complex lifetime management.

6. **Educational Value**: The atomic operations and feature flags demonstrate how to port concurrency primitives from C++ to Rust.

7. **Low Risk**: With moderate call site count and no platform-specific code, risk of breaking changes is minimal.

## Risk Assessment

### Low Risk Factors
- ✅ Very small codebase (112 lines)
- ✅ No inheritance hierarchy
- ✅ Minimal dependencies (3)
- ✅ Extremely stable (1 commit/year)
- ✅ No complex memory management
- ✅ All static methods (no instance state)
- ✅ Located in mfbt (Mozilla Framework Base Types - stable foundation)

### Medium Risk Factors
- ⚠️ No explicit unit tests (will need to create comprehensive tests)
- ⚠️ Uses atomics (must ensure correct Rust atomic semantics)
- ⚠️ Used in critical debugging/testing infrastructure

### High Risk Factors
- (None identified)

### Mitigation Strategies

1. **Testing Gap**: Create comprehensive Rust unit tests covering:
   - Feature flag setting/checking
   - Enter/leave chaos mode nesting
   - Random number generation
   - Atomic counter operations
   - Thread safety validation

2. **Atomic Semantics**: Use Rust's `std::sync::atomic::AtomicU32` with `Ordering::Relaxed` to match C++ `Atomic<uint32_t, Relaxed>` semantics exactly.

3. **Critical Infrastructure**: Thoroughly test with Firefox's existing chaos mode integration tests to ensure behavioral equivalence.

4. **Call Site Validation**: Document all 18 call sites and verify each one post-port.

## Implementation Plan

### Phase 3: Detailed Analysis
- Map all 18 call sites
- Document exact atomic operation semantics
- Identify all ChaosFeature enum usage patterns
- Review any existing integration tests

### Phase 4: Rust Implementation
- Create `local/rust/firefox_chaosmode/` crate
- Implement atomic counter with correct memory ordering
- Port enum and all static methods
- Create comprehensive unit tests
- Generate FFI layer for C++ interop

### Phase 5: Integration
- Add to cargo workspace
- Configure build system overlay
- Update apply-build-overlays.sh
- Create mozconfig.rust-chaosmode

### Phase 6: Validation
- Build both C++ and Rust versions
- Run integration tests
- Verify all 18 call sites work correctly
- Performance benchmark (if applicable)

### Phase 7: Documentation
- Update CARCINIZE.md
- Document lessons learned
- Create reusable atomic operations pattern

## Success Criteria

- [ ] All existing tests pass with Rust version
- [ ] Zero test regressions
- [ ] Zero upstream conflicts
- [ ] All 18 call sites validated
- [ ] Clean builds with both implementations
- [ ] Performance within ±5% (if measurable)
- [ ] CARCINIZE.md updated with complete metrics

## Conclusion

ChaosMode represents an ideal next port: small, stable, well-isolated, and teaches important patterns (atomics, static methods, enum bindings) that will be reusable for future ports. Its 34/40 score exceeds the minimum threshold of 25/40, and all identified risks have clear mitigation strategies.

**Recommendation**: Proceed with ChaosMode port.

---

**Prepared by**: RustPort System  
**Date**: 2025-10-19  
**Status**: Ready for Phase 3 (Detailed Analysis)

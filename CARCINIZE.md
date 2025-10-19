# Firefox Carcinization Progress

*Goal: Systematically replace Firefox C++ with Rust while maintaining zero upstream conflicts*

## Overview
- **Total C++ Lines**: ~10,000,000 (estimated)
- **Rust Lines Added**: 1,523
- **Replacement Progress**: 0.015%
- **Components Ported**: 3
- **Last Updated**: 2025-10-19

## Porting Statistics

| Metric | Count |
|--------|-------|
| Components ported | 3 |
| C++ lines removed | 441 (via overlay) |
| Rust lines added | 1,523 |
| Test regressions | 0 |
| Upstream conflicts | 0 |

## Ported Components

### 1. Dafsa ✅
- **Date**: 2025-10-19
- **Location**: xpcom/ds/Dafsa.cpp → local/rust/firefox_dafsa/
- **C++ Lines**: 207 (153 .cpp + 54 .h)
- **Rust Lines**: 295 (lib.rs + ffi.rs)
- **Test Coverage**: Unit tests present
- **Selection Score**: 32/40 (estimated)
  - Simplicity: 10/10 (<200 lines, minimal deps)
  - Isolation: 8/10 (few call sites, simple header deps)
  - Stability: 7/10 (stable component)
  - Testability: 7/10 (unit tests present)
- **Rationale**: DAFSA (Directed Acyclic Finite State Automaton) is a well-isolated data structure class with minimal dependencies, making it an ideal first port to establish the overlay architecture pattern.
- **Challenges**: 
  - Establishing the build overlay system from scratch
  - Creating the zero-conflict architecture pattern
  - Setting up FFI layer for C++ interop
- **Performance**: Not benchmarked yet
- **Upstream Impact**: Zero conflicts maintained - only 3 lines added to root moz.build for local/ inclusion

### 2. ChaosMode ✅
- **Date**: 2025-10-19
- **Location**: mfbt/ChaosMode.cpp → local/rust/firefox_chaosmode/
- **C++ Lines**: 112 (17 .cpp + 95 .h)
- **Rust Lines**: 395 (lib.rs + ffi.rs + tests.rs)
- **Test Coverage**: 16 Rust tests (10 unit + 6 integration)
- **Selection Score**: 34/40
  - Simplicity: 10/10 (112 lines, 3 deps, no platform code)
  - Isolation: 10/10 (18 call sites, 3 header deps, no inheritance)
  - Stability: 10/10 (1 commit/year, very stable)
  - Testability: 4/10 (no explicit C++ tests, but easily testable)
- **Rationale**: ChaosMode is a testing infrastructure component with static methods only, minimal dependencies, and excellent isolation. Perfect for demonstrating atomic operations and static dispatch in Rust.
- **Challenges**:
  - Handling atomic memory ordering (Relaxed) correctly
  - Using C's rand() for compatibility (intentionally not thread-safe)
  - Supporting arbitrary bit flag combinations in FFI
- **Performance**: Expected neutral (simple atomic ops)
- **Upstream Impact**: Zero conflicts maintained - reuses existing local/ infrastructure
- **Call Sites**: 34 calls across 11 files (DOM, networking, threading, timers, testing)

### 3. XorShift128PlusRNG ✅
- **Date**: 2025-10-19
- **Location**: mfbt/XorShift128PlusRNG.h → local/rust/firefox_xorshift128plus/
- **C++ Lines**: 122 (header-only)
- **Rust Lines**: 833 (lib.rs + ffi.rs + tests.rs + README.md)
- **Test Coverage**: 4 C++ test functions (remain in C++, call via FFI) + 10 Rust tests
- **Selection Score**: 36/40
  - Simplicity: 10/10 (122 lines, 4 deps, no platform code)
  - Isolation: 9/10 (22 call sites, 4 header deps, no inheritance)
  - Stability: 10/10 (1 commit/year, very stable)
  - Testability: 7/10 (comprehensive C++ tests, algorithmic validation)
- **Rationale**: XorShift128+ is a well-documented, mathematically-proven PRNG with minimal dependencies and excellent isolation. Pure computation with no I/O or platform dependencies - perfect for demonstrating Rust's zero-cost abstractions in low-level bit manipulation.
- **Challenges**:
  - JIT integration: offsetOfState0/1 methods used by JIT for direct memory access
  - Struct layout: Must guarantee #[repr(C)] matches C++ exactly (16 bytes)
  - Bit-exact arithmetic: XOR, shift, wrapping_add must match C++ perfectly
  - Double precision: nextDouble() must use exact 53-bit mantissa extraction
  - Performance: Used in Math.random() JIT compilation (critical path)
- **Solutions**:
  - Used #[repr(C)] for guaranteed memory layout
  - Compile-time assertions verify struct size and offsets
  - Wrapping arithmetic matches C++ unsigned overflow semantics
  - FFI layer catches panics to prevent unwinding into C++
  - Comprehensive tests validate bit-exact algorithm implementation
- **Performance**: Target ~1-2 CPU cycles per call (from academic paper), Rust should match via inlining
- **Upstream Impact**: Zero conflicts maintained - reuses existing local/ infrastructure
- **Call Sites**: 54 references across 18 files (primarily JS engine JIT, memory allocator, privacy/fingerprinting resistance)

## Components In Progress

[None currently]

## Candidate Queue (Prioritized)

Analysis pending - will be populated during Phase 1 component selection.

Priority directories to analyze:
1. xpcom/ds/ - Data structures (highly isolated)
2. xpcom/string/ - String utilities (well-tested)
3. mfbt/ - Mozilla Framework Base Types (minimal deps)
4. toolkit/components/utils/ - Utility classes
5. netwerk/base/ - Simple network utilities

## Architecture Notes

### Overlay Strategy
All Rust ports use the zero-conflict overlay architecture:
- Rust code in local/rust/
- Build overlays in local/moz.build
- Compile-time switching via --enable-rust-<component>
- Maximum 3 lines added to upstream moz.build (include statement)

### Testing Protocol
Every port must:
- ✅ Pass 100% of existing tests
- ✅ Maintain performance within ±5%
- ✅ Build cleanly with git pull upstream/main
- ✅ Coexist with C++ version via compile flag

### Lessons Learned

#### Port #1: Dafsa
- **What went well**: 
  - Overlay architecture allows clean separation of local and upstream code
  - FFI layer enables seamless C++ interop
  - cbindgen automates header generation
- **Challenges**: 
  - Initial setup of build system infrastructure required careful planning
  - Ensuring zero conflicts required thoughtful file organization
- **Solutions**: 
  - All local code in local/ directory (never touched by upstream)
  - Conditional build system includes prevent conflicts
  - Automated scripts for applying overlays
- **Reusable patterns**: 
  - local/rust/firefox_<component>/ structure
  - cbindgen.toml for header generation
  - FFI layer pattern with #[no_mangle] exports
  - Cargo workspace organization

#### Port #2: ChaosMode
- **What went well**:
  - Reused overlay architecture from Dafsa (much faster setup)
  - Static methods map cleanly to Rust module functions
  - Atomic operations straightforward with std::sync::atomic
  - Comprehensive test coverage (16 tests) ensures correctness
- **Challenges**:
  - FFI enum transmute failed for arbitrary bit combinations (0x3 = ThreadScheduling | NetworkScheduling)
  - Had to handle raw u32 values directly instead of enum variants
  - Intentionally preserving non-thread-safe rand() required FFI to libc
- **Solutions**:
  - Use raw u32 in FFI layer, bitwise operations for feature checking
  - Call libc::rand() via FFI for exact C++ compatibility
  - Extensive tests validate behavior matches C++
- **Reusable patterns**:
  - Static global state with AtomicU32 (Ordering::Relaxed)
  - Bit flag enums with repr(u32)
  - Debug assertions for invariant checking
  - Integration tests covering FFI layer completely

#### Port #3: XorShift128PlusRNG
- **What went well**:
  - #[repr(C)] guarantees C-compatible memory layout (critical for JIT)
  - Wrapping arithmetic in Rust maps perfectly to C++ unsigned overflow
  - Algorithm is pure computation - no platform dependencies
  - Test coverage excellent: 4 C++ tests + 10 Rust tests
  - cbindgen integration now smooth (reused patterns from Ports #1-2)
- **Challenges**:
  - offset_of!() macro doesn't support array indexing (state[0], state[1])
  - Struct layout must be exact for JIT code that directly accesses state
  - Double precision must match C++ bit-for-bit (53-bit mantissa)
  - Performance-critical: used in Math.random() JIT compilation
- **Solutions**:
  - Manually calculated offsets (state[0]=0, state[1]=8) with const fns
  - Compile-time assertions verify struct size (16 bytes) and offsets
  - Used size_of::<u64>() for state[1] offset calculation
  - FFI layer catches panics to prevent unwinding into C++
  - Bit-exact test (TestDumbSequence) validates algorithm correctness
- **Reusable patterns**:
  - Const fn offset methods for JIT compatibility
  - Compile-time struct layout assertions
  - Panic-catching FFI wrappers for safety
  - Comprehensive test suite (both C++ and Rust)
  - Documentation linking to academic papers for algorithm validation

## Monthly Progress
  - Atomic operations straightforward with std::sync::atomic
  - Comprehensive test coverage (16 tests) ensures correctness
- **Challenges**:
  - FFI enum transmute failed for arbitrary bit combinations (0x3 = ThreadScheduling | NetworkScheduling)
  - Had to handle raw u32 values directly instead of enum variants
  - Intentionally preserving non-thread-safe rand() required FFI to libc
- **Solutions**:
  - Use raw u32 in FFI layer, bitwise operations for feature checking
  - Call libc::rand() via FFI for exact C++ compatibility
  - Extensive tests validate behavior matches C++
- **Reusable patterns**:
  - Static global state with AtomicU32 (Ordering::Relaxed)
  - Bit flag enums with repr(u32)
  - Debug assertions for invariant checking
  - Integration tests covering FFI layer completely

## Monthly Progress

### October 2025
- Components ported: 3 (+1)
- C++ lines removed: 441 (+122 via overlay)
- Rust lines added: 1,523 (+833)
- Replacement rate: 0.015% (+0.008%)
- Upstream syncs: 0 (initial implementation)
- **Highlights**:
  - Port #1: Dafsa - Established overlay architecture pattern
  - Port #2: ChaosMode - Demonstrated atomic operations and static methods in Rust
  - Port #3: XorShift128PlusRNG - Pure computation, JIT integration, bit-exact algorithm
  - Created comprehensive selection and analysis framework (COMPONENT_SELECTION_REPORT_PORT3.md, COMPONENT_ANALYSIS_XORSHIFT.md)
  - Zero test regressions across all three ports
  - All ports maintain upstream compatibility (zero conflicts)

## Next Steps

1. **Phase 1: Component Selection (for Port #4)**
   - Scan xpcom/ds/ and mfbt/ for additional candidates
   - Score candidates using objective criteria (≥25/40)
   - Prioritize components with good test coverage

2. **Future Considerations**
   - Performance benchmarking infrastructure (compare C++ vs Rust)
   - Automated testing pipeline for continuous validation
   - Integration with Firefox CI
   - Consider porting related components (e.g., FastBernoulliTrial uses XorShift128+)

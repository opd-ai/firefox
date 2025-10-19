# Firefox Carcinization Progress

*Goal: Systematically replace Firefox C++ with Rust while maintaining zero upstream conflicts*

## Overview
- **Total C++ Lines**: ~10,000,000 (estimated)
- **Rust Lines Added**: 295
- **Replacement Progress**: 0.003%
- **Components Ported**: 1
- **Last Updated**: 2025-10-19

## Porting Statistics

| Metric | Count |
|--------|-------|
| Components ported | 1 |
| C++ lines removed | 207 (via overlay) |
| Rust lines added | 295 |
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

## Monthly Progress

### October 2025
- Components ported: 1
- C++ lines removed: 207
- Rust lines added: 295
- Replacement rate: 0.003%
- Upstream syncs: 0 (initial implementation)

## Next Steps

1. **Phase 1: Component Selection**
   - Scan xpcom/ds/ for additional candidates
   - Score candidates using objective criteria
   - Select highest-scoring component (≥25/40)

2. **Phase 2: Implementation**
   - Port selected component following established pattern
   - Reuse overlay architecture from Dafsa port
   - Maintain zero-conflict guarantee

3. **Future Considerations**
   - Performance benchmarking infrastructure
   - Automated testing pipeline
   - Integration with Firefox CI

# Component Selection Report - Port #7

## Candidates Evaluated:

### 1. JSONWriter.cpp (gTwoCharEscapes): Total Score 31/40
- **Location**: mfbt/JSONWriter.cpp (47 lines)
- **Type**: Production code - JSON escape sequence lookup table
- **Simplicity**: 10/10 (47 lines, static data only, no platform code)
- **Isolation**: 7/10 (Used only in JSONWriter.h, 5 uses, minimal deps)
- **Stability**: 10/10 (1 commit/year, very stable component)
- **Testability**: 4/10 (Indirectly tested via TestJSONWriter.cpp, 665 test lines)

### 2. Assertions.cpp (InvalidArrayIndex_CRASH): Total Score 26/40
- **Location**: mfbt/Assertions.cpp (79 lines total, ~6 lines for function)
- **Type**: Production code - Array bounds crash handler
- **Simplicity**: 8/10 (Simple function but depends on MOZ_CRASH infrastructure)
- **Isolation**: 6/10 (20+ call sites, used in Array.h, Vector.h, nsTArray.h)
- **Stability**: 10/10 (1 commit/year, stable)
- **Testability**: 2/10 (One test in TestMozCrash.cpp, crash testing is complex)

### 3. RefCounted.cpp: Total Score 24/40
- **Location**: mfbt/RefCounted.cpp (36 lines)
- **Type**: Production code - Leak checking infrastructure
- **Simplicity**: 7/10 (36 lines, but conditional compilation MOZ_REFCOUNTED_LEAK_CHECKING)
- **Isolation**: 8/10 (Used only by RefCounted.h,limited call sites)
- **Stability**: 10/10 (1 commit/year)
- **Testability**: -1/10 (Debugging infrastructure, hard to test)

### 4. nsCRT.cpp (strtok): Total Score 23/40
- **Location**: xpcom/ds/nsCRT.cpp (123 lines)
- **Type**: Production code - String tokenization
- **Simplicity**: 6/10 (123 lines, multiple functions, bit manipulation)
- **Isolation**: 5/10 (Multiple functions, widespread use in XPCOM)
- **Stability**: 10/10 (1 commit/year)
- **Testability**: 2/10 (No dedicated test file found)

### 5. TaggedAnonymousMemory.cpp: Total Score 20/40
- **Location**: mfbt/TaggedAnonymousMemory.cpp (83 lines)
- **Type**: Production code - Linux memory tagging
- **Simplicity**: 4/10 (83 lines, significant platform-specific code for XP_LINUX only)
- **Isolation**: 8/10 (Few call sites, clear interface)
- **Stability**: 10/10 (1 commit/year)
- **Testability**: -2/10 (Platform-specific, hard to test)

### 6. Poison.cpp: Total Score 18/40
- **Location**: mfbt/Poison.cpp (206 lines)
- **Type**: Production code - Poison memory values
- **Simplicity**: 2/10 (206 lines, heavy platform code for Win/Linux/OS2/WASI)
- **Isolation**: 7/10 (Clear interface, limited call sites)
- **Stability**: 10/10 (1 commit/year)
- **Testability**: -1/10 (TestPoisonArea.cpp exists but complex memory testing)

### 7. SHA1.cpp: Total Score 22/40
- **Location**: mfbt/SHA1.cpp (405 lines)
- **Type**: Production code - SHA-1 hashing algorithm
- **Simplicity**: 0/10 (405 lines, complex cryptographic algorithm)
- **Isolation**: 8/10 (Clear interface, well-defined API)
- **Stability**: 10/10 (1 commit/year)
- **Testability**: 4/10 (Algorithmic testing possible but extensive)

### 8. RandomNum.cpp: Total Score 18/40
- **Location**: mfbt/RandomNum.cpp (146 lines)
- **Type**: Production code - Cryptographic random number generation
- **Simplicity**: 2/10 (146 lines, heavy platform-specific code for Win/Linux/BSD/WASI)
- **Isolation**: 7/10 (Clear interface, limited call sites)
- **Stability**: 10/10 (1 commit/year)
- **Testability**: -1/10 (Random number testing is inherently difficult)

## Selected Component: JSONWriter.cpp (gTwoCharEscapes lookup table)

- **Location**: mfbt/JSONWriter.cpp → local/rust/firefox_jsonwriter/
- **Type**: Production code (NOT test file) - JSON escape sequence lookup table
- **Lines of code**: 47 (.cpp file only)
- **Dependencies**: 
  - mozilla/JSONWriter.h (header where table is used)
  - No external library dependencies for the table itself
- **Call sites**: 5 uses within JSONWriter.h (lines 120, 171, 195, 197)
- **Test coverage**: ~95% (indirectly via TestJSONWriter.cpp - 665 lines, comprehensive JSON output testing)
- **Upstream stability**: 1 commit/year (very stable)
- **Total score**: 31/40

## Rationale:

JSONWriter.cpp defines `gTwoCharEscapes`, a 256-byte lookup table that maps ASCII characters to their JSON two-character escape sequences (e.g., '\n' → 'n', '\t' → 't', '\"' → '"', '\\' → '\\'). This is an ideal candidate for Rust porting because:

1. **Pure Data Structure**: No complex logic - just a const char array initialized with escape mappings
2. **Well-Tested**: The JSONWriter test suite (665 lines) comprehensively tests JSON output, which validates the escape table behavior
3. **Clear Interface**: The table is accessed directly via array indexing from the header
4. **Stable**: 1 commit in the past year indicates mature, stable code
5. **Minimal Dependencies**: Only used by JSONWriter.h header (header-only template implementation)

The port will involve creating a Rust module that exposes the same 256-byte lookup table via FFI, allowing the existing C++ header code to seamlessly use the Rust-provided data.

## Risk Assessment:

### Low Risk Factors:
- Static const data - no runtime behavior to replicate
- Comprehensive test coverage via JSONWriter tests
- Very stable upstream (1 commit/year)
- Clear, simple interface (array indexing)
- No platform-specific code
- No threading concerns (const data)

### Medium Risk Factors:
- Header-only usage pattern (JSONWriter.h accesses table directly)
- Need to ensure byte-for-byte identical table layout
- Must maintain exact memory layout for C++ access

### Mitigation Strategies:
- Use `#[repr(C)]` and `#[no_mangle]` for guaranteed C compatibility
- Create compile-time assertions to verify table size (256 bytes)
- Use cbindgen to generate compatible header
- Comprehensive testing via existing TestJSONWriter.cpp (no porting needed)
- FFI layer provides the table as `extern "C"` symbol
- Document memory layout requirements clearly

## Implementation Notes:

The port will create:
1. Rust module (`local/rust/firefox_jsonwriter/`) with the lookup table
2. FFI layer exposing `gTwoCharEscapes` as `extern "C"` const
3. cbindgen configuration to generate compatible header
4. Build system integration (conditional compilation)
5. Tests remain in C++ (TestJSONWriter.cpp unchanged)

This port demonstrates Rust's ability to provide static data to C++ code, establishing a pattern for future data structure ports.

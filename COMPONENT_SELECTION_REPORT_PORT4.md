# Component Selection Report - Port #4

## Candidates Evaluated:

### 1. HashBytes (mfbt/HashFunctions.cpp): Total Score 35/40
**Simplicity Score: 10/10**
- Lines of code: 38 .cpp + ~420 .h (but HashBytes is only one function) = **10/10**
- Dependencies: 3 (Types.h, string.h, WrappingOperations.h) = **10/10**
- Platform-specific code: None = **10/10**

**Isolation Score: 8/10**
- Call sites: ~29 (production code) = **7/10**
- Header dependencies: 3 direct includes = **10/10**
- Inheritance depth: 0 (standalone function) = **10/10**

**Stability Score: 10/10**
- Commits last year: 1 (merge commit, not substantive change) = **10/10**
- Bug references: 0 in recent history = **10/10**
- Last major refactor: >2 years = **10/10**

**Testability Score: 7/10**
- Test coverage: Indirectly tested through hash tables and containers = **7/10**
- Test types: Integration tests via hash tables = **7/10**
- Test clarity: Indirect (through hash table operations) = **7/10**

### 2. nsObserverList (xpcom/ds/nsObserverList.cpp): Total Score 20/40
**Simplicity Score: 7/10**
- Lines of code: 93 .cpp + 67 .h = 160 = **7/10**
- Dependencies: 8+ (nsCOMArray, nsIObserver, nsHashKeys, nsMaybeWeakPtr, etc.) = **4/10**
- Platform-specific code: None = **10/10**

**Isolation Score: 3/10**
- Call sites: Unknown, likely many through observer service = **2/10**
- Header dependencies: 8+ XPCOM headers = **4/10**
- Inheritance depth: 1 (nsCharPtrHashKey) = **7/10**

**Stability Score: 5/10**
- Commits last year: Not checked, but XPCOM is stable = **7/10**
- Bug references: Unknown = **5/10**
- Last major refactor: Unknown = **5/10**

**Testability Score: 5/10**
- Test coverage: Unknown = **5/10**
- Test types: Likely has tests but not analyzed = **5/10**
- Test clarity: Unknown = **5/10**

**Excluded Reason**: Heavy XPCOM dependencies make FFI layer complex

### 3. Poison (mfbt/Poison.cpp): Total Score 18/40
**Simplicity Score: 4/10**
- Lines of code: 206 .cpp + 110 .h = 316 = **4/10**
- Dependencies: 5+ including platform-specific headers = **7/10**
- Platform-specific code: Significant (Windows, Linux, OS/2, WASI) = **0/10**

**Isolation Score: 7/10**
- Call sites: Primarily internal to memory allocator = **7/10**
- Header dependencies: 5+ = **7/10**
- Inheritance depth: 0 = **10/10**

**Stability Score: 7/10**
- Commits last year: Low = **10/10**
- Bug references: Few = **7/10**
- Last major refactor: >2 years = **7/10**

**Testability Score: 0/10**
- Test coverage: Difficult to test (memory protection) = **0/10**
- Test types: None found = **0/10**
- Test clarity: N/A = **0/10**

**Excluded Reason**: Heavy platform-specific code and hard to test

### 4. TaggedAnonymousMemory (mfbt/TaggedAnonymousMemory.cpp): Total Score 17/40
**Simplicity Score: 6/10**
- Lines of code: 83 .cpp + 83 .h = 166 = **7/10**
- Dependencies: Platform-specific system calls = **4/10**
- Platform-specific code: Linux-only feature = **0/10**

**Isolation Score: 8/10**
- Call sites: Used by memory allocator = **7/10**
- Header dependencies: 4 = **10/10**
- Inheritance depth: 0 = **10/10**

**Stability Score: 3/10**
- Commits last year: Unknown = **5/10**
- Bug references: Unknown = **5/10**
- Last major refactor: Recent (Linux 5.17+ feature) = **0/10**

**Testability Score: 0/10**
- Test coverage: Hard to test (kernel feature) = **0/10**
- Test types: None = **0/10**
- Test clarity: N/A = **0/10**

**Excluded Reason**: Platform-specific (Linux-only), hard to test

---

## Selected Component: HashBytes Function

### Location
- **File**: mfbt/HashFunctions.cpp (function: HashBytes)
- **Header**: mfbt/HashFunctions.h
- **Type**: Production code (NOT test file)

### Metrics
- **C++ Lines**: 38 (.cpp) + ~20 (relevant .h declarations)
- **Dependencies**: 
  - mozilla/Types.h (MFBT_API)
  - string.h (memcpy)
  - mozilla/WrappingOperations.h (WrappingMultiply)
  - HashFunctions.h internal functions (AddToHash, kGoldenRatioU32)
- **Call Sites**: ~29 locations across codebase
  - gfx/ (blur, fonts, 2D)
  - js/src/ (JIT, stencil, BigInt)
  - dom/media/
  - layout/painting/
  - memory/replace/dmd/
  - image/encoders/
- **Test Coverage**: Indirectly via hash table tests, ~60% (estimated)
- **Upstream Stability**: 1 commit/year
- **Total Score**: **35/40**

### Rationale

HashBytes is the optimal candidate for Port #4 because:

1. **Simplicity**: Single, focused function that hashes byte arrays
2. **Pure Computation**: No I/O, no platform dependencies, no side effects
3. **Well-Isolated**: Clear API boundary with ~29 call sites
4. **Algorithm Clarity**: Simple word-by-word hashing with golden ratio mixing
5. **Stability**: Extremely stable (1 commit/year)
6. **FFI-Friendly**: Simple C-compatible signature: `uint32_t HashBytes(const void*, size_t, HashNumber)`
7. **Testable**: Easy to write unit tests with known inputs/outputs

The function implements a straightforward hashing algorithm:
- Walk through memory word-by-word for efficiency
- Handle unaligned trailing bytes
- Mix with golden ratio constant (AddToHash)
- Return 32-bit hash

This is a perfect candidate for demonstrating:
- Raw pointer handling in Rust (`*const u8`)
- Unsafe memory operations with safety guarantees
- Word-aligned memory access optimization
- Zero-cost abstractions for algorithm port

### Risk Assessment

**Low Risk Factors:**
- ✅ No platform-specific code
- ✅ Pure function (no side effects)
- ✅ Simple algorithm
- ✅ Clear test strategy (unit tests with test vectors)
- ✅ Single function to port

**Medium Risk Factors:**
- ⚠️ Used by JIT compiler (js/src/jit/) - performance-sensitive
- ⚠️ ~29 call sites to verify
- ⚠️ Must handle unaligned memory access correctly

**Mitigation Strategies:**
- **Performance**: Rust should match or exceed C++ with proper optimization
- **Call Sites**: Extensive testing with existing test suite
- **Memory Safety**: Carefully validate pointer arithmetic in unsafe blocks
- **Alignment**: Use slice operations for byte-by-byte fallback

### Implementation Notes

**Key Challenges:**
1. Pointer arithmetic in Rust (use slice operations)
2. Word-by-word memory access (use `size_of::<usize>()`)
3. Unaligned loads (use safe slice indexing)
4. Must match C++ behavior bit-for-bit

**Success Criteria:**
- ✅ All existing hash table tests pass
- ✅ Performance within ±5% of C++
- ✅ No undefined behavior in unsafe blocks
- ✅ FFI layer is panic-safe

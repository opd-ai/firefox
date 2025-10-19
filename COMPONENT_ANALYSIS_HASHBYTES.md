# Component Analysis: HashBytes

## Overview
HashBytes is a function in mfbt/HashFunctions.cpp that hashes a byte array into a 32-bit hash value. It's part of Firefox's hash function suite and is used throughout the codebase for creating hash codes from arbitrary byte sequences.

## API Surface

```cpp
namespace mozilla {

// Constants
static const HashNumber kGoldenRatioU32 = 0x9E3779B9U;

// Core function to port
uint32_t HashBytes(const void* aBytes, size_t aLength,
                   HashNumber startingHash = 0);

// Helper functions (inline in header, will be ported for completeness)
constexpr HashNumber RotateLeft5(HashNumber aValue) {
  return (aValue << 5) | (aValue >> 27);
}

constexpr HashNumber AddU32ToHash(HashNumber aHash, uint32_t aValue) {
  return mozilla::WrappingMultiply(kGoldenRatioU32,
                                   RotateLeft5(aHash) ^ aValue);
}

} // namespace mozilla
```

### Function Signature
- **Name**: `HashBytes`
- **Parameters**:
  - `const void* aBytes`: Pointer to byte array to hash
  - `size_t aLength`: Length of byte array in bytes
  - `HashNumber startingHash = 0`: Optional starting hash value for chaining
- **Return**: `uint32_t` (HashNumber) - 32-bit hash value
- **Namespace**: `mozilla`

### Algorithm Description

1. **Initialize** hash with `startingHash` parameter
2. **Word-by-word hashing**:
   - Iterate through memory in `sizeof(size_t)` chunks
   - For each word: use `memcpy` for unaligned load, then call `AddToHash`
3. **Remaining bytes**:
   - Hash any trailing bytes (< sizeof(size_t)) individually
4. **Return** final hash value

The hash mixing uses:
- Golden ratio constant: `0x9E3779B9U`
- Rotation: Left-rotate by 5 bits
- XOR with input value
- Wrapping multiply (unsigned overflow behavior)

## Dependencies

### Direct Dependencies
1. **mozilla/Types.h**
   - `MFBT_API` macro for export
   - `HashNumber` type alias (uint32_t)

2. **string.h**
   - `memcpy()` for unaligned loads

3. **mozilla/WrappingOperations.h**
   - `WrappingMultiply()` for wrapping arithmetic

4. **Internal (from HashFunctions.h)**
   - `kGoldenRatioU32` constant
   - `RotateLeft5()` inline function
   - `AddU32ToHash()` inline function

### Indirect Dependencies
None - this is a leaf function in the dependency graph.

## Call Sites Analysis

Total: **29 non-test call sites** (approximate)

### By Category:

**Graphics (gfx/)**: ~10 call sites
- `gfx/2d/SFNTData.cpp`: Font data hashing (4 calls)
- `gfx/thebes/gfxBlur.cpp`: Shadow color hashing (4 calls)
- `gfx/thebes/gfxFont.cpp`: Font variation settings
- `gfx/thebes/gfxUserFontSet.h`: Font features hashing (2 calls)

**JavaScript Engine (js/src/)**: ~7 call sites
- `js/src/frontend/StencilXdr.cpp`: Script hashing (2 calls)
- `js/src/jit/CacheIRCompiler.cpp`: Code cache hashing
- `js/src/jit/IonTypes.h`: Value hashing
- `js/src/jit/MacroAssembler.cpp`: Inline hash (comment reference)
- `js/src/vm/BigIntType.cpp`: BigInt hashing
- `js/src/vm/SharedStencil.h`: Immutable data hashing

**Other Modules**: ~12 call sites
- `dom/media/doctor/DDLifetimes.h`: Media logging
- `layout/painting/BorderCache.h`: Border cache key
- `memory/replace/dmd/DMD.cpp`: Stack trace hashing
- `image/encoders/png/nsPNGEncoder.cpp`: Image buffer hashing

### Usage Patterns:
1. **Direct hashing**: `HashBytes(data, len)`
2. **Hash chaining**: `AddToHash(hash, HashBytes(data, len))`
3. **Struct field hashing**: `HashBytes(&field, sizeof(field))`

## Test Coverage

### Direct Tests
❌ **No dedicated unit tests found** for HashBytes specifically.

### Indirect Tests
✅ HashBytes is exercised through:
- **js/src/jsapi-tests/testHashTable.cpp**: Hash table operations
- **mfbt/tests/TestHashTable.cpp**: MFBT hash table tests
- Various integration tests that use hash tables and containers

### Test Strategy for Port
We will create:
1. **Rust unit tests** (`src/tests.rs`):
   - Test with empty array
   - Test with single byte
   - Test with word-aligned data
   - Test with unaligned data
   - Test with starting hash (chaining)
   - Test that matches C++ output exactly

2. **FFI integration tests**:
   - Verify C++ callers work unchanged
   - Test all call sites still function

3. **Property-based tests** (optional):
   - Hash determinism (same input → same output)
   - Avalanche effect (small input change → large hash change)
   - Distribution quality

## Memory & Threading

### Ownership Model
- **Input**: Borrows const pointer (read-only access)
- **Output**: Returns owned `u32` value
- **No heap allocation**: Stack-only function

### Thread Safety
- ✅ **Thread-safe**: Pure function, no shared state
- ✅ **Reentrant**: No global state modified
- ✅ **Const-correct**: Input is read-only

### Memory Safety Considerations
- ⚠️ **Unsafe block required**: Raw pointer dereference
- ⚠️ **Bounds checking**: Must trust caller that `aBytes[0..aLength]` is valid
- ✅ **No UAF**: Function doesn't store pointers
- ✅ **No double-free**: Function doesn't own memory

## Algorithm Analysis

### Complexity
- **Time**: O(n) where n = aLength
- **Space**: O(1) - constant stack usage
- **Cache behavior**: Sequential access (cache-friendly)

### Word-by-Word Optimization
```cpp
// Optimize by processing size_t words instead of individual bytes
for (; i < aLength - (aLength % sizeof(size_t)); i += sizeof(size_t)) {
  size_t data;
  memcpy(&data, b + i, sizeof(size_t));  // Unaligned load
  hash = AddToHash(hash, data);
}
```

Benefits:
- Fewer loop iterations (8x on 64-bit)
- Better ILP (instruction-level parallelism)
- Reduced function call overhead

### Rust Implementation Strategy

```rust
pub fn hash_bytes(bytes: &[u8], starting_hash: u32) -> u32 {
    let mut hash = starting_hash;
    let len = bytes.len();
    let ptr = bytes.as_ptr();
    
    // Word-by-word hashing
    let word_size = std::mem::size_of::<usize>();
    let num_words = len / word_size;
    
    for i in 0..num_words {
        let offset = i * word_size;
        // Safe: bounds checked by num_words calculation
        let word = unsafe {
            std::ptr::read_unaligned(ptr.add(offset) as *const usize)
        };
        hash = add_to_hash(hash, word as u32);
    }
    
    // Remaining bytes
    let remaining_start = num_words * word_size;
    for byte in &bytes[remaining_start..] {
        hash = add_to_hash(hash, *byte as u32);
    }
    
    hash
}
```

## Performance Considerations

### Critical Paths
- **JIT compiler**: Used in js/src/jit/ for code cache keys (HOT PATH)
- **Font rendering**: gfx/ font cache lookups (WARM PATH)
- **BigInt operations**: Hashing large integers (COLD PATH)

### Optimization Targets
1. **Inline aggressively**: Mark `#[inline]` for hot path
2. **SIMD potential**: Future enhancement for larger buffers
3. **Branch prediction**: Loop structure should be branch-predictor friendly

### Benchmarking Plan
- Small buffers (< 64 bytes): Most common case
- Medium buffers (64 - 1024 bytes): Font/image data
- Large buffers (> 1024 bytes): Rare, but test worst case
- Target: Within ±5% of C++ performance

## Security Considerations

### Non-Cryptographic Hash
⚠️ **This is NOT a cryptographic hash function**
- Uses simple golden ratio mixing
- Vulnerable to hash collision attacks
- Should NOT be used for security purposes

### Use Cases
✅ Appropriate for:
- Hash table keys (internal)
- Cache keys (internal)
- Checksums (non-security)

❌ NOT appropriate for:
- Password hashing
- Cryptographic signatures
- Security tokens

### Privacy
- Deterministic output may leak information about input data
- Consider HashCodeScrambler class for privacy-sensitive data

## Implementation Checklist

- [x] Understand algorithm
- [ ] Implement core logic in Rust
- [ ] Create FFI layer
- [ ] Write comprehensive tests
- [ ] Validate against C++ output
- [ ] Performance benchmarking
- [ ] Documentation
- [ ] Integration with build system

## References

- **Golden Ratio Hashing**: Knuth, "The Art of Computer Programming", 6.4
- **Fibonacci Hashing**: https://en.wikipedia.org/wiki/Hash_function#Fibonacci_hashing
- **SipHash** (used by HashCodeScrambler): https://131002.net/siphash/

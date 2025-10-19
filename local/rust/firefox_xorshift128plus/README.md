# XorShift128+ PRNG - Rust Port

Rust implementation of Firefox's XorShift128+ pseudo-random number generator (mfbt/XorShift128PlusRNG.h).

## Overview

This is Port #3 in Firefox's systematic C++ to Rust migration. XorShift128+ is a fast, non-cryptographic PRNG based on the xorshift128+ algorithm described in:

> Vigna, Sebastiano (2014). "Further scramblings of Marsaglia's xorshift generators". arXiv:1404.0390

## Component Details

- **Original**: mfbt/XorShift128PlusRNG.h (122 lines, header-only)
- **Rust Port**: local/rust/firefox_xorshift128plus/ (390 lines including tests)
- **Lines Removed**: 0 (overlay architecture, C++ coexists)
- **Test Coverage**: 4 C++ test functions (remain in C++, call via FFI)
- **Selection Score**: 36/40

### Why This Component?

1. **Simplicity**: Pure algorithm, no I/O, no allocations, no platform dependencies
2. **Isolation**: Well-defined API, 22 call sites (manageable)
3. **Stability**: 1 commit in last year, very stable
4. **Testability**: Comprehensive C++ tests validate FFI boundary
5. **Performance**: Perfect for demonstrating Rust's zero-cost abstractions

## Algorithm

XorShift128+ is a xorshift-based PRNG with these properties:

- **Period**: 2^128 - 1 calls before repetition
- **Performance**: ~1-2 CPU cycles per call
- **Quality**: Passes BigCrush statistical test suite
- **Size**: 16 bytes (2 × u64 state)
- **Thread safety**: NOT thread-safe (no internal locking)

## API

### Rust API

```rust
use firefox_xorshift128plus::XorShift128PlusRNG;

// Create RNG with seeds
let mut rng = XorShift128PlusRNG::new(seed0, seed1);

// Generate random numbers
let random_u64 = rng.next();
let random_f64 = rng.next_double();  // [0, 1)

// Reset state
rng.set_state(new_seed0, new_seed1);

// JIT offsets
let offset0 = XorShift128PlusRNG::offset_of_state0();  // 0
let offset1 = XorShift128PlusRNG::offset_of_state1();  // 8
```

### FFI API (C++)

```cpp
// Constructor
XorShift128PlusRNG* rng = xorshift128plus_new(seed0, seed1);

// Generate random numbers
uint64_t val = xorshift128plus_next(rng);
double d = xorshift128plus_next_double(rng);

// Reset state
xorshift128plus_set_state(rng, new_seed0, new_seed1);

// JIT offsets
size_t offset0 = xorshift128plus_offset_of_state0();  // 0
size_t offset1 = xorshift128plus_offset_of_state1();  // 8

// Destructor
xorshift128plus_destroy(rng);
```

## Testing Strategy

This Rust port maintains 100% compatibility with existing C++ tests.

### Test Architecture

- **C++ tests remain unchanged** (mfbt/tests/TestXorShift128PlusRNG.cpp)
- **C++ tests call Rust implementation** via FFI layer (src/ffi.rs)
- **No C++ test ports** were created
- **Additional Rust tests** (src/lib.rs) provide supplementary validation

### FFI Test Support

The FFI layer (src/ffi.rs) exposes all methods needed by:
- Production code call sites (54 references across 18 files)
- Unit test call sites (4 test functions)
- JIT code generation (offset methods)

### C++ Tests (Remain in C++)

1. **TestDumbSequence**: Verifies bit-exact algorithm implementation
2. **TestPopulation**: Validates statistical bit distribution
3. **TestSetState**: Tests state reset and reproducibility
4. **TestDoubleDistribution**: Validates uniform distribution in [0, 1)

### Rust Tests (Supplementary)

Additional tests in `src/lib.rs`:
- `test_struct_size`: Verify 16-byte layout
- `test_struct_offsets`: Verify state[0] at 0, state[1] at 8
- `test_dumb_sequence`: Match C++ TestDumbSequence
- `test_set_state`: Match C++ TestSetState
- `test_population`: Match C++ TestPopulation
- `test_next_double_range`: Validate [0, 1) range
- `test_zero_state_panics`: Debug assertion validation

### Running Tests

```bash
# Build Rust crate and run Rust tests
cd local/rust/firefox_xorshift128plus
cargo test

# Run C++ tests with Rust backend (via overlay system)
export MOZ_RUST_XORSHIFT128PLUS=1
./local/scripts/apply-build-overlays.sh
./mach build
./mach test mfbt/tests/TestXorShift128PlusRNG
```

## Memory Layout

The struct uses `#[repr(C)]` to guarantee C-compatible layout:

```
Offset  | Field      | Type | Size
--------|------------|------|-----
0       | state[0]   | u64  | 8
8       | state[1]   | u64  | 8
--------|------------|------|-----
Total:  16 bytes
```

This layout is critical for JIT code that directly accesses state via computed offsets.

### Compile-Time Guarantees

```rust
// In src/lib.rs
const _: () = {
    assert!(size_of::<XorShift128PlusRNG>() == 16);
    assert!(offset_of_state0() == 0);
    assert!(offset_of_state1() == 8);
};
```

## Performance

### Target

- C++ inline version: ~1-2 CPU cycles per call
- Rust version: Should match via zero-cost abstractions and inlining

### Critical Code Paths

1. **JS Engine**: Math.random() JIT compilation
2. **Memory Allocator**: Allocation randomization
3. **Fingerprinting Resistance**: Privacy-protecting randomization

### Benchmarking

Performance validation will occur in Phase 5 (Validation).

## Integration

### Build System

Uses Firefox's overlay architecture:
- Rust code in `local/rust/firefox_xorshift128plus/`
- Build configuration in `local/mozconfig.rust-xorshift128plus`
- Conditional compilation via `--enable-rust-xorshift128plus`
- Zero conflicts with upstream (all changes in `local/`)

### Dependencies

**None** - Pure Rust standard library

### Platform Support

- All platforms supported by Firefox
- No platform-specific code
- Requires 64-bit integers (u64)
- IEEE 754 double precision floating point

## Safety

### Panic Safety

All FFI functions catch panics to prevent unwinding into C++:

```rust
pub extern "C" fn xorshift128plus_next(rng: *mut RNG) -> u64 {
    let result = panic::catch_unwind(|| unsafe { (*rng).next() });
    result.unwrap_or(0)
}
```

### Null Pointer Safety

All FFI functions check for null pointers:

```rust
if rng.is_null() {
    return 0;
}
```

### Debug Assertions

```rust
debug_assert!(
    state0 != 0 || state1 != 0,
    "At least one state value must be non-zero"
);
```

## Known Limitations

1. **Not thread-safe**: Use one RNG per thread or external synchronization
2. **Not cryptographically secure**: Don't use for security-sensitive operations
3. **Initial zeros**: If seeds are small, initial outputs will have many zeros
4. **Zero state**: Both state values being zero breaks the algorithm (asserted)

## Future Work

1. Performance benchmarking against C++ version
2. Integration with Firefox CI
3. Consider porting related components (FastBernoulliTrial uses this RNG)

## References

- **Original C++**: mfbt/XorShift128PlusRNG.h
- **Paper**: Vigna, S. (2014). "Further scramblings of Marsaglia's xorshift generators". arXiv:1404.0390
- **Test Suite**: mfbt/tests/TestXorShift128PlusRNG.cpp
- **Component Analysis**: COMPONENT_ANALYSIS_XORSHIFT.md

## License

Mozilla Public License 2.0 (MPL-2.0)

# Firefox ChaosMode - Rust Implementation

Rust port of Firefox's ChaosMode testing infrastructure from `mfbt/ChaosMode.{h,cpp}`.

## Overview

ChaosMode is a testing tool that introduces controlled non-determinism into Firefox to help uncover race conditions, timing bugs, and other concurrency issues. When enabled, it randomizes various behaviors like:

- Thread scheduling
- Network request ordering
- Timer delays
- I/O amounts
- Hash table iteration order
- Image cache usage
- Task dispatching and running

## API

### Rust API

```rust
use firefox_chaosmode::*;

// Set which features should be chaotic
set_chaos_feature(ChaosFeature::ThreadScheduling);

// Enable chaos mode
enter_chaos_mode();

// Check if a feature is active
if is_active(ChaosFeature::ThreadScheduling) {
    // Use random values to create chaos
    let random_val = random_u32_less_than(100);
}

// Disable chaos mode
leave_chaos_mode();
```

### C++ FFI API

```cpp
// C++ can call via FFI
extern "C" {
    void mozilla_chaosmode_set_chaos_feature(uint32_t feature);
    bool mozilla_chaosmode_is_active(uint32_t feature);
    void mozilla_chaosmode_enter_chaos_mode();
    void mozilla_chaosmode_leave_chaos_mode();
    uint32_t mozilla_chaosmode_random_u32_less_than(uint32_t bound);
    int32_t mozilla_chaosmode_random_i32_in_range(int32_t low, int32_t high);
}
```

## Features

### ChaosFeature Enum

```rust
pub enum ChaosFeature {
    None = 0x0,
    ThreadScheduling = 0x1,
    NetworkScheduling = 0x2,
    TimerScheduling = 0x4,
    IOAmounts = 0x8,
    HashTableIteration = 0x10,
    ImageCache = 0x20,
    TaskDispatching = 0x40,
    TaskRunning = 0x80,
    Any = 0xffffffff,
}
```

Features can be combined using bitwise OR.

## Thread Safety

- **Counter operations**: Thread-safe (uses `AtomicU32` with `Relaxed` ordering)
- **Feature setting**: NOT thread-safe (must be done before threading starts)
- **Random functions**: NOT thread-safe (uses C `rand()` for compatibility)

## Implementation Notes

### Atomic Memory Ordering

Uses `Ordering::Relaxed` to match C++ `Atomic<uint32_t, Relaxed>`:
- No ordering guarantees beyond atomicity
- Appropriate for simple counters where ordering doesn't matter
- Matches C++ behavior exactly

### Random Number Generation

Uses `libc::rand()` via FFI:
- NOT thread-safe (intentional, matches C++ behavior)
- NOT cryptographically secure
- Simple modulo for range reduction (preserves bias for compatibility)

### Nesting

Chaos mode can be nested:
```rust
enter_chaos_mode();  // counter = 1
enter_chaos_mode();  // counter = 2
leave_chaos_mode();  // counter = 1
leave_chaos_mode();  // counter = 0
```

## Building

```bash
cargo build
cargo test
```

## Testing

```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test chaosmode_tests
```

## Usage in Firefox

### With Rust ChaosMode

```bash
export MOZ_RUST_CHAOSMODE=1
./local/scripts/apply-build-overlays.sh
./mach build
```

### With C++ ChaosMode (default)

```bash
./mach build
```

## Differences from C++

None - this is a direct port with identical behavior:
- Same atomic semantics (Relaxed ordering)
- Same random number generation (uses C `rand()`)
- Same API signatures
- Same nesting behavior
- Same thread safety characteristics

## Call Sites

ChaosMode is used in 34 locations across 11 Firefox files:

- DOM utilities
- Image loading
- Network scheduling
- Thread scheduling
- Timer scheduling
- Hash table iteration
- Testing infrastructure

## Performance

No performance difference expected:
- Simple atomic operations
- Minimal overhead when not active
- Direct FFI calls (no wrapper overhead)

## License

MPL-2.0 (Mozilla Public License 2.0)

## Authors

Firefox Developers

---

**Port Date**: 2025-10-19  
**Original C++**: mfbt/ChaosMode.{h,cpp}  
**Port #**: 2

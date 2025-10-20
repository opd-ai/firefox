# Component Analysis: nsTObserverArray_base

## API Surface:

```cpp
class nsTObserverArray_base {
 public:
  typedef size_t index_type;
  typedef size_t size_type;
  typedef ptrdiff_t diff_type;

 protected:
  // Nested iterator base class
  class Iterator_base {
   protected:
    friend class nsTObserverArray_base;
    Iterator_base(index_type aPosition, Iterator_base* aNext);
    
    index_type mPosition;      // Current iterator position
    Iterator_base* mNext;      // Next iterator in linked list
  };

  // Constructor/Destructor (inline in header - NOT porting)
  nsTObserverArray_base();
  ~nsTObserverArray_base();

  // *** METHODS TO PORT (in .cpp file) ***
  
  /**
   * Adjusts all active iterators after array modification
   * @param aModPos Position where element was added/removed
   * @param aAdjustment -1 (removal) or +1 (insertion)
   */
  void AdjustIterators(index_type aModPos, diff_type aAdjustment);

  /**
   * Resets all iterators to position 0
   * Called during array Clear()
   */
  void ClearIterators();

  // Member variable
  mutable Iterator_base* mIterators;  // Head of iterator linked list
};
```

## Core Algorithm:

The base class maintains a linked list of active iterators (`mIterators`). When the array is modified:

1. **AdjustIterators**: Walks the iterator linked list and adjusts positions of any iterators pointing beyond the modification point
   - Insertion (+1): Increments positions after insert point
   - Removal (-1): Decrements positions after removal point
   
2. **ClearIterators**: Walks the iterator linked list and resets all positions to 0

This ensures iterators remain valid even when the underlying array is modified during iteration.

## Dependencies:

### Direct includes (.cpp file):
- `nsTObserverArray.h` - The header declaring the class

### Indirect dependencies (in .h file - NOT porting):
- `mozilla/MemoryReporting.h` - Memory reporting utilities
- `mozilla/ReverseIterator.h` - Reverse iterator support
- `nsTArray.h` - The underlying array storage
- `nsCycleCollectionNoteChild.h` - Cycle collection support

**Port Strategy**: We port ONLY the .cpp file (2 methods). The template header stays in C++, calling our Rust implementation via FFI for these base class methods.

## Call Sites (total: 11 internal uses in header):

All calls are from the template class `nsAutoTObserverArray<T, N>` in the same header file:

1. **InsertElementAt** (line 163): `AdjustIterators(aIndex, 1);`
2. **InsertElementAt** (line 170): `AdjustIterators(aIndex, 1);`
3. **PrependElementUnlessExists** (line 181): `AdjustIterators(0, 1);`
4. **RemoveElementAt** (line 211): `AdjustIterators(aIndex, -1);`
5. **RemoveElement** (line 227): `AdjustIterators(index, -1);`
6. **NonObservingRemoveElementsBy** (line 239): `AdjustIterators(i, -1);` (in loop)
7. **Clear** (line 252): `ClearIterators();`

**External Usage**: The methods are `protected`, so only called by derived template classes in the same header. No external .cpp files directly call these methods.

## Test Coverage (tests remain in C++):

**Test File**: xpcom/tests/gtest/TestObserverArray.cpp (573 lines)

**Test Structure**:
- Uses gtest framework
- Single comprehensive TEST(ObserverArray, Tests) with 40+ test scenarios
- Tests forward iteration, backward iteration, end-limited iteration
- Tests concurrent modifications: append during iteration, remove during iteration, insert during iteration
- Tests edge cases: empty arrays, single elements, multiple iterators
- Tests iterator stability guarantees

**Key Test Scenarios**:
1. Basic iteration (forward, backward, end-limited)
2. Append during iteration (elements added at end)
3. Prepend during iteration (elements added at start)
4. Insert during iteration (elements added in middle)
5. Remove during iteration (elements deleted)
6. Combined operations (insert + remove during same iteration)
7. Multiple concurrent iterators
8. Iterator destruction order validation
9. Clear() operation
10. AppendElementUnlessExists, PrependElementUnlessExists

**Coverage Estimate**: ~90% (comprehensive functional testing, all public API exercised)

**Note**: All tests will continue calling through C++ template → FFI → Rust implementation. We do NOT port the test file.

## Memory & Threading:

**Ownership Model**:
- Iterator linked list managed by base class
- Each Iterator_base is owned by its iterator object (created on stack in template code)
- No heap allocation in these two methods
- Lifetime: Iterators must be destroyed in reverse construction order (stack-based LIFO)

**Thread Safety**: 
- **NOT thread-safe** by design
- Observers are typically used on single thread (main thread or specific worker)
- No synchronization primitives used
- `mIterators` is mutable to allow iteration over const arrays

**Resource Cleanup**:
- No dynamic memory allocation in AdjustIterators or ClearIterators
- Iterators manage their own lifecycle (constructor adds to list, destructor removes)
- Base class destructor asserts that mIterators == nullptr (no leaked iterators)

**Memory Layout Requirements**:
- `Iterator_base` struct must maintain C-compatible layout for FFI
- `mPosition`: size_t (8 bytes on 64-bit)
- `mNext`: raw pointer (8 bytes on 64-bit)
- Total: 16 bytes, natural alignment

## FFI Considerations:

**Challenges**:
1. Raw pointer manipulation (`Iterator_base* mIterators`)
2. Walking linked list (need unsafe Rust)
3. Mutable pointer traversal (`iter->mPosition`, `iter->mNext`)
4. NULL pointer handling (end of list)

**Solutions**:
1. Use #[repr(C)] for Iterator_base struct
2. Use `*mut Iterator_base` for raw pointers
3. Carefully validate NULL before dereferencing
4. Panic boundary in FFI layer
5. Preserve exact C++ semantics (no Rust ownership transfer)

**FFI Exports Needed**:
```rust
#[no_mangle]
pub extern "C" fn nsTObserverArray_base_AdjustIterators(
    this: *mut nsTObserverArray_base,
    mod_pos: usize,
    adjustment: isize
);

#[no_mangle]
pub extern "C" fn nsTObserverArray_base_ClearIterators(
    this: *mut nsTObserverArray_base
);
```

## Integration Strategy:

**Conditional Compilation**:
```cpp
// In nsTObserverArray.cpp
#ifdef MOZ_RUST_OBSERVER_ARRAY
  // Call Rust implementation via FFI
  extern "C" void nsTObserverArray_base_AdjustIterators(...);
  extern "C" void nsTObserverArray_base_ClearIterators(...);
#else
  // Original C++ implementation
  void nsTObserverArray_base::AdjustIterators(...) { /* ... */ }
  void nsTObserverArray_base::ClearIterators() { /* ... */ }
#endif
```

**Build Flag**: `--enable-rust-observer-array`

## Validation Plan:

1. **Unit Tests**: All 573 lines of TestObserverArray.cpp must pass
2. **Build Test**: Both C++ and Rust versions must compile cleanly
3. **Behavior Verification**: Bit-exact behavior match with C++ version
4. **Memory Safety**: No segfaults, no memory leaks (valgrind if available)
5. **Performance**: Within ±5% of C++ version (not performance-critical)
6. **Upstream Merge**: Zero conflicts with git pull upstream/main

## Risk Mitigation:

**Pointer Safety**:
- Comprehensive NULL checks before dereferencing
- Use `is_null()` checks in Rust
- Panic boundaries to prevent unwinding into C++

**Testing Strategy**:
- Leverage comprehensive C++ test suite (573 lines)
- Add Rust-side unit tests for pointer manipulation edge cases
- Validate with both C++ and Rust implementations

**Rollback Plan**:
- Conditional compilation allows instant fallback to C++
- No changes to test files (tests work with both versions)
- Overlay architecture ensures zero upstream conflicts

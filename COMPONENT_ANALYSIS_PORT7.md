# Component Analysis: JSONWriter.cpp (gTwoCharEscapes)

## API Surface:

```cpp
namespace mozilla {
namespace detail {

// 256-byte lookup table mapping ASCII characters to their JSON two-char escape sequences
// Non-zero entries indicate the character requires a two-char escape:
//   - 0x08 ('\b') → 'b'  
//   - 0x09 ('\t') → 't'
//   - 0x0A ('\n') → 'n'
//   - 0x0C ('\f') → 'f'
//   - 0x0D ('\r') → 'r'
//   - 0x22 ('"')  → '"'
//   - 0x5C ('\\') → '\\'
// Zero entries indicate no two-char escape is needed (may need \uXXXX format instead)

extern MFBT_DATA const char gTwoCharEscapes[256];

}  // namespace detail
}  // namespace mozilla
```

**Memory Layout:**
- Type: `const char[256]`
- Size: 256 bytes
- Alignment: 1 byte (char alignment)
- Linkage: External with C++ name mangling

**Usage Pattern:**
- Read-only array access: `detail::gTwoCharEscapes[u]` where `u` is uint8_t
- Used for conditional checks: `if (detail::gTwoCharEscapes[u])` - non-zero means escape needed
- Used to get escape character: `mOwnedStr[i++] = detail::gTwoCharEscapes[u]`

## Dependencies:

### Direct includes (JSONWriter.cpp):
```cpp
#include "mozilla/JSONWriter.h"  // Self-header, defines extern declaration
```

### Direct includes (JSONWriter.h - where table is used):
```cpp
#include "double-conversion/double-conversion.h"  // External library for double formatting
#include "mozilla/Assertions.h"                    // MOZ_ASSERT
#include "mozilla/IntegerPrintfMacros.h"           // Integer formatting
#include "mozilla/PodOperations.h"                 // POD utilities
#include "mozilla/Span.h"                          // Span<T> type
#include "mozilla/Sprintf.h"                       // String formatting
#include "mozilla/UniquePtr.h"                     // Smart pointers
#include "mozilla/Vector.h"                        // Vector container
#include <utility>                                 // std::move
```

**Note:** The gTwoCharEscapes table itself has NO dependencies. It's pure static data.

### Indirect dependencies:
- None - the table is self-contained

### External libraries:
- None for the table itself
- JSONWriter.h depends on double-conversion library, but gTwoCharEscapes doesn't

## Call Sites (total: 4 uses in JSONWriter.h):

### 1. mfbt/JSONWriter.h:120 - Extern declaration
```cpp
namespace detail {
extern MFBT_DATA const char gTwoCharEscapes[256];
}  // namespace detail
```
**Context:** Forward declaration for use in EscapedString class

### 2. mfbt/JSONWriter.h:171 - First escape check
```cpp
if (detail::gTwoCharEscapes[u]) {
  nExtra += 1;  // Count characters that need escaping
}
```
**Context:** First pass to calculate how many extra bytes are needed for escaping

### 3. mfbt/JSONWriter.h:195 - Second escape check
```cpp
if (detail::gTwoCharEscapes[u]) {
  mOwnedStr[i++] = '\\';
  mOwnedStr[i++] = detail::gTwoCharEscapes[u];  // Use next line
}
```
**Context:** Second pass - actually perform the escaping

### 4. mfbt/JSONWriter.h:197 - Get escape character
```cpp
mOwnedStr[i++] = detail::gTwoCharEscapes[u];
```
**Context:** Write the escape character (e.g., 'n' for '\n')

## Test Coverage (tests remain in C++):

### Unit tests: mfbt/tests/TestJSONWriter.cpp (665 lines)
- **Test functions**: 8 test functions
  - `TestBasicProperties()` - Tests null, bool, int, double, string properties
  - `TestVeryLongString()` - Tests strings exceeding buffer size
  - `TestIndentation()` - Tests pretty-printing with various indent levels
  - `TestEscaping()` - **PRIMARY TEST for gTwoCharEscapes** - Tests all escape sequences
  - `TestStringObjectWithEscaping()` - Tests escaped strings in objects
  - `TestAllWhitespaceInlineOnlyAndWithoutIndent()` - Tests inline formatting
  - `TestShortInlineAndInline()` - Tests mixed formatting
  - `TestSpanProperties()` - Tests Span-based string properties

### Key escape test scenarios (TestEscaping):
```cpp
// Tests all characters that need two-char escapes:
w.StringProperty("string", "\" \\ \x07 \b \t \n \x0b \f \r");

// Expected output validates gTwoCharEscapes table:
// "string": "\" \\ \u0007 \b \t \n \u000b \f \r"
//           ^^  ^^         ^^ ^^ ^^ ^^      ^^ ^^
//           Two-char escapes from gTwoCharEscapes table
```

### Integration tests:
- Used indirectly in:
  - `memory/replace/dmd/test/SmokeDMD.cpp` - Memory profiling output
  - `mozglue/tests/TestBaseProfiler.cpp` - Profiler JSON output
  - `tools/profiler/tests/gtest/GeckoProfiler.cpp` - Profiler output
  - Numerous other components that generate JSON output

### Coverage estimate: ~95%
- All escape sequences comprehensively tested
- Edge cases (empty strings, long strings, nested objects) covered
- Control characters (0x00-0x1F) tested
- Escape characters (", \, control chars) tested
- **Note**: All tests will continue calling through Rust FFI

## Table Structure Analysis:

### Populated entries (characters requiring two-char escapes):
```
Index (hex) | ASCII | Escape | Description
------------|-------|--------|-------------
0x08        | \b    | 'b'    | Backspace
0x09        | \t    | 't'    | Tab
0x0A        | \n    | 'n'    | Newline
0x0C        | \f    | 'f'    | Form feed
0x0D        | \r    | 'r'    | Carriage return
0x22        | "     | '"'    | Double quote
0x5C        | \     | '\'    | Backslash
```

### All other entries (250 of 256):
- Value: 0 (null character)
- Meaning: Either character doesn't need escaping (0x20-0x7E except " and \)
           or needs \uXXXX six-char escape (0x00-0x1F except those listed above)

### Memory layout verification:
```
Total size: 256 bytes
Alignment: 1 byte (char)
Entries: 256 (complete ASCII table)
Non-zero entries: 7 (as listed above)
Zero entries: 249
```

## Memory & Threading:

### Ownership model:
- **Static const data** - no ownership, never freed
- Defined once at compile time with static storage duration
- Lives for entire program lifetime

### Thread safety:
- **Thread-safe** - const data, read-only access
- No writes, no synchronization needed
- Multiple threads can safely read simultaneously
- No mutex or atomic operations required

### Resource cleanup:
- **No cleanup needed** - static storage duration
- Automatically deallocated at program termination
- No manual memory management required

## JSON Escape Sequence Specification (RFC 4627):

From the RFC and code comments:
- All characters 0x00-0x1F (control characters) must be escaped
- Two-char escapes preferred when available: \", \\, \b, \f, \n, \r, \t
- Six-char \uXXXX format used for other control characters
- Regular printable characters (0x20-0x7E except " and \) need no escaping

## Implementation Notes:

### Original C++ implementation:
```cpp
#define ___ 0
const char gTwoCharEscapes[256] = {
    /*          0    1    2    3    4    5    6    7    8    9 */
    /*   0+ */ ___, ___, ___,  ___, ___, ___, ___, ___, 'b', 't',
    /*  10+ */ 'n', ___, 'f',  'r', ___, ___, ___, ___, ___, ___,
    //  ... (pattern continues)
    /*  30+ */ ___, ___, ___,  ___, '"', ___, ___, ___, ___, ___,
    //  ... (pattern continues)
    /*  90+ */ ___, ___, '\\', ___, ___, ___, ___, ___, ___, ___,
    //  ... (pattern continues)
};
#undef ___
```

### Rust implementation considerations:
- Use `const` array with explicit initialization
- Maintain exact byte-for-byte compatibility
- Use `#[repr(C)]` for memory layout guarantee
- Export via `#[no_mangle]` for C++ access
- Consider using match or lookup for clarity vs. direct array init

## FFI Requirements:

### C++ access pattern:
```cpp
uint8_t u = static_cast<uint8_t>(c);
if (detail::gTwoCharEscapes[u]) {
    // Character needs escaping
    char escapeChar = detail::gTwoCharEscapes[u];
    // Use escapeChar
}
```

### Rust FFI must provide:
1. Symbol name: `_ZN7mozilla6detail15gTwoCharEscapesE` (C++ mangled)
   OR: `mozilla_detail_gTwoCharEscapes` (C compatible)
2. Type: `[u8; 256]` or `[i8; 256]` (depends on platform char signedness)
3. Linkage: `extern "C"` with `#[no_mangle]`
4. Visibility: Public, available to C++ linker
5. Lifetime: `'static` - must live for program duration

### Platform considerations:
- **char signedness**: C++ `char` may be signed or unsigned (platform-dependent)
- Rust should use `i8` or `u8` based on target platform
- Values 0-127 work identically for both signed/unsigned
- No values ≥128 in table, so signedness doesn't matter here

## Port Strategy:

### What to port:
- ✅ gTwoCharEscapes[256] const array
- ✅ FFI export for C++ access
- ✅ cbindgen header generation

### What NOT to port:
- ❌ JSONWriter.h (545 lines, complex header-only template code)
- ❌ JSONWriter class and EscapedString class (header-only)
- ❌ TestJSONWriter.cpp (test file remains in C++, will call via FFI)

### Success criteria:
1. Rust module exports gTwoCharEscapes with C linkage
2. C++ code in JSONWriter.h can access table via FFI
3. TestJSONWriter.cpp passes 100% (8/8 tests)
4. No changes to test files required
5. Byte-for-byte identical table layout

## Compilation Model:

### Current C++ build:
```
mfbt/JSONWriter.cpp  →  JSONWriter.o  →  libmozglue.so
                                           (or libxul.so)
```

### Proposed Rust build:
```
local/rust/firefox_jsonwriter/  →  libfirefox_jsonwriter.rlib  →  libmozglue.so
   src/lib.rs                                                        (or libxul.so)
   src/ffi.rs
```

### Conditional compilation:
```cpp
#ifdef MOZ_RUST_JSONWRITER
  // Use Rust implementation (link against Rust library)
  extern "C" const char mozilla_detail_gTwoCharEscapes[256];
  namespace mozilla::detail {
    const char* const gTwoCharEscapes = mozilla_detail_gTwoCharEscapes;
  }
#else
  // Use C++ implementation (current)
  namespace mozilla::detail {
    const char gTwoCharEscapes[256] = { /* ... */ };
  }
#endif
```

## Risk Mitigation:

### Compilation verification:
- Use `static_assert(sizeof(gTwoCharEscapes) == 256)` in Rust (compile-time check)
- Verify table contents match with integration test

### Testing verification:
- Run TestJSONWriter.cpp with both C++ and Rust backends
- Compare outputs byte-for-byte
- Ensure all 8 test functions pass with Rust table

### Documentation:
- README.md in Rust module documenting table layout
- Comments explaining each escape mapping
- Link to RFC 4627 specification
- Note that tests remain in C++

## Summary:

The gTwoCharEscapes table is an ideal candidate for Rust porting:
- **Simplicity**: Pure static const data, no logic
- **Isolation**: Self-contained, no dependencies
- **Testability**: Comprehensive test coverage via TestJSONWriter.cpp
- **Stability**: Core JSON escaping functionality, stable for years
- **FFI-friendly**: Simple array access pattern, no complex types

The port will establish a pattern for providing static data from Rust to C++, demonstrating zero-overhead FFI for read-only tables.

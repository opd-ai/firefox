//! Firefox JSONWriter - gTwoCharEscapes Lookup Table
//!
//! This module provides a Rust implementation of the JSON character escape lookup table
//! originally defined in mfbt/JSONWriter.cpp. The table maps ASCII characters to their
//! two-character JSON escape sequences per RFC 4627.
//!
//! # Background
//!
//! JSON strings must escape certain characters:
//! - Control characters (U+0000 through U+001F) must be escaped
//! - Quote marks (") and backslashes (\) must be escaped
//!
//! This implementation uses two-character escape sequences where possible:
//! - `\b` (backspace, 0x08)
//! - `\t` (tab, 0x09)
//! - `\n` (newline, 0x0A)
//! - `\f` (form feed, 0x0C)
//! - `\r` (carriage return, 0x0D)
//! - `\"` (quote, 0x22)
//! - `\\` (backslash, 0x5C)
//!
//! All other control characters use six-character `\uXXXX` format.
//!
//! # Table Structure
//!
//! The `gTwoCharEscapes` table is a 256-byte array where:
//! - Non-zero values indicate the character needs a two-char escape
//!   (the value is the second character of the escape sequence)
//! - Zero values indicate either:
//!   - No escaping needed (printable ASCII 0x20-0x7E except " and \)
//!   - Six-char `\uXXXX` escape needed (other control characters)
//!
//! # FFI Interface
//!
//! The table is exposed to C++ via FFI as `mozilla_detail_gTwoCharEscapes`.
//! C++ code in `mfbt/JSONWriter.h` accesses this table for JSON string escaping.
//!
//! # Example Usage (from C++):
//!
//! ```cpp
//! uint8_t u = static_cast<uint8_t>(someChar);
//! if (mozilla::detail::gTwoCharEscapes[u]) {
//!     // Character needs two-char escape
//!     char escapeChar = mozilla::detail::gTwoCharEscapes[u];
//!     // Write '\\' followed by escapeChar
//! }
//! ```
//!
//! # Memory Layout
//!
//! - Size: 256 bytes (complete ASCII table)
//! - Alignment: 1 byte (char alignment)
//! - Lifetime: Static (lives for program duration)
//! - Thread-safety: Read-only, no synchronization needed

/// JSON Two-Character Escape Lookup Table
///
/// Maps ASCII character codes (0-255) to their two-character JSON escape sequences.
/// Non-zero entries indicate the character requires escaping; the value is the
/// second character of the escape sequence (after the backslash).
///
/// # Populated Entries (7 total):
/// - `0x08` ('\b') → `'b'`
/// - `0x09` ('\t') → `'t'`
/// - `0x0A` ('\n') → `'n'`
/// - `0x0C` ('\f') → `'f'`
/// - `0x0D` ('\r') → `'r'`
/// - `0x22` ('"')  → `'"'`
/// - `0x5C` ('\\') → `'\\'`
///
/// # Zero Entries (249 total):
/// - Regular printable characters (no escaping needed)
/// - Control characters requiring `\uXXXX` format
///
/// # Thread Safety
/// This is a const lookup table, safe for concurrent access from multiple threads.
pub static TWO_CHAR_ESCAPES: [i8; 256] = [
    // Row 0 (0x00-0x09): Control characters
    // 0x00-0x07: NULL through BEL - no two-char escape (use \uXXXX)
    0, 0, 0, 0, 0, 0, 0, 0,
    // 0x08: Backspace (\b)
    b'b' as i8,
    // 0x09: Tab (\t)
    b't' as i8,
    
    // Row 1 (0x0A-0x13): More control characters
    // 0x0A: Newline (\n)
    b'n' as i8,
    // 0x0B: Vertical tab - no two-char escape
    0,
    // 0x0C: Form feed (\f)
    b'f' as i8,
    // 0x0D: Carriage return (\r)
    b'r' as i8,
    // 0x0E-0x13: Shift out through DC3 - no two-char escapes
    0, 0, 0, 0, 0, 0,
    
    // Row 2 (0x14-0x1D): Control characters
    // 0x14-0x1D: DC4 through GS - no two-char escapes
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    
    // Row 3 (0x1E-0x27): Last control chars and start of printable ASCII
    // 0x1E-0x1F: RS, US - no two-char escapes
    0, 0,
    // 0x20-0x21: Space, ! - no escaping needed
    0, 0,
    // 0x22: Double quote (\")
    b'"' as i8,
    // 0x23-0x27: #, $, %, &, ' - no escaping needed
    0, 0, 0, 0, 0,
    
    // Rows 4-9 (0x28-0x5B): Printable ASCII
    // 0x28-0x5B: ( through [ - no escaping needed
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,  // 0x28-0x31
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,  // 0x32-0x3B
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,  // 0x3C-0x45
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,  // 0x46-0x4F
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,  // 0x50-0x59
    0, 0,                          // 0x5A-0x5B
    
    // Row 10 (0x5C-0x65): Backslash and more printable ASCII
    // 0x5C: Backslash (\\)
    b'\\' as i8,
    // 0x5D-0x65: ] through e - no escaping needed
    0, 0, 0, 0, 0, 0, 0, 0, 0,
    
    // Rows 11-25 (0x66-0xFF): Rest of printable ASCII and extended ASCII
    // All no escaping needed (extended ASCII not used in JSON strings typically)
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,  // 0x66-0x6F
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,  // 0x70-0x79
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,  // 0x7A-0x83
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,  // 0x84-0x8D
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,  // 0x8E-0x97
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,  // 0x98-0xA1
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,  // 0xA2-0xAB
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,  // 0xAC-0xB5
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,  // 0xB6-0xBF
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,  // 0xC0-0xC9
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,  // 0xCA-0xD3
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,  // 0xD4-0xDD
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,  // 0xDE-0xE7
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,  // 0xE8-0xF1
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,  // 0xF2-0xFB
    0, 0, 0, 0,                    // 0xFC-0xFF
];

/// Compile-time verification that the table is exactly 256 bytes
const _: () = assert!(std::mem::size_of_val(&TWO_CHAR_ESCAPES) == 256);

// FFI exports for C++ interop
pub mod ffi;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_size() {
        assert_eq!(TWO_CHAR_ESCAPES.len(), 256);
        assert_eq!(std::mem::size_of_val(&TWO_CHAR_ESCAPES), 256);
    }

    #[test]
    fn test_escape_mappings() {
        // Test all seven two-char escapes
        assert_eq!(TWO_CHAR_ESCAPES[0x08], b'b' as i8);  // \b
        assert_eq!(TWO_CHAR_ESCAPES[0x09], b't' as i8);  // \t
        assert_eq!(TWO_CHAR_ESCAPES[0x0A], b'n' as i8);  // \n
        assert_eq!(TWO_CHAR_ESCAPES[0x0C], b'f' as i8);  // \f
        assert_eq!(TWO_CHAR_ESCAPES[0x0D], b'r' as i8);  // \r
        assert_eq!(TWO_CHAR_ESCAPES[0x22], b'"' as i8);  // \"
        assert_eq!(TWO_CHAR_ESCAPES[0x5C], b'\\' as i8); // \\
    }

    #[test]
    fn test_no_other_escapes() {
        // Verify all other control characters (0x00-0x1F except the 7 above) are zero
        let control_chars_no_escape = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,  // NULL through BEL
            // 0x08 is \b
            // 0x09 is \t
            // 0x0A is \n
            0x0B,  // Vertical tab
            // 0x0C is \f
            // 0x0D is \r
            0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15,  // SO through NAK
            0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D,  // SYN through GS
            0x1E, 0x1F,  // RS, US
        ];

        for &ch in &control_chars_no_escape {
            assert_eq!(
                TWO_CHAR_ESCAPES[ch as usize], 0,
                "Control character 0x{:02X} should not have two-char escape", ch
            );
        }
    }

    #[test]
    fn test_printable_ascii_no_escape() {
        // Test printable ASCII (0x20-0x7E) except " and \
        for ch in 0x20u8..=0x7Eu8 {
            if ch == 0x22 || ch == 0x5C {
                // Skip " and \, they have escapes
                continue;
            }
            assert_eq!(
                TWO_CHAR_ESCAPES[ch as usize], 0,
                "Printable character '{}' (0x{:02X}) should not need escaping",
                ch as char, ch
            );
        }
    }

    #[test]
    fn test_extended_ascii_no_escape() {
        // Test extended ASCII (0x7F-0xFF) - none should have two-char escapes
        for ch in 0x7Fu8..=0xFFu8 {
            assert_eq!(
                TWO_CHAR_ESCAPES[ch as usize], 0,
                "Extended ASCII character 0x{:02X} should not have two-char escape", ch
            );
        }
    }

    #[test]
    fn test_escape_char_values() {
        // Verify escape characters are valid ASCII letters
        assert!(TWO_CHAR_ESCAPES[0x08] > 0);  // 'b'
        assert!(TWO_CHAR_ESCAPES[0x09] > 0);  // 't'
        assert!(TWO_CHAR_ESCAPES[0x0A] > 0);  // 'n'
        assert!(TWO_CHAR_ESCAPES[0x0C] > 0);  // 'f'
        assert!(TWO_CHAR_ESCAPES[0x0D] > 0);  // 'r'
        assert!(TWO_CHAR_ESCAPES[0x22] > 0);  // '"'
        assert!(TWO_CHAR_ESCAPES[0x5C] > 0);  // '\'
    }

    #[test]
    fn test_only_seven_escapes() {
        // Count non-zero entries - should be exactly 7
        let count = TWO_CHAR_ESCAPES.iter().filter(|&&x| x != 0).count();
        assert_eq!(count, 7, "Table should have exactly 7 non-zero entries");
    }

    #[test]
    fn test_escape_usage_pattern() {
        // Simulate the usage pattern from JSONWriter.h
        let test_string = b"Hello\tWorld\n\"Quoted\"\x0B\\Path";
        let mut escaped_count = 0;

        for &byte in test_string {
            if TWO_CHAR_ESCAPES[byte as usize] != 0 {
                escaped_count += 1;
                // In real code: output '\\' followed by TWO_CHAR_ESCAPES[byte]
            }
        }

        // Count expected escapes: \t, \n, \", \", \\
        // \x0B (vertical tab) doesn't have two-char escape
        assert_eq!(escaped_count, 5, "Should find 5 escapable characters");
    }

    #[test]
    fn test_json_spec_compliance() {
        // From RFC 4627, these are the mandatory two-char escapes that exist:
        // \" \\ \/ \b \f \n \r \t
        // (Note: \/ is optional in practice, we don't use it)
        
        // We implement 7 of them (excluding \/ which is not in gTwoCharEscapes)
        assert_eq!(TWO_CHAR_ESCAPES[b'"' as usize], b'"' as i8);   // \"
        assert_eq!(TWO_CHAR_ESCAPES[b'\\' as usize], b'\\' as i8); // \\
        assert_eq!(TWO_CHAR_ESCAPES[b'\x08' as usize], b'b' as i8); // \b
        assert_eq!(TWO_CHAR_ESCAPES[b'\x0C' as usize], b'f' as i8); // \f
        assert_eq!(TWO_CHAR_ESCAPES[b'\n' as usize], b'n' as i8);   // \n
        assert_eq!(TWO_CHAR_ESCAPES[b'\r' as usize], b'r' as i8);   // \r
        assert_eq!(TWO_CHAR_ESCAPES[b'\t' as usize], b't' as i8);   // \t
        
        // Forward slash (/) does NOT have a two-char escape in our implementation
        assert_eq!(TWO_CHAR_ESCAPES[b'/' as usize], 0);
    }
}

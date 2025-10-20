/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Firefox ASCIIMask - Rust Port
//!
//! This module provides compile-time boolean lookup tables for fast ASCII character
//! classification. It's a direct port of xpcom/string/nsASCIIMask.cpp.
//!
//! ## Original C++ Implementation
//! - File: xpcom/string/nsASCIIMask.cpp (38 lines)
//! - Purpose: Fast ASCII character set membership testing
//! - Pattern: Static const data - 4 boolean arrays (128 bytes each)
//!
//! ## Usage
//! ```rust
//! use firefox_asciimask::*;
//!
//! // Check if a character is whitespace
//! let c = b' ';
//! if c < 128 && WHITESPACE_MASK[c as usize] {
//!     println!("Character is whitespace");
//! }
//!
//! // Or use the helper
//! if is_masked(&WHITESPACE_MASK, ' ' as u8) {
//!     println!("Character is whitespace");
//! }
//! ```
//!
//! ## Architecture
//! - Pure const data (no heap allocation, no runtime initialization)
//! - Thread-safe (immutable data)
//! - Cache-friendly (128-byte arrays fit in L1 cache)
//! - Zero-cost abstraction (array access compiles to single memory load)

#![no_std]

pub mod ffi;

/// Type alias for ASCII mask arrays (128 booleans, one per ASCII character)
pub type ASCIIMaskArray = [bool; 128];

// ============================================================================
// Compile-Time Mask Generation
// ============================================================================

/// Helper macro to create ASCII mask at compile time.
/// Generates an array of 128 booleans by calling test function for each index.
macro_rules! create_mask {
    ($test:expr) => {{
        [
            $test(0), $test(1), $test(2), $test(3), $test(4), $test(5), $test(6), $test(7),
            $test(8), $test(9), $test(10), $test(11), $test(12), $test(13), $test(14), $test(15),
            $test(16), $test(17), $test(18), $test(19), $test(20), $test(21), $test(22), $test(23),
            $test(24), $test(25), $test(26), $test(27), $test(28), $test(29), $test(30), $test(31),
            $test(32), $test(33), $test(34), $test(35), $test(36), $test(37), $test(38), $test(39),
            $test(40), $test(41), $test(42), $test(43), $test(44), $test(45), $test(46), $test(47),
            $test(48), $test(49), $test(50), $test(51), $test(52), $test(53), $test(54), $test(55),
            $test(56), $test(57), $test(58), $test(59), $test(60), $test(61), $test(62), $test(63),
            $test(64), $test(65), $test(66), $test(67), $test(68), $test(69), $test(70), $test(71),
            $test(72), $test(73), $test(74), $test(75), $test(76), $test(77), $test(78), $test(79),
            $test(80), $test(81), $test(82), $test(83), $test(84), $test(85), $test(86), $test(87),
            $test(88), $test(89), $test(90), $test(91), $test(92), $test(93), $test(94), $test(95),
            $test(96), $test(97), $test(98), $test(99), $test(100), $test(101), $test(102), $test(103),
            $test(104), $test(105), $test(106), $test(107), $test(108), $test(109), $test(110), $test(111),
            $test(112), $test(113), $test(114), $test(115), $test(116), $test(117), $test(118), $test(119),
            $test(120), $test(121), $test(122), $test(123), $test(124), $test(125), $test(126), $test(127),
        ]
    }};
}

// ============================================================================
// Character Test Predicates
// ============================================================================

/// Test if character is whitespace: \f, \t, \r, \n, or space
const fn is_whitespace(c: u8) -> bool {
    c == b'\x0C' || // \f (form feed)
    c == b'\t' ||
    c == b'\r' ||
    c == b'\n' ||
    c == b' '
}

/// Test if character is CRLF: \r or \n
const fn is_crlf(c: u8) -> bool {
    c == b'\r' || c == b'\n'
}

/// Test if character is CRLF or tab: \r, \n, or \t
const fn is_crlf_tab(c: u8) -> bool {
    c == b'\r' || c == b'\n' || c == b'\t'
}

/// Test if character is a digit: 0-9
const fn is_zero_to_nine(c: u8) -> bool {
    c == b'0' || c == b'1' || c == b'2' || c == b'3' || c == b'4' ||
    c == b'5' || c == b'6' || c == b'7' || c == b'8' || c == b'9'
}

// ============================================================================
// Static Mask Arrays (Compile-Time Initialized)
// ============================================================================

/// Mask for whitespace characters: \f, \t, \r, \n, space
pub static WHITESPACE_MASK: ASCIIMaskArray = create_mask!(is_whitespace);

/// Mask for CRLF characters: \r, \n
pub static CRLF_MASK: ASCIIMaskArray = create_mask!(is_crlf);

/// Mask for CRLF and tab characters: \r, \n, \t
pub static CRLF_TAB_MASK: ASCIIMaskArray = create_mask!(is_crlf_tab);

/// Mask for digit characters: 0-9
pub static ZERO_TO_NINE_MASK: ASCIIMaskArray = create_mask!(is_zero_to_nine);

// ============================================================================
// Helper Functions
// ============================================================================

/// Check if a character is masked (with bounds checking)
///
/// Equivalent to C++: `aChar < 128 && aMask[aChar]`
///
/// # Arguments
/// * `mask` - The ASCII mask array to check against
/// * `ch` - The character to test (any u8, but only ASCII range is valid)
///
/// # Returns
/// `true` if ch < 128 and mask[ch] is true, `false` otherwise
///
/// # Examples
/// ```
/// use firefox_asciimask::*;
///
/// assert!(is_masked(&CRLF_MASK, b'\n'));
/// assert!(is_masked(&CRLF_MASK, b'\r'));
/// assert!(!is_masked(&CRLF_MASK, b'a'));
/// assert!(!is_masked(&CRLF_MASK, 200)); // > 127, always false
/// ```
#[inline(always)]
pub fn is_masked(mask: &ASCIIMaskArray, ch: u8) -> bool {
    ch < 128 && mask[ch as usize]
}

// ============================================================================
// Compile-Time Assertions
// ============================================================================

const _: () = {
    // Ensure ASCIIMaskArray is exactly 128 bytes
    assert!(core::mem::size_of::<ASCIIMaskArray>() == 128);
    
    // Verify masks are populated correctly (spot checks)
    assert!(WHITESPACE_MASK[b' ' as usize]);
    assert!(WHITESPACE_MASK[b'\t' as usize]);
    assert!(WHITESPACE_MASK[b'\n' as usize]);
    assert!(WHITESPACE_MASK[b'\r' as usize]);
    assert!(WHITESPACE_MASK[b'\x0C' as usize]); // \f
    
    assert!(CRLF_MASK[b'\n' as usize]);
    assert!(CRLF_MASK[b'\r' as usize]);
    assert!(!CRLF_MASK[b' ' as usize]);
    
    assert!(CRLF_TAB_MASK[b'\n' as usize]);
    assert!(CRLF_TAB_MASK[b'\r' as usize]);
    assert!(CRLF_TAB_MASK[b'\t' as usize]);
    assert!(!CRLF_TAB_MASK[b' ' as usize]);
    
    assert!(ZERO_TO_NINE_MASK[b'0' as usize]);
    assert!(ZERO_TO_NINE_MASK[b'9' as usize]);
    assert!(ZERO_TO_NINE_MASK[b'5' as usize]);
    assert!(!ZERO_TO_NINE_MASK[b'a' as usize]);
};

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_size() {
        assert_eq!(core::mem::size_of::<ASCIIMaskArray>(), 128);
    }

    #[test]
    fn test_whitespace_mask() {
        // Whitespace characters
        assert!(WHITESPACE_MASK[b' ' as usize]);
        assert!(WHITESPACE_MASK[b'\t' as usize]);
        assert!(WHITESPACE_MASK[b'\n' as usize]);
        assert!(WHITESPACE_MASK[b'\r' as usize]);
        assert!(WHITESPACE_MASK[b'\x0C' as usize]); // \f
        
        // Non-whitespace characters
        assert!(!WHITESPACE_MASK[b'a' as usize]);
        assert!(!WHITESPACE_MASK[b'0' as usize]);
        assert!(!WHITESPACE_MASK[0]);
    }

    #[test]
    fn test_crlf_mask() {
        // CRLF characters
        assert!(CRLF_MASK[b'\n' as usize]);
        assert!(CRLF_MASK[b'\r' as usize]);
        
        // Non-CRLF characters
        assert!(!CRLF_MASK[b' ' as usize]);
        assert!(!CRLF_MASK[b'\t' as usize]);
        assert!(!CRLF_MASK[b'a' as usize]);
        assert!(!CRLF_MASK[0]);
    }

    #[test]
    fn test_crlf_tab_mask() {
        // CRLF + tab characters
        assert!(CRLF_TAB_MASK[b'\n' as usize]);
        assert!(CRLF_TAB_MASK[b'\r' as usize]);
        assert!(CRLF_TAB_MASK[b'\t' as usize]);
        
        // Non-CRLF-tab characters
        assert!(!CRLF_TAB_MASK[b' ' as usize]);
        assert!(!CRLF_TAB_MASK[b'a' as usize]);
        assert!(!CRLF_TAB_MASK[0]);
    }

    #[test]
    fn test_zero_to_nine_mask() {
        // Digit characters
        assert!(ZERO_TO_NINE_MASK[b'0' as usize]);
        assert!(ZERO_TO_NINE_MASK[b'1' as usize]);
        assert!(ZERO_TO_NINE_MASK[b'5' as usize]);
        assert!(ZERO_TO_NINE_MASK[b'9' as usize]);
        
        // Non-digit characters
        assert!(!ZERO_TO_NINE_MASK[b'a' as usize]);
        assert!(!ZERO_TO_NINE_MASK[b' ' as usize]);
        assert!(!ZERO_TO_NINE_MASK[0]);
    }

    #[test]
    fn test_is_masked_helper() {
        // Valid ASCII range
        assert!(is_masked(&CRLF_MASK, b'\n'));
        assert!(is_masked(&CRLF_MASK, b'\r'));
        assert!(!is_masked(&CRLF_MASK, b'a'));
        
        // Boundary: 127 is valid ASCII
        assert!(!is_masked(&CRLF_MASK, 127));
        
        // Out of range: > 127 always returns false
        assert!(!is_masked(&CRLF_MASK, 128));
        assert!(!is_masked(&CRLF_MASK, 200));
        assert!(!is_masked(&CRLF_MASK, 255));
    }

    #[test]
    fn test_all_digits() {
        for c in b'0'..=b'9' {
            assert!(ZERO_TO_NINE_MASK[c as usize], "Digit {} should be masked", c as char);
            assert!(is_masked(&ZERO_TO_NINE_MASK, c), "is_masked should work for {}", c as char);
        }
    }

    #[test]
    fn test_all_whitespace() {
        let whitespace_chars = [b' ', b'\t', b'\n', b'\r', b'\x0C'];
        for &c in &whitespace_chars {
            assert!(WHITESPACE_MASK[c as usize], "Whitespace char 0x{:02X} should be masked", c);
            assert!(is_masked(&WHITESPACE_MASK, c), "is_masked should work for 0x{:02X}", c);
        }
    }
}

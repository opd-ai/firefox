// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Rust port of nsCRT::atoll - ASCII string to int64 conversion
//!
//! This module provides a drop-in replacement for Firefox's `nsCRT::atoll()`
//! function, which converts ASCII decimal digit strings to 64-bit signed integers.
//!
//! # Behavior
//!
//! The function matches the C++ implementation exactly:
//! - Returns 0 on null input
//! - Parses decimal digits (0-9) only
//! - Stops at first non-digit character
//! - Does NOT handle signs ('+' or '-')
//! - Does NOT skip whitespace
//! - Does NOT check for overflow (wraps on overflow)
//!
//! # Examples
//!
//! ```
//! use firefox_atoll::atoll;
//!
//! assert_eq!(atoll("123"), 123);
//! assert_eq!(atoll("456abc"), 456);  // Stops at 'a'
//! assert_eq!(atoll("abc123"), 0);     // Stops at 'a'
//! assert_eq!(atoll("  123"), 0);      // Stops at space
//! assert_eq!(atoll("-123"), 0);       // Stops at '-'
//! ```

/// Converts an ASCII string of decimal digits to an i64.
///
/// This function mimics the exact behavior of the C++ `nsCRT::atoll()`:
/// - Null-terminated string parsing
/// - Stops at first non-digit (0-9)
/// - No sign handling
/// - No whitespace skipping
/// - No overflow detection (wraps per Rust semantics)
///
/// # Arguments
///
/// * `s` - A string slice containing ASCII characters
///
/// # Returns
///
/// * `i64` - The parsed integer value, or 0 if:
///   - String is empty
///   - First character is not a digit
///   - Any other parsing failure
///
/// # Examples
///
/// ```
/// use firefox_atoll::atoll;
///
/// assert_eq!(atoll("0"), 0);
/// assert_eq!(atoll("123"), 123);
/// assert_eq!(atoll("9223372036854775807"), 9223372036854775807); // INT64_MAX
/// ```
#[inline]
pub fn atoll(s: &str) -> i64 {
    let bytes = s.as_bytes();
    
    // Empty string returns 0
    if bytes.is_empty() {
        return 0;
    }
    
    let mut result: i64 = 0;
    
    for &byte in bytes {
        // Check if character is a digit (0-9)
        if byte >= b'0' && byte <= b'9' {
            // Use wrapping arithmetic to match C++ overflow behavior
            result = result.wrapping_mul(10);
            result = result.wrapping_add((byte - b'0') as i64);
        } else {
            // Stop at first non-digit (matching C++ behavior)
            break;
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_string() {
        assert_eq!(atoll(""), 0);
    }

    #[test]
    fn test_zero() {
        assert_eq!(atoll("0"), 0);
    }

    #[test]
    fn test_simple_positive() {
        assert_eq!(atoll("123"), 123);
        assert_eq!(atoll("456"), 456);
        assert_eq!(atoll("1"), 1);
        assert_eq!(atoll("42"), 42);
    }

    #[test]
    fn test_large_numbers() {
        assert_eq!(atoll("1234567890"), 1234567890);
        assert_eq!(atoll("9223372036854775807"), i64::MAX);  // INT64_MAX
    }

    #[test]
    fn test_stops_at_non_digit() {
        assert_eq!(atoll("123abc"), 123);
        assert_eq!(atoll("456def789"), 456);
        assert_eq!(atoll("42!"), 42);
    }

    #[test]
    fn test_leading_non_digit() {
        assert_eq!(atoll("abc123"), 0);
        assert_eq!(atoll("!456"), 0);
        assert_eq!(atoll("xyz"), 0);
    }

    #[test]
    fn test_whitespace_not_skipped() {
        assert_eq!(atoll("  123"), 0);  // Leading space stops parsing
        assert_eq!(atoll("123  "), 123); // Trailing space stops but value is captured
        assert_eq!(atoll(" "), 0);
        assert_eq!(atoll("\t123"), 0);
        assert_eq!(atoll("123\n"), 123);
    }

    #[test]
    fn test_signs_not_handled() {
        assert_eq!(atoll("-123"), 0);  // '-' is not a digit
        assert_eq!(atoll("+123"), 0);  // '+' is not a digit
        assert_eq!(atoll("- 123"), 0);
    }

    #[test]
    fn test_overflow_wraps() {
        // Test value just beyond INT64_MAX
        // "9223372036854775808" would overflow
        // In C++, this would wrap. We use wrapping_mul/wrapping_add to match.
        // Since we're testing exact behavior, we verify wrapping happens
        let overflow_str = "9223372036854775808"; // INT64_MAX + 1
        let result = atoll(overflow_str);
        // The result will wrap around (negative value)
        assert!(result < 0); // Wrapped around
    }

    #[test]
    fn test_very_long_number() {
        // Very long number will overflow and wrap
        let long_num = "99999999999999999999999999999";
        let result = atoll(long_num);
        // Just verify it doesn't panic - actual value depends on wrapping behavior
        let _ = result;
    }

    #[test]
    fn test_leading_zeros() {
        assert_eq!(atoll("0000"), 0);
        assert_eq!(atoll("00123"), 123);
        assert_eq!(atoll("007"), 7);
    }

    #[test]
    fn test_all_digits() {
        assert_eq!(atoll("0123456789"), 123456789);
    }

    #[test]
    fn test_single_digit() {
        for i in 0..=9 {
            let s = i.to_string();
            assert_eq!(atoll(&s), i as i64);
        }
    }
}

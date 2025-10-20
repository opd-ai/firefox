// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Rust port of nsCRT.cpp utility functions
//!
//! This module implements three string/number utility functions from Firefox's
//! nsCRT class:
//! - `strtok`: Thread-safe string tokenizer (modifies input in-place)
//! - `strcmp_char16`: UTF-16 string comparison  
//! - `atoll`: String to 64-bit integer conversion
//!
//! # Safety
//!
//! All functions handle null pointers safely and match C++ behavior exactly.

#![allow(non_camel_case_types)]

pub mod ffi;

use std::ptr;

const DELIM_TABLE_SIZE: usize = 32;

/// Build a delimiter lookup table for fast character checking.
/// Uses a bitmap where each bit represents whether a byte value is a delimiter.
///
/// # Algorithm
/// - Table is 32 bytes (256 bits), one bit per possible byte value
/// - Bit position = byte_value / 8, bit mask = 1 << (byte_value % 8)
/// - SET_DELIM sets the bit, IS_DELIM checks the bit
#[inline]
fn build_delim_table(delims: &[u8]) -> [u8; DELIM_TABLE_SIZE] {
    let mut table = [0u8; DELIM_TABLE_SIZE];
    for &ch in delims {
        // SET_DELIM: table[ch >> 3] |= (1 << (ch & 7))
        table[(ch >> 3) as usize] |= 1 << (ch & 7);
    }
    table
}

/// Check if a character is in the delimiter table.
#[inline]
fn is_delim(table: &[u8; DELIM_TABLE_SIZE], ch: u8) -> bool {
    // IS_DELIM: table[ch >> 3] & (1 << (ch & 7))
    (table[(ch >> 3) as usize] & (1 << (ch & 7))) != 0
}

/// Thread-safe string tokenizer (Rust implementation of nsCRT::strtok)
///
/// This function tokenizes a null-terminated C string by replacing delimiters
/// with '\0' and returning pointers to tokens.
///
/// # Safety
///
/// This function is unsafe because it:
/// - Dereferences raw pointers
/// - Modifies the input string in-place
/// - Assumes null-terminated strings
///
/// # Arguments
///
/// * `string` - Mutable pointer to the string to tokenize (must be non-null)
/// * `delims` - Pointer to null-terminated delimiter string
/// * `new_str` - Output pointer to update with continuation point
///
/// # Returns
///
/// Pointer to the next token, or null if no more tokens found.
///
/// # Behavior (matches C++):
///
/// 1. Build delimiter lookup table from `delims`
/// 2. Skip leading delimiters in `string`
/// 3. Mark token start
/// 4. Find next delimiter
/// 5. Replace delimiter with '\0'
/// 6. Update `new_str` to point after the '\0'
/// 7. Return token start (or null if no token found)
///
/// # Example (C-style usage):
///
/// ```c
/// char str[] = "a,b,c";
/// char* newStr;
/// char* token = nsCRT_strtok(str, ",", &newStr);
/// while (token != NULL) {
///     printf("%s\n", token);
///     token = nsCRT_strtok(newStr, ",", &newStr);
/// }
/// ```
pub unsafe fn strtok(
    string: *mut i8,
    delims: *const i8,
    new_str: *mut *mut i8,
) -> *mut i8 {
    debug_assert!(!string.is_null(), "string must not be null");
    
    if string.is_null() || delims.is_null() || new_str.is_null() {
        return ptr::null_mut();
    }

    // Build delimiter table
    let delim_table = {
        let mut delims_vec = Vec::new();
        let mut delim_ptr = delims;
        while *delim_ptr != 0 {
            delims_vec.push(*delim_ptr as u8);
            delim_ptr = delim_ptr.offset(1);
        }
        build_delim_table(&delims_vec)
    };

    // Skip to beginning (skip leading delimiters)
    let mut str_ptr = string;
    while *str_ptr != 0 && is_delim(&delim_table, *str_ptr as u8) {
        str_ptr = str_ptr.offset(1);
    }
    let result = str_ptr;

    // Fix up the end of the token
    while *str_ptr != 0 {
        if is_delim(&delim_table, *str_ptr as u8) {
            *str_ptr = 0; // Replace delimiter with null terminator
            str_ptr = str_ptr.offset(1);
            break;
        }
        str_ptr = str_ptr.offset(1);
    }

    // Update continuation pointer
    *new_str = str_ptr;

    // Return null if no token found (result == str_ptr means empty)
    if str_ptr == result {
        ptr::null_mut()
    } else {
        result
    }
}

/// UTF-16 string comparison (Rust implementation of nsCRT::strcmp for char16_t*)
///
/// Compares two null-terminated UTF-16 strings lexicographically.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers and assumes
/// null-terminated UTF-16 strings.
///
/// # Arguments
///
/// * `str1` - Pointer to first UTF-16 string
/// * `str2` - Pointer to second UTF-16 string
///
/// # Returns
///
/// - `-1` if str1 < str2
/// - `0` if str1 == str2
/// - `1` if str1 > str2
///
/// # Null Handling (matches C++ exactly):
///
/// - Both null → 0
/// - str1 null, str2 non-null → -1
/// - str1 non-null, str2 null → 1
///
/// # Example:
///
/// ```c
/// char16_t* s1 = u"hello";
/// char16_t* s2 = u"world";
/// int32_t result = nsCRT_strcmp_char16(s1, s2); // Returns -1
/// ```
pub unsafe fn strcmp_char16(str1: *const u16, str2: *const u16) -> i32 {
    // Handle null pointer cases
    if str1.is_null() && str2.is_null() {
        return 0;
    }
    if str1.is_null() {
        return -1;
    }
    if str2.is_null() {
        return 1;
    }

    // Character-by-character comparison
    let mut s1 = str1;
    let mut s2 = str2;
    loop {
        let c1 = *s1;
        let c2 = *s2;
        
        if c1 != c2 {
            return if c1 < c2 { -1 } else { 1 };
        }
        
        if c1 == 0 || c2 == 0 {
            break;
        }
        
        s1 = s1.offset(1);
        s2 = s2.offset(1);
    }

    0
}

/// String to 64-bit integer conversion (Rust implementation of nsCRT::atoll)
///
/// Parses a null-terminated C string as a decimal integer.
///
/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer and assumes
/// a null-terminated string.
///
/// # Arguments
///
/// * `str` - Pointer to null-terminated ASCII string
///
/// # Returns
///
/// The parsed integer value, or 0 if the string is null or contains no digits.
///
/// # Behavior (matches C++ exactly):
///
/// - Null pointer → 0
/// - Empty string → 0
/// - Parses digits from the start until first non-digit
/// - No overflow checking (matches C++ behavior)
/// - No sign handling (only positive integers, matches C++ implementation)
///
/// # Example:
///
/// ```c
/// char* str = "12345";
/// int64_t value = nsCRT_atoll(str); // Returns 12345
/// ```
pub unsafe fn atoll(str: *const i8) -> i64 {
    if str.is_null() {
        return 0;
    }

    let mut result: i64 = 0;
    let mut ptr = str;

    while *ptr != 0 && (*ptr as u8 >= b'0') && (*ptr as u8 <= b'9') {
        result = result.wrapping_mul(10);
        result = result.wrapping_add(((*ptr as u8) - b'0') as i64);
        ptr = ptr.offset(1);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_strtok_basic() {
        unsafe {
            let input = CString::new("a,b,c").unwrap().into_raw();
            let delims = CString::new(",").unwrap().into_raw();
            let mut new_str: *mut i8 = ptr::null_mut();

            let token1 = strtok(input, delims, &mut new_str);
            assert!(!token1.is_null());
            assert_eq!(std::ffi::CStr::from_ptr(token1).to_str().unwrap(), "a");

            let token2 = strtok(new_str, delims, &mut new_str);
            assert!(!token2.is_null());
            assert_eq!(std::ffi::CStr::from_ptr(token2).to_str().unwrap(), "b");

            let token3 = strtok(new_str, delims, &mut new_str);
            assert!(!token3.is_null());
            assert_eq!(std::ffi::CStr::from_ptr(token3).to_str().unwrap(), "c");

            let token4 = strtok(new_str, delims, &mut new_str);
            assert!(token4.is_null());

            let _ = CString::from_raw(input);
            let _ = CString::from_raw(delims);
        }
    }

    #[test]
    fn test_strtok_multiple_delimiters() {
        unsafe {
            let input = CString::new("a  b\tc").unwrap().into_raw();
            let delims = CString::new(" \t").unwrap().into_raw();
            let mut new_str: *mut i8 = ptr::null_mut();

            let token1 = strtok(input, delims, &mut new_str);
            assert_eq!(std::ffi::CStr::from_ptr(token1).to_str().unwrap(), "a");

            let token2 = strtok(new_str, delims, &mut new_str);
            assert_eq!(std::ffi::CStr::from_ptr(token2).to_str().unwrap(), "b");

            let token3 = strtok(new_str, delims, &mut new_str);
            assert_eq!(std::ffi::CStr::from_ptr(token3).to_str().unwrap(), "c");

            let _ = CString::from_raw(input);
            let _ = CString::from_raw(delims);
        }
    }

    #[test]
    fn test_strtok_leading_delimiters() {
        unsafe {
            let input = CString::new(",,a,b").unwrap().into_raw();
            let delims = CString::new(",").unwrap().into_raw();
            let mut new_str: *mut i8 = ptr::null_mut();

            let token1 = strtok(input, delims, &mut new_str);
            assert_eq!(std::ffi::CStr::from_ptr(token1).to_str().unwrap(), "a");

            let _ = CString::from_raw(input);
            let _ = CString::from_raw(delims);
        }
    }

    #[test]
    fn test_strcmp_char16_equal() {
        unsafe {
            let s1: Vec<u16> = "hello".encode_utf16().chain(std::iter::once(0)).collect();
            let s2: Vec<u16> = "hello".encode_utf16().chain(std::iter::once(0)).collect();
            assert_eq!(strcmp_char16(s1.as_ptr(), s2.as_ptr()), 0);
        }
    }

    #[test]
    fn test_strcmp_char16_less_than() {
        unsafe {
            let s1: Vec<u16> = "abc".encode_utf16().chain(std::iter::once(0)).collect();
            let s2: Vec<u16> = "xyz".encode_utf16().chain(std::iter::once(0)).collect();
            assert_eq!(strcmp_char16(s1.as_ptr(), s2.as_ptr()), -1);
        }
    }

    #[test]
    fn test_strcmp_char16_greater_than() {
        unsafe {
            let s1: Vec<u16> = "xyz".encode_utf16().chain(std::iter::once(0)).collect();
            let s2: Vec<u16> = "abc".encode_utf16().chain(std::iter::once(0)).collect();
            assert_eq!(strcmp_char16(s1.as_ptr(), s2.as_ptr()), 1);
        }
    }

    #[test]
    fn test_strcmp_char16_null_handling() {
        unsafe {
            let s: Vec<u16> = "hello".encode_utf16().chain(std::iter::once(0)).collect();
            
            // Both null
            assert_eq!(strcmp_char16(ptr::null(), ptr::null()), 0);
            
            // First null
            assert_eq!(strcmp_char16(ptr::null(), s.as_ptr()), -1);
            
            // Second null
            assert_eq!(strcmp_char16(s.as_ptr(), ptr::null()), 1);
        }
    }

    #[test]
    fn test_strcmp_char16_empty_strings() {
        unsafe {
            let s1: Vec<u16> = vec![0];
            let s2: Vec<u16> = vec![0];
            assert_eq!(strcmp_char16(s1.as_ptr(), s2.as_ptr()), 0);
        }
    }

    #[test]
    fn test_atoll_basic() {
        unsafe {
            let s = CString::new("12345").unwrap();
            assert_eq!(atoll(s.as_ptr()), 12345);
        }
    }

    #[test]
    fn test_atoll_zero() {
        unsafe {
            let s = CString::new("0").unwrap();
            assert_eq!(atoll(s.as_ptr()), 0);
        }
    }

    #[test]
    fn test_atoll_stops_at_non_digit() {
        unsafe {
            let s = CString::new("123abc").unwrap();
            assert_eq!(atoll(s.as_ptr()), 123);
        }
    }

    #[test]
    fn test_atoll_null_pointer() {
        unsafe {
            assert_eq!(atoll(ptr::null()), 0);
        }
    }

    #[test]
    fn test_atoll_no_digits() {
        unsafe {
            let s = CString::new("abc").unwrap();
            assert_eq!(atoll(s.as_ptr()), 0);
        }
    }

    #[test]
    fn test_atoll_empty_string() {
        unsafe {
            let s = CString::new("").unwrap();
            assert_eq!(atoll(s.as_ptr()), 0);
        }
    }

    #[test]
    fn test_build_delim_table() {
        let delims = b",;:";
        let table = build_delim_table(delims);
        
        assert!(is_delim(&table, b','));
        assert!(is_delim(&table, b';'));
        assert!(is_delim(&table, b':'));
        assert!(!is_delim(&table, b'a'));
        assert!(!is_delim(&table, b'0'));
    }
}

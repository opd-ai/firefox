// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Unit tests for UTF-8 validation
//!
//! These tests supplement the C++ tests in mfbt/tests/TestUtf8.cpp.
//! The C++ tests will call the Rust implementation via FFI and remain
//! the source of truth for behavioral compatibility.
//!
//! These Rust tests provide additional coverage and serve as
//! documentation for the expected behavior.

use crate::is_valid_utf8;

#[test]
fn test_empty_string() {
    assert!(is_valid_utf8(b""));
}

#[test]
fn test_ascii_only() {
    assert!(is_valid_utf8(b"Hello, world!"));
    assert!(is_valid_utf8(b"0123456789"));
    assert!(is_valid_utf8(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ"));
    assert!(is_valid_utf8(b"abcdefghijklmnopqrstuvwxyz"));
    assert!(is_valid_utf8(b" !\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~"));
}

#[test]
fn test_two_byte_sequences() {
    // é (U+00E9): C3 A9
    assert!(is_valid_utf8(&[0xC3, 0xA9]));
    assert!(is_valid_utf8("Café".as_bytes()));

    // ñ (U+00F1): C3 B1
    assert!(is_valid_utf8(&[0xC3, 0xB1]));

    // Range: U+0080 to U+07FF (2-byte UTF-8)
    // å (U+00E5): C3 A5
    assert!(is_valid_utf8(&[0xC3, 0xA5]));
}

#[test]
fn test_three_byte_sequences() {
    // € (U+20AC): E2 82 AC
    assert!(is_valid_utf8(&[0xE2, 0x82, 0xAC]));

    // ☕ (U+2615): E2 98 95
    assert!(is_valid_utf8(&[0xE2, 0x98, 0x95]));

    // 日 (U+65E5): E6 97 A5
    assert!(is_valid_utf8("日本語".as_bytes()));

    // 中 (U+4E2D): E4 B8 AD
    assert!(is_valid_utf8("中文".as_bytes()));
}

#[test]
fn test_four_byte_sequences() {
    // 🦀 (U+1F980, Rust crab): F0 9F A6 80
    assert!(is_valid_utf8("🦀".as_bytes()));

    // 𐍈 (U+10348, Gothic letter hwair): F0 90 8D 88
    assert!(is_valid_utf8(&[0xF0, 0x90, 0x8D, 0x88]));

    // 😀 (U+1F600): F0 9F 98 80
    assert!(is_valid_utf8("😀".as_bytes()));
}

#[test]
fn test_max_codepoint() {
    // U+10FFFF (max valid Unicode): F4 8F BF BF
    assert!(is_valid_utf8(&[0xF4, 0x8F, 0xBF, 0xBF]));
}

#[test]
fn test_invalid_beyond_max_codepoint() {
    // U+110000 (beyond Unicode range): F4 90 80 80
    assert!(!is_valid_utf8(&[0xF4, 0x90, 0x80, 0x80]));

    // U+200000 (way beyond): F8 88 80 80 80
    assert!(!is_valid_utf8(&[0xF8, 0x88, 0x80, 0x80, 0x80]));
}

#[test]
fn test_invalid_lead_bytes() {
    // 0xC0, 0xC1 are invalid lead bytes (would create overlong encodings)
    assert!(!is_valid_utf8(&[0xC0]));
    assert!(!is_valid_utf8(&[0xC1]));

    // 0xF5-0xFF are invalid lead bytes (beyond U+10FFFF)
    assert!(!is_valid_utf8(&[0xF5]));
    assert!(!is_valid_utf8(&[0xF6]));
    assert!(!is_valid_utf8(&[0xF7]));
    assert!(!is_valid_utf8(&[0xF8]));
    assert!(!is_valid_utf8(&[0xFF]));
}

#[test]
fn test_invalid_surrogates() {
    // U+D800 (low surrogate start): ED A0 80
    assert!(!is_valid_utf8(&[0xED, 0xA0, 0x80]));

    // U+DABC (arbitrary low surrogate): ED AA BC
    assert!(!is_valid_utf8(&[0xED, 0xAA, 0xBC]));

    // U+DFFF (low surrogate end): ED BF BF
    assert!(!is_valid_utf8(&[0xED, 0xBF, 0xBF]));

    // U+D7FF (just before surrogates) - should be VALID
    assert!(is_valid_utf8(&[0xED, 0x9F, 0xBF]));

    // U+E000 (just after surrogates) - should be VALID
    assert!(is_valid_utf8(&[0xEE, 0x80, 0x80]));
}

#[test]
fn test_overlong_encodings() {
    // Overlong encoding of U+0000 (should be 00, not C0 80)
    assert!(!is_valid_utf8(&[0xC0, 0x80]));

    // Overlong encoding of 'A' (U+0041, should be 41, not C1 81)
    assert!(!is_valid_utf8(&[0xC1, 0x81]));

    // Overlong encoding of '€' (should be E2 82 AC, not F0 82 82 AC)
    assert!(!is_valid_utf8(&[0xF0, 0x82, 0x82, 0xAC]));
}

#[test]
fn test_truncated_sequences() {
    // 2-byte sequence, missing continuation byte
    assert!(!is_valid_utf8(&[0xC3]));

    // 3-byte sequence, missing continuation bytes
    assert!(!is_valid_utf8(&[0xE2]));
    assert!(!is_valid_utf8(&[0xE2, 0x82]));

    // 4-byte sequence, missing continuation bytes
    assert!(!is_valid_utf8(&[0xF0]));
    assert!(!is_valid_utf8(&[0xF0, 0x9F]));
    assert!(!is_valid_utf8(&[0xF0, 0x9F, 0xA6]));
}

#[test]
fn test_invalid_continuation_bytes() {
    // Continuation byte without lead byte
    assert!(!is_valid_utf8(&[0x80]));
    assert!(!is_valid_utf8(&[0xBF]));

    // Valid lead byte followed by invalid continuation
    // 0xC3 expects 10xxxxxx, but gets 11xxxxxx
    assert!(!is_valid_utf8(&[0xC3, 0xC0]));

    // 0xE2 expects two continuation bytes
    assert!(!is_valid_utf8(&[0xE2, 0xFF, 0xAC]));
    assert!(!is_valid_utf8(&[0xE2, 0x82, 0xFF]));
}

#[test]
fn test_mixed_valid_and_invalid() {
    // Valid sequence followed by invalid
    let mut data = "Hello".as_bytes().to_vec();
    data.push(0xFF);
    assert!(!is_valid_utf8(&data));

    // Invalid followed by valid
    let mut data = vec![0xFF];
    data.extend_from_slice("World".as_bytes());
    assert!(!is_valid_utf8(&data));
}

#[test]
fn test_property_valid_utf8_strings() {
    // All valid Rust strings are valid UTF-8
    let test_strings = vec![
        "",
        "a",
        "Hello, world!",
        "Café",
        "日本語",
        "中文",
        "العربية",
        "Русский",
        "한국어",
        "🦀🎉🌟",
        "Mixed: Hello 世界 🌍",
    ];

    for s in test_strings {
        assert!(
            is_valid_utf8(s.as_bytes()),
            "Valid Rust string should be valid UTF-8: {}",
            s
        );
    }
}

#[test]
fn test_property_length_preserved() {
    // Validation doesn't modify the byte count
    let data = "Hello, 世界!".as_bytes();
    let len_before = data.len();
    let _result = is_valid_utf8(data);
    let len_after = data.len();
    assert_eq!(len_before, len_after, "Length should not change");
}

#[test]
fn test_deterministic() {
    // Same input always produces same output
    let data = "Test 🦀".as_bytes();
    assert_eq!(is_valid_utf8(data), is_valid_utf8(data));

    let invalid = &[0xFF, 0xFE];
    assert_eq!(is_valid_utf8(invalid), is_valid_utf8(invalid));
}

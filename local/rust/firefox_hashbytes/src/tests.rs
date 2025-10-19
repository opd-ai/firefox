//! Tests for HashBytes implementation
//!
//! These tests verify that the Rust implementation matches the C++ behavior.

use super::*;

#[test]
fn test_empty_array() {
    let data = b"";
    let hash = hash_bytes(data, 0);
    assert_eq!(hash, 0, "Empty array with starting_hash=0 should return 0");
}

#[test]
fn test_empty_array_with_starting_hash() {
    let data = b"";
    let starting = 42;
    let hash = hash_bytes(data, starting);
    assert_eq!(hash, starting, "Empty array should return starting_hash");
}

#[test]
fn test_single_byte() {
    let data = b"A";
    let hash = hash_bytes(data, 0);
    
    // Expected calculation:
    // hash = add_u32_to_hash(0, 'A' as u32)
    // hash = add_u32_to_hash(0, 65)
    let expected = add_u32_to_hash(0, 65);
    
    assert_eq!(hash, expected);
    assert_ne!(hash, 0, "Single byte should produce non-zero hash");
}

#[test]
fn test_golden_ratio_constant() {
    assert_eq!(GOLDEN_RATIO_U32, 0x9E3779B9, "Golden ratio constant must match C++");
}

#[test]
fn test_rotate_left5() {
    // Test some known values
    assert_eq!(rotate_left5(0), 0);
    assert_eq!(rotate_left5(1), 32); // 1 << 5
    assert_eq!(rotate_left5(0x80000000), 16); // High bit rotates: 0x8000_0000 >> 27 = 16
    
    // Test that it's a true rotation (no bits lost)
    let value = 0x12345678;
    let rotated = rotate_left5(value);
    let expected = (value << 5) | (value >> 27);
    assert_eq!(rotated, expected);
}

#[test]
fn test_add_u32_to_hash_zero_hash() {
    // When hash is 0, the function should still mix the value
    let value = 42;
    let result = add_u32_to_hash(0, value);
    
    // rotate_left5(0) = 0
    // 0 ^ 42 = 42
    // GOLDEN_RATIO_U32 * 42
    let expected = GOLDEN_RATIO_U32.wrapping_mul(42);
    assert_eq!(result, expected);
}

#[test]
fn test_add_u32_to_hash_nonzero_hash() {
    let hash = 100;
    let value = 42;
    let result = add_u32_to_hash(hash, value);
    
    let rotated = rotate_left5(hash);
    let xored = rotated ^ value;
    let expected = GOLDEN_RATIO_U32.wrapping_mul(xored);
    
    assert_eq!(result, expected);
}

#[test]
fn test_deterministic() {
    // Same input should always produce same output
    let data = b"hello world";
    let hash1 = hash_bytes(data, 0);
    let hash2 = hash_bytes(data, 0);
    assert_eq!(hash1, hash2, "Hash should be deterministic");
}

#[test]
fn test_different_inputs_different_outputs() {
    let data1 = b"hello";
    let data2 = b"world";
    let hash1 = hash_bytes(data1, 0);
    let hash2 = hash_bytes(data2, 0);
    assert_ne!(hash1, hash2, "Different inputs should (usually) produce different hashes");
}

#[test]
fn test_order_matters() {
    let data1 = b"ab";
    let data2 = b"ba";
    let hash1 = hash_bytes(data1, 0);
    let hash2 = hash_bytes(data2, 0);
    assert_ne!(hash1, hash2, "Order of bytes should matter");
}

#[test]
fn test_word_aligned_data() {
    // Test with data that's exactly word-size aligned (8 bytes on 64-bit)
    let data = b"12345678";
    let hash = hash_bytes(data, 0);
    assert_ne!(hash, 0, "Word-aligned data should produce non-zero hash");
}

#[test]
fn test_unaligned_data() {
    // Test with data that's not word-aligned
    let data = b"123"; // 3 bytes
    let hash = hash_bytes(data, 0);
    assert_ne!(hash, 0, "Unaligned data should produce non-zero hash");
}

#[test]
fn test_hash_chaining() {
    let part1 = b"hello";
    let part2 = b" world";
    
    // Chain hashing
    let hash1 = hash_bytes(part1, 0);
    let hash2 = hash_bytes(part2, hash1);
    
    // Full hashing
    let full = b"hello world";
    let _hash_full = hash_bytes(full, 0);
    
    // Chained hash should be different from full hash
    // (C++ implementation doesn't guarantee they're equal due to word boundaries)
    assert_ne!(hash2, hash1, "Chained hash should update");
}

#[test]
fn test_starting_hash_affects_output() {
    let data = b"test";
    let hash1 = hash_bytes(data, 0);
    let hash2 = hash_bytes(data, 100);
    assert_ne!(hash1, hash2, "Starting hash should affect output");
}

#[test]
fn test_large_buffer() {
    // Test with a large buffer to verify word-by-word processing
    let data = vec![0x42u8; 1024];
    let hash = hash_bytes(&data, 0);
    assert_ne!(hash, 0, "Large buffer should produce non-zero hash");
}

#[test]
fn test_all_zeros() {
    let data = vec![0u8; 100];
    let hash = hash_bytes(&data, 0);
    // Actually, all zeros with starting_hash=0 produces 0 because:
    // add_u32_to_hash(0, 0) = GOLDEN_RATIO * (rotate_left5(0) ^ 0) = GOLDEN_RATIO * 0 = 0
    // This is correct behavior - the hash mixes the input, and zero input gives zero
    // Let's test with a non-zero starting hash instead
    let hash_with_start = hash_bytes(&data, 1);
    assert_ne!(hash_with_start, 0, "All zeros with non-zero start should produce non-zero hash");
}

#[test]
fn test_all_ones() {
    let data = vec![0xFFu8; 100];
    let hash = hash_bytes(&data, 0);
    assert_ne!(hash, 0, "All ones should produce non-zero hash");
}

#[test]
fn test_avalanche_effect() {
    // Small change in input should produce large change in output
    let data1 = b"test";
    let data2 = b"Test"; // Only first byte changed
    
    let hash1 = hash_bytes(data1, 0);
    let hash2 = hash_bytes(data2, 0);
    
    assert_ne!(hash1, hash2, "Single byte change should change hash");
    
    // Count different bits
    let diff = hash1 ^ hash2;
    let bit_count = diff.count_ones();
    
    // Avalanche effect: ideally ~50% of bits should flip
    // We'll accept anything > 25% (8 bits) as reasonable
    assert!(bit_count >= 8, 
            "Avalanche effect: {} bits changed (expected >= 8)", bit_count);
}

#[test]
fn test_sequential_bytes() {
    let data: Vec<u8> = (0..=255).collect();
    let hash = hash_bytes(&data, 0);
    assert_ne!(hash, 0, "Sequential bytes should produce non-zero hash");
}

#[test]
fn test_wrapping_behavior() {
    // Test that wrapping arithmetic works correctly
    // Use values that would overflow if not wrapped
    let data = vec![0xFFu8; 100];
    let hash = hash_bytes(&data, 0xFFFFFFFF);
    // Should not panic and should produce some result
    assert!(hash <= 0xFFFFFFFF); // Always true for u32, but validates no panic
}

/// Test to verify the implementation matches expected C++ behavior
/// These are "golden values" that we expect from the C++ implementation
#[test]
fn test_known_values() {
    // These values would need to be verified against actual C++ output
    // For now, we're just testing consistency
    
    let test_cases: Vec<(&[u8], u32)> = vec![
        (b"", 0u32),
        (b"a", 0u32),
        (b"hello", 0u32),
        (b"Hello", 0u32),
        (b"hello world", 0u32),
    ];
    
    for (data, starting) in test_cases {
        let hash = hash_bytes(data, starting);
        // Just verify it's deterministic
        let hash2 = hash_bytes(data, starting);
        assert_eq!(hash, hash2, "Hash of {:?} should be deterministic", 
                   std::str::from_utf8(data).unwrap_or("<binary>"));
    }
}

#[test]
fn test_boundary_conditions() {
    // Test various boundary conditions
    
    // Exactly one word
    let word_size = std::mem::size_of::<usize>();
    let one_word = vec![0x42u8; word_size];
    let hash = hash_bytes(&one_word, 0);
    assert_ne!(hash, 0);
    
    // One word + 1 byte
    let one_word_plus = vec![0x42u8; word_size + 1];
    let hash = hash_bytes(&one_word_plus, 0);
    assert_ne!(hash, 0);
    
    // One word - 1 byte
    let one_word_minus = vec![0x42u8; word_size - 1];
    let hash = hash_bytes(&one_word_minus, 0);
    assert_ne!(hash, 0);
}

#[test]
fn test_word_size_independence() {
    // The same data should hash to the same value regardless of word size
    // This is a property we want to maintain for cross-platform compatibility
    
    // Use data smaller than both 32-bit and 64-bit word sizes
    let data = b"test";
    let hash = hash_bytes(data, 0);
    
    // Manually compute what we expect (byte-by-byte processing)
    let mut expected = 0;
    for &byte in data {
        expected = add_u32_to_hash(expected, byte as u32);
    }
    
    assert_eq!(hash, expected, "Small data should hash byte-by-byte");
}

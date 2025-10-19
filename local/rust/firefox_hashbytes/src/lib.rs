//! Rust port of Firefox's HashBytes function from mfbt/HashFunctions.cpp
//!
//! This module provides a fast, non-cryptographic hash function for hashing
//! arbitrary byte sequences. It's used throughout Firefox for creating hash
//! codes for hash tables, caches, and other data structures.
//!
//! # Algorithm
//!
//! The hash function uses the golden ratio (0x9E3779B9) for mixing, based on
//! Fibonacci hashing as described in Knuth's "The Art of Computer Programming".
//!
//! The algorithm:
//! 1. Processes memory word-by-word (sizeof(usize)) for performance
//! 2. For each word: rotate left by 5, XOR with input, multiply by golden ratio
//! 3. Handles remaining bytes individually
//! 4. Returns 32-bit hash value
//!
//! # Safety
//!
//! This implementation uses unsafe code for performance (unaligned memory reads)
//! but maintains safety through careful bounds checking and slice operations.
//!
//! # Non-Cryptographic
//!
//! ⚠️ This is NOT a cryptographic hash function. Do not use for:
//! - Password hashing
//! - Cryptographic signatures
//! - Security tokens
//!
//! # Examples
//!
//! ```
//! use firefox_hashbytes::hash_bytes;
//!
//! let data = b"hello world";
//! let hash = hash_bytes(data, 0);
//! println!("Hash: 0x{:08x}", hash);
//! ```

/// The golden ratio as a 32-bit fixed-point value.
/// Used for hash mixing to ensure good distribution of hash values.
pub const GOLDEN_RATIO_U32: u32 = 0x9E3779B9;

/// Type alias for hash numbers (32-bit unsigned integer).
pub type HashNumber = u32;

/// Rotate a 32-bit value left by 5 bits.
///
/// This is an inline function used for hash mixing. The rotation by 5 bits
/// is arbitrary but provides good mixing properties.
///
/// # Examples
///
/// ```
/// use firefox_hashbytes::rotate_left5;
///
/// let value = 0x12345678;
/// let rotated = rotate_left5(value);
/// assert_eq!(rotated, (value << 5) | (value >> 27));
/// ```
#[inline(always)]
pub const fn rotate_left5(value: HashNumber) -> HashNumber {
    (value << 5) | (value >> 27)
}

/// Add a 32-bit value to a hash.
///
/// This is the core mixing function used by all hash routines. It combines:
/// - Rotation (rotate left by 5 bits)
/// - XOR with input value
/// - Wrapping multiplication by golden ratio
///
/// The order of operations is important:
/// - We XOR *before* multiplying to avoid losing information when hash=0
/// - We use wrapping multiply to allow unsigned overflow (matches C++ behavior)
///
/// # Examples
///
/// ```
/// use firefox_hashbytes::add_u32_to_hash;
///
/// let hash = 0;
/// let value = 42;
/// let new_hash = add_u32_to_hash(hash, value);
/// ```
#[inline(always)]
pub const fn add_u32_to_hash(hash: HashNumber, value: u32) -> HashNumber {
    // Wrapping multiply is used to match C++ unsigned overflow semantics
    GOLDEN_RATIO_U32.wrapping_mul(rotate_left5(hash) ^ value)
}

/// Hash a byte array into a 32-bit hash value.
///
/// This function hashes arbitrary byte sequences using a fast, non-cryptographic
/// algorithm. It processes memory word-by-word for performance and handles any
/// remaining bytes individually.
///
/// # Arguments
///
/// * `bytes` - The byte slice to hash
/// * `starting_hash` - Optional starting hash value for chaining (default: 0)
///
/// # Returns
///
/// A 32-bit hash value
///
/// # Examples
///
/// ```
/// use firefox_hashbytes::hash_bytes;
///
/// // Simple hashing
/// let data = b"hello";
/// let hash = hash_bytes(data, 0);
///
/// // Hash chaining
/// let part1 = b"hello";
/// let part2 = b" world";
/// let hash1 = hash_bytes(part1, 0);
/// let hash2 = hash_bytes(part2, hash1);
/// ```
///
/// # Performance
///
/// This function is optimized for performance:
/// - Word-by-word processing (8 bytes at a time on 64-bit systems)
/// - Unaligned memory reads for efficiency
/// - Inline-friendly implementation
///
/// # Safety
///
/// This function uses unsafe code for unaligned memory reads, but maintains
/// safety through proper bounds checking via slice operations.
pub fn hash_bytes(bytes: &[u8], starting_hash: HashNumber) -> HashNumber {
    let mut hash = starting_hash;
    let len = bytes.len();

    if len == 0 {
        return hash;
    }

    // Process word-by-word for performance
    // On 64-bit systems, this processes 8 bytes at a time
    // On 32-bit systems, this processes 4 bytes at a time
    let word_size = std::mem::size_of::<usize>();
    let num_full_words = len / word_size;

    // Hash full words
    for i in 0..num_full_words {
        let offset = i * word_size;
        // SAFETY: We've calculated num_full_words to ensure we don't read past
        // the end of the slice. The unaligned read is safe because we're reading
        // from a valid slice.
        let word = unsafe {
            let ptr = bytes.as_ptr().add(offset);
            std::ptr::read_unaligned(ptr as *const usize)
        };

        // Hash lower 32 bits first (matches C++ behavior)
        hash = add_u32_to_hash(hash, word as u32);

        // On 64-bit systems, also hash upper 32 bits
        if word_size == 8 {
            let upper = (word >> 32) as u32;
            hash = add_u32_to_hash(hash, upper);
        }
    }

    // Hash remaining bytes (less than word_size)
    let remaining_start = num_full_words * word_size;
    for byte in &bytes[remaining_start..] {
        hash = add_u32_to_hash(hash, *byte as u32);
    }

    hash
}

// FFI layer for C++ interop
pub mod ffi;

#[cfg(test)]
mod tests;

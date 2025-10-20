//! Rust port of nsTArray.cpp
//!
//! This module provides the core data structures and functions for nsTArray support.
//! It exports two critical symbols used by the nsTArray<T> template in C++:
//!
//! 1. `sEmptyTArrayHeader` - A const struct representing an empty array
//! 2. `IsTwiceTheRequiredBytesRepresentableAsUint32` - Overflow validation function
//!
//! # Memory Layout
//!
//! The nsTArrayHeader struct must match the C++ layout exactly:
//! - Size: 8 bytes (two uint32_t fields)
//! - Alignment: 8 bytes (via alignas(8) in C++)
//! - With alignment, total size is 16 bytes (8 bytes data + 8 bytes padding)
//!
//! # Thread Safety
//!
//! Both exports are inherently thread-safe:
//! - `sEmptyTArrayHeader` is const/immutable (read-only)
//! - `IsTwiceTheRequiredBytesRepresentableAsUint32` is a pure function (no state)
//!
//! # Original C++ Implementation
//!
//! Located in: mozilla-central/xpcom/ds/nsTArray.cpp
//!
//! ```cpp
//! alignas(8) const nsTArrayHeader sEmptyTArrayHeader = {0, 0, 0};
//!
//! bool IsTwiceTheRequiredBytesRepresentableAsUint32(size_t aCapacity,
//!                                                   size_t aElemSize) {
//!   using mozilla::CheckedUint32;
//!   return ((CheckedUint32(aCapacity) * aElemSize) * 2).isValid();
//! }
//! ```

pub mod ffi;

/// nsTArrayHeader - Array header structure matching C++ layout
///
/// This struct must match the C++ nsTArrayHeader exactly:
/// ```cpp
/// struct nsTArrayHeader {
///   uint32_t mLength;
///   uint32_t mCapacity : 31;
///   uint32_t mIsAutoArray : 1;
/// };
/// ```
///
/// **Bit Field Handling**:
/// C++ packs mCapacity (31 bits) and mIsAutoArray (1 bit) into a single uint32_t.
/// For sEmptyTArrayHeader (all zeros), we simply store 0 in m_capacity_and_flags.
///
/// Layout:
/// - Offset 0: m_length (uint32_t, 4 bytes)
/// - Offset 4: m_capacity_and_flags (uint32_t, 4 bytes)
///   - Bits 0-30: capacity (31 bits)
///   - Bit 31: is_auto_array (1 bit)
/// - Total: 8 bytes
/// - With alignment: 16 bytes (8 bytes padding)
#[repr(C)]
#[repr(align(8))]  // alignas(8) in C++
pub struct nsTArrayHeader {
    /// Array length (number of elements)
    m_length: u32,
    /// Packed field: 31 bits capacity + 1 bit is_auto_array
    /// Bit layout: [is_auto_array:1][capacity:31]
    m_capacity_and_flags: u32,
}

// Compile-time assertions to verify memory layout
const _: () = {
    // Verify struct size is 8 bytes (before padding)
    let _ = std::mem::transmute::<nsTArrayHeader, [u8; 8]>;
    
    // Verify alignment is 8 bytes
    assert!(std::mem::align_of::<nsTArrayHeader>() == 8);
    
    // Verify total size with alignment is 16 bytes
    // Note: With repr(C) and align(8), the struct itself is 8 bytes,
    // but when stored as static, it gets padded to alignment boundary
    assert!(std::mem::size_of::<nsTArrayHeader>() == 8);
};

/// Check if twice the required bytes fits in uint32_t
///
/// This function validates that `(capacity * elem_size * 2)` doesn't overflow uint32_t.
/// It's used by nsTArray to ensure the capacity doubling strategy won't cause overflow.
///
/// # Algorithm
///
/// 1. Multiply capacity by elem_size (checked)
/// 2. Multiply result by 2 (checked)
/// 3. Check if result fits in uint32_t
///
/// Returns true if no overflow, false on overflow.
///
/// # Examples
///
/// ```rust
/// // Small values - no overflow
/// assert_eq!(is_twice_required_bytes_representable_as_uint32(100, 8), true);
///
/// // Large values - would overflow
/// assert_eq!(is_twice_required_bytes_representable_as_uint32(usize::MAX, 8), false);
/// ```
///
/// # C++ Equivalent
///
/// ```cpp
/// bool IsTwiceTheRequiredBytesRepresentableAsUint32(size_t aCapacity,
///                                                   size_t aElemSize) {
///   using mozilla::CheckedUint32;
///   return ((CheckedUint32(aCapacity) * aElemSize) * 2).isValid();
/// }
/// ```
pub fn is_twice_required_bytes_representable_as_uint32(
    capacity: usize,
    elem_size: usize,
) -> bool {
    // Use Rust's checked arithmetic (equivalent to CheckedUint32 in C++)
    capacity
        .checked_mul(elem_size)  // Step 1: capacity * elem_size
        .and_then(|bytes| bytes.checked_mul(2))  // Step 2: result * 2
        .map(|total| total <= u32::MAX as usize)  // Step 3: check fits in uint32_t
        .unwrap_or(false)  // Return false on overflow
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_size() {
        // Verify struct is 8 bytes (two uint32_t)
        assert_eq!(std::mem::size_of::<nsTArrayHeader>(), 8);
    }

    #[test]
    fn test_header_alignment() {
        // Verify alignment is 8 bytes (alignas(8))
        assert_eq!(std::mem::align_of::<nsTArrayHeader>(), 8);
    }

    #[test]
    fn test_overflow_check_small_values() {
        // Small values should not overflow
        assert!(is_twice_required_bytes_representable_as_uint32(100, 8));
        assert!(is_twice_required_bytes_representable_as_uint32(1000, 4));
        assert!(is_twice_required_bytes_representable_as_uint32(0, 0));
    }

    #[test]
    fn test_overflow_check_edge_cases() {
        // Edge case: zero values
        assert!(is_twice_required_bytes_representable_as_uint32(0, 8));
        assert!(is_twice_required_bytes_representable_as_uint32(100, 0));
        
        // Edge case: maximum safe value (uint32_t max / 2)
        let max_safe = (u32::MAX as usize) / 2;
        assert!(is_twice_required_bytes_representable_as_uint32(max_safe, 1));
    }

    #[test]
    fn test_overflow_check_large_values() {
        // Large values should overflow
        assert!(!is_twice_required_bytes_representable_as_uint32(usize::MAX, 8));
        assert!(!is_twice_required_bytes_representable_as_uint32(u32::MAX as usize, u32::MAX as usize));
        
        // Just over the limit: (u32::MAX / 2) + 1, elem_size=1 should still be OK
        // because ((u32::MAX/2) + 1) * 1 * 2 = u32::MAX + 1, which is just over
        let over_limit = ((u32::MAX as usize) / 2) + 1;
        // over_limit * 1 * 2 = u32::MAX + 2, which overflows uint32
        assert!(!is_twice_required_bytes_representable_as_uint32(over_limit, 1));
    }

    #[test]
    fn test_overflow_check_deterministic() {
        // Same inputs should always produce same outputs
        for _ in 0..100 {
            assert!(is_twice_required_bytes_representable_as_uint32(1000, 8));
            assert!(!is_twice_required_bytes_representable_as_uint32(usize::MAX, 8));
        }
    }
}

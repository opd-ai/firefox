//! FFI layer for nsTArray exports
//!
//! This module provides C-compatible exports for both production code and tests.
//! All C++ code (including tests) calls Rust implementation through these FFI functions.

use crate::{is_twice_required_bytes_representable_as_uint32, nsTArrayHeader};

/// Empty array header constant (matches C++ sEmptyTArrayHeader)
///
/// C++ Declaration:
/// ```cpp
/// alignas(8) const nsTArrayHeader sEmptyTArrayHeader = {0, 0, 0};
/// ```
///
/// Memory layout:
/// - Offset 0: mLength = 0 (uint32_t)
/// - Offset 4: mCapacity = 0, mIsAutoArray = 0 (uint32_t, bit-packed)
/// - Total: 8 bytes + 8 bytes padding (alignas(8)) = 16 bytes
///
/// This is a shared constant used by all empty nsTArray instances.
/// It's never modified, always read-only.
#[no_mangle]
pub static sEmptyTArrayHeader: nsTArrayHeader = nsTArrayHeader {
    m_length: 0,
    m_capacity_and_flags: 0,  // Both capacity and is_auto_array are 0
};

/// Check if twice the required bytes fits in uint32_t (FFI wrapper)
///
/// C++ Signature:
/// ```cpp
/// bool IsTwiceTheRequiredBytesRepresentableAsUint32(size_t aCapacity,
///                                                   size_t aElemSize);
/// ```
///
/// This function is called by nsTArray::EnsureCapacityImpl() before allocating
/// memory to ensure the capacity doubling strategy won't overflow uint32_t.
///
/// # Safety
///
/// This function is pure (no side effects) and safe to call from any thread.
///
/// # Parameters
///
/// - `capacity`: Desired array capacity (number of elements)
/// - `elem_size`: Size of each element in bytes
///
/// # Returns
///
/// - `true` if `(capacity * elem_size * 2) <= UINT32_MAX`
/// - `false` if overflow would occur
///
/// # Examples
///
/// ```cpp
/// // C++ usage (from nsTArray.h)
/// if (!IsTwiceTheRequiredBytesRepresentableAsUint32(aCapacity, aElemSize)) {
///   Alloc::SizeTooBig((size_t)aCapacity * aElemSize);
///   return Alloc::FailureResult();
/// }
/// ```
#[no_mangle]
pub extern "C" fn IsTwiceTheRequiredBytesRepresentableAsUint32(
    capacity: usize,
    elem_size: usize,
) -> bool {
    // Pure function - direct call, no panic catching needed
    is_twice_required_bytes_representable_as_uint32(capacity, elem_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_header_values() {
        // Verify sEmptyTArrayHeader is all zeros
        assert_eq!(sEmptyTArrayHeader.m_length, 0);
        assert_eq!(sEmptyTArrayHeader.m_capacity_and_flags, 0);
    }

    #[test]
    fn test_empty_header_address() {
        // Verify we can get a pointer to sEmptyTArrayHeader
        let ptr: *const nsTArrayHeader = &sEmptyTArrayHeader;
        assert!(!ptr.is_null());
        
        // Verify pointer is properly aligned (8-byte alignment)
        assert_eq!((ptr as usize) % 8, 0);
    }

    #[test]
    fn test_ffi_overflow_check_basic() {
        // Test FFI function directly
        assert!(IsTwiceTheRequiredBytesRepresentableAsUint32(100, 8));
        assert!(!IsTwiceTheRequiredBytesRepresentableAsUint32(usize::MAX, 8));
    }

    #[test]
    fn test_ffi_overflow_check_edge_cases() {
        // Zero values
        assert!(IsTwiceTheRequiredBytesRepresentableAsUint32(0, 0));
        assert!(IsTwiceTheRequiredBytesRepresentableAsUint32(0, 100));
        assert!(IsTwiceTheRequiredBytesRepresentableAsUint32(100, 0));
    }

    #[test]
    fn test_ffi_overflow_check_boundary() {
        // Test at uint32_t boundary
        let max_safe = (u32::MAX as usize) / 2;
        
        // max_safe * 1 * 2 = u32::MAX - should be OK (just at boundary)
        assert!(IsTwiceTheRequiredBytesRepresentableAsUint32(max_safe, 1));
        
        // (max_safe + 1) * 1 * 2 = u32::MAX + 2 - overflow!
        assert!(!IsTwiceTheRequiredBytesRepresentableAsUint32(max_safe + 1, 1));
        
        // max_safe * 2 * 2 would be 2 * u32::MAX - overflow!
        assert!(!IsTwiceTheRequiredBytesRepresentableAsUint32(max_safe, 2));
    }

    #[test]
    fn test_ffi_function_is_pure() {
        // Verify function is deterministic (pure)
        for _ in 0..100 {
            assert!(IsTwiceTheRequiredBytesRepresentableAsUint32(12345, 67));
        }
    }
}

// -*- Mode: rust; rust-indent-offset: 4 -*-
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! FFI layer for nsTObserverArray_base
//!
//! This module provides C-compatible exports for the Rust implementation
//! of nsTObserverArray_base methods. These functions are called from C++
//! code (both production code and tests).
//!
//! ## Function Signatures
//!
//! All functions match the C++ method signatures exactly:
//! - void nsTObserverArray_base::AdjustIterators(index_type, diff_type)
//! - void nsTObserverArray_base::ClearIterators()
//!
//! ## Safety
//!
//! All FFI functions include panic boundaries to prevent unwinding into C++.
//! Null pointer checks are performed where appropriate.

use std::panic;

use crate::nsTObserverArray_base;

/// FFI export for nsTObserverArray_base::AdjustIterators
///
/// Adjusts all active iterators after an array modification.
///
/// # Arguments
///
/// * `this` - Pointer to the nsTObserverArray_base instance
/// * `mod_pos` - Position where the modification occurred (index_type = size_t)
/// * `adjustment` - -1 for removal, +1 for insertion (diff_type = ptrdiff_t)
///
/// # Safety
///
/// This function is safe to call from C++ as long as:
/// - `this` is a valid pointer to a nsTObserverArray_base instance
/// - The instance is not accessed concurrently from multiple threads
/// - All iterators in the linked list are valid
///
/// # Panics
///
/// Any panics are caught and logged. The function returns normally to
/// prevent unwinding into C++ code.
#[no_mangle]
pub unsafe extern "C" fn nsTObserverArray_base_AdjustIterators(
    this: *mut nsTObserverArray_base,
    mod_pos: usize,
    adjustment: isize,
) {
    // Catch any panics to prevent unwinding into C++
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        // Null pointer check
        if this.is_null() {
            eprintln!("nsTObserverArray_base_AdjustIterators: null this pointer");
            return;
        }

        // SAFETY: We checked for null above
        // The caller guarantees this is a valid pointer
        let array_base = &mut *this;
        
        // Call the Rust implementation
        array_base.adjust_iterators(mod_pos, adjustment);
    }));

    // Log any panic but don't propagate it
    if let Err(e) = result {
        eprintln!(
            "nsTObserverArray_base_AdjustIterators panicked: {:?}",
            e
        );
    }
}

/// FFI export for nsTObserverArray_base::ClearIterators
///
/// Resets all iterators to position 0.
///
/// # Arguments
///
/// * `this` - Pointer to the nsTObserverArray_base instance
///
/// # Safety
///
/// This function is safe to call from C++ as long as:
/// - `this` is a valid pointer to a nsTObserverArray_base instance
/// - The instance is not accessed concurrently from multiple threads
/// - All iterators in the linked list are valid
///
/// # Panics
///
/// Any panics are caught and logged. The function returns normally to
/// prevent unwinding into C++ code.
#[no_mangle]
pub unsafe extern "C" fn nsTObserverArray_base_ClearIterators(
    this: *mut nsTObserverArray_base,
) {
    // Catch any panics to prevent unwinding into C++
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        // Null pointer check
        if this.is_null() {
            eprintln!("nsTObserverArray_base_ClearIterators: null this pointer");
            return;
        }

        // SAFETY: We checked for null above
        // The caller guarantees this is a valid pointer
        let array_base = &mut *this;
        
        // Call the Rust implementation
        array_base.clear_iterators();
    }));

    // Log any panic but don't propagate it
    if let Err(e) = result {
        eprintln!(
            "nsTObserverArray_base_ClearIterators panicked: {:?}",
            e
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{create_test_array_base, create_test_iterator, Iterator_base};
    use std::ptr;

    #[test]
    fn test_ffi_adjust_iterators_null_this() {
        // Should not crash with null pointer
        unsafe {
            nsTObserverArray_base_AdjustIterators(ptr::null_mut(), 0, 1);
        }
    }

    #[test]
    fn test_ffi_clear_iterators_null_this() {
        // Should not crash with null pointer
        unsafe {
            nsTObserverArray_base_ClearIterators(ptr::null_mut());
        }
    }

    #[test]
    fn test_ffi_adjust_iterators_single_iterator() {
        // Create iterator at position 5
        let mut iter = create_test_iterator(5, ptr::null_mut());
        let mut array = create_test_array_base(&mut iter as *mut Iterator_base);

        // Insert at position 3 (adjustment +1)
        unsafe {
            nsTObserverArray_base_AdjustIterators(&mut array as *mut _, 3, 1);
        }

        // Position should be adjusted to 6
        assert_eq!(iter.m_position, 6);
    }

    #[test]
    fn test_ffi_adjust_iterators_no_change() {
        // Create iterator at position 2
        let mut iter = create_test_iterator(2, ptr::null_mut());
        let mut array = create_test_array_base(&mut iter as *mut Iterator_base);

        // Insert at position 3 (after iterator)
        unsafe {
            nsTObserverArray_base_AdjustIterators(&mut array as *mut _, 3, 1);
        }

        // Position should not change (2 <= 3)
        assert_eq!(iter.m_position, 2);
    }

    #[test]
    fn test_ffi_clear_iterators_single_iterator() {
        // Create iterator at position 10
        let mut iter = create_test_iterator(10, ptr::null_mut());
        let mut array = create_test_array_base(&mut iter as *mut Iterator_base);

        // Clear all iterators
        unsafe {
            nsTObserverArray_base_ClearIterators(&mut array as *mut _);
        }

        // Position should be reset to 0
        assert_eq!(iter.m_position, 0);
    }

    #[test]
    fn test_ffi_adjust_iterators_multiple_iterators() {
        // Create chain of 3 iterators
        let mut iter3 = create_test_iterator(10, ptr::null_mut());
        let mut iter2 = create_test_iterator(5, &mut iter3 as *mut _);
        let mut iter1 = create_test_iterator(3, &mut iter2 as *mut _);
        let mut array = create_test_array_base(&mut iter1 as *mut Iterator_base);

        // Remove at position 4 (adjustment -1)
        unsafe {
            nsTObserverArray_base_AdjustIterators(&mut array as *mut _, 4, -1);
        }

        // Check results:
        // iter1 at 3: 3 <= 4, no change -> 3
        // iter2 at 5: 5 > 4, adjust -1 -> 4
        // iter3 at 10: 10 > 4, adjust -1 -> 9
        assert_eq!(iter1.m_position, 3);
        assert_eq!(iter2.m_position, 4);
        assert_eq!(iter3.m_position, 9);
    }

    #[test]
    fn test_ffi_clear_iterators_multiple_iterators() {
        // Create chain of 3 iterators with different positions
        let mut iter3 = create_test_iterator(100, ptr::null_mut());
        let mut iter2 = create_test_iterator(50, &mut iter3 as *mut _);
        let mut iter1 = create_test_iterator(25, &mut iter2 as *mut _);
        let mut array = create_test_array_base(&mut iter1 as *mut Iterator_base);

        // Clear all iterators
        unsafe {
            nsTObserverArray_base_ClearIterators(&mut array as *mut _);
        }

        // All positions should be 0
        assert_eq!(iter1.m_position, 0);
        assert_eq!(iter2.m_position, 0);
        assert_eq!(iter3.m_position, 0);
    }
}

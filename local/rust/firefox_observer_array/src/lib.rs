// -*- Mode: rust; rust-indent-offset: 4 -*-
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Rust port of nsTObserverArray_base
//!
//! This module provides the base class functionality for nsTObserverArray,
//! which implements an array that supports stable iterators even when the
//! array is modified during iteration.
//!
//! The base class maintains a linked list of active iterators and provides
//! two methods to keep them synchronized with array modifications:
//! - AdjustIterators: Updates iterator positions after insert/remove
//! - ClearIterators: Resets all iterators to position 0
//!
//! ## Memory Layout
//!
//! The Iterator_base struct must have C-compatible layout:
//! ```text
//! struct Iterator_base {
//!     mPosition: usize,           // 8 bytes on 64-bit
//!     mNext: *mut Iterator_base,  // 8 bytes on 64-bit
//! }  // Total: 16 bytes
//! ```
//!
//! ## Safety
//!
//! This code manipulates raw pointers to maintain the iterator linked list.
//! All unsafe operations are carefully documented with safety invariants.

pub mod ffi;

#[cfg(test)]
mod tests;

/// C-compatible representation of nsTObserverArray_base::Iterator_base
///
/// This struct matches the C++ memory layout exactly.
/// Fields correspond to the C++ class members:
/// - mPosition: Current iterator position (index into array)
/// - mNext: Next iterator in the linked list (or null)
#[repr(C)]
pub struct Iterator_base {
    pub m_position: usize,              // index_type in C++ (size_t)
    pub m_next: *mut Iterator_base,     // Iterator_base* in C++
}

/// C-compatible representation of nsTObserverArray_base
///
/// This struct matches the C++ memory layout exactly.
/// Contains only the mIterators field which is the head of the
/// iterator linked list.
#[repr(C)]
pub struct nsTObserverArray_base {
    pub m_iterators: *mut Iterator_base,  // mutable pointer to allow iteration
}

impl nsTObserverArray_base {
    /// Adjusts all active iterators after an array modification.
    ///
    /// When an element is inserted or removed from the array, this method
    /// walks the linked list of active iterators and adjusts their positions
    /// if they point beyond the modification point.
    ///
    /// # Arguments
    ///
    /// * `mod_pos` - Position where the modification occurred
    /// * `adjustment` - -1 for removal, +1 for insertion
    ///
    /// # Safety
    ///
    /// This method performs pointer manipulation. It is safe as long as:
    /// - All iterators in the linked list are valid
    /// - The linked list is properly terminated (ends with null)
    /// - No iterator is accessed after being destroyed
    ///
    /// # Algorithm
    ///
    /// ```text
    /// for each iterator in mIterators linked list:
    ///     if iterator.mPosition > mod_pos:
    ///         iterator.mPosition += adjustment
    /// ```
    ///
    /// # Examples
    ///
    /// ```text
    /// Array: [A, B, C, D]
    /// Iterator at position 2 (pointing to C)
    ///
    /// Insert at position 1:
    ///   AdjustIterators(1, 1)
    ///   Iterator position becomes 3 (still points to C)
    ///   Result: [A, X, B, C, D]
    ///
    /// Remove at position 1:
    ///   AdjustIterators(1, -1)
    ///   Iterator position becomes 1 (still points to C)
    ///   Result: [A, C, D]
    /// ```
    pub fn adjust_iterators(&mut self, mod_pos: usize, adjustment: isize) {
        // C++ asserts: aAdjustment == -1 || aAdjustment == 1
        debug_assert!(
            adjustment == -1 || adjustment == 1,
            "invalid adjustment: must be -1 or +1, got {}",
            adjustment
        );

        // Walk the linked list of iterators
        let mut iter = self.m_iterators;
        
        while !iter.is_null() {
            // SAFETY: We check for null before dereferencing
            // The linked list is maintained by C++ code and is guaranteed
            // to be valid during the lifetime of the array
            unsafe {
                let iter_ref = &mut *iter;
                
                // Adjust position if iterator points beyond modification point
                if iter_ref.m_position > mod_pos {
                    // Apply adjustment (wrapping arithmetic for safety)
                    // In practice, adjustment is always -1 or +1
                    if adjustment > 0 {
                        iter_ref.m_position = iter_ref.m_position.wrapping_add(adjustment as usize);
                    } else {
                        iter_ref.m_position = iter_ref.m_position.wrapping_sub((-adjustment) as usize);
                    }
                }
                
                // Move to next iterator in linked list
                iter = iter_ref.m_next;
            }
        }
    }

    /// Resets all iterators to position 0.
    ///
    /// Called when the array is cleared (Clear() method).
    /// This ensures all iterators start from the beginning of the
    /// (now empty) array.
    ///
    /// # Safety
    ///
    /// This method performs pointer manipulation. It is safe as long as:
    /// - All iterators in the linked list are valid
    /// - The linked list is properly terminated (ends with null)
    ///
    /// # Algorithm
    ///
    /// ```text
    /// for each iterator in mIterators linked list:
    ///     iterator.mPosition = 0
    /// ```
    pub fn clear_iterators(&mut self) {
        // Walk the linked list of iterators
        let mut iter = self.m_iterators;
        
        while !iter.is_null() {
            // SAFETY: We check for null before dereferencing
            // The linked list is maintained by C++ code and is guaranteed
            // to be valid during the lifetime of the array
            unsafe {
                let iter_ref = &mut *iter;
                
                // Reset position to 0
                iter_ref.m_position = 0;
                
                // Move to next iterator in linked list
                iter = iter_ref.m_next;
            }
        }
    }
}

/// Helper function to create a test iterator (for testing only)
#[cfg(test)]
pub fn create_test_iterator(position: usize, next: *mut Iterator_base) -> Iterator_base {
    Iterator_base {
        m_position: position,
        m_next: next,
    }
}

/// Helper function to create a test array base (for testing only)
#[cfg(test)]
pub fn create_test_array_base(iterators: *mut Iterator_base) -> nsTObserverArray_base {
    nsTObserverArray_base {
        m_iterators: iterators,
    }
}

// -*- Mode: rust; rust-indent-offset: 4 -*-
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Unit tests for nsTObserverArray_base implementation
//!
//! These tests validate the Rust implementation of the iterator management
//! methods. The C++ test suite (TestObserverArray.cpp) provides comprehensive
//! integration testing through the FFI layer.

use crate::{create_test_array_base, create_test_iterator, Iterator_base};
use std::ptr;

#[test]
fn test_adjust_iterators_empty_list() {
    // Empty iterator list (no iterators)
    let mut array = create_test_array_base(ptr::null_mut());
    
    // Should not crash
    array.adjust_iterators(5, 1);
    array.adjust_iterators(3, -1);
}

#[test]
fn test_adjust_iterators_single_iterator_insert() {
    // Create single iterator at position 5
    let mut iter = create_test_iterator(5, ptr::null_mut());
    let mut array = create_test_array_base(&mut iter as *mut Iterator_base);
    
    // Insert at position 3 (before iterator)
    array.adjust_iterators(3, 1);
    
    // Position should be incremented to 6
    assert_eq!(iter.m_position, 6);
}

#[test]
fn test_adjust_iterators_single_iterator_remove() {
    // Create single iterator at position 10
    let mut iter = create_test_iterator(10, ptr::null_mut());
    let mut array = create_test_array_base(&mut iter as *mut Iterator_base);
    
    // Remove at position 5 (before iterator)
    array.adjust_iterators(5, -1);
    
    // Position should be decremented to 9
    assert_eq!(iter.m_position, 9);
}

#[test]
fn test_adjust_iterators_at_exact_position() {
    // Create iterator at position 5
    let mut iter = create_test_iterator(5, ptr::null_mut());
    let mut array = create_test_array_base(&mut iter as *mut Iterator_base);
    
    // Modify at position 5 (exact match)
    array.adjust_iterators(5, 1);
    
    // Position should NOT change (condition is mPosition > aModPos)
    assert_eq!(iter.m_position, 5);
}

#[test]
fn test_adjust_iterators_before_position() {
    // Create iterator at position 3
    let mut iter = create_test_iterator(3, ptr::null_mut());
    let mut array = create_test_array_base(&mut iter as *mut Iterator_base);
    
    // Modify at position 10 (after iterator)
    array.adjust_iterators(10, 1);
    
    // Position should NOT change
    assert_eq!(iter.m_position, 3);
}

#[test]
fn test_adjust_iterators_multiple_iterators() {
    // Create chain of 4 iterators at different positions
    let mut iter4 = create_test_iterator(15, ptr::null_mut());
    let mut iter3 = create_test_iterator(10, &mut iter4 as *mut _);
    let mut iter2 = create_test_iterator(5, &mut iter3 as *mut _);
    let mut iter1 = create_test_iterator(2, &mut iter2 as *mut _);
    
    let mut array = create_test_array_base(&mut iter1 as *mut Iterator_base);
    
    // Insert at position 7
    array.adjust_iterators(7, 1);
    
    // Check results:
    // iter1 at 2: 2 <= 7, no change -> 2
    // iter2 at 5: 5 <= 7, no change -> 5
    // iter3 at 10: 10 > 7, adjust +1 -> 11
    // iter4 at 15: 15 > 7, adjust +1 -> 16
    assert_eq!(iter1.m_position, 2);
    assert_eq!(iter2.m_position, 5);
    assert_eq!(iter3.m_position, 11);
    assert_eq!(iter4.m_position, 16);
}

#[test]
fn test_adjust_iterators_all_before() {
    // Create iterators all before modification point
    let mut iter3 = create_test_iterator(3, ptr::null_mut());
    let mut iter2 = create_test_iterator(2, &mut iter3 as *mut _);
    let mut iter1 = create_test_iterator(1, &mut iter2 as *mut _);
    
    let mut array = create_test_array_base(&mut iter1 as *mut Iterator_base);
    
    // Modify at position 10 (after all iterators)
    array.adjust_iterators(10, -1);
    
    // No iterators should change
    assert_eq!(iter1.m_position, 1);
    assert_eq!(iter2.m_position, 2);
    assert_eq!(iter3.m_position, 3);
}

#[test]
fn test_adjust_iterators_all_after() {
    // Create iterators all after modification point
    let mut iter3 = create_test_iterator(15, ptr::null_mut());
    let mut iter2 = create_test_iterator(12, &mut iter3 as *mut _);
    let mut iter1 = create_test_iterator(10, &mut iter2 as *mut _);
    
    let mut array = create_test_array_base(&mut iter1 as *mut Iterator_base);
    
    // Remove at position 5 (before all iterators)
    array.adjust_iterators(5, -1);
    
    // All iterators should be decremented
    assert_eq!(iter1.m_position, 9);
    assert_eq!(iter2.m_position, 11);
    assert_eq!(iter3.m_position, 14);
}

#[test]
fn test_clear_iterators_empty_list() {
    // Empty iterator list
    let mut array = create_test_array_base(ptr::null_mut());
    
    // Should not crash
    array.clear_iterators();
}

#[test]
fn test_clear_iterators_single_iterator() {
    // Create iterator at arbitrary position
    let mut iter = create_test_iterator(42, ptr::null_mut());
    let mut array = create_test_array_base(&mut iter as *mut Iterator_base);
    
    // Clear iterators
    array.clear_iterators();
    
    // Position should be reset to 0
    assert_eq!(iter.m_position, 0);
}

#[test]
fn test_clear_iterators_multiple_iterators() {
    // Create chain of iterators at various positions
    let mut iter5 = create_test_iterator(100, ptr::null_mut());
    let mut iter4 = create_test_iterator(75, &mut iter5 as *mut _);
    let mut iter3 = create_test_iterator(50, &mut iter4 as *mut _);
    let mut iter2 = create_test_iterator(25, &mut iter3 as *mut _);
    let mut iter1 = create_test_iterator(10, &mut iter2 as *mut _);
    
    let mut array = create_test_array_base(&mut iter1 as *mut Iterator_base);
    
    // Clear all iterators
    array.clear_iterators();
    
    // All positions should be 0
    assert_eq!(iter1.m_position, 0);
    assert_eq!(iter2.m_position, 0);
    assert_eq!(iter3.m_position, 0);
    assert_eq!(iter4.m_position, 0);
    assert_eq!(iter5.m_position, 0);
}

#[test]
fn test_clear_iterators_position_zero() {
    // Create iterator already at position 0
    let mut iter = create_test_iterator(0, ptr::null_mut());
    let mut array = create_test_array_base(&mut iter as *mut Iterator_base);
    
    // Clear iterators
    array.clear_iterators();
    
    // Position should remain 0
    assert_eq!(iter.m_position, 0);
}

#[test]
fn test_sequential_operations() {
    // Create iterator chain
    let mut iter3 = create_test_iterator(10, ptr::null_mut());
    let mut iter2 = create_test_iterator(5, &mut iter3 as *mut _);
    let mut iter1 = create_test_iterator(3, &mut iter2 as *mut _);
    
    let mut array = create_test_array_base(&mut iter1 as *mut Iterator_base);
    
    // Perform sequence of operations
    array.adjust_iterators(4, 1);   // Insert at 4
    assert_eq!(iter1.m_position, 3);  // No change (3 <= 4)
    assert_eq!(iter2.m_position, 6);  // Adjusted (5 > 4)
    assert_eq!(iter3.m_position, 11); // Adjusted (10 > 4)
    
    array.adjust_iterators(5, -1);  // Remove at 5
    assert_eq!(iter1.m_position, 3);  // No change (3 <= 5)
    assert_eq!(iter2.m_position, 5);  // Adjusted (6 > 5, becomes 5)
    assert_eq!(iter3.m_position, 10); // Adjusted (11 > 5, becomes 10)
    
    array.clear_iterators();
    assert_eq!(iter1.m_position, 0);
    assert_eq!(iter2.m_position, 0);
    assert_eq!(iter3.m_position, 0);
}

#[test]
fn test_boundary_conditions() {
    // Test with position 0
    let mut iter = create_test_iterator(0, ptr::null_mut());
    let mut array = create_test_array_base(&mut iter as *mut Iterator_base);
    
    // Insert at position 0
    array.adjust_iterators(0, 1);
    assert_eq!(iter.m_position, 0); // No change (0 <= 0)
    
    // Remove at position 0
    array.adjust_iterators(0, -1);
    assert_eq!(iter.m_position, 0); // No change (0 <= 0)
}

#[test]
#[should_panic(expected = "invalid adjustment")]
fn test_invalid_adjustment_debug_assert() {
    // This test validates that debug assertions catch invalid adjustments
    let mut iter = create_test_iterator(5, ptr::null_mut());
    let mut array = create_test_array_base(&mut iter as *mut Iterator_base);
    
    // Invalid adjustment (not -1 or +1)
    array.adjust_iterators(3, 2);
}

#[test]
fn test_large_positions() {
    // Test with large position values
    let mut iter = create_test_iterator(usize::MAX - 10, ptr::null_mut());
    let mut array = create_test_array_base(&mut iter as *mut Iterator_base);
    
    // Clear should work with large positions
    array.clear_iterators();
    assert_eq!(iter.m_position, 0);
}

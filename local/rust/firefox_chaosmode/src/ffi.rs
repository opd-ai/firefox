// -*- Mode: Rust; tab-width: 4; indent-tabs-mode: nil; c-basic-offset: 4 -*-
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! FFI layer for ChaosMode - C-compatible exports
//!
//! This module provides C-compatible functions that can be called from C++.
//! All functions use #[no_mangle] and extern "C" for ABI compatibility.

use crate::{enter_chaos_mode, leave_chaos_mode, random_i32_in_range, random_u32_less_than};

/// Set which chaos features should be active.
/// 
/// # Safety
/// Must be called before threading starts. Not thread-safe.
/// 
/// # Arguments
/// * `feature` - The chaos feature flags to enable (as u32)
#[no_mangle]
pub extern "C" fn mozilla_chaosmode_set_chaos_feature(feature: u32) {
    // We don't convert to enum - just set the raw u32 value directly
    // This allows arbitrary bit combinations like 0x3 (ThreadScheduling | NetworkScheduling)
    unsafe {
        crate::CHAOS_FEATURES = feature;
    }
}

/// Check if a specific chaos feature is currently active.
/// 
/// Thread-safe: Uses atomic operations.
/// 
/// # Arguments
/// * `feature` - The chaos feature to check (as u32)
/// 
/// # Returns
/// true if the feature is active, false otherwise
#[no_mangle]
pub extern "C" fn mozilla_chaosmode_is_active(feature: u32) -> bool {
    let counter = crate::CHAOS_MODE_COUNTER.load(std::sync::atomic::Ordering::Relaxed);
    let features = unsafe { crate::CHAOS_FEATURES };
    counter > 0 && (features & feature) != 0
}

/// Increase the chaos mode activation level.
/// 
/// Thread-safe: Uses atomic operations.
#[no_mangle]
pub extern "C" fn mozilla_chaosmode_enter_chaos_mode() {
    enter_chaos_mode();
}

/// Decrease the chaos mode activation level.
/// 
/// Thread-safe: Uses atomic operations.
/// 
/// # Safety
/// Will panic in debug builds if counter is 0.
#[no_mangle]
pub extern "C" fn mozilla_chaosmode_leave_chaos_mode() {
    leave_chaos_mode();
}

/// Return a pseudo-random u32 < bound.
/// 
/// # Arguments
/// * `bound` - Upper bound (exclusive)
/// 
/// # Returns
/// A pseudo-random u32 in range [0, bound)
/// 
/// # Safety
/// Not thread-safe - uses C rand().
/// Will panic in debug builds if bound is 0.
#[no_mangle]
pub extern "C" fn mozilla_chaosmode_random_u32_less_than(bound: u32) -> u32 {
    random_u32_less_than(bound)
}

/// Return a pseudo-random i32 between low and high (inclusive).
/// 
/// # Arguments
/// * `low` - Lower bound (inclusive)
/// * `high` - Upper bound (inclusive)
/// 
/// # Returns
/// A pseudo-random i32 in range [low, high]
/// 
/// # Safety
/// Not thread-safe - uses C rand().
/// Will panic in debug builds if high < low.
#[no_mangle]
pub extern "C" fn mozilla_chaosmode_random_i32_in_range(low: i32, high: i32) -> i32 {
    random_i32_in_range(low, high)
}

#[cfg(test)]
mod ffi_tests {
    use super::*;

    #[test]
    fn test_ffi_set_and_check() {
        // Set ThreadScheduling feature
        mozilla_chaosmode_set_chaos_feature(0x1);
        
        // Should not be active yet (counter is 0)
        assert!(!mozilla_chaosmode_is_active(0x1));
        
        // Enter chaos mode
        mozilla_chaosmode_enter_chaos_mode();
        
        // Now should be active
        assert!(mozilla_chaosmode_is_active(0x1));
        
        // NetworkScheduling should not be active
        assert!(!mozilla_chaosmode_is_active(0x2));
        
        // Leave chaos mode
        mozilla_chaosmode_leave_chaos_mode();
        
        // Should not be active anymore
        assert!(!mozilla_chaosmode_is_active(0x1));
    }

    #[test]
    fn test_ffi_random_functions() {
        // Test random u32
        for _ in 0..10 {
            let val = mozilla_chaosmode_random_u32_less_than(100);
            assert!(val < 100);
        }
        
        // Test random i32
        for _ in 0..10 {
            let val = mozilla_chaosmode_random_i32_in_range(-50, 50);
            assert!(val >= -50 && val <= 50);
        }
    }

    #[test]
    fn test_ffi_any_feature() {
        // Test Any feature (0xffffffff)
        mozilla_chaosmode_set_chaos_feature(0xffffffff);
        mozilla_chaosmode_enter_chaos_mode();
        
        // All features should be active
        assert!(mozilla_chaosmode_is_active(0x1));  // ThreadScheduling
        assert!(mozilla_chaosmode_is_active(0x2));  // NetworkScheduling
        assert!(mozilla_chaosmode_is_active(0x4));  // TimerScheduling
        assert!(mozilla_chaosmode_is_active(0xffffffff));  // Any
        
        mozilla_chaosmode_leave_chaos_mode();
    }
}

// -*- Mode: Rust; tab-width: 4; indent-tabs-mode: nil; c-basic-offset: 4 -*-
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Integration tests for ChaosMode
//!
//! These tests verify the complete ChaosMode implementation including FFI layer.

use firefox_chaosmode::*;
use firefox_chaosmode::ffi::*;

#[test]
fn test_full_integration() {
    // Set feature via FFI
    mozilla_chaosmode_set_chaos_feature(ChaosFeature::ThreadScheduling as u32);
    
    // Verify not active initially
    assert!(!mozilla_chaosmode_is_active(ChaosFeature::ThreadScheduling as u32));
    assert!(!is_active(ChaosFeature::ThreadScheduling));
    
    // Enter chaos mode
    mozilla_chaosmode_enter_chaos_mode();
    enter_chaos_mode();  // Nest it
    
    // Verify active
    assert!(mozilla_chaosmode_is_active(ChaosFeature::ThreadScheduling as u32));
    assert!(is_active(ChaosFeature::ThreadScheduling));
    
    // Verify other features not active
    assert!(!is_active(ChaosFeature::NetworkScheduling));
    
    // Leave chaos mode (both levels)
    leave_chaos_mode();
    mozilla_chaosmode_leave_chaos_mode();
    
    // Verify not active anymore
    assert!(!is_active(ChaosFeature::ThreadScheduling));
}

#[test]
fn test_random_distribution() {
    // Basic distribution test for random_u32_less_than
    let mut histogram = [0u32; 10];
    for _ in 0..1000 {
        let val = mozilla_chaosmode_random_u32_less_than(10);
        histogram[val as usize] += 1;
    }
    
    // Each bucket should have received some values
    // (this is probabilistic but 1000 samples should hit all buckets)
    for count in &histogram {
        assert!(*count > 0, "Random distribution issue: bucket has 0 samples");
    }
}

#[test]
fn test_feature_combinations() {
    // Test multiple features enabled
    let combined = ChaosFeature::ThreadScheduling as u32 | ChaosFeature::NetworkScheduling as u32;
    mozilla_chaosmode_set_chaos_feature(combined);
    mozilla_chaosmode_enter_chaos_mode();
    
    assert!(mozilla_chaosmode_is_active(ChaosFeature::ThreadScheduling as u32));
    assert!(mozilla_chaosmode_is_active(ChaosFeature::NetworkScheduling as u32));
    assert!(!mozilla_chaosmode_is_active(ChaosFeature::TimerScheduling as u32));
    
    mozilla_chaosmode_leave_chaos_mode();
}

#[test]
fn test_deep_nesting() {
    let depth = 100;
    
    // Enter deeply
    for _ in 0..depth {
        mozilla_chaosmode_enter_chaos_mode();
    }
    
    // Leave deeply
    for _ in 0..depth {
        mozilla_chaosmode_leave_chaos_mode();
    }
    
    // Should be back to inactive
    assert!(!mozilla_chaosmode_is_active(ChaosFeature::Any as u32));
}

#[test]
fn test_random_edge_cases() {
    // Test with bound of 1 (should always return 0)
    for _ in 0..100 {
        assert_eq!(mozilla_chaosmode_random_u32_less_than(1), 0);
    }
    
    // Test with same low and high (should always return that value)
    for _ in 0..100 {
        assert_eq!(mozilla_chaosmode_random_i32_in_range(42, 42), 42);
    }
    
    // Test negative range
    for _ in 0..100 {
        let val = mozilla_chaosmode_random_i32_in_range(-100, -50);
        assert!(val >= -100 && val <= -50);
    }
}

#[test]
fn test_concurrent_checks() {
    // This test doesn't spawn threads (we'd need std::thread for that)
    // but it verifies that the atomic operations work correctly
    // when called in sequence
    
    mozilla_chaosmode_set_chaos_feature(ChaosFeature::Any as u32);
    
    // Simulate multiple "threads" entering/leaving
    mozilla_chaosmode_enter_chaos_mode();
    assert!(mozilla_chaosmode_is_active(ChaosFeature::ThreadScheduling as u32));
    
    mozilla_chaosmode_enter_chaos_mode();
    assert!(mozilla_chaosmode_is_active(ChaosFeature::ThreadScheduling as u32));
    
    mozilla_chaosmode_leave_chaos_mode();
    assert!(mozilla_chaosmode_is_active(ChaosFeature::ThreadScheduling as u32));
    
    mozilla_chaosmode_leave_chaos_mode();
    assert!(!mozilla_chaosmode_is_active(ChaosFeature::ThreadScheduling as u32));
}

// -*- Mode: Rust; tab-width: 4; indent-tabs-mode: nil; c-basic-offset: 4 -*-
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Rust implementation of Firefox ChaosMode
//!
//! ChaosMode provides a testing infrastructure that introduces controlled
//! non-determinism into Firefox to uncover race conditions and timing bugs.
//!
//! # Safety
//! - Uses atomic operations with Relaxed ordering (matches C++ implementation)
//! - Random functions are NOT thread-safe (intentional, matches C++ behavior)
//! - SetChaosFeature must be called before threading starts

use std::sync::atomic::{AtomicU32, Ordering};

// FFI layer for C++ interop
pub mod ffi;

/// Chaos features that can be enabled for testing.
/// These are bit flags that can be combined.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChaosFeature {
    None = 0x0,
    /// Altering thread scheduling
    ThreadScheduling = 0x1,
    /// Altering network request scheduling
    NetworkScheduling = 0x2,
    /// Altering timer scheduling
    TimerScheduling = 0x4,
    /// Read and write less-than-requested amounts
    IOAmounts = 0x8,
    /// Iterate over hash tables in random order
    HashTableIteration = 0x10,
    /// Randomly refuse to use cached version of image
    ImageCache = 0x20,
    /// Delay dispatching threads to encourage dispatched tasks to run
    TaskDispatching = 0x40,
    /// Delay task running to encourage sending threads to run
    TaskRunning = 0x80,
    /// All features enabled
    Any = 0xffffffff,
}

/// Global chaos mode counter (tracks nesting depth)
/// Uses Relaxed ordering to match C++ Atomic<uint32_t, Relaxed>
static CHAOS_MODE_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Global chaos features configuration
/// This is NOT atomic - must be set before threading starts
static mut CHAOS_FEATURES: u32 = ChaosFeature::Any as u32;

/// Set which chaos features should be active when chaos mode is enabled.
/// 
/// # Safety
/// Must be called before any threads are started. Not thread-safe.
pub fn set_chaos_feature(feature: ChaosFeature) {
    unsafe {
        CHAOS_FEATURES = feature as u32;
    }
}

/// Check if a specific chaos feature is currently active.
/// 
/// A feature is active when:
/// 1. Chaos mode counter > 0 (enterChaosMode has been called)
/// 2. The feature is enabled in CHAOS_FEATURES
/// 
/// Thread-safe: Uses atomic load with Relaxed ordering.
pub fn is_active(feature: ChaosFeature) -> bool {
    let counter = CHAOS_MODE_COUNTER.load(Ordering::Relaxed);
    let features = unsafe { CHAOS_FEATURES };
    counter > 0 && (features & (feature as u32)) != 0
}

/// Increase the chaos mode activation level.
/// 
/// Chaos mode can be nested - each call to enterChaosMode must be
/// matched by a call to leaveChaosMode.
/// 
/// Thread-safe: Uses atomic increment with Relaxed ordering.
pub fn enter_chaos_mode() {
    CHAOS_MODE_COUNTER.fetch_add(1, Ordering::Relaxed);
}

/// Decrease the chaos mode activation level.
/// 
/// # Panics
/// Debug builds will panic if counter is already 0.
/// Release builds will underflow (undefined behavior - don't do this).
/// 
/// Thread-safe: Uses atomic decrement with Relaxed ordering.
pub fn leave_chaos_mode() {
    let prev = CHAOS_MODE_COUNTER.fetch_sub(1, Ordering::Relaxed);
    debug_assert!(prev > 0, "leaveChaosMode called without matching enterChaosMode");
}

/// Return a pseudo-random uint32_t < aBound.
/// 
/// Uses C's rand() function for compatibility with C++ implementation.
/// NOT thread-safe, NOT cryptographically secure.
/// Only for chaos testing where deterministic results aren't needed.
/// 
/// # Panics
/// Debug builds will panic if aBound is 0.
/// 
/// # Safety
/// Uses unsafe FFI call to libc::rand().
/// Not thread-safe - matches C++ behavior.
pub fn random_u32_less_than(bound: u32) -> u32 {
    debug_assert!(bound != 0, "bound must not be zero");
    unsafe {
        (libc::rand() as u32) % bound
    }
}

/// Return a pseudo-random int32_t between aLow and aHigh (inclusive).
/// 
/// Uses C's rand() function for compatibility with C++ implementation.
/// NOT thread-safe, NOT cryptographically secure.
/// 
/// # Panics
/// Debug builds will panic if aHigh < aLow.
/// 
/// # Safety
/// Uses unsafe FFI call to libc::rand().
/// Not thread-safe - matches C++ behavior.
pub fn random_i32_in_range(low: i32, high: i32) -> i32 {
    debug_assert!(high >= low, "high must be >= low");
    let range = high - low + 1;
    unsafe {
        ((libc::rand() as i32) % range) + low
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        // Counter should start at 0
        assert_eq!(CHAOS_MODE_COUNTER.load(Ordering::Relaxed), 0);
        // Any feature should not be active with counter=0
        assert!(!is_active(ChaosFeature::Any));
    }

    #[test]
    fn test_enter_leave_chaos_mode() {
        let initial = CHAOS_MODE_COUNTER.load(Ordering::Relaxed);
        
        enter_chaos_mode();
        assert_eq!(CHAOS_MODE_COUNTER.load(Ordering::Relaxed), initial + 1);
        assert!(is_active(ChaosFeature::Any));
        
        leave_chaos_mode();
        assert_eq!(CHAOS_MODE_COUNTER.load(Ordering::Relaxed), initial);
    }

    #[test]
    fn test_nesting() {
        let initial = CHAOS_MODE_COUNTER.load(Ordering::Relaxed);
        
        enter_chaos_mode();
        enter_chaos_mode();
        enter_chaos_mode();
        assert_eq!(CHAOS_MODE_COUNTER.load(Ordering::Relaxed), initial + 3);
        
        leave_chaos_mode();
        leave_chaos_mode();
        leave_chaos_mode();
        assert_eq!(CHAOS_MODE_COUNTER.load(Ordering::Relaxed), initial);
    }

    #[test]
    fn test_feature_checking() {
        set_chaos_feature(ChaosFeature::ThreadScheduling);
        
        let initial = CHAOS_MODE_COUNTER.load(Ordering::Relaxed);
        assert!(!is_active(ChaosFeature::ThreadScheduling));
        
        enter_chaos_mode();
        assert!(is_active(ChaosFeature::ThreadScheduling));
        assert!(!is_active(ChaosFeature::NetworkScheduling));
        
        leave_chaos_mode();
        assert!(!is_active(ChaosFeature::ThreadScheduling));
        assert_eq!(CHAOS_MODE_COUNTER.load(Ordering::Relaxed), initial);
    }

    #[test]
    fn test_random_u32_less_than() {
        // Test bounds checking
        for _ in 0..100 {
            let val = random_u32_less_than(10);
            assert!(val < 10);
        }
        
        // Test bound of 1 always returns 0
        for _ in 0..10 {
            assert_eq!(random_u32_less_than(1), 0);
        }
    }

    #[test]
    fn test_random_i32_in_range() {
        // Test range bounds
        for _ in 0..100 {
            let val = random_i32_in_range(-10, 10);
            assert!(val >= -10 && val <= 10);
        }
        
        // Test single value range
        for _ in 0..10 {
            assert_eq!(random_i32_in_range(5, 5), 5);
        }
        
        // Test positive range
        for _ in 0..100 {
            let val = random_i32_in_range(0, 100);
            assert!(val >= 0 && val <= 100);
        }
    }

    #[test]
    fn test_chaos_feature_values() {
        // Verify enum values match C++ constants
        assert_eq!(ChaosFeature::None as u32, 0x0);
        assert_eq!(ChaosFeature::ThreadScheduling as u32, 0x1);
        assert_eq!(ChaosFeature::NetworkScheduling as u32, 0x2);
        assert_eq!(ChaosFeature::TimerScheduling as u32, 0x4);
        assert_eq!(ChaosFeature::IOAmounts as u32, 0x8);
        assert_eq!(ChaosFeature::HashTableIteration as u32, 0x10);
        assert_eq!(ChaosFeature::ImageCache as u32, 0x20);
        assert_eq!(ChaosFeature::TaskDispatching as u32, 0x40);
        assert_eq!(ChaosFeature::TaskRunning as u32, 0x80);
        assert_eq!(ChaosFeature::Any as u32, 0xffffffff);
    }
}

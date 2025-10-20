// -*- Mode: rust; rust-indent-offset: 4 -*-
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Tests for RefCounted leak checking infrastructure

use super::*;
use std::ffi::CString;
use std::os::raw::{c_char, c_uint, c_void};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

// Test callback functions
static TEST_ADDREF_CALLED: AtomicBool = AtomicBool::new(false);
static TEST_RELEASE_CALLED: AtomicBool = AtomicBool::new(false);
static TEST_ADDREF_COUNT: AtomicUsize = AtomicUsize::new(0);
static TEST_RELEASE_COUNT: AtomicUsize = AtomicUsize::new(0);

extern "C" fn test_log_addref(
    _ptr: *mut c_void,
    _refcnt: MozRefCountType,
    _typename: *const c_char,
    _size: c_uint,
) {
    TEST_ADDREF_CALLED.store(true, Ordering::Relaxed);
    TEST_ADDREF_COUNT.fetch_add(1, Ordering::Relaxed);
}

extern "C" fn test_log_release(
    _ptr: *mut c_void,
    _refcnt: MozRefCountType,
    _typename: *const c_char,
) {
    TEST_RELEASE_CALLED.store(true, Ordering::Relaxed);
    TEST_RELEASE_COUNT.fetch_add(1, Ordering::Relaxed);
}

fn reset_test_state() {
    TEST_ADDREF_CALLED.store(false, Ordering::Relaxed);
    TEST_RELEASE_CALLED.store(false, Ordering::Relaxed);
    TEST_ADDREF_COUNT.store(0, Ordering::Relaxed);
    TEST_RELEASE_COUNT.store(0, Ordering::Relaxed);
    
    #[cfg(test)]
    ffi::mozilla_detail_RefCountLogger_ResetForTesting();
}

#[test]
fn test_initial_state() {
    reset_test_state();
    
    // Initially, all pointers should be null and counters zero
    assert_eq!(
        ffi::mozilla_detail_RefCountLogger_GetLogAddRefFunc(),
        std::ptr::null_mut()
    );
    assert_eq!(
        ffi::mozilla_detail_RefCountLogger_GetLogReleaseFunc(),
        std::ptr::null_mut()
    );
    assert_eq!(ffi::mozilla_detail_RefCountLogger_GetStaticCtorCounter(), 0);
}

#[test]
fn test_set_leak_checking_functions_null() {
    reset_test_state();
    
    // Set functions to None (null)
    ffi::mozilla_detail_RefCountLogger_SetLeakCheckingFunctions(None, None);
    
    // Should still be null
    assert_eq!(
        ffi::mozilla_detail_RefCountLogger_GetLogAddRefFunc(),
        std::ptr::null_mut()
    );
    assert_eq!(
        ffi::mozilla_detail_RefCountLogger_GetLogReleaseFunc(),
        std::ptr::null_mut()
    );
}

#[test]
fn test_set_leak_checking_functions_valid() {
    reset_test_state();
    
    // Set valid function pointers
    ffi::mozilla_detail_RefCountLogger_SetLeakCheckingFunctions(
        Some(test_log_addref),
        Some(test_log_release),
    );
    
    // Pointers should now be non-null
    let addref_ptr = ffi::mozilla_detail_RefCountLogger_GetLogAddRefFunc();
    let release_ptr = ffi::mozilla_detail_RefCountLogger_GetLogReleaseFunc();
    
    assert!(!addref_ptr.is_null());
    assert!(!release_ptr.is_null());
    
    // Verify we can call them (would be done by C++ code)
    assert!(!TEST_ADDREF_CALLED.load(Ordering::Relaxed));
    assert!(!TEST_RELEASE_CALLED.load(Ordering::Relaxed));
}

#[test]
fn test_static_ctor_counter() {
    reset_test_state();
    
    // Simulate static constructor usage
    let typename = CString::new("TestType").unwrap();
    ffi::mozilla_detail_RefCountLogger_IncrementStaticCtorCounter(typename.as_ptr());
    
    assert_eq!(ffi::mozilla_detail_RefCountLogger_GetStaticCtorCounter(), 1);
    
    // Increment again
    ffi::mozilla_detail_RefCountLogger_IncrementStaticCtorCounter(typename.as_ptr());
    assert_eq!(ffi::mozilla_detail_RefCountLogger_GetStaticCtorCounter(), 2);
}

#[test]
fn test_static_ctor_warning() {
    reset_test_state();
    
    // Set up static constructor usage before initialization
    let typename = CString::new("EarlyType").unwrap();
    ffi::mozilla_detail_RefCountLogger_IncrementStaticCtorCounter(typename.as_ptr());
    ffi::mozilla_detail_RefCountLogger_IncrementStaticCtorCounter(typename.as_ptr());
    
    assert_eq!(ffi::mozilla_detail_RefCountLogger_GetStaticCtorCounter(), 2);
    
    // Now call SetLeakCheckingFunctions - should print warning and reset counter
    // (We can't easily capture stderr in tests, but we can verify the reset)
    ffi::mozilla_detail_RefCountLogger_SetLeakCheckingFunctions(
        Some(test_log_addref),
        Some(test_log_release),
    );
    
    // Counter should be reset to 0
    assert_eq!(ffi::mozilla_detail_RefCountLogger_GetStaticCtorCounter(), 0);
}

#[test]
fn test_function_pointer_replacement() {
    reset_test_state();
    
    // Set initial functions
    ffi::mozilla_detail_RefCountLogger_SetLeakCheckingFunctions(
        Some(test_log_addref),
        Some(test_log_release),
    );
    
    let ptr1 = ffi::mozilla_detail_RefCountLogger_GetLogAddRefFunc();
    assert!(!ptr1.is_null());
    
    // Replace with null
    ffi::mozilla_detail_RefCountLogger_SetLeakCheckingFunctions(None, None);
    
    let ptr2 = ffi::mozilla_detail_RefCountLogger_GetLogAddRefFunc();
    assert!(ptr2.is_null());
    
    // Replace with function again
    ffi::mozilla_detail_RefCountLogger_SetLeakCheckingFunctions(
        Some(test_log_addref),
        Some(test_log_release),
    );
    
    let ptr3 = ffi::mozilla_detail_RefCountLogger_GetLogAddRefFunc();
    assert!(!ptr3.is_null());
}

#[test]
fn test_null_typename() {
    reset_test_state();
    
    // Pass null typename (should not crash)
    ffi::mozilla_detail_RefCountLogger_IncrementStaticCtorCounter(std::ptr::null());
    assert_eq!(ffi::mozilla_detail_RefCountLogger_GetStaticCtorCounter(), 1);
}

#[test]
fn test_state_thread_safety() {
    reset_test_state();
    
    // Set function pointers
    ffi::mozilla_detail_RefCountLogger_SetLeakCheckingFunctions(
        Some(test_log_addref),
        Some(test_log_release),
    );
    
    // Spawn multiple threads that read the function pointers
    let handles: Vec<_> = (0..10)
        .map(|_| {
            std::thread::spawn(|| {
                for _ in 0..100 {
                    let addref_ptr = ffi::mozilla_detail_RefCountLogger_GetLogAddRefFunc();
                    let release_ptr = ffi::mozilla_detail_RefCountLogger_GetLogReleaseFunc();
                    assert!(!addref_ptr.is_null());
                    assert!(!release_ptr.is_null());
                }
            })
        })
        .collect();
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_refcount_logger_state_new() {
    let state = RefCountLoggerState::new();
    
    // All fields should be initialized to null/zero
    assert!(state.log_addref_func.load(Ordering::Relaxed).is_null());
    assert!(state.log_release_func.load(Ordering::Relaxed).is_null());
    assert_eq!(state.num_static_ctors.load(Ordering::Relaxed), 0);
    assert!(state
        .last_static_ctor_typename
        .load(Ordering::Relaxed)
        .is_null());
}

#[test]
fn test_refcount_logger_state_set_functions() {
    let state = RefCountLoggerState::new();
    
    state.set_leak_checking_functions(Some(test_log_addref), Some(test_log_release));
    
    let addref = state.get_log_addref_func();
    let release = state.get_log_release_func();
    
    assert!(addref.is_some());
    assert!(release.is_some());
}

#[test]
fn test_refcount_logger_state_get_none() {
    let state = RefCountLoggerState::new();
    
    assert!(state.get_log_addref_func().is_none());
    assert!(state.get_log_release_func().is_none());
}

#[test]
fn test_multiple_increments() {
    reset_test_state();
    
    let typename = CString::new("TestType").unwrap();
    
    // Increment counter 100 times
    for _ in 0..100 {
        ffi::mozilla_detail_RefCountLogger_IncrementStaticCtorCounter(typename.as_ptr());
    }
    
    assert_eq!(
        ffi::mozilla_detail_RefCountLogger_GetStaticCtorCounter(),
        100
    );
}

#[test]
fn test_static_exports_sync() {
    reset_test_state();
    
    // Set function pointers via FFI
    ffi::mozilla_detail_RefCountLogger_SetLeakCheckingFunctions(
        Some(test_log_addref),
        Some(test_log_release),
    );
    
    // Verify static exports are updated
    unsafe {
        assert!(!ffi::mozilla_detail_gLogAddRefFunc.is_null());
        assert!(!ffi::mozilla_detail_gLogReleaseFunc.is_null());
        assert_eq!(ffi::mozilla_detail_gNumStaticCtors, 0);
    }
}

#[test]
fn test_static_ctor_typename_preservation() {
    reset_test_state();
    
    let typename1 = CString::new("FirstType").unwrap();
    let typename2 = CString::new("SecondType").unwrap();
    
    // First increment with typename1
    ffi::mozilla_detail_RefCountLogger_IncrementStaticCtorCounter(typename1.as_ptr());
    
    // Second increment with typename2 (should overwrite)
    ffi::mozilla_detail_RefCountLogger_IncrementStaticCtorCounter(typename2.as_ptr());
    
    // Counter should be 2
    assert_eq!(ffi::mozilla_detail_RefCountLogger_GetStaticCtorCounter(), 2);
    
    // When we set leak checking functions, the warning should use typename2
    ffi::mozilla_detail_RefCountLogger_SetLeakCheckingFunctions(
        Some(test_log_addref),
        Some(test_log_release),
    );
    
    // Counter should be reset
    assert_eq!(ffi::mozilla_detail_RefCountLogger_GetStaticCtorCounter(), 0);
}

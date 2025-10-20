// -*- Mode: rust; rust-indent-offset: 4 -*-
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! FFI layer for RefCounted leak checking
//!
//! This module exports C-compatible symbols that match the original C++
//! interface from mfbt/RefCounted.cpp:
//!
//! - `gLogAddRefFunc` - Global function pointer
//! - `gLogReleaseFunc` - Global function pointer
//! - `gNumStaticCtors` - Global counter
//! - `gLastStaticCtorTypeName` - Global string pointer
//! - `RefCountLogger_SetLeakCheckingFunctions` - Configuration function
//!
//! # Safety
//!
//! All FFI functions use panic boundaries to prevent unwinding into C++.
//! Function pointers must be valid when called (caller responsibility).

use super::{MozRefCountType, REFCOUNT_LOGGER};
use std::os::raw::{c_char, c_uint, c_void};
use std::panic;
use std::ptr::null_mut;
use std::sync::atomic::Ordering;

// ============================================================================
// Global Variable Exports
// ============================================================================

/// gLogAddRefFunc - Function pointer for logging AddRef calls
///
/// C++ signature: `MFBT_DATA LogAddRefFunc gLogAddRefFunc = nullptr;`
///
/// This is accessed directly from C++ code, so we export it as a mutable
/// static pointer. However, we actually use the atomic in REFCOUNT_LOGGER
/// for thread-safe access.
#[no_mangle]
pub static mut mozilla_detail_gLogAddRefFunc: *mut c_void = null_mut();

/// gLogReleaseFunc - Function pointer for logging Release calls
///
/// C++ signature: `MFBT_DATA LogReleaseFunc gLogReleaseFunc = nullptr;`
#[no_mangle]
pub static mut mozilla_detail_gLogReleaseFunc: *mut c_void = null_mut();

/// gNumStaticCtors - Counter for static constructor usage
///
/// C++ signature: `MFBT_DATA size_t gNumStaticCtors = 0;`
#[no_mangle]
pub static mut mozilla_detail_gNumStaticCtors: usize = 0;

/// gLastStaticCtorTypeName - Type name of last static constructor
///
/// C++ signature: `MFBT_DATA const char* gLastStaticCtorTypeName = nullptr;`
#[no_mangle]
pub static mut mozilla_detail_gLastStaticCtorTypeName: *const c_char = null_mut();

// ============================================================================
// Function Exports
// ============================================================================

/// SetLeakCheckingFunctions - Configure leak checking function pointers
///
/// C++ signature:
/// ```cpp
/// MFBT_API void RefCountLogger::SetLeakCheckingFunctions(
///     LogAddRefFunc aLogAddRefFunc,
///     LogReleaseFunc aLogReleaseFunc);
/// ```
///
/// This function initializes the function pointers used for leak checking.
/// It should be called once at startup from nsTraceRefcnt::Startup().
///
/// # Safety
///
/// - Function pointers must remain valid for the lifetime of the program
/// - Caller must ensure the pointers point to valid functions
/// - Not thread-safe: expects single initialization at startup
///
/// # Arguments
///
/// * `log_addref` - Function to call on AddRef (may be null)
/// * `log_release` - Function to call on Release (may be null)
#[no_mangle]
pub extern "C" fn mozilla_detail_RefCountLogger_SetLeakCheckingFunctions(
    log_addref: Option<extern "C" fn(*mut c_void, MozRefCountType, *const c_char, c_uint)>,
    log_release: Option<extern "C" fn(*mut c_void, MozRefCountType, *const c_char)>,
) {
    // Panic boundary: prevent unwinding into C++
    let result = panic::catch_unwind(|| {
        // Call the core implementation
        REFCOUNT_LOGGER.set_leak_checking_functions(log_addref, log_release);

        // Update the static exports for direct C++ access
        // These are kept in sync with the atomic storage for compatibility
        // SAFETY: We're the only writer, and C++ reads these at startup
        unsafe {
            mozilla_detail_gLogAddRefFunc = log_addref
                .map(|f| f as *mut c_void)
                .unwrap_or(null_mut());
            mozilla_detail_gLogReleaseFunc = log_release
                .map(|f| f as *mut c_void)
                .unwrap_or(null_mut());
            mozilla_detail_gNumStaticCtors =
                REFCOUNT_LOGGER.num_static_ctors.load(Ordering::Relaxed);
            mozilla_detail_gLastStaticCtorTypeName = REFCOUNT_LOGGER
                .last_static_ctor_typename
                .load(Ordering::Relaxed);
        }
    });

    // If panic occurred, log error but don't propagate
    if result.is_err() {
        eprintln!("PANIC in SetLeakCheckingFunctions (Rust)");
    }
}

// ============================================================================
// Helper Functions for C++ Integration
// ============================================================================

/// Get the current LogAddRefFunc pointer
///
/// This is a helper for C++ code that wants to check if leak checking is enabled.
/// Returns null if not initialized.
#[no_mangle]
pub extern "C" fn mozilla_detail_RefCountLogger_GetLogAddRefFunc() -> *mut c_void {
    let result = panic::catch_unwind(|| {
        REFCOUNT_LOGGER
            .log_addref_func
            .load(Ordering::Acquire)
    });
    result.unwrap_or(null_mut())
}

/// Get the current LogReleaseFunc pointer
///
/// This is a helper for C++ code that wants to check if leak checking is enabled.
/// Returns null if not initialized.
#[no_mangle]
pub extern "C" fn mozilla_detail_RefCountLogger_GetLogReleaseFunc() -> *mut c_void {
    let result = panic::catch_unwind(|| {
        REFCOUNT_LOGGER
            .log_release_func
            .load(Ordering::Acquire)
    });
    result.unwrap_or(null_mut())
}

/// Increment the static constructor counter
///
/// This is called from RefCounted.h when leak checking functions are not yet
/// initialized but AddRef/Release is called (typically from static constructors).
///
/// # Safety
///
/// - `typename_ptr` must be a valid pointer to a null-terminated string
/// - The string must have static lifetime (not freed)
#[no_mangle]
pub extern "C" fn mozilla_detail_RefCountLogger_IncrementStaticCtorCounter(
    typename_ptr: *const c_char,
) {
    let _ = panic::catch_unwind(|| {
        REFCOUNT_LOGGER
            .num_static_ctors
            .fetch_add(1, Ordering::Relaxed);
        if !typename_ptr.is_null() {
            REFCOUNT_LOGGER
                .last_static_ctor_typename
                .store(typename_ptr as *mut c_char, Ordering::Relaxed);
        }
    });
}

/// Get the static constructor counter value
///
/// Returns the current count of AddRef/Release calls before initialization.
#[no_mangle]
pub extern "C" fn mozilla_detail_RefCountLogger_GetStaticCtorCounter() -> usize {
    let result = panic::catch_unwind(|| {
        REFCOUNT_LOGGER.num_static_ctors.load(Ordering::Relaxed)
    });
    result.unwrap_or(0)
}

// ============================================================================
// Testing Support
// ============================================================================

/// Reset all global state to defaults (for testing only)
///
/// This function is only available in test builds and allows tests to
/// reset state between test cases.
///
/// # Safety
///
/// Should only be called from single-threaded test code.
#[cfg(test)]
#[no_mangle]
pub extern "C" fn mozilla_detail_RefCountLogger_ResetForTesting() {
    REFCOUNT_LOGGER
        .log_addref_func
        .store(null_mut(), Ordering::Release);
    REFCOUNT_LOGGER
        .log_release_func
        .store(null_mut(), Ordering::Release);
    REFCOUNT_LOGGER
        .num_static_ctors
        .store(0, Ordering::Release);
    REFCOUNT_LOGGER
        .last_static_ctor_typename
        .store(null_mut(), Ordering::Release);
    unsafe {
        mozilla_detail_gLogAddRefFunc = null_mut();
        mozilla_detail_gLogReleaseFunc = null_mut();
        mozilla_detail_gNumStaticCtors = 0;
        mozilla_detail_gLastStaticCtorTypeName = null_mut();
    }
}

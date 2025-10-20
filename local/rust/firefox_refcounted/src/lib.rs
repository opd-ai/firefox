// -*- Mode: rust; rust-indent-offset: 4 -*-
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Rust port of mfbt/RefCounted.cpp - RefCount leak checking infrastructure
//!
//! This module provides the global state and configuration for Mozilla's
//! RefCounted leak detection system. When the `leak-checking` feature is
//! enabled (corresponding to MOZ_REFCOUNTED_LEAK_CHECKING in C++), it exports:
//!
//! - `gLogAddRefFunc` - Function pointer for logging AddRef calls
//! - `gLogReleaseFunc` - Function pointer for logging Release calls
//! - `gNumStaticCtors` - Counter for static constructor usage
//! - `gLastStaticCtorTypeName` - Type name of last static constructor
//! - `SetLeakCheckingFunctions()` - Configuration function
//!
//! # Thread Safety
//!
//! The current implementation matches C++ semantics:
//! - `SetLeakCheckingFunctions` expects single-threaded initialization (called once at startup)
//! - Function pointers are read by many threads concurrently after initialization
//! - Uses atomic operations for thread-safe concurrent reads
//!
//! # FFI Safety
//!
//! All exports use C linkage (#[no_mangle], extern "C") for compatibility with
//! C++ code. Function pointers must match exact signatures expected by C++.

#![cfg_attr(feature = "leak-checking", allow(static_mut_refs))]

use std::os::raw::{c_char, c_uint, c_void};
use std::ptr::null_mut;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

/// MozRefCountType - typically uint32_t in C++
pub type MozRefCountType = c_uint;

/// Function pointer type for logging AddRef calls
///
/// Signature: void (*)(void* aPtr, MozRefCountType aNewRefCnt, 
///                     const char* aTypeName, uint32_t aClassSize)
#[cfg(feature = "leak-checking")]
pub type LogAddRefFunc =
    Option<extern "C" fn(*mut c_void, MozRefCountType, *const c_char, c_uint)>;

/// Function pointer type for logging Release calls
///
/// Signature: void (*)(void* aPtr, MozRefCountType aNewRefCnt,
///                     const char* aTypeName)
#[cfg(feature = "leak-checking")]
pub type LogReleaseFunc =
    Option<extern "C" fn(*mut c_void, MozRefCountType, *const c_char)>;

/// Global state for RefCount leak checking
#[cfg(feature = "leak-checking")]
pub struct RefCountLoggerState {
    /// Function pointer for logging AddRef calls
    pub log_addref_func: AtomicPtr<c_void>,
    /// Function pointer for logging Release calls
    pub log_release_func: AtomicPtr<c_void>,
    /// Counter for static constructor usage (warning detection)
    pub num_static_ctors: AtomicUsize,
    /// Type name of last static constructor
    pub last_static_ctor_typename: AtomicPtr<c_char>,
}

#[cfg(feature = "leak-checking")]
impl RefCountLoggerState {
    /// Create new state with all fields initialized to null/zero
    #[must_use]
    pub const fn new() -> Self {
        RefCountLoggerState {
            log_addref_func: AtomicPtr::new(null_mut()),
            log_release_func: AtomicPtr::new(null_mut()),
            num_static_ctors: AtomicUsize::new(0),
            last_static_ctor_typename: AtomicPtr::new(null_mut()),
        }
    }

    /// Set leak checking function pointers
    ///
    /// This function is expected to be called once at startup from
    /// nsTraceRefcnt::Startup(). It is NOT thread-safe and assumes
    /// single-threaded initialization.
    ///
    /// # Arguments
    ///
    /// * `log_addref` - Function to call when AddRef is invoked
    /// * `log_release` - Function to call when Release is invoked
    ///
    /// # Side Effects
    ///
    /// - If `num_static_ctors` > 0, prints warning to stderr
    /// - Resets `num_static_ctors` and `last_static_ctor_typename`
    /// - Updates `log_addref_func` and `log_release_func`
    pub fn set_leak_checking_functions(
        &self,
        log_addref: LogAddRefFunc,
        log_release: LogReleaseFunc,
    ) {
        // Check if RefCounted was used before initialization
        let num_ctors = self.num_static_ctors.load(Ordering::Relaxed);
        if num_ctors > 0 {
            // Get the last type name before resetting
            let last_typename = self.last_static_ctor_typename.load(Ordering::Relaxed);
            let typename_str = if last_typename.is_null() {
                "unknown"
            } else {
                // SAFETY: last_typename points to a static string literal from C++
                // (typeName() returns const char* to static data)
                unsafe {
                    std::ffi::CStr::from_ptr(last_typename)
                        .to_str()
                        .unwrap_or("invalid utf-8")
                }
            };

            // Print warning to stderr (matches C++ fprintf behavior)
            eprintln!(
                "RefCounted objects addrefed/released (static ctor?) total: {}, last type: {}",
                num_ctors, typename_str
            );

            // Reset counters
            self.num_static_ctors.store(0, Ordering::Relaxed);
            self.last_static_ctor_typename
                .store(null_mut(), Ordering::Relaxed);
        }

        // Store function pointers
        // Cast Option<fn> to *mut c_void for storage
        let addref_ptr = log_addref
            .map(|f| f as *mut c_void)
            .unwrap_or(null_mut());
        let release_ptr = log_release
            .map(|f| f as *mut c_void)
            .unwrap_or(null_mut());

        self.log_addref_func.store(addref_ptr, Ordering::Release);
        self.log_release_func
            .store(release_ptr, Ordering::Release);
    }

    /// Get the LogAddRefFunc function pointer
    ///
    /// Returns None if not initialized, Some(fn) otherwise.
    /// Uses Acquire ordering for synchronization.
    #[inline]
    #[must_use]
    pub fn get_log_addref_func(&self) -> LogAddRefFunc {
        let ptr = self.log_addref_func.load(Ordering::Acquire);
        if ptr.is_null() {
            None
        } else {
            // SAFETY: We stored this as a function pointer in set_leak_checking_functions
            Some(unsafe { std::mem::transmute::<*mut c_void, extern "C" fn(*mut c_void, MozRefCountType, *const c_char, c_uint)>(ptr) })
        }
    }

    /// Get the LogReleaseFunc function pointer
    ///
    /// Returns None if not initialized, Some(fn) otherwise.
    /// Uses Acquire ordering for synchronization.
    #[inline]
    #[must_use]
    pub fn get_log_release_func(&self) -> LogReleaseFunc {
        let ptr = self.log_release_func.load(Ordering::Acquire);
        if ptr.is_null() {
            None
        } else {
            // SAFETY: We stored this as a function pointer in set_leak_checking_functions
            Some(unsafe { std::mem::transmute::<*mut c_void, extern "C" fn(*mut c_void, MozRefCountType, *const c_char)>(ptr) })
        }
    }
}

impl Default for RefCountLoggerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Global instance of RefCountLoggerState
///
/// This is a static variable with interior mutability via atomic types.
/// It's safe to access from multiple threads concurrently.
#[cfg(feature = "leak-checking")]
pub static REFCOUNT_LOGGER: RefCountLoggerState = RefCountLoggerState::new();

// Re-export FFI module
#[cfg(feature = "leak-checking")]
pub mod ffi;

#[cfg(feature = "leak-checking")]
#[cfg(test)]
mod tests;

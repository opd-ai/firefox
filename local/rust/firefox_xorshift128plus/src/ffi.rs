// -*- Mode: rust; tab-width: 4; indent-tabs-mode: nil; c-basic-offset: 2 -*-
// vim: set ts=4 sts=2 et sw=2 tw=80:
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! FFI bindings for XorShift128PlusRNG
//!
//! This module provides C-compatible FFI exports for the XorShift128+ PRNG,
//! allowing C++ code to use the Rust implementation. This includes both
//! production code and test code.
//!
//! ## Memory Management
//!
//! The FFI layer uses opaque pointers for RNG instances. C++ code is responsible
//! for calling the destructor when done.
//!
//! ## Naming Convention
//!
//! All FFI functions are prefixed with `xorshift128plus_` to avoid naming conflicts.

use crate::XorShift128PlusRNG;
use std::panic;

/// FFI-safe constructor: Create new XorShift128+ RNG
///
/// # Safety
///
/// This function is safe to call from C++. Returns a heap-allocated RNG instance.
/// Caller must call `xorshift128plus_destroy` when done.
///
/// # Arguments
///
/// * `initial0` - First seed value
/// * `initial1` - Second seed value
///
/// # Returns
///
/// Pointer to newly allocated RNG, or null on allocation failure
///
/// # Note
///
/// At least one of initial0, initial1 should be non-zero for proper operation.
#[no_mangle]
pub extern "C" fn xorshift128plus_new(initial0: u64, initial1: u64) -> *mut XorShift128PlusRNG {
    // Catch panics to prevent unwinding into C++
    let result = panic::catch_unwind(|| {
        Box::into_raw(Box::new(XorShift128PlusRNG::new(initial0, initial1)))
    });

    match result {
        Ok(ptr) => ptr,
        Err(_) => std::ptr::null_mut(),
    }
}

/// FFI-safe destructor: Destroy XorShift128+ RNG
///
/// # Safety
///
/// `rng` must be a valid pointer returned from `xorshift128plus_new` and not
/// previously destroyed. Passing null is safe (no-op).
#[no_mangle]
pub unsafe extern "C" fn xorshift128plus_destroy(rng: *mut XorShift128PlusRNG) {
    if !rng.is_null() {
        // Catch panics during destruction
        let _ = panic::catch_unwind(|| {
            unsafe {
                let _ = Box::from_raw(rng);
            }
        });
    }
}

/// FFI-safe next: Generate next pseudo-random 64-bit number
///
/// # Safety
///
/// `rng` must be a valid pointer to an XorShift128PlusRNG instance.
///
/// # Returns
///
/// Next pseudo-random u64 value, or 0 if rng is null or panic occurs
#[no_mangle]
pub unsafe extern "C" fn xorshift128plus_next(rng: *mut XorShift128PlusRNG) -> u64 {
    if rng.is_null() {
        return 0;
    }

    // Catch panics
    let result = panic::catch_unwind(|| unsafe { (*rng).next() });

    result.unwrap_or(0)
}

/// FFI-safe nextDouble: Generate next pseudo-random double in [0, 1)
///
/// # Safety
///
/// `rng` must be a valid pointer to an XorShift128PlusRNG instance.
///
/// # Returns
///
/// Next pseudo-random f64 value in [0.0, 1.0), or 0.0 if rng is null or panic occurs
#[no_mangle]
pub unsafe extern "C" fn xorshift128plus_next_double(rng: *mut XorShift128PlusRNG) -> f64 {
    if rng.is_null() {
        return 0.0;
    }

    // Catch panics
    let result = panic::catch_unwind(|| unsafe { (*rng).next_double() });

    result.unwrap_or(0.0)
}

/// FFI-safe setState: Set RNG state to specific values
///
/// # Safety
///
/// `rng` must be a valid pointer to an XorShift128PlusRNG instance.
///
/// # Note
///
/// At least one of state0, state1 should be non-zero for proper operation.
#[no_mangle]
pub unsafe extern "C" fn xorshift128plus_set_state(
    rng: *mut XorShift128PlusRNG,
    state0: u64,
    state1: u64,
) {
    if rng.is_null() {
        return;
    }

    // Catch panics
    let _ = panic::catch_unwind(|| unsafe {
        (*rng).set_state(state0, state1);
    });
}

/// FFI-safe offsetOfState0: Get byte offset of state[0]
///
/// This is used by JIT code for direct memory access.
///
/// # Returns
///
/// Byte offset of state[0] from start of struct (always 0)
#[no_mangle]
pub extern "C" fn xorshift128plus_offset_of_state0() -> usize {
    XorShift128PlusRNG::offset_of_state0()
}

/// FFI-safe offsetOfState1: Get byte offset of state[1]
///
/// This is used by JIT code for direct memory access.
///
/// # Returns
///
/// Byte offset of state[1] from start of struct (always 8)
#[no_mangle]
pub extern "C" fn xorshift128plus_offset_of_state1() -> usize {
    XorShift128PlusRNG::offset_of_state1()
}

// Additional FFI helpers for C++ compatibility

/// Get the size of XorShift128PlusRNG struct
///
/// Used by C++ code to verify struct size matches expectations.
///
/// # Returns
///
/// Size in bytes (always 16)
#[no_mangle]
pub extern "C" fn xorshift128plus_size_of() -> usize {
    std::mem::size_of::<XorShift128PlusRNG>()
}

#[cfg(test)]
mod ffi_tests {
    use super::*;

    #[test]
    fn test_ffi_lifecycle() {
        // Test FFI constructor, methods, and destructor
        unsafe {
            let rng = xorshift128plus_new(1, 4);
            assert!(!rng.is_null());

            // Test next()
            let val1 = xorshift128plus_next(rng);
            assert_eq!(val1, 0x800049);

            // Test setState() and reproducibility
            xorshift128plus_set_state(rng, 1, 4);
            let val2 = xorshift128plus_next(rng);
            assert_eq!(val2, 0x800049);

            // Test nextDouble()
            let d = xorshift128plus_next_double(rng);
            assert!(d >= 0.0 && d < 1.0);

            // Test destructor
            xorshift128plus_destroy(rng);
        }
    }

    #[test]
    fn test_ffi_null_safety() {
        // Test that null pointers don't crash
        unsafe {
            let val = xorshift128plus_next(std::ptr::null_mut());
            assert_eq!(val, 0);

            let d = xorshift128plus_next_double(std::ptr::null_mut());
            assert_eq!(d, 0.0);

            xorshift128plus_set_state(std::ptr::null_mut(), 1, 2);

            xorshift128plus_destroy(std::ptr::null_mut());
        }
    }

    #[test]
    fn test_ffi_offsets() {
        // Verify offset functions return correct values
        assert_eq!(xorshift128plus_offset_of_state0(), 0);
        assert_eq!(xorshift128plus_offset_of_state1(), 8);
        assert_eq!(xorshift128plus_size_of(), 16);
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! FFI bindings for nsQueryArrayElementAt
//!
//! This module provides the C-compatible FFI interface for the
//! nsQueryArrayElementAt::operator() function.
//!
//! ## FFI Safety
//!
//! All functions in this module are marked #[no_mangle] and use extern "C"
//! calling convention for C++ interoperability. Each function:
//!
//! 1. Validates all pointer parameters (null checks)
//! 2. Wraps calls in panic::catch_unwind to prevent unwinding into C++
//! 3. Returns appropriate nsresult error codes
//! 4. Never panics across FFI boundary
//!
//! ## Usage from C++
//!
//! The C++ code calls this function as:
//! ```cpp
//! extern "C" nsresult nsQueryArrayElementAt_operator(
//!     nsIArray* array,
//!     uint32_t index,
//!     const nsIID* iid,
//!     void** result,
//!     nsresult* error_ptr
//! );
//! ```

use crate::{nsIArray, nsIID, nsresult, query_array_element_at_impl};
use crate::{NS_ERROR_NULL_POINTER, NS_OK};
use std::os::raw::c_void;
use std::panic;

/// XPCOM error code for unexpected failure
const NS_ERROR_FAILURE: nsresult = 0x80004005;

/// FFI function for nsQueryArrayElementAt::operator()
///
/// This is the main entry point called by C++ code.
///
/// # Parameters
///
/// - `array`: Pointer to nsIArray (may be null, checked)
/// - `index`: Element index (0-based)
/// - `iid`: Interface ID to query for (must not be null)
/// - `result`: Output parameter for the resulting interface pointer (must not be null)
/// - `error_ptr`: Optional output for error code (may be null)
///
/// # Returns
///
/// nsresult status code:
/// - NS_OK (0) on success
/// - NS_ERROR_NULL_POINTER if array/iid/result is null
/// - NS_ERROR_FAILURE if a panic occurs
/// - Other error codes from nsIArray::QueryElementAt
///
/// # Safety
///
/// This function is safe to call from C++ if:
/// - `iid` is a valid pointer to nsIID
/// - `result` is a valid pointer to void*
/// - `array` may be null (will return error)
/// - `error_ptr` may be null (error won't be stored)
///
/// The function will never panic into C++ code - all panics are caught
/// and converted to NS_ERROR_FAILURE.
#[no_mangle]
pub extern "C" fn nsQueryArrayElementAt_operator(
    array: *mut nsIArray,
    index: u32,
    iid: *const nsIID,
    result: *mut *mut c_void,
    error_ptr: *mut nsresult,
) -> nsresult {
    // Catch any panics to prevent unwinding into C++
    let result_code = panic::catch_unwind(|| {
        // Validate required pointers
        if iid.is_null() {
            return NS_ERROR_NULL_POINTER;
        }
        if result.is_null() {
            return NS_ERROR_NULL_POINTER;
        }

        // Call the implementation
        query_array_element_at_impl(array, index, iid, result, error_ptr)
    });

    // Handle panic case
    match result_code {
        Ok(status) => status,
        Err(_) => {
            // Panic occurred - return error
            // Also store in error_ptr if provided
            if !error_ptr.is_null() {
                unsafe {
                    *error_ptr = NS_ERROR_FAILURE;
                }
            }
            NS_ERROR_FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nsIID;

    #[test]
    fn test_ffi_null_iid_returns_error() {
        let array = 0x1234 as *mut nsIArray;
        let mut result: *mut c_void = std::ptr::null_mut();
        let mut error: nsresult = 0;

        let status = nsQueryArrayElementAt_operator(
            array,
            0,
            std::ptr::null(),
            &mut result,
            &mut error,
        );

        assert_eq!(status, NS_ERROR_NULL_POINTER);
    }

    #[test]
    fn test_ffi_null_result_returns_error() {
        let array = 0x1234 as *mut nsIArray;
        let iid = nsIID {
            m0: 0,
            m1: 0,
            m2: 0,
            m3: [0; 8],
        };
        let mut error: nsresult = 0;

        let status = nsQueryArrayElementAt_operator(
            array,
            0,
            &iid,
            std::ptr::null_mut(),
            &mut error,
        );

        assert_eq!(status, NS_ERROR_NULL_POINTER);
    }

    #[test]
    fn test_ffi_null_array_returns_error() {
        let iid = nsIID {
            m0: 0,
            m1: 0,
            m2: 0,
            m3: [0; 8],
        };
        let mut result: *mut c_void = std::ptr::null_mut();
        let mut error: nsresult = 0;

        let status = nsQueryArrayElementAt_operator(
            std::ptr::null_mut(),
            0,
            &iid,
            &mut result,
            &mut error,
        );

        assert_eq!(status, NS_ERROR_NULL_POINTER);
        assert_eq!(error, NS_ERROR_NULL_POINTER);
    }

    #[test]
    fn test_ffi_valid_call_succeeds() {
        let array = 0x1234 as *mut nsIArray;
        let iid = nsIID {
            m0: 0,
            m1: 0,
            m2: 0,
            m3: [0; 8],
        };
        let mut result: *mut c_void = std::ptr::null_mut();
        let mut error: nsresult = 0;

        let status = nsQueryArrayElementAt_operator(
            array,
            0,
            &iid,
            &mut result,
            &mut error,
        );

        assert_eq!(status, NS_OK);
        assert_eq!(error, NS_OK);
    }

    #[test]
    fn test_ffi_null_error_ptr_works() {
        let array = 0x1234 as *mut nsIArray;
        let iid = nsIID {
            m0: 0,
            m1: 0,
            m2: 0,
            m3: [0; 8],
        };
        let mut result: *mut c_void = std::ptr::null_mut();

        let status = nsQueryArrayElementAt_operator(
            array,
            0,
            &iid,
            &mut result,
            std::ptr::null_mut(),
        );

        assert_eq!(status, NS_OK);
    }
}

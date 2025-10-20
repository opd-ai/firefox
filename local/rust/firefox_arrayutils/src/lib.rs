/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Rust port of Firefox nsArrayUtils (nsQueryArrayElementAt helper)
//!
//! This module implements the nsQueryArrayElementAt::operator() function,
//! which provides type-safe element queries from nsIArray XPCOM interfaces.
//!
//! ## Original C++ Implementation
//!
//! ```cpp
//! nsresult nsQueryArrayElementAt::operator()(const nsIID& aIID,
//!                                            void** aResult) const {
//!   nsresult status = mArray ? mArray->QueryElementAt(mIndex, aIID, aResult)
//!                            : NS_ERROR_NULL_POINTER;
//!
//!   if (mErrorPtr) {
//!     *mErrorPtr = status;
//!   }
//!
//!   return status;
//! }
//! ```
//!
//! ## Design
//!
//! The function is a pure helper that:
//! 1. Checks if mArray is null → return NS_ERROR_NULL_POINTER
//! 2. Calls nsIArray::QueryElementAt(mIndex, aIID, aResult)
//! 3. Stores the error code in *mErrorPtr if provided
//! 4. Returns the nsresult status code
//!
//! This is used by the do_QueryElementAt inline helper function to provide
//! type-safe element retrieval from XPCOM arrays, integrated with nsCOMPtr.
//!
//! ## Safety
//!
//! All FFI boundaries are protected with:
//! - Null pointer checks
//! - Panic boundaries (catch_unwind)
//! - Error propagation via nsresult codes
//!
//! ## Examples
//!
//! ```cpp
//! // C++ usage (via do_QueryElementAt helper)
//! nsCOMPtr<nsIFoo> foo = do_QueryElementAt(array, 0);
//! if (!foo) {
//!   return NS_ERROR_FAILURE;
//! }
//! ```

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod ffi;

use std::os::raw::{c_void, c_uint};

/// nsresult type (XPCOM error code)
pub type nsresult = u32;

/// nsIID type (128-bit interface identifier)
///
/// This is a 128-bit GUID used to identify COM interfaces.
/// We treat it as an opaque type passed by pointer from C++.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct nsIID {
    m0: u32,
    m1: u16,
    m2: u16,
    m3: [u8; 8],
}

/// nsIArray opaque type
///
/// We don't need to know the layout, just pass the pointer through.
#[repr(C)]
pub struct nsIArray {
    _private: [u8; 0],
}

// XPCOM error codes (from nsError.h)
/// Success code
pub const NS_OK: nsresult = 0;

/// Null pointer error
pub const NS_ERROR_NULL_POINTER: nsresult = 0x80004003;

/// Core implementation of nsQueryArrayElementAt::operator()
///
/// This function matches the C++ implementation exactly:
/// 1. Check if array is null → return NS_ERROR_NULL_POINTER
/// 2. Call nsIArray::QueryElementAt (via FFI)
/// 3. Store error code in error_ptr if provided
/// 4. Return status code
///
/// # Parameters
///
/// - `array`: Pointer to nsIArray (may be null)
/// - `index`: Element index (0-based)
/// - `iid`: Interface ID to query for
/// - `result`: Output parameter for the resulting interface pointer
/// - `error_ptr`: Optional output for error code (may be null)
///
/// # Returns
///
/// nsresult status code (NS_OK on success, error code on failure)
///
/// # Safety
///
/// This function assumes:
/// - `iid` is a valid pointer to nsIID
/// - `result` is a valid pointer to void*
/// - `array` and `error_ptr` may be null (checked)
/// - Caller manages lifetime of all pointers
pub fn query_array_element_at_impl(
    array: *mut nsIArray,
    index: u32,
    iid: *const nsIID,
    result: *mut *mut c_void,
    error_ptr: *mut nsresult,
) -> nsresult {
    // Step 1: Check if array is null
    if array.is_null() {
        let status = NS_ERROR_NULL_POINTER;
        
        // Store error code if requested
        if !error_ptr.is_null() {
            unsafe {
                *error_ptr = status;
            }
        }
        
        return status;
    }

    // Step 2: Call nsIArray::QueryElementAt through FFI
    // In real implementation, this would call the C++ method
    // In test mode, calls our safe mock function
    #[cfg(not(test))]
    let status = unsafe {
        nsIArray_QueryElementAt(array, index, iid, result)
    };
    #[cfg(test)]
    let status = nsIArray_QueryElementAt(array, index, iid, result);

    // Step 3: Store error code if requested
    if !error_ptr.is_null() {
        unsafe {
            *error_ptr = status;
        }
    }

    // Step 4: Return status
    status
}

/// External declaration for nsIArray::QueryElementAt
///
/// This is the actual C++ method we need to call.
/// The C++ signature is:
/// ```cpp
/// nsresult QueryElementAt(uint32_t index, const nsIID& uuid, void** result);
/// ```
#[cfg(not(test))]
extern "C" {
    fn nsIArray_QueryElementAt(
        this: *mut nsIArray,
        index: c_uint,
        uuid: *const nsIID,
        result: *mut *mut c_void,
    ) -> nsresult;
}

// Mock implementation for tests
#[cfg(test)]
#[no_mangle]
pub extern "C" fn nsIArray_QueryElementAt(
    _this: *mut nsIArray,
    _index: c_uint,
    _uuid: *const nsIID,
    _result: *mut *mut c_void,
) -> nsresult {
    // Mock implementation for tests
    NS_OK
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_array_returns_error() {
        let iid = nsIID {
            m0: 0,
            m1: 0,
            m2: 0,
            m3: [0; 8],
        };
        let mut result: *mut c_void = std::ptr::null_mut();
        let mut error: nsresult = 0;

        let status = query_array_element_at_impl(
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
    fn test_null_error_ptr_works() {
        // Create a dummy array pointer (won't be dereferenced in mock)
        let array = 0x1234 as *mut nsIArray;
        let iid = nsIID {
            m0: 0,
            m1: 0,
            m2: 0,
            m3: [0; 8],
        };
        let mut result: *mut c_void = std::ptr::null_mut();

        let status = query_array_element_at_impl(
            array,
            0,
            &iid,
            &mut result,
            std::ptr::null_mut(),
        );

        // Should succeed without error_ptr
        assert_eq!(status, NS_OK);
    }

    #[test]
    fn test_valid_call_succeeds() {
        // Create a dummy array pointer (won't be dereferenced in mock)
        let array = 0x1234 as *mut nsIArray;
        let iid = nsIID {
            m0: 0,
            m1: 0,
            m2: 0,
            m3: [0; 8],
        };
        let mut result: *mut c_void = std::ptr::null_mut();
        let mut error: nsresult = 0;

        let status = query_array_element_at_impl(
            array,
            0,
            &iid,
            &mut result,
            &mut error,
        );

        assert_eq!(status, NS_OK);
        assert_eq!(error, NS_OK);
    }
}

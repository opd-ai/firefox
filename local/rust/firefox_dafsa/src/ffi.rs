/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! FFI bindings for C++ interoperability.
//!
//! This module provides C-compatible exports that match the C++ Dafsa API.

use crate::{Dafsa, KEY_NOT_FOUND};
use nsstring::nsACString;
use std::slice;

/// Opaque handle for C++ to hold a Dafsa instance.
#[repr(C)]
pub struct RustDafsa {
    inner: Box<Dafsa>,
}

/// Creates a new RustDafsa instance from a data span.
///
/// # Safety
/// - `data` must be a valid pointer to `length` bytes
/// - The data must remain valid for the lifetime of the RustDafsa
#[no_mangle]
pub unsafe extern "C" fn rust_dafsa_new(data: *const u8, length: usize) -> *mut RustDafsa {
    if data.is_null() || length == 0 {
        return std::ptr::null_mut();
    }

    let slice = slice::from_raw_parts(data, length);
    let dafsa = Dafsa::from_slice(slice);

    Box::into_raw(Box::new(RustDafsa {
        inner: Box::new(dafsa),
    }))
}

/// Destroys a RustDafsa instance.
///
/// # Safety
/// - `dafsa` must be a valid pointer returned from `rust_dafsa_new`
/// - `dafsa` must not be used after this call
#[no_mangle]
pub unsafe extern "C" fn rust_dafsa_destroy(dafsa: *mut RustDafsa) {
    if !dafsa.is_null() {
        drop(Box::from_raw(dafsa));
    }
}

/// Looks up a key in the DAFSA.
///
/// # Safety
/// - `dafsa` must be a valid pointer returned from `rust_dafsa_new`
/// - `key` must be a valid pointer to an nsACString
#[no_mangle]
pub unsafe extern "C" fn rust_dafsa_lookup(
    dafsa: *const RustDafsa,
    key: *const nsACString,
) -> i32 {
    if dafsa.is_null() || key.is_null() {
        return KEY_NOT_FOUND;
    }

    let dafsa = &(*dafsa).inner;
    let key_str = (*key).as_str_unchecked();

    dafsa.lookup(key_str)
}

/// Returns the KEY_NOT_FOUND constant for C++ code.
#[no_mangle]
pub extern "C" fn rust_dafsa_key_not_found() -> i32 {
    KEY_NOT_FOUND
}

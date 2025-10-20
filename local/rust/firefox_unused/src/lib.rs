/* -*- Mode: Rust; tab-width: 4; indent-tabs-mode: nil; c-basic-offset: 4 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Rust port of mozilla::Unused
//!
//! This module provides a Rust implementation of the `mozilla::Unused` global,
//! which is used throughout Firefox to suppress compiler warnings for unused
//! return values. The pattern is: `Unused << expr;`
//!
//! # Design
//!
//! The C++ version defines:
//! ```cpp
//! struct unused_t {
//!   template <typename T>
//!   MOZ_ALWAYS_INLINE_EVEN_DEBUG void operator<<(const T& /*unused*/) const {}
//! };
//! extern MFBT_DATA const unused_t Unused;
//! ```
//!
//! Since Rust cannot provide C++ template operator overloads, we use a hybrid approach:
//! - **Rust side**: Exports a static const `mozilla_Unused` object
//! - **C++ side**: Keeps the template operator<< in the header (Unused.h)
//! - **Integration**: C++ header references the Rust-exported symbol
//!
//! # Memory Layout
//!
//! The `UnusedT` struct is 1 byte (same as C++ `unused_t`). We use a dummy
//! byte field because Rust would make a zero-sized type (ZST) 0 bytes, but
//! C++ makes an empty struct 1 byte.
//!
//! # FFI Safety
//!
//! This is a static const export with no function calls. The Rust static is
//! immutable and has 'static lifetime, making it safe for C++ to reference
//! indefinitely.
//!
//! # Usage
//!
//! From C++:
//! ```cpp
//! mozilla::Unused << SomeFunctionReturningValue();
//! ```
//!
//! The left-shift operator is defined in the C++ header and always inlined,
//! providing zero runtime overhead.

#![deny(unsafe_op_in_unsafe_fn)]

/// Rust representation of mozilla::unused_t
///
/// This struct must be 1 byte to match C++ sizeof(unused_t) = 1.
/// C++ makes empty structs 1 byte, but Rust would make ZST 0 bytes.
///
/// # Memory Layout
///
/// - Size: 1 byte
/// - Alignment: 1 byte
/// - Repr: C (for FFI compatibility)
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct UnusedT {
    /// Dummy field to ensure sizeof(UnusedT) = 1 byte (matches C++)
    _private: u8,
}

/// Static const instance of UnusedT, exported to C++ as `mozilla_Unused`
///
/// This is the Rust equivalent of:
/// ```cpp
/// const unused_t Unused = unused_t();
/// ```
///
/// # FFI Safety
///
/// - Symbol: `mozilla_Unused` (C linkage)
/// - Type: const UnusedT (immutable static)
/// - Lifetime: 'static (never deallocated)
/// - Thread-safe: Yes (immutable, no state)
///
/// # C++ Integration
///
/// C++ code accesses this via:
/// ```cpp
/// extern "C" {
///   extern const unused_t mozilla_Unused;
/// }
/// namespace mozilla {
///   static const unused_t& Unused = mozilla_Unused;
/// }
/// ```
#[no_mangle]
pub static mozilla_Unused: UnusedT = UnusedT { _private: 0 };

// Compile-time assertions to verify memory layout matches C++
const _: () = {
    // Verify UnusedT is exactly 1 byte (same as C++ unused_t)
    const UNUSED_SIZE: usize = std::mem::size_of::<UnusedT>();
    assert!(UNUSED_SIZE == 1, "UnusedT must be 1 byte to match C++ unused_t");

    // Verify alignment is 1 byte
    const UNUSED_ALIGN: usize = std::mem::align_of::<UnusedT>();
    assert!(
        UNUSED_ALIGN == 1,
        "UnusedT alignment must be 1 byte to match C++ unused_t"
    );
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unused_size() {
        // Verify UnusedT is 1 byte (same as C++ unused_t)
        assert_eq!(std::mem::size_of::<UnusedT>(), 1);
    }

    #[test]
    fn test_unused_alignment() {
        // Verify UnusedT has 1-byte alignment
        assert_eq!(std::mem::align_of::<UnusedT>(), 1);
    }

    #[test]
    fn test_unused_is_copy() {
        // Verify UnusedT implements Copy (needed for const usage)
        let unused1 = mozilla_Unused;
        let unused2 = unused1; // Should copy, not move
        let _unused3 = unused1; // Should still be usable
        let _unused4 = unused2;
    }

    #[test]
    fn test_unused_is_const() {
        // Verify mozilla_Unused can be used in const context
        const _UNUSED_REF: &UnusedT = &mozilla_Unused;
    }

    #[test]
    fn test_unused_private_field() {
        // Verify _private field is initialized to 0
        assert_eq!(mozilla_Unused._private, 0);
    }

    #[test]
    fn test_unused_multiple_copies() {
        // Verify we can create multiple copies (all identical)
        let copies: [UnusedT; 10] = [mozilla_Unused; 10];
        for copy in &copies {
            assert_eq!(copy._private, 0);
        }
    }
}

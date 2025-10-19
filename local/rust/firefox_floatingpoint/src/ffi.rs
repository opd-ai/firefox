//! FFI (Foreign Function Interface) layer for IsFloat32Representable
//!
//! This module provides C-compatible exports for the Rust implementation,
//! allowing C++ code to call into the Rust version of IsFloat32Representable.
//!
//! # Safety
//!
//! The FFI layer includes panic safety to prevent unwinding across the FFI
//! boundary, which would cause undefined behavior in C++.

use std::panic;

/// C-compatible export of IsFloat32Representable.
///
/// This function provides the FFI boundary for C++ code to call the Rust
/// implementation of IsFloat32Representable. It matches the exact signature
/// and behavior of the C++ function defined in mfbt/FloatingPoint.cpp.
///
/// # Arguments
///
/// * `value` - A double-precision floating point value to check
///
/// # Returns
///
/// * `true` (1) if representable as float32
/// * `false` (0) if not representable or if a panic occurred
///
/// # Safety
///
/// This function is safe to call from C++ code. It:
/// - Uses `#[no_mangle]` to preserve the symbol name for C++ linking
/// - Uses `extern "C"` for C calling convention compatibility
/// - Catches panics to prevent unwinding into C++ (undefined behavior)
/// - Returns a safe default (false) if a panic occurs
///
/// # Panics
///
/// In the extremely unlikely event of a panic (which should never happen for
/// pure mathematical operations), the panic is caught and `false` is returned.
/// This provides a safe fallback behavior.
///
/// # Examples (from C++)
///
/// ```cpp
/// // In C++ code:
/// extern "C" bool IsFloat32Representable(double value);
///
/// bool result = IsFloat32Representable(1.0);  // Returns true
/// bool overflow = IsFloat32Representable(1e50);  // Returns false
/// ```
#[no_mangle]
pub extern "C" fn IsFloat32Representable(value: f64) -> bool {
    // Catch any potential panics to prevent unwinding into C++
    // This is a critical safety measure for FFI boundaries
    match panic::catch_unwind(|| crate::is_float32_representable(value)) {
        Ok(result) => result,
        Err(_) => {
            // In the extremely unlikely event of a panic, return false
            // (conservative: assume not representable if something went wrong)
            //
            // NOTE: This should never happen for pure mathematical operations,
            // but we include it for defense-in-depth safety.
            eprintln!("PANIC in IsFloat32Representable FFI - returning false");
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_basic() {
        // Test that FFI function works identically to Rust function
        assert_eq!(IsFloat32Representable(1.0), true);
        assert_eq!(IsFloat32Representable(2147483647.0), false);
        assert_eq!(IsFloat32Representable(f64::NAN), true);
        assert_eq!(IsFloat32Representable(f64::INFINITY), true);
    }

    #[test]
    fn test_ffi_special_values() {
        // Zeroes
        assert_eq!(IsFloat32Representable(0.0), true);
        assert_eq!(IsFloat32Representable(-0.0), true);

        // Infinities
        assert_eq!(IsFloat32Representable(f64::INFINITY), true);
        assert_eq!(IsFloat32Representable(f64::NEG_INFINITY), true);

        // NaN
        assert_eq!(IsFloat32Representable(f64::NAN), true);
    }

    #[test]
    fn test_ffi_overflow() {
        let max_as_f64 = f32::MAX as f64;
        assert_eq!(IsFloat32Representable(max_as_f64 * 2.0), false);
        assert_eq!(IsFloat32Representable(-max_as_f64 * 2.0), false);
    }

    #[test]
    fn test_ffi_precision() {
        // Exact values
        assert_eq!(IsFloat32Representable(1.0), true);
        assert_eq!(IsFloat32Representable(2.5), true);

        // Precision loss
        assert_eq!(IsFloat32Representable(2147483647.0), false);
        assert_eq!(IsFloat32Representable(16777217.0), false);
    }

    #[test]
    fn test_ffi_powers_of_two() {
        // Representable powers of 2
        assert_eq!(IsFloat32Representable(2.0_f64.powi(0)), true);
        assert_eq!(IsFloat32Representable(2.0_f64.powi(10)), true);
        assert_eq!(IsFloat32Representable(2.0_f64.powi(127)), true);

        // Non-representable powers of 2
        assert_eq!(IsFloat32Representable(2.0_f64.powi(128)), false);
        assert_eq!(IsFloat32Representable(2.0_f64.powi(-150)), false);
    }
}

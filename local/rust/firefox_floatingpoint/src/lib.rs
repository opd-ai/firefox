//! Firefox IsFloat32Representable - Rust Port
//!
//! This module implements the IsFloat32Representable function from
//! mfbt/FloatingPoint.cpp, which determines whether a double-precision
//! floating point value can be losslessly represented as a single-precision
//! (float32) value.
//!
//! # Algorithm
//!
//! The function implements IEEE-754 representability checking:
//! 1. NaN and ±∞ values are representable (same in float32 and float64)
//! 2. Values exceeding FLT_MAX are not representable
//! 3. For finite values, a round-trip conversion test detects precision loss:
//!    - If value is exactly representable, round-trip preserves it
//!    - If value is between two adjacent float32 values, round-trip changes it
//!
//! # Examples
//!
//! ```
//! use firefox_floatingpoint::is_float32_representable;
//!
//! // Exact values are representable
//! assert!(is_float32_representable(1.0));
//! assert!(is_float32_representable(2.5));
//!
//! // Special values are representable
//! assert!(is_float32_representable(f64::NAN));
//! assert!(is_float32_representable(f64::INFINITY));
//! assert!(is_float32_representable(f64::NEG_INFINITY));
//!
//! // Values exceeding float32 range are not representable
//! assert!(!is_float32_representable(f64::from(f32::MAX) * 2.0));
//!
//! // INT32_MAX is not exactly representable as float32
//! assert!(!is_float32_representable(2147483647.0));
//! ```

// FFI layer for C++ interoperability
pub mod ffi;

/// Determines whether a double-precision value can be losslessly represented as float32.
///
/// # Arguments
///
/// * `value` - A double-precision floating point value to check
///
/// # Returns
///
/// * `true` if the value can be represented as float32 without precision loss
/// * `false` if the value would lose precision when converted to float32
///
/// # IEEE-754 Behavior
///
/// - NaN values (any NaN) → `true` (NaN representation is compatible)
/// - Positive infinity → `true` (±∞ same in both formats)
/// - Negative infinity → `true`
/// - Zero (±0) → `true` (both signed zeros exist in float32)
/// - Values with |value| > f32::MAX → `false` (overflow)
/// - Values requiring more than 23 significand bits → `false` (precision loss)
/// - Exact float32 values → `true` (e.g., 1.0, 2.5, 128.0)
///
/// # Examples
///
/// ```
/// use firefox_floatingpoint::is_float32_representable;
///
/// // Representable values
/// assert!(is_float32_representable(0.0));
/// assert!(is_float32_representable(-0.0));
/// assert!(is_float32_representable(1.0));
/// assert!(is_float32_representable(f64::NAN));
///
/// // Not representable (precision or range)
/// assert!(!is_float32_representable(2147483647.0)); // INT32_MAX
/// assert!(!is_float32_representable(f64::from(f32::MAX) * 2.0)); // overflow
/// ```
#[inline]
pub fn is_float32_representable(value: f64) -> bool {
    // Step 1: NaN and infinities are representable in both formats
    // IEEE-754 guarantees compatible NaN and ±∞ representations
    if !value.is_finite() {
        return true;
    }

    // Step 2: Check if the value exceeds the finite float32 range
    // f32::MAX is approximately 3.402823e+38
    // Any value with absolute value greater than this cannot fit in float32
    if value.abs() > f32::MAX as f64 {
        return false;
    }

    // Step 3: Round-trip conversion test for precision loss
    // If the value is exactly representable, converting to float32 and back
    // to float64 will preserve the value. If precision is lost, the round-trip
    // will produce a different value.
    //
    // This works because:
    // - Exact float32 values (e.g., 1.0, 2.5) have exact representations
    // - Values between adjacent float32 values (e.g., 2147483647.0) will
    //   round to the nearest float32, changing the value
    let as_float32 = value as f32;
    let round_trip = as_float32 as f64;

    // Value is representable if and only if the round-trip preserves it
    round_trip == value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero() {
        // Positive and negative zero
        assert!(is_float32_representable(0.0));
        assert!(is_float32_representable(-0.0));
    }

    #[test]
    fn test_special_values() {
        // NaN (any NaN is representable)
        assert!(is_float32_representable(f64::NAN));
        
        // Infinities
        assert!(is_float32_representable(f64::INFINITY));
        assert!(is_float32_representable(f64::NEG_INFINITY));
    }

    #[test]
    fn test_exact_representable_values() {
        // Simple integers that fit exactly in float32
        assert!(is_float32_representable(1.0));
        assert!(is_float32_representable(2.0));
        assert!(is_float32_representable(100.0));
        assert!(is_float32_representable(-42.0));

        // Simple fractions representable in binary
        assert!(is_float32_representable(0.5));
        assert!(is_float32_representable(0.25));
        assert!(is_float32_representable(2.5));
        assert!(is_float32_representable(3.125));
    }

    #[test]
    fn test_powers_of_two() {
        // Powers of 2 from 2^-149 to 2^127 are exactly representable
        // (This covers the normal and denormal range of float32)
        
        // Test some representative powers of two
        assert!(is_float32_representable(2.0_f64.powi(-149))); // Smallest denormal
        assert!(is_float32_representable(2.0_f64.powi(-10)));
        assert!(is_float32_representable(2.0_f64.powi(0)));
        assert!(is_float32_representable(2.0_f64.powi(10)));
        assert!(is_float32_representable(2.0_f64.powi(64)));
        assert!(is_float32_representable(2.0_f64.powi(127))); // Largest normal power of 2
    }

    #[test]
    fn test_overflow_values() {
        // Values exceeding f32::MAX are not representable
        let max_f32_as_f64 = f32::MAX as f64;
        
        assert!(!is_float32_representable(max_f32_as_f64 * 1.1));
        assert!(!is_float32_representable(max_f32_as_f64 * 2.0));
        assert!(!is_float32_representable(max_f32_as_f64 * 10.0));
        
        // Negative overflow
        assert!(!is_float32_representable(-max_f32_as_f64 * 1.1));
        assert!(!is_float32_representable(-max_f32_as_f64 * 2.0));
    }

    #[test]
    fn test_underflow_denormals() {
        // Powers of 2 smaller than 2^-149 are not representable
        // (Below the denormal range of float32)
        assert!(!is_float32_representable(2.0_f64.powi(-150)));
        assert!(!is_float32_representable(2.0_f64.powi(-160)));
        assert!(!is_float32_representable(2.0_f64.powi(-200)));
        assert!(!is_float32_representable(2.0_f64.powi(-1000)));
    }

    #[test]
    fn test_too_large_powers_of_two() {
        // Powers of 2 from 2^128 onwards are not representable
        // (Exceed the exponent range of float32)
        assert!(!is_float32_representable(2.0_f64.powi(128)));
        assert!(!is_float32_representable(2.0_f64.powi(200)));
        assert!(!is_float32_representable(2.0_f64.powi(500)));
    }

    #[test]
    fn test_precision_loss() {
        // INT32_MAX (2147483647) is too large to fit exactly in float32
        // float32 has only 23+1 significand bits, needs 31 bits for INT32_MAX
        assert!(!is_float32_representable(2147483647.0));
        assert!(!is_float32_representable(-2147483647.0));
        
        // Large odd numbers lose precision
        assert!(!is_float32_representable(16777217.0)); // 2^24 + 1, exceeds precision
    }

    #[test]
    fn test_max_float32() {
        // f32::MAX itself is exactly representable
        assert!(is_float32_representable(f32::MAX as f64));
        assert!(is_float32_representable(f32::MIN as f64)); // MIN is -MAX
        
        // Just beyond f32::MAX is not representable
        let next_after_max = f32::MAX as f64 + 1e30; // Much larger than MAX
        assert!(!is_float32_representable(next_after_max));
    }

    #[test]
    fn test_min_positive_float32() {
        // Smallest positive normal float32
        let min_positive = f32::MIN_POSITIVE as f64;
        assert!(is_float32_representable(min_positive));
        
        // Half of that (denormal) is still representable
        assert!(is_float32_representable(min_positive / 2.0));
    }

    #[test]
    fn test_denormal_boundary() {
        // Smallest positive denormal float32 is 2^-149
        // This is approximately 1.4e-45
        let smallest_denormal = 2.0_f64.powi(-149);
        assert!(is_float32_representable(smallest_denormal));
        
        // Double it (still denormal)
        assert!(is_float32_representable(smallest_denormal * 2.0));
        
        // Half of it (underflow to zero is still representable as zero)
        // Note: This will round to zero, which IS representable
        let half_smallest = smallest_denormal / 2.0;
        assert!(!is_float32_representable(half_smallest) || half_smallest == 0.0);
    }

    #[test]
    fn test_random_representable() {
        // Values that are known to be exactly representable
        let representable = vec![
            0.0,
            1.0,
            -1.0,
            2.0,
            0.5,
            0.25,
            0.125,
            10.0,
            100.0,
            1000.0,
            -42.5,
            3.14159_f32 as f64, // A float32 value cast to f64 is always representable
        ];

        for &val in &representable {
            assert!(
                is_float32_representable(val),
                "Expected {} to be representable as float32",
                val
            );
        }
    }

    #[test]
    fn test_random_non_representable() {
        // Values that lose precision when converted to float32
        let non_representable = vec![
            f64::from(f32::MAX) * 1.1, // Overflow
            2147483647.0,              // INT32_MAX
            -2147483647.0,
            16777217.0,                // 2^24 + 1
            std::f64::consts::PI,      // Pi needs more precision than float32
            2.0_f64.powi(-150),        // Underflow
        ];

        for &val in &non_representable {
            assert!(
                !is_float32_representable(val),
                "Expected {} to NOT be representable as float32",
                val
            );
        }
    }

    #[test]
    fn test_edge_cases() {
        // Additional edge cases
        
        // Subnormal boundaries
        assert!(is_float32_representable(f32::MIN_POSITIVE as f64));
        
        // Very small numbers (denormals)
        // 2^-140 is a denormal float32 (between MIN_POSITIVE and smallest denormal)
        assert!(is_float32_representable(2.0_f64.powi(-140)));
        
        // Very large numbers
        assert!(!is_float32_representable(1e50)); // Too large
    }
}

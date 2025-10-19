// -*- Mode: rust; tab-width: 4; indent-tabs-mode: nil; c-basic-offset: 2 -*-
// vim: set ts=4 sts=2 et sw=2 tw=80:
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Rust implementation of XorShift128+ PRNG
//!
//! This is a port of mfbt/XorShift128PlusRNG.h from C++ to Rust.
//!
//! XorShift128+ is a fast, non-cryptographic pseudo-random number generator
//! based on the xorshift128+ algorithm described in:
//!
//! Vigna, Sebastiano (2014). "Further scramblings of Marsaglia's xorshift
//! generators". arXiv:1404.0390 (http://arxiv.org/abs/1404.0390)
//!
//! ## Properties
//!
//! - **Period**: 2^128 - 1 calls before repetition
//! - **Performance**: ~1-2 CPU cycles per call
//! - **Quality**: Passes BigCrush statistical tests
//! - **Thread safety**: NOT thread-safe (no internal synchronization)
//! - **Cryptographic security**: NOT suitable for cryptographic use
//!
//! ## Memory Layout
//!
//! The struct uses `#[repr(C)]` to guarantee C-compatible memory layout.
//! Size is exactly 16 bytes (2 × u64). This is critical for JIT code that
//! directly accesses the state fields via computed offsets.

use std::mem::size_of;

// Export FFI module
pub mod ffi;

/// XorShift128+ pseudo-random number generator
///
/// A stream of pseudo-random numbers generated using the xorshift+ technique.
/// The stream repeats every 2^128 - 1 calls. Zero appears 2^64 - 1 times;
/// all other numbers appear 2^64 times.
///
/// # Thread Safety
///
/// This generator is NOT thread-safe. Methods mutate internal state without
/// synchronization. Use one RNG per thread or provide external synchronization.
///
/// # Example
///
/// ```
/// use firefox_xorshift128plus::XorShift128PlusRNG;
///
/// let mut rng = XorShift128PlusRNG::new(1, 4);
/// let random_u64 = rng.next();
/// let random_f64 = rng.next_double(); // in range [0, 1)
/// ```
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct XorShift128PlusRNG {
    /// Internal state: two 64-bit values
    /// At least one must be non-zero for proper operation
    state: [u64; 2],
}

impl XorShift128PlusRNG {
    /// Construct a xorshift128+ PRNG with initial seed values
    ///
    /// # Arguments
    ///
    /// * `initial0` - First seed value
    /// * `initial1` - Second seed value
    ///
    /// # Panics
    ///
    /// Panics in debug mode if both initial values are zero.
    ///
    /// # Note
    ///
    /// If the initial states contain many zeros, for a few iterations you'll
    /// see many zeroes in the generated numbers. It's suggested to seed a
    /// SplitMix64 generator and use its first two outputs to seed xorshift128+.
    pub fn new(initial0: u64, initial1: u64) -> Self {
        let mut rng = Self { state: [0, 0] };
        rng.set_state(initial0, initial1);
        rng
    }

    /// Generate the next pseudo-random 64-bit number
    ///
    /// Uses wrapping arithmetic (overflow is intentional and part of the algorithm).
    ///
    /// # Returns
    ///
    /// A pseudo-random u64 value
    #[inline]
    pub fn next(&mut self) -> u64 {
        // Algorithm from Vigna 2014:
        // s1 = state[0]
        // s0 = state[1]
        // state[0] = s0
        // s1 ^= s1 << 23
        // state[1] = s1 ^ s0 ^ (s1 >> 17) ^ (s0 >> 26)
        // return state[1] + s0

        let mut s1 = self.state[0];
        let s0 = self.state[1];
        
        self.state[0] = s0;
        s1 ^= s1 << 23;
        self.state[1] = s1 ^ s0 ^ (s1 >> 17) ^ (s0 >> 26);
        
        // Wrapping addition (intentional overflow)
        self.state[1].wrapping_add(s0)
    }

    /// Generate a pseudo-random floating-point value in the range [0, 1)
    ///
    /// More precisely, choose an integer in the range [0, 2^53) and divide it
    /// by 2^53. This uses the full 53-bit mantissa precision of IEEE 754 double.
    ///
    /// # Returns
    ///
    /// A pseudo-random f64 value in [0.0, 1.0)
    #[inline]
    pub fn next_double(&mut self) -> f64 {
        // Because IEEE 64-bit floating point format stores the leading '1' bit
        // of the mantissa implicitly, it effectively represents a mantissa in
        // the range [0, 2^53) in only 52 bits. We need 53 bits for the mantissa.
        const MANTISSA_BITS: u32 = 53; // f64::MANTISSA_DIGITS is 52, but we need 53
        
        // Get next random value and extract 53-bit mantissa
        let mantissa = self.next() & ((1u64 << MANTISSA_BITS) - 1);
        
        // Divide by 2^53 to get [0, 1) range
        // This is exact because all integers in [0, 2^53) are exactly representable
        (mantissa as f64) / ((1u64 << MANTISSA_BITS) as f64)
    }

    /// Set the RNG state to specific values
    ///
    /// # Arguments
    ///
    /// * `state0` - New value for state[0]
    /// * `state1` - New value for state[1]
    ///
    /// # Panics
    ///
    /// Panics in debug mode if both state values are zero.
    ///
    /// # Use Cases
    ///
    /// - Reproducible sequences for testing
    /// - Serialization/deserialization
    /// - Forking RNG state
    pub fn set_state(&mut self, state0: u64, state1: u64) {
        debug_assert!(
            state0 != 0 || state1 != 0,
            "XorShift128PlusRNG: At least one state value must be non-zero"
        );
        self.state[0] = state0;
        self.state[1] = state1;
    }

    /// Get the byte offset of state[0] within the struct
    ///
    /// This is used by JIT code for direct memory access.
    ///
    /// # Returns
    ///
    /// The offset in bytes of state[0] from the start of the struct
    #[inline]
    pub const fn offset_of_state0() -> usize {
        // state[0] is at offset 0
        0
    }

    /// Get the byte offset of state[1] within the struct
    ///
    /// This is used by JIT code for direct memory access.
    ///
    /// # Returns
    ///
    /// The offset in bytes of state[1] from the start of the struct
    #[inline]
    pub const fn offset_of_state1() -> usize {
        // state[1] is at offset 8 (after first u64)
        size_of::<u64>()
    }
}

// Compile-time assertions to ensure struct layout matches C++
const _: () = {
    // Struct must be exactly 16 bytes (2 × u64)
    assert!(size_of::<XorShift128PlusRNG>() == 2 * size_of::<u64>());
    
    // Verify offsets match expected C++ layout
    assert!(XorShift128PlusRNG::offset_of_state0() == 0);
    assert!(XorShift128PlusRNG::offset_of_state1() == 8);
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_struct_size() {
        // Verify struct is exactly 16 bytes
        assert_eq!(size_of::<XorShift128PlusRNG>(), 16);
        assert_eq!(size_of::<XorShift128PlusRNG>(), 2 * size_of::<u64>());
    }

    #[test]
    fn test_struct_offsets() {
        // Verify state[0] is at offset 0
        assert_eq!(XorShift128PlusRNG::offset_of_state0(), 0);
        // Verify state[1] is at offset 8 (after first u64)
        assert_eq!(XorShift128PlusRNG::offset_of_state1(), 8);
    }

    #[test]
    fn test_dumb_sequence() {
        // Test from TestXorShift128PlusRNG.cpp::TestDumbSequence()
        // Verifies bit-exact algorithm implementation
        let mut rng = XorShift128PlusRNG::new(1, 4);

        // Expected values calculated by hand following the algorithm
        assert_eq!(rng.next(), 0x800049);
        assert_eq!(rng.next(), 0x3000186);
        assert_eq!(rng.next(), 0x400003001145);
    }

    #[test]
    fn test_set_state() {
        // Test from TestXorShift128PlusRNG.cpp::TestSetState()
        const SEED: [u64; 2] = [1795644156779822404, 14162896116325912595];
        let mut rng = XorShift128PlusRNG::new(SEED[0], SEED[1]);

        // Generate sequence
        let log: Vec<u64> = (0..10).map(|_| rng.next()).collect();

        // Reset to same state
        rng.set_state(SEED[0], SEED[1]);

        // Verify same sequence
        for expected in log {
            assert_eq!(rng.next(), expected);
        }
    }

    #[test]
    fn test_next_double_range() {
        // Verify nextDouble() returns values in [0, 1)
        let mut rng = XorShift128PlusRNG::new(
            0xa207aaede6859736,
            0xaca6ca5060804791,
        );

        for _ in 0..1000 {
            let d = rng.next_double();
            assert!(d >= 0.0, "nextDouble() returned {}, expected >= 0.0", d);
            assert!(d < 1.0, "nextDouble() returned {}, expected < 1.0", d);
        }
    }

    #[test]
    #[should_panic(expected = "At least one state value must be non-zero")]
    fn test_zero_state_panics() {
        // Both seeds zero should panic in debug mode
        XorShift128PlusRNG::new(0, 0);
    }

    #[test]
    fn test_population() {
        // Test from TestXorShift128PlusRNG.cpp::TestPopulation()
        let mut rng = XorShift128PlusRNG::new(
            698079309544035222,
            6012389156611637584,
        );

        // Warm up
        for _ in 0..40 {
            rng.next();
        }

        // Check bit population (should be roughly even)
        for _ in 0..40 {
            let val = rng.next();
            let pop = val.count_ones();
            assert!(
                pop >= 24 && pop <= 40,
                "Bit population {} out of expected range [24, 40]",
                pop
            );
        }
    }
}

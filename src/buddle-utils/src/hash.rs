//! Implementation of dictionary hash functions commonly
//! used throughout the game.

use crate::bitmask;

/// Produces a String ID of `data`.
///
/// This hash function is a custom design by KingsIsle.
///
/// Similar enough strings frequently produce collisions
/// and it is not optimized for performance due to its
/// use of shifts by non-constant values.
///
/// It is officially deprecated in favor of [`djb2`] in
/// Pirate101.
#[inline(always)]
pub const fn string_id(data: &str) -> u32 {
    StringIdBuilder::new().feed(data).finish()
}

/// A builder for String IDs which repeatedly accepts
/// data and produces the final hash.
pub struct StringIdBuilder {
    state: i32,
    byte_index: u32,
}

impl StringIdBuilder {
    /// Produces a new builder with default configuration.
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            state: 0,
            byte_index: 0,
        }
    }

    /// Consumes the previous builder object and returns a
    /// new one, with `data` hashed into the state.
    ///
    /// This may be called repeatedly to add more substrings
    /// to the final hash.
    #[inline(never)]
    // LLVM overeagerly tries to vectorize this loop in
    // optimized builds which results in large codegen.
    //
    // But due to our inputs being rather small in most
    // cases, this is not only a slowdown but also a
    // revolting source of binary overhead for no gain.
    #[optimize(size)]
    pub const fn feed(mut self, data: &str) -> Self {
        let bytes = data.as_bytes();

        // Iterate over all the bytes in the string.
        let mut i = 0;
        while i < bytes.len() {
            // Compute the current value to process and the
            // shift to use based on previous feed() calls.
            let c = bytes[i] as i32 - 32;
            let shift = (self.byte_index + i as u32) * 5 % 32;

            // Perform the hashing operation.
            self.state ^= c.wrapping_shl(shift);
            if shift > 24 {
                self.state ^= c.wrapping_shr(32 - shift);
            }

            // Advance to the next byte.
            i += 1;
        }

        // Advance the byte index for the next feed() call.
        self.byte_index += i as u32;

        self
    }

    /// Consumes the builder and returns the final hash.
    #[inline(always)]
    pub const fn finish(self) -> u32 {
        self.state.unsigned_abs()
    }
}

/// Implementation of the [DJB2] hash function.
///
/// [DJB2]: https://theartincode.stanis.me/008-djb2/
#[inline(always)]
pub const fn djb2(input: &str) -> u32 {
    let bytes = input.as_bytes();
    let mut state: u32 = 5381;

    let mut i = 0;
    while i < bytes.len() {
        // state * 33 + bytes[i]
        state = (state << 5)
            .wrapping_add(state)
            .wrapping_add(bytes[i] as u32);

        i += 1;
    }

    // XXX: KingsIsle's implementation strips the MSB.
    state & bitmask!(u32::BITS - 1)
}

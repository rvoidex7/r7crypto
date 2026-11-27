//! Pluggable `EntropySource` API; see
//! [Architecture](https://rvoidex7.github.io/r7notes/Projects/r7crypto/Architecture#the-one-abstraction).

mod biased;
mod chacha;
mod counter;
mod kind;
mod lcg;
mod lfsr;
mod mock;
mod system;

pub use biased::BiasedSource;
pub use chacha::ChaCha20Source;
pub use counter::CounterSource;
pub use kind::{SourceKind, ARENA_SEED};
pub use lcg::{LcgPreset, LcgSource};
pub use lfsr::Lfsr16Source;
pub use mock::MockEntropy;
pub use system::SystemEntropy;

/// Core trait for entropy sources
///
/// One trait for quality, NIST and benchmark paths; `Use` takes explicit keys.
/// See [The RNG Arena](https://rvoidex7.github.io/r7notes/Projects/r7crypto/Architecture#the-rng-arena).
///
/// # Examples
///
/// ```
/// use r7crypto::entropy::EntropySource;
///
/// struct MyRNG {
///     state: u64,
/// }
///
/// impl EntropySource for MyRNG {
///     fn fill_bytes(&mut self, dest: &mut [u8]) {
///         for byte in dest.iter_mut() {
///             // Simple LCG for example (DO NOT use in production!)
///             self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
///             *byte = (self.state >> 24) as u8;
///         }
///     }
///     
///     fn name(&self) -> &str {
///         "My Custom RNG"
///     }
/// }
/// ```
///
/// Full example: [Using the library in code](https://rvoidex7.github.io/r7notes/Projects/r7crypto/Architecture#using-the-library-in-code).
pub trait EntropySource {
    /// Fill `dest` completely with fresh bytes; overwrite, never XOR.
    ///
    /// `next_byte`, `next_u32` and `next_u64` delegate here.
    fn fill_bytes(&mut self, dest: &mut [u8]);

    /// One byte via [`Self::fill_bytes`].
    fn next_byte(&mut self) -> u8 {
        let mut buf = [0u8; 1];
        self.fill_bytes(&mut buf);
        buf[0]
    }

    /// Little-endian u32 via [`Self::fill_bytes`].
    fn next_u32(&mut self) -> u32 {
        let mut buf = [0u8; 4];
        self.fill_bytes(&mut buf);
        u32::from_le_bytes(buf)
    }

    /// Little-endian u64 via [`Self::fill_bytes`].
    fn next_u64(&mut self) -> u64 {
        let mut buf = [0u8; 8];
        self.fill_bytes(&mut buf);
        u64::from_le_bytes(buf)
    }

    /// Display label for the source, shown in UI labels and examples.
    fn name(&self) -> &str {
        "Unknown Source"
    }

    /// Rewind to the initial state; deterministic sources must reproduce their first bytes.
    fn reset(&mut self) {}
}

impl EntropySource for Box<dyn EntropySource> {
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        (**self).fill_bytes(dest)
    }

    fn name(&self) -> &str {
        (**self).name()
    }

    fn reset(&mut self) {
        (**self).reset()
    }
}

impl<T: ?Sized + EntropySource> EntropySource for &mut T {
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        (**self).fill_bytes(dest)
    }

    fn name(&self) -> &str {
        (**self).name()
    }

    fn reset(&mut self) {
        (**self).reset()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_entropy() {
        let mut entropy = SystemEntropy::new();
        let mut buf = [0u8; 32];
        entropy.fill_bytes(&mut buf);

        // Should not be all zeros (probability is negligible)
        assert!(buf.iter().any(|&b| b != 0));
    }

    #[test]
    fn test_next_methods() {
        let mut entropy = SystemEntropy::new();

        let byte = entropy.next_byte();
        let u32_val = entropy.next_u32();
        let u64_val = entropy.next_u64();

        let _ = (byte, u32_val, u64_val);
    }

    #[test]
    fn boxed_source_forwards_reset() {
        // Regression: the blanket impl used to inherit the no-op default, so
        // resetting a `Box<dyn EntropySource>` silently did nothing.
        let mut boxed: Box<dyn EntropySource> = Box::new(MockEntropy::new(42));

        let mut first = [0u8; 16];
        let mut second = [0u8; 16];
        boxed.fill_bytes(&mut first);
        boxed.reset();
        boxed.fill_bytes(&mut second);

        assert_eq!(first, second);
    }
}

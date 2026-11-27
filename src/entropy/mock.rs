//! Deterministic test helper; see
//! [Architecture](https://rvoidex7.github.io/r7notes/Projects/r7crypto/Architecture#the-one-abstraction).

use super::EntropySource;

/// Deterministic test source. DO NOT use in production!
///
/// # Examples
///
/// ```
/// use r7crypto::entropy::{EntropySource, MockEntropy};
///
/// let mut entropy = MockEntropy::new(42);
/// let mut buffer = [0u8; 4];
/// entropy.fill_bytes(&mut buffer);
/// assert_eq!(buffer, [0xCAu8, 0xC2, 0xBB, 0xE3]);
/// ```
#[derive(Debug, Clone)]
pub struct MockEntropy {
    state: u64,
    initial_state: u64,
}

impl MockEntropy {
    /// Create a new mock entropy source with given seed
    pub fn new(seed: u64) -> Self {
        Self {
            state: seed,
            initial_state: seed,
        }
    }

    // Cheap deterministic test bytes, not crypto: classic ANSI C LCG constants.
    fn next(&mut self) -> u8 {
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        (self.state >> 24) as u8
    }
}

impl Default for MockEntropy {
    /// Create with the default seed (42).
    fn default() -> Self {
        Self::new(42)
    }
}

impl EntropySource for MockEntropy {
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for byte in dest.iter_mut() {
            *byte = self.next();
        }
    }

    fn name(&self) -> &str {
        "Mock RNG (for testing only)"
    }

    fn reset(&mut self) {
        self.state = self.initial_state;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_deterministic() {
        let mut entropy1 = MockEntropy::new(42);
        let mut entropy2 = MockEntropy::new(42);

        let mut buf1 = [0u8; 32];
        let mut buf2 = [0u8; 32];

        entropy1.fill_bytes(&mut buf1);
        entropy2.fill_bytes(&mut buf2);

        assert_eq!(buf1, buf2);
    }

    #[test]
    fn test_mock_different_seeds() {
        let mut entropy1 = MockEntropy::new(42);
        let mut entropy2 = MockEntropy::new(43);

        let mut buf1 = [0u8; 32];
        let mut buf2 = [0u8; 32];

        entropy1.fill_bytes(&mut buf1);
        entropy2.fill_bytes(&mut buf2);

        assert_ne!(buf1, buf2);
    }

    #[test]
    fn test_mock_reset() {
        let mut entropy = MockEntropy::new(42);

        let mut buf1 = [0u8; 32];
        let mut buf2 = [0u8; 32];

        entropy.fill_bytes(&mut buf1);
        entropy.reset();
        entropy.fill_bytes(&mut buf2);

        assert_eq!(buf1, buf2);
    }
}

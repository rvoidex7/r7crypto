//! Seeded ChaCha20 entropy source.

use chacha20::cipher::{KeyIvInit, StreamCipher as CipherTrait};
use chacha20::ChaCha20;
use sha2::{Digest, Sha256};

use super::EntropySource;

/// ChaCha20 used as a deterministic, seeded CSPRNG.
///
/// The 256-bit key comes from the `u64` seed via SHA-256 and the nonce is zero.
/// Same seed, same stream — that reproducibility is the point for Test and
/// Benchmark, not secrecy. See
/// [Seeded CSPRNG](https://rvoidex7.github.io/r7notes/Cryptography/Random-Number-Generators#seeded-csprng).
///
/// # Security Note
///
/// A guessable seed makes the stream predictable. Use the OS source for real keys.
///
/// # Examples
///
/// ```
/// use r7crypto::entropy::{ChaCha20Source, EntropySource};
///
/// let mut source = ChaCha20Source::new(42);
/// let mut buffer = [0u8; 32];
/// source.fill_bytes(&mut buffer);
///
/// // The same seed reproduces the same stream.
/// let mut again = ChaCha20Source::new(42);
/// let mut second = [0u8; 32];
/// again.fill_bytes(&mut second);
/// assert_eq!(buffer, second);
/// ```
pub struct ChaCha20Source {
    cipher: ChaCha20,
    key: [u8; 32],
    nonce: [u8; 12],
}

impl ChaCha20Source {
    /// Create a source whose key is derived from `seed`.
    pub fn new(seed: u64) -> Self {
        let digest = Sha256::digest(seed.to_le_bytes());
        let mut key = [0u8; 32];
        key.copy_from_slice(&digest);
        let nonce = [0u8; 12];

        Self {
            cipher: ChaCha20::new((&key).into(), (&nonce).into()),
            key,
            nonce,
        }
    }
}

impl EntropySource for ChaCha20Source {
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        // `apply_keystream` XORs, so start from a clean buffer to make this an
        // overwrite like every other source.
        dest.fill(0);
        self.cipher.apply_keystream(dest);
    }

    fn name(&self) -> &str {
        "ChaCha20 CSPRNG (seeded)"
    }

    fn reset(&mut self) {
        self.cipher = ChaCha20::new((&self.key).into(), (&self.nonce).into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_produces_the_same_stream() {
        let mut first = ChaCha20Source::new(42);
        let mut second = ChaCha20Source::new(42);

        let mut a = [0u8; 64];
        let mut b = [0u8; 64];
        first.fill_bytes(&mut a);
        second.fill_bytes(&mut b);

        assert_eq!(a, b);
    }

    #[test]
    fn different_seeds_produce_different_streams() {
        let mut first = ChaCha20Source::new(42);
        let mut second = ChaCha20Source::new(43);

        let mut a = [0u8; 64];
        let mut b = [0u8; 64];
        first.fill_bytes(&mut a);
        second.fill_bytes(&mut b);

        assert_ne!(a, b);
    }

    #[test]
    fn reset_rewinds_the_keystream() {
        let mut source = ChaCha20Source::new(7);

        let mut first = [0u8; 32];
        let mut second = [0u8; 32];
        source.fill_bytes(&mut first);
        source.reset();
        source.fill_bytes(&mut second);

        assert_eq!(first, second);
    }

    #[test]
    fn overwrites_existing_buffer_content() {
        let mut source = ChaCha20Source::new(1);
        let mut buffer = [0xFFu8; 16];
        source.fill_bytes(&mut buffer);

        assert_ne!(buffer, [0xFFu8; 16], "keystream must overwrite, not XOR");
    }

    #[test]
    fn consecutive_calls_continue_the_stream() {
        let mut split = ChaCha20Source::new(99);
        let mut whole = ChaCha20Source::new(99);

        let mut first = [0u8; 8];
        let mut second = [0u8; 8];
        split.fill_bytes(&mut first);
        split.fill_bytes(&mut second);

        let mut joined = [0u8; 16];
        whole.fill_bytes(&mut joined);

        assert_eq!(&joined[..8], &first);
        assert_eq!(&joined[8..], &second);
    }

    #[test]
    fn name_identifies_the_source() {
        assert!(ChaCha20Source::new(0).name().contains("ChaCha20"));
    }
}

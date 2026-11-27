//! Stream ciphers.
//!
//! [`StreamCipher`] is an educational one-time pad. [`KeyedStreamCipher`] uses
//! ChaCha20 and supports real, reversible encryption. Open [Use](https://rvoidex7.github.io/r7crypto/#/use).

use crate::entropy::EntropySource;
use chacha20::cipher::{KeyIvInit, StreamCipher as CipherTrait};
use chacha20::ChaCha20;
use sha2::{Digest, Sha256};

/// Simple XOR stream cipher (one-time pad style).
///
/// Each call to [`StreamCipher::process`] pulls a fresh keystream from the
/// entropy source and XORs it with the input. Because XOR is symmetric the
/// same function encrypts and decrypts, but only if the **same keystream** is
/// reused. This type does not store a reusable keystream.
///
/// # Security Note
///
/// Educational only; malleable, no plaintext avalanche effect. Do not use
/// for real data. See [The misconception](https://rvoidex7.github.io/r7notes/Cryptography/Avalanche-Effect#the-misconception).
///
/// # Examples
///
/// ```
/// use r7crypto::entropy::MockEntropy;
/// use r7crypto::crypto::StreamCipher;
///
/// let mut cipher = StreamCipher::new(MockEntropy::new(42));
/// let ciphertext = cipher.process(b"Hello, World!");
/// assert_eq!(ciphertext.len(), 13);
/// ```
pub struct StreamCipher<E: EntropySource> {
    entropy: E,
    state: Vec<u8>,
    bytes_processed: usize,
}

impl<E: EntropySource> StreamCipher<E> {
    /// Create a new cipher with the given entropy source.
    pub fn new(entropy: E) -> Self {
        Self {
            entropy,
            state: Vec::new(),
            bytes_processed: 0,
        }
    }

    /// Process data (encrypt or decrypt — XOR is symmetric).
    pub fn process(&mut self, data: &[u8]) -> Vec<u8> {
        let mut keystream = vec![0u8; data.len()];
        self.entropy.fill_bytes(&mut keystream);

        self.state = keystream.iter().take(64).copied().collect();

        let output = data
            .iter()
            .zip(keystream.iter())
            .map(|(data_byte, key_byte)| data_byte ^ key_byte)
            .collect();

        self.bytes_processed += data.len();
        output
    }

    /// Last (up to) 64 keystream bytes, for visualization.
    pub fn state(&self) -> &[u8] {
        &self.state
    }

    /// Total bytes processed.
    pub fn bytes_processed(&self) -> usize {
        self.bytes_processed
    }

    /// Reset the cipher state and the underlying entropy source.
    pub fn reset(&mut self) {
        self.state.clear();
        self.bytes_processed = 0;
        self.entropy.reset();
    }
}

/// Real ChaCha20 stream cipher with a 256-bit key and 96-bit nonce.
///
/// Encryption and decryption use the exact same operation and are inverses
/// for the same key and nonce.
///
/// # Security
///
/// Never reuse a nonce under one key: reuse leaks `P1 ^ P2`. See
/// [Why nonce reuse is fatal](https://rvoidex7.github.io/r7notes/Cryptography/ChaCha20#why-nonce-reuse-is-fatal).
///
/// # Examples
///
/// ```
/// use r7crypto::crypto::KeyedStreamCipher;
///
/// let key = [0x42u8; 32];
/// let nonce = [0x24u8; 12];
/// let plaintext = "Merhaba dünya!".as_bytes();
///
/// let ciphertext = KeyedStreamCipher::process_chacha20(&key, &nonce, plaintext);
/// let decrypted = KeyedStreamCipher::process_chacha20(&key, &nonce, &ciphertext);
/// assert_eq!(decrypted, plaintext);
/// ```
pub struct KeyedStreamCipher;

impl KeyedStreamCipher {
    /// Encrypt or decrypt `data` with ChaCha20 under the given key and nonce.
    pub fn process_chacha20(key: &[u8; 32], nonce: &[u8; 12], data: &[u8]) -> Vec<u8> {
        let mut output = data.to_vec();
        let mut cipher = ChaCha20::new(key.into(), nonce.into());
        cipher.apply_keystream(&mut output);
        output
    }

    /// Derive a 256-bit ChaCha20 key from a passphrase with SHA-256.
    ///
    /// Demonstration only — not a password KDF; see
    /// [Key derivation from a passphrase](https://rvoidex7.github.io/r7notes/Cryptography/ChaCha20#key-derivation-from-a-passphrase).
    pub fn derive_key(passphrase: &str) -> [u8; 32] {
        let digest = Sha256::digest(passphrase.as_bytes());
        let mut key = [0u8; 32];
        key.copy_from_slice(&digest);
        key
    }

    /// Generate a random 96-bit nonce from the platform CSPRNG
    /// (OS on native, Web Crypto on wasm).
    pub fn random_nonce() -> [u8; 12] {
        let mut nonce = [0u8; 12];
        getrandom::getrandom(&mut nonce).expect("operating system RNG failed");
        nonce
    }

    /// Convenience wrapper: derive a key from a passphrase and process data.
    pub fn process_with_passphrase(passphrase: &str, nonce: &[u8; 12], data: &[u8]) -> Vec<u8> {
        let key = Self::derive_key(passphrase);
        Self::process_chacha20(&key, nonce, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entropy::MockEntropy;

    #[test]
    fn cipher_basic() {
        let mut cipher = StreamCipher::new(MockEntropy::new(42));
        let plaintext = b"Hello, World!";
        let ciphertext = cipher.process(plaintext);

        assert_ne!(plaintext.as_slice(), ciphertext.as_slice());
        assert_eq!(plaintext.len(), ciphertext.len());
    }

    #[test]
    fn cipher_is_deterministic_for_the_same_seed() {
        let mut cipher1 = StreamCipher::new(MockEntropy::new(42));
        let mut cipher2 = StreamCipher::new(MockEntropy::new(42));

        assert_eq!(
            cipher1.process(b"Test message"),
            cipher2.process(b"Test message")
        );
    }

    #[test]
    fn cipher_state_is_limited_to_64_bytes() {
        let mut cipher = StreamCipher::new(MockEntropy::new(42));
        let ciphertext = cipher.process(b"Some data");

        let mut entropy = MockEntropy::new(42);
        let mut keystream = vec![0u8; 9];
        entropy.fill_bytes(&mut keystream);
        assert_eq!(cipher.state(), &keystream);
        assert_eq!(
            ciphertext,
            b"Some data"
                .iter()
                .zip(&keystream)
                .map(|(byte, key)| byte ^ key)
                .collect::<Vec<_>>()
        );

        let mut long_cipher = StreamCipher::new(MockEntropy::new(42));
        long_cipher.process(&[0u8; 100]);
        assert_eq!(long_cipher.state().len(), 64);
    }

    #[test]
    fn bytes_processed_is_tracked() {
        let mut cipher = StreamCipher::new(MockEntropy::new(42));
        cipher.process(b"12345");
        assert_eq!(cipher.bytes_processed(), 5);
        cipher.process(b"67890");
        assert_eq!(cipher.bytes_processed(), 10);
    }

    #[test]
    fn xor_cipher_is_malleable_not_avalanche() {
        // With a fixed keystream, one plaintext bit flip changes exactly one
        // ciphertext bit — malleability, not diffusion.
        let mut entropy = MockEntropy::new(7);
        let mut keystream = vec![0u8; 32];
        entropy.fill_bytes(&mut keystream);

        let plaintext = [0u8; 32];
        let mut modified = plaintext;
        modified[0] ^= 0b1000_0000;

        let ciphertext: Vec<u8> = plaintext
            .iter()
            .zip(&keystream)
            .map(|(p, k)| p ^ k)
            .collect();
        let modified_ciphertext: Vec<u8> = modified
            .iter()
            .zip(&keystream)
            .map(|(p, k)| p ^ k)
            .collect();

        let changed: u32 = ciphertext
            .iter()
            .zip(&modified_ciphertext)
            .map(|(a, b)| (a ^ b).count_ones())
            .sum();
        assert_eq!(changed, 1);
    }

    #[test]
    fn chacha20_roundtrip_preserves_turkish_utf8() {
        let plaintext = "Şifreleme testi: ğüşıöç ĞÜŞİÖÇ 🔐 12345!";
        let key = [0x42u8; 32];
        let nonce = [0x24u8; 12];

        let ciphertext = KeyedStreamCipher::process_chacha20(&key, &nonce, plaintext.as_bytes());
        assert_ne!(ciphertext, plaintext.as_bytes());

        let decrypted = KeyedStreamCipher::process_chacha20(&key, &nonce, &ciphertext);
        assert_eq!(String::from_utf8(decrypted).unwrap(), plaintext);
    }

    #[test]
    fn chacha20_keystream_reuse_leaks_plaintext_xor() {
        // Two-time pad: C1 xor C2 == P1 xor P2.
        let key = [0x11u8; 32];
        let nonce = [0x22u8; 12];
        let p1 = b"attack at dawn";
        let p2 = b"defend at dusk";

        let c1 = KeyedStreamCipher::process_chacha20(&key, &nonce, p1);
        let c2 = KeyedStreamCipher::process_chacha20(&key, &nonce, p2);

        let c_xor: Vec<u8> = c1.iter().zip(&c2).map(|(a, b)| a ^ b).collect();
        let p_xor: Vec<u8> = p1.iter().zip(p2).map(|(a, b)| a ^ b).collect();
        assert_eq!(c_xor, p_xor);
    }

    #[test]
    fn distinct_nonces_do_not_leak_the_plaintext_xor() {
        // Same key, different nonce: the two-time pad identity must not hold.
        let key = [0x11u8; 32];
        let p1 = b"attack at dawn";
        let p2 = b"defend at dusk";

        let c1 = KeyedStreamCipher::process_chacha20(&key, &[0x22u8; 12], p1);
        let c2 = KeyedStreamCipher::process_chacha20(&key, &[0x33u8; 12], p2);

        let c_xor: Vec<u8> = c1.iter().zip(&c2).map(|(a, b)| a ^ b).collect();
        let p_xor: Vec<u8> = p1.iter().zip(p2).map(|(a, b)| a ^ b).collect();
        assert_ne!(c_xor, p_xor);
    }

    #[test]
    fn chacha20_is_malleable_but_not_single_bit_avalanche() {
        // Stream-cipher malleability applies to ChaCha20 too: flipping one
        // ciphertext bit flips exactly one plaintext bit.
        let key = [0x55u8; 32];
        let nonce = [0x66u8; 12];
        let plaintext = [0u8; 32];
        let ciphertext = KeyedStreamCipher::process_chacha20(&key, &nonce, &plaintext);

        let mut tampered = ciphertext.clone();
        tampered[0] ^= 0b1000_0000;
        let decrypted = KeyedStreamCipher::process_chacha20(&key, &nonce, &tampered);

        let changed: u32 = plaintext
            .iter()
            .zip(&decrypted)
            .map(|(a, b)| (a ^ b).count_ones())
            .sum();
        assert_eq!(changed, 1);
    }

    #[test]
    fn derived_key_is_deterministic() {
        assert_eq!(
            KeyedStreamCipher::derive_key("correct horse battery staple"),
            KeyedStreamCipher::derive_key("correct horse battery staple")
        );
        assert_ne!(
            KeyedStreamCipher::derive_key("key one"),
            KeyedStreamCipher::derive_key("key two")
        );
    }
}

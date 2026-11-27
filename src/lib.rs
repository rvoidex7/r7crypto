//! # r7 Crypto Lab
//!
//! Library first, optional GUI. See
//! [Architecture](https://rvoidex7.github.io/r7notes/Projects/r7crypto/Architecture).
//!
//! ## Overview
//!
//! Capabilities: pluggable sources, quality and NIST checks, source
//! benchmarks, and teaching views. See
//! [Why the layers exist](https://rvoidex7.github.io/r7notes/Projects/r7crypto/Architecture#why-the-layers-exist).
//!
//! ## Quick Start
//!
//! ```rust
//! use r7crypto::entropy::{EntropySource, SystemEntropy};
//! use r7crypto::crypto::StreamCipher;
//!
//! let entropy = SystemEntropy::new();
//! let mut cipher = StreamCipher::new(entropy);
//!
//! let plaintext = b"Hello, World!";
//! let ciphertext = cipher.process(plaintext);
//! ```
//!
//! This is the teaching one-time pad; use [`KeyedStreamCipher`] for ChaCha20.
//! See [Two ciphers, on purpose](https://rvoidex7.github.io/r7notes/Projects/r7crypto/Architecture#two-ciphers-on-purpose).
//!
//! ## Custom Entropy Sources
//!
//! See [Using the library in code](https://rvoidex7.github.io/r7notes/Projects/r7crypto/Architecture#using-the-library-in-code)
//! for a custom source example:
//!
//! ```rust
//! use r7crypto::entropy::EntropySource;
//!
//! struct MyRNG {
//!     // Your state
//! }
//!
//! impl EntropySource for MyRNG {
//!     fn fill_bytes(&mut self, dest: &mut [u8]) {
//!         // Your implementation
//!     }
//!     
//!     fn name(&self) -> &str {
//!         "My Custom RNG"
//!     }
//! }
//! ```

pub(crate) mod time;

pub mod bench;
pub mod crypto;
pub mod entropy;
pub mod learn;
pub mod quality;

#[cfg(feature = "gui")]
pub mod viz;

#[cfg(all(feature = "gui", target_arch = "wasm32"))]
mod web;

pub use crypto::{AvalancheAnalyzer, KeyedStreamCipher, StreamCipher};
pub use entropy::{EntropySource, SourceKind, SystemEntropy};
pub use quality::{NistTests, QualityMetrics, QualityReport};

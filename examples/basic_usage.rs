//! Library-only tour of the APIs the GUI calls; see
//! [Using the library in code](https://rvoidex7.github.io/r7notes/Projects/r7crypto/Architecture#using-the-library-in-code).

use r7crypto::crypto::StreamCipher;
use r7crypto::entropy::{EntropySource, SystemEntropy};
use r7crypto::quality::QualityMetrics;

fn main() {
    println!("=== r7 Crypto Lab - Basic Usage ===\n");

    let mut entropy = SystemEntropy::new();
    println!("Using entropy source: {}", entropy.name());

    let mut buffer = [0u8; 16];
    entropy.fill_bytes(&mut buffer);
    println!("\nGenerated 16 random bytes:");
    println!("{}", hex::encode(buffer));

    println!("\n--- Stream Cipher Example ---");
    let mut cipher = StreamCipher::new(SystemEntropy::new());
    let plaintext = b"Hello, r7 Crypto Lab!";
    let ciphertext = cipher.process(plaintext);
    println!("Plaintext:  {}", String::from_utf8_lossy(plaintext));
    println!("Ciphertext: {}", hex::encode(&ciphertext));

    println!("\n--- Quality Analysis ---");
    let mut entropy_for_test = SystemEntropy::new();
    let metrics = QualityMetrics::analyze(&mut entropy_for_test, 10_000);

    println!("Shannon Entropy: {:.4} bits/byte", metrics.shannon_entropy);
    println!("Min-Entropy:     {:.4} bits/byte", metrics.min_entropy);
    println!("Mean byte value: {:.2}", metrics.mean);
    println!("Overall Score:   {:.1}/100", metrics.overall_score());

    println!("\nDone!");
}

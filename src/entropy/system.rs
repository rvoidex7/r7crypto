//! OS CSPRNG via `getrandom`; on wasm this is Web Crypto.
//! See [The operating system CSPRNG](https://rvoidex7.github.io/r7notes/Cryptography/Random-Number-Generators#the-operating-system-csprng).

use super::EntropySource;
use getrandom::getrandom;

/// System entropy source via `getrandom` (Web Crypto on wasm).
///
/// This is the reference source in the RNG Arena.
///
/// # Examples
///
/// ```
/// use r7crypto::entropy::{EntropySource, SystemEntropy};
///
/// let mut entropy = SystemEntropy::new();
/// let mut buffer = [0u8; 32];
/// entropy.fill_bytes(&mut buffer);
/// ```
#[derive(Debug, Default, Clone)]
pub struct SystemEntropy;

impl SystemEntropy {
    /// Create a new system entropy source
    pub fn new() -> Self {
        Self
    }
}

impl EntropySource for SystemEntropy {
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        // Panic by design: a silent fallback would be worse.
        getrandom(dest).expect("system CSPRNG failed");
    }

    fn name(&self) -> &str {
        #[cfg(target_os = "android")]
        return "System RNG (Android getrandom / /dev/urandom)";

        #[cfg(target_os = "linux")]
        return "System RNG (Linux /dev/urandom)";

        #[cfg(target_os = "macos")]
        return "System RNG (macOS SecRandomCopyBytes)";

        #[cfg(target_os = "windows")]
        return "System RNG (Windows BCryptGenRandom)";

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        return "System RNG";
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_entropy_basic() {
        let mut entropy = SystemEntropy::new();
        let mut buf1 = [0u8; 32];
        let mut buf2 = [0u8; 32];

        entropy.fill_bytes(&mut buf1);
        entropy.fill_bytes(&mut buf2);

        assert_ne!(buf1, buf2);
    }

    #[test]
    fn test_system_entropy_large() {
        let mut entropy = SystemEntropy::new();
        let mut buf = vec![0u8; 1_000_000];

        entropy.fill_bytes(&mut buf);

        // Around 1 in 256 bytes should be zero; 2x slack for sampling.
        let zeros = buf.iter().filter(|&&b| b == 0).count();
        let expected_zeros = 1_000_000 / 256;

        assert!(zeros > expected_zeros / 2);
        assert!(zeros < expected_zeros * 2);
    }

    #[test]
    fn test_name() {
        let entropy = SystemEntropy::new();
        let name = entropy.name();
        assert!(name.contains("System RNG"));
    }
}

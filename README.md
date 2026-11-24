<p align="center">
  <img src="assets/icons/128x128.png" width="96" alt="r7 Crypto Lab icon">
</p>

<h1 align="center">r7 Crypto Lab</h1>

<p align="center"><em>Learn · Try · Verify · Make it visible</em></p>

<p align="center"><img src="https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white" alt="Rust"> <img src="https://img.shields.io/github/license/rvoidex7/r7crypto?style=for-the-badge" alt="License"> <img src="https://github.com/rvoidex7/r7crypto/actions/workflows/ci.yml/badge.svg?style=for-the-badge" alt="CI"></p>

An interactive cryptography workbench built on a pluggable entropy framework,
with quality testing, visualization and **interactive educational tools**.

**Documentation:** architecture, design decisions, dependencies, build/CI and roadmap are in [r7notes / r7crypto](https://rvoidex7.github.io/r7notes/Projects/r7crypto).

## Features

- **🔌 Trait-based Design**: Any RNG can be plugged in via the `EntropySource` trait
- **🎛️ RNG Arena**: Switch between seven built-in sources and tune bias live
- **🔬 Quality Testing**: NIST SP 800-22 tests, Shannon entropy, min-entropy calculations
- **Cryptographic Use**: Real ChaCha20 encryption plus an educational XOR one-time pad
- **📊 Real-time Visualization**: egui-based GUI for interactive exploration
- **🔀 Avalanche**: See SHA-256 single-bit diffusion in a 16×16 output-bit grid
- **⚡ Performance Benchmarks**: Measure throughput and latency of entropy sources
- **🎓 Interactive Learning**: Step-by-step visualizations of cryptographic operations
- **🌐 One codebase**: Native Linux/Windows binary and browser/WASM build from the same commit

## Quick Start

### Try it in the browser

Open <https://rvoidex7.github.io/r7crypto/> — no installation needed.

### Download a desktop binary

Every release has Linux (`.tar.gz`) and Windows (`.zip`) builds on the
[releases page](https://github.com/rvoidex7/r7crypto/releases). Unzip and run
`r7crypto` / `r7crypto.exe`; no Rust toolchain is needed. Android builds will
appear here when that platform ships.

### Build from source

```bash
cargo run --release
```

Local web builds and the full quality gate (`./scripts/check.sh`) are documented in
[Build and Release](https://rvoidex7.github.io/r7notes/Projects/r7crypto/Build-and-Release).

## GUI Tabs

The header selects the active entropy source (RNG Arena) and, when supported,
its `P(1)` bias. The Test and Benchmark tabs use that selection.

### 📝 Use Tab

Encrypt and decrypt UTF-8 text with ChaCha20 (256-bit key, 96-bit nonce). The
passphrase is hashed with SHA-256 and the nonce can be regenerated at any time.
Encryption and decryption are exact inverses for the same key and nonce.
The XOR one-time-pad view is a teaching toy that draws fresh keystream; it is
not a substitute for ChaCha20. The code and its tests are public — inspect them
and decide for yourself.

### 🔬 Test Tab

Analyze entropy quality with:
- Shannon Entropy (bits/byte)
- Min-Entropy (NIST SP 800-90B MCV, 99% upper confidence bound)
- Markov-1 conditional min-entropy and lag-1 serial correlation
- Chi-Square uniformity test (with a suspiciously-uniform warning)
- NIST SP 800-22 statistical tests (Frequency, Block Frequency, Runs,
  Longest Run, Serial, Cumulative Sums)

### ⚡ Benchmark Tab

Measure entropy source performance:
- Throughput (MB/s)
- Latency per byte (µs)

These numbers describe source generation only, not cryptographic operation
speed. Operation benchmarks are planned separately.

### 🎓 Learn Tab

Step-by-step visualizations of cryptographic operations:

- **XOR Cipher Visualization**: see each character encrypted bit by bit
  - Character → ASCII → Binary conversion
  - Bit-by-bit XOR with colour coding
  - Keystream from the selected entropy source
  - Previous / Next / Play / Pause and adjustable speed

### 🔀 Avalanche Tab

Flip one input bit of a SHA-256 message and inspect which of the 256 output bits
changed. The grid is a deterministic teaching aid; the full distribution and
weak-function comparison are still being expanded.

## Quality Metrics

The framework measures:

- **Shannon Entropy**: Average information content (max: 8.0 bits/byte)
- **Min-Entropy**: Conservative estimate based on most probable outcome
- **Chi-Square**: Tests uniformity of byte distribution
- **NIST SP 800-22 Tests**: Industry-standard randomness tests
  - Frequency Test
  - Runs Test
  - Longest Run Test
  - Serial Test
  - Chi-Square Test

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the GNU General Public License v3.0 — see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- NIST SP 800-22 for statistical test specifications
- The Rust community for excellent cryptographic libraries
- egui for the immediate-mode GUI framework

---

**Made with 🦀 Rust**

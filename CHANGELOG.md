# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Hash deep links such as `#/learn/nist-frequency`, so a note can open the exact
  screen and lesson instead of just the app. The web bootstrap reads the
  fragment; malformed fragments leave the default screen in place.
- Header links to [crypto notes](https://rvoidex7.github.io/r7notes/Cryptography),
  [project docs](https://rvoidex7.github.io/r7notes/Projects/r7crypto) and the
  [repository](https://github.com/rvoidex7/r7crypto). Every screen also links to
  its exact background note and the precise source range that implements it.
  The notes link back to the same screen and source lines, so the connection is
  specific rather than a generic entry point.

### Changed
- Comment audit across all `.rs` files: wrong or stale claims corrected,
  obvious noise removed, and long explanations replaced by links to the exact
  r7notes section. Module docs now also link back to the app screen they
  implement.

### Fixed
- `benchmark_avg(..., iterations: 0)` returned `NaN`; it now returns a zeroed
  result, with a regression test.
- `examples/quality_check.rs` printed `/5` and used a 4/5 threshold although
  `run_all_tests` returns nine P-values. The total is now derived from the
  result and the threshold is a plain majority.

### Tests
- `tests/deep_link_ui_test.rs`: fragment parsing, exact lesson selection and
  per-screen note/code links.
- `tests/mobile_ui_test.rs` now checks that all three header links are present
  and stay inside a 360 px viewport.

## [0.5.0] - 2026-11-25

### Added
- **Browser/WASM build**: `src/web.rs`, `index.html` and `Trunk.toml` provide a
  Trunk-powered web app from the same Rust code as the native GUI. The public
  URL is `/r7crypto/` for GitHub Pages.
- Separate `build-pages` and `deploy` jobs in `.github/workflows/ci.yml`.
  `build-pages` builds with `trunk build --release` and uploads `dist/`;
  `deploy` publishes it after the checks pass. Generated site files are never
  committed.
- A wasm type-check job in CI
  (`cargo check --lib --bins --target wasm32-unknown-unknown --all-features --locked`).
- `src/viz/job.rs`: shared incremental jobs for native and web. Native runs them
  on a worker thread; wasm advances them in bounded frame steps, so the UI
  never blocks. Benchmark timing counts only active generation time.
- `tests/use_ui_test.rs` and `tests/learn_ui_test.rs`: widget-level coverage for
  the Use tab and all three Learn lessons (input, buttons, output, error state
  and determinism).
- More unit and headless tests across jobs, crypto and the UI: job failure
  propagation, poll-after-completion errors and native/incremental parity;
  ordered NIST steps; SHA-256 avalanche determinism and bit order; distinct
  nonces and ChaCha20 malleability; benchmark repeatability and zero-byte
  handling; spinner/error visibility; bias and sample-size sliders reaching
  jobs; the full NIST verdict table; and native clock monotonicity. The suite
  now has 198 tests.
- `scripts/check.sh`: one local command that mirrors the full CI gate, including
  the feature matrix and the wasm-specific checks.
- A wasm API guard in `ci/wasm-clippy/clippy.toml` plus a CI/Clippy step that denies
  `clippy::disallowed_methods` for the web build. Host-only calls such as
  `std::time::Instant::now` compile on `wasm32-unknown-unknown` but panic in the
  browser; they now fail the build before runtime.
- Bounded completion tests for every entropy source and both job types, boundary
  sample sizes, a self-check that the bounded helper detects a hanging job, and
  UI tests asserting that the Test/Benchmark spinners clear within a poll limit.
- `scripts/wasm-smoke.mjs`: a zero-dependency browser runtime smoke test. It
  serves the Trunk build locally, opens it in headless Chromium over CDP, and
  fails on console errors, uncaught exceptions, a missing/unsized canvas or an
  unexpected title. CI runs it in its own `browser-smoke` job, and `deploy`
  waits for it. This covers wasm-bindgen init, WebGL and frame-loop failures
  that compilation and native tests cannot see. The script has a `--self-test`
  mode that injects a known console error to prove the detector is attached.

### Changed
- **License changed from MIT to GNU General Public License v3.0** in `LICENSE`,
  `Cargo.toml`, the README and CONTRIBUTING. Releases published under MIT keep
  their original license; GPLv3 applies from 0.5.0 onward.
- Repository documentation now names
  [r7notes / r7crypto](https://rvoidex7.github.io/r7notes/Projects/r7crypto)
  as the single home for architecture, design decisions, dependencies,
  build/CI and roadmap. The README keeps only the public entry point.
- Removed the placeholder "Screenshots" section from the README; no screenshots
  exist, so it only advertised the web app under a misleading heading.
- Test and Benchmark jobs now use the same incremental pipeline. NIST tests run
  one per step and metrics reuse those results instead of recomputing the suite.
- `SourceKind::build` returns `Box<dyn EntropySource + Send>` so seeded sources
  can move into native worker threads.
- On wasm, `getrandom/js` makes the **System** source use the browser's
  `crypto.getRandomValues()`; the UI states this explicitly.
- `rust-toolchain.toml` installs the `wasm32-unknown-unknown` target with the
  pinned toolchain.
- CI installs Trunk from a prebuilt release instead of compiling it on every
  run, and the deployment job is named "Deploy web app" — the published build
  is the full workbench, not a reduced demo.
- Pages artifact upload and deployment run in separate jobs. `deploy` waits for
  `build-pages`, so the artifact lookup happens after the upload has settled
  instead of racing it in the same job (actions/deploy-pages#451).

### Fixed
- Trunk also builds Cargo's binary target for `wasm32`, where `eframe`'s native
  API is compiled out. `src/main.rs` now has a wasm no-op `main`, so an
  all-features wasm build succeeds; the actual web entry point remains
  `src/web.rs` in the library.
- The bin and cdylib share the crate name, so a plain Trunk build saw two
  artifacts called `r7crypto` and aborted. The new `desktop` feature gates the
  native bin; Trunk builds with `--no-default-features --features gui`, leaving
  only the cdylib. CI type-checks that exact feature set too.
- The wasm CI check now covers `--bins` as well as `--lib`, so this class of
  failure is caught before the deploy job runs.
- Verified the full static site locally with Trunk 0.22.2: `dist/` contains the
  hashed WASM/JS assets and `/r7crypto/` base URLs.
- The Learn lesson speed controls and the Shannon/NIST input rows now wrap and
  constrain their widths, so every control stays visible and clickable at
  360 px. Benchmarking zero bytes reports zero throughput/latency instead of
  `NaN`.
- **Browser benchmarks no longer freeze.** Timing now goes through
  `src/time.rs`: `performance.now()` on wasm and `Instant` natively, instead of
  calling `std::time::Instant::now()` in the browser. The host-only API guard
  stays in CI to prevent regression.
- Updated `url` to 2.5.9 and `form_urlencoded` to 1.2.3
  (RUSTSEC-2026-0592, stack overflow in `Url::origin()` on nested `blob:` URLs).
  No first-party code calls `Url::origin()`.
- Linux CI installs `libwayland-dev` before all-features builds; `wayland-sys`
  probes for `wayland-client.pc`, which the Ubuntu runners do not ship.

## [0.4.0] - 2026-11-21

### Changed
- **Renamed the project to `r7crypto`** (display name **r7 Crypto Lab**). The
  package, library, binary, app id, `.desktop` entry, icon asset names, docs
  links and release artifacts now use the new name. Previous release notes keep
  the original `entropy-forge` name as history.

### Added
- **Avalanche tab**: visualize the SHA-256 strict avalanche criterion. Pick a
  message and an input bit, then see which of the 256 output bits flipped in a
  16×16 grid, alongside mean/min/max diffusion over up to 200 single-bit flips
- `tests/avalanche_ui_test.rs`: headless tests for the tab button, message
  field, input-bit slider and Run button
- A 360px overflow test for the Avalanche grid, and the new tab in the general
  narrow-screen tab sweep

## [0.3.0] - 2026-11-19

### Added
- **RNG Arena**: a header source selector plus six new generators — seeded
  ChaCha20, LCG (glibc and RANDU presets), 16-bit LFSR, biased coin with an
  adjustable `P(1)`, and a deterministic counter. The Test and Benchmark tabs
  now run against the selected source instead of hardcoding `SystemEntropy`.
  Changing the source or bias clears stale results and cancels running jobs.
  Deterministic sources use a fixed seed so runs are reproducible.
- `tests/arena_ui_test.rs`: headless interaction tests for the source selector,
  the bias slider, and the Test/Benchmark jobs
- The bundled application icon is used for the window, the in-app header and
  the README instead of a lock emoji
- Windows release builds embed `assets/icons/icon.ico` into the executable, so
  Explorer and the taskbar show the real icon
- Linux release archives include a `.desktop` entry and a 256x256 hicolor icon

### Changed
- `Box<dyn EntropySource>` and `&mut T` now forward `reset()` to the wrapped
  source instead of silently using the no-op default
- Release builds on Windows no longer open a console window next to the GUI
- The window declares the `entropy-forge` app id, so Wayland panels can match
  it with the installed `.desktop` entry
- Upgraded `statrs` to 0.19, which drops the unmaintained `paste` transitive
  dependency (RUSTSEC-2024-0436)

### Removed
- Unused direct dependencies: `rand`, `thiserror` and `criterion`

## [0.2.0] - 2026-11-15

### Added
- Interactive "Learn" tab with step-by-step XOR visualization
- Bit-level encryption visualization
- Auto-play animation with speed control
- Educational explanations for cryptographic concepts
- `KeyedStreamCipher`: real ChaCha20 encryption/decryption with a 256-bit key
  and 96-bit nonce, plus SHA-256 passphrase key derivation
- `AvalancheAnalyzer`: real SHA-256 strict avalanche criterion analysis
- Shared zero-allocation MSB-first bit reader (`quality::bits`)
- NIST SP 800-22 Block Frequency and Cumulative Sums tests
- NIST SP 800-90B MCV min-entropy (99% confidence bound), Markov-1
  conditional min-entropy and lag-1 serial correlation metrics
- `tests/crypto_nist_golden.rs` NIST golden vectors and adversarial cases
- `tests/mobile_ui_test.rs` headless 360x800 narrow-screen rendering test
- Use tab: explicit Encrypt/Decrypt actions and a "Ciphertext as hex" option
  that describes only the ciphertext representation
- Shared GUI spacing tokens (`FIELD_GAP`, `SECTION_GAP`) for consistent form
  layout across screens
- Widget-level encrypt/decrypt round-trip test and 360 px bounding-box
  assertions for every tab

### Changed
- Rewrote the Runs, Longest Run, Serial and Frequency tests against the NIST
  SP 800-22 algorithms and validated them with the official sample vectors
- Quality score is capped at 30 for deterministic or structurally unsound
  sequences, even when Shannon entropy is 8.0
- `Test` and `Benchmark` jobs run on background threads instead of blocking the
  UI thread
- Learn tab now operates on UTF-8 bytes instead of truncating characters, and
  reports entropy against the absolute 8 bits/byte scale
- GUI migrated to egui/eframe 0.37 and egui_kittest 0.37
- Text fields and sliders expose accessible labels; fixed widths that overflow
  a 360 px phone screen were removed

### Fixed
- Decrypting in the GUI no longer hex-encodes the recovered plaintext; the hex
  option applies only to the ciphertext representation
- Invalid hex ciphertext is reported instead of being silently treated as raw
  text bytes

### Removed
- Misleading `avalanche_effect` method on the educational XOR `StreamCipher`
  (XOR has no plaintext avalanche effect by construction)

## [0.1.0] - 2025-01-XX

### Added
- Initial release
- Pluggable `EntropySource` trait system
- System entropy implementation (Windows BCryptGenRandom, Linux /dev/urandom, macOS SecRandomCopyBytes)
- Stream cipher implementation with XOR keystream
- Quality metrics (Shannon entropy, min-entropy, chi-square)
- NIST SP 800-22 statistical tests (Frequency, Runs, Longest Run, Chi-Square, Serial)
- Performance benchmarking
- egui-based GUI with four tabs:
  - **Use**: Stream cipher encryption/decryption
  - **Test**: Quality analysis and NIST tests
  - **Benchmark**: Performance measurement
  - **Learn**: Educational visualization
- Cross-platform support (Windows, Linux, macOS)

### Security
- Note: This is an educational tool. Do not use for production cryptography.

---

[Unreleased]: https://github.com/rvoidex7/r7crypto/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/rvoidex7/r7crypto/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/rvoidex7/r7crypto/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/rvoidex7/r7crypto/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/rvoidex7/r7crypto/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/rvoidex7/r7crypto/releases/tag/v0.1.0

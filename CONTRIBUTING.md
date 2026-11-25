# Contributing to r7 Crypto Lab

Thanks for your interest in contributing. This file covers the repository workflow. The engineering notes — architecture, dependencies, build/CI, roadmap — are in [r7notes / r7crypto](https://rvoidex7.github.io/r7notes/Projects/r7crypto).

## Code of Conduct

Please be respectful and constructive in all interactions. See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## How to contribute

### Reporting bugs

1. Check [Issues](https://github.com/rvoidex7/r7crypto/issues) for an existing report.
2. Open a new issue with the [Bug Report template](.github/ISSUE_TEMPLATE/bug_report.md) and include reproduction steps, platform and version.

### Suggesting features

1. Check existing issues for a similar suggestion.
2. Open one with the [Feature Request template](.github/ISSUE_TEMPLATE/feature_request.md), describing the use case first and the implementation second. A feature that only adds information belongs in the notes, not in the app.

### Submitting code

1. Fork the repository and clone your fork.
2. Create a branch: `git checkout -b feature/your-feature-name`.
3. Make your changes following the style notes below.
4. Run the full local gate: `./scripts/check.sh`.
5. Commit with a clear message, push to your fork and open a Pull Request.

CI is authoritative; the script is the local mirror. The wasm-only guard rejects host-only APIs such as `Instant::now` in the web build — use the shared `time::Clock` for timing. See the [Roadmap](https://rvoidex7.github.io/r7notes/Projects/r7crypto/Roadmap).

## Development setup

Rust stable (edition 2021) and Git are enough for the native app. Web work additionally needs the `wasm32-unknown-unknown` target, `trunk`, Node 22+ and a Chromium binary (`CHROME_BIN` if it is not in a standard location).

```bash
git clone https://github.com/rvoidex7/r7crypto.git
cd r7crypto
cargo run --release
./scripts/check.sh
```

## Code style

- `cargo fmt --all -- --check` and `cargo clippy --all-targets --all-features --locked -- -D warnings`; `./scripts/check.sh` runs both.
- Write documentation comments for public APIs.
- Every new feature gets its own tests covering input, button, output format, error state and determinism; UI changes also get a headless 360×800 render check and a human UX review. See [Architecture](https://rvoidex7.github.io/r7notes/Projects/r7crypto/Architecture).
- New crates are added with the feature that needs them, not before — see [Dependencies](https://rvoidex7.github.io/r7notes/Projects/r7crypto/Dependencies).

Commit messages use imperative present tense and a first line under 72 characters, for example `feat: add step-by-step XOR visualization` or `fix: correct entropy calculation for edge cases`.

## Project structure

```text
r7crypto/
├── src/
│   ├── entropy/     EntropySource trait and implementations
│   ├── crypto/      Cryptographic operations
│   ├── quality/     Quality metrics and NIST tests
│   ├── bench/       Performance benchmarking
│   ├── learn/       Teaching models
│   ├── viz/         GUI components and jobs
│   └── web.rs       WASM bootstrap
├── index.html       Trunk page for the web build
├── Trunk.toml       Web build configuration
├── examples/        Runnable API examples
├── scripts/         Local gate and browser smoke test
└── tests/           Integration and headless UI tests
```

## Getting help

Open an issue for questions, check the [project notes](https://rvoidex7.github.io/r7notes/Projects/r7crypto), or look at the existing code and examples.

## License

By contributing, you agree that your contributions are licensed under the GNU General Public License v3.0 (see [LICENSE](LICENSE)).

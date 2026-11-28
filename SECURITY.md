# Security Policy

GitHub uses `SECURITY.md` to surface a private **Report a vulnerability** entry in the Security tab and to tell reporters which versions are maintained. That is why this file stays in the repository; design and dependency rationale live in the [project notes](https://rvoidex7.github.io/r7notes/Projects/r7crypto).

## Supported versions

| Version | Status |
|---|---|
| Latest release | Maintained |
| Older releases | Not maintained |

Release assets and notes are on the [releases page](https://github.com/rvoidex7/r7crypto/releases).

## Reporting a vulnerability

Use the repository's **Security → Report a vulnerability** link (private GitHub security advisory). If that option is not available, open a draft security advisory or contact the maintainer through the channels on the [Connect page](https://rvoidex7.github.io/r7notes/Connect).

Please do not open a public issue before a fix is available. Include the affected version, reproduction steps and impact, and allow reasonable time for a fix before public disclosure.

## Scope

The ChaCha20 path uses RustCrypto's RFC 8439 implementation. The XOR one-time-pad view is a teaching toy, not a cipher. Implementation details and test coverage are documented in the [Architecture notes](https://rvoidex7.github.io/r7notes/Projects/r7crypto/Architecture).

Known RUSTSEC advisories on transitive dependencies are **unmaintained, not vulnerable** reports; each has its own tracking issue linked from the [Dependencies notes](https://rvoidex7.github.io/r7notes/Projects/r7crypto/Dependencies).

## Dependency monitoring

Dependabot monitors dependency updates and `cargo audit` runs weekly via `audit.yml`, filing an issue per advisory.

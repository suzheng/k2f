# Compatibility

K2F compatibility has three layers: the **on-disk format** ([spec v0.3](docs/spec/k2f-v0.3.md)), the **SDK release line** (**0.3.x** on PyPI, npm, and crates.io), and the **compiler identity** stored on each lock (`engine_version`, `engine_commit_sha`). This page states policy for those layers. Field rules are normative in the [format spec](docs/spec/k2f-v0.3.md); viewer banners and verify hash codes are summarized on [Integrity](/integrity) and [Verify](/verify).

## At a glance

| Topic | Policy |
| --- | --- |
| Format contract | [Spec v0.3](docs/spec/k2f-v0.3.md), released with SDK **0.3.x** |
| Older release lines | **0.2.x and earlier** are not supported ([Security](SECURITY.md#supported-versions)) |
| SDK patch releases | Keep spec **v0.3** unless JSON schemas or the lock format break |
| Lock `engine_version` | Full compiler semver on the lock; not a second public spec number |
| `ENGINE_MISMATCH` | Provenance when the reader engine differs — not a broken package |
| Security fixes | Current **0.3.x** line only ([Security](SECURITY.md)) |

## Supported SDK runtimes

Reference SDKs in this repository target:

| Runtime | Version | Install |
| --- | --- | --- |
| Python | 3.9+ | `pip install k2f` |
| Node.js | 18+ | `npm i @openk2f/k2f` (scoped; unscoped `k2f` on npm is taken) |
| Rust | latest stable | `cargo install k2f` |

Building the engine or desktop reader from source requires **Rust (stable)**. See [Contributing](CONTRIBUTING.md#prerequisites) for the contributor toolchain.

Packaged [desktop reader](/download) installers bundle an engine build; they follow the same format contract as the CLI/SDK, not a separate semver product line ([Desktop reader](desktop/k2f_reader/README.md)).

## Format and SDK versioning

Format spec **0.3** is the current on-disk contract, aligned with the **0.3.x** SDK release line. Patch SDK releases do not rename the spec file unless embedded schemas or the lock format break compatibility.

Breaking schema or lock-format changes bump the spec version and ship in the same release as the SDK. Release notes: [CHANGELOG](CHANGELOG.md). Shipped vs planned features: [Project status](docs/guide/status.md).

## Engine identity on locks

Every compiled lock records:

- **`engine_version`** — semver of the compiling engine (same value as `k2f_core` at build time)
- **`engine_commit_sha`** — git commit the engine was built from

`appearance_hash` binds those fields **as stored on the lock**. Verification recomputes hashes using that recorded identity. When the hash chain is valid but the **reader** was built from a different version or commit, verify reports **`ENGINE_MISMATCH`**. That is **provenance**, not tampering: viewers stay `UNSIGNED` or `SIGNED` and paint the published lock.

Do not map `ENGINE_MISMATCH` to `BROKEN_INTEGRITY` or `SIGNED_BUT_BROKEN`. Details: [Integrity verification](docs/spec/k2f-v0.3.md#integrity-verification) in the spec and [Security](SECURITY.md).

Rewriting `engine_version` or `engine_commit_sha` on the lock without refreshing `appearance_hash` is **`APPEARANCE_CHANGED`** (tamper), not a quiet mismatch.

## What to do when verify reports a code

| Code | Typical action |
| --- | --- |
| `ENGINE_MISMATCH` | Read and export quietly; relock only when recompiling with the current engine |
| `CONTENT_CHANGED` | Recompile from the updated semantic tree |
| `APPEARANCE_CHANGED` | Recompile after theme, font, page, or unbound engine-field changes |
| `FONT_MISSING` | Repack with required embedded fonts |

Full code list and banner rules: [Integrity](/integrity).

## Release builds and `K2F_ENGINE_COMMIT_SHA`

Release builds of `k2f_core` panic if `engine_commit_sha` would be `UNKNOWN`. Set **`K2F_ENGINE_COMMIT_SHA`** in environments without a git checkout (for example some CI images). Debug builds may record `UNKNOWN` unless the variable is set.

## Cross-platform locks

Layout is intended to be deterministic for a given engine build and semantic tree. Example locks under `examples/published/` are verified in CI on macOS (`lock-macos` job) to catch platform-specific layout differences.

## Related

- [Integrity](/integrity) — viewer banners and verify codes
- [Changelog](CHANGELOG.md) — releases and spec bumps
- [Contributing](CONTRIBUTING.md) — development setup and golden suite

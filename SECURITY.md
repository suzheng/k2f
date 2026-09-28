# Security Policy

This page covers how to report vulnerabilities in the K2F reference engine, SDK, and official tooling. Normative format rules (integrity codes, banners, prohibitions) live in the [format spec](docs/spec/k2f-v0.3.md); user-facing banner behavior is summarized on [Integrity](/integrity) and [Verify](/verify).

## Reporting a vulnerability

Report security issues through [GitHub Security Advisories](https://github.com/suzheng/k2f/security/advisories/new). **Do not** open public GitHub issues, discussions, or pull requests for undisclosed vulnerabilities.

Include a clear description, steps to reproduce, and the impact you see (confidentiality, integrity, availability, or cross-user risk on [k2f.dev](https://k2f.dev) if relevant). We aim to acknowledge reports within **five business days**.

## Scope

**In scope**

- Rust engine and layout compiler (`engine/`, including WASM builds used by the site and SDK)
- JavaScript / TypeScript SDK (`@openk2f/k2f`, npm package in `sdk/js`)
- MCP stdio server (`engine/k2f_mcp`)
- Desktop reader (`desktop/`)
- Official web surfaces on k2f.dev (playground uploads, publish links, verify page)

**Out of scope**

- Vulnerabilities in third-party dependencies that are already fixed in a supported release line — please report those projects directly
- Issues in unsupported versions (see below)
- General product feedback or format debates — use public issues instead

## Supported versions

Security fixes are released for the current **0.3.x** SDK and matching engine. Format spec **v0.3** is the on-disk contract ([compatibility policy](COMPATIBILITY.md)).

| Release line | Supported for security fixes |
|--------------|------------------------------|
| **0.3.x**    | Yes                          |
| 0.2.x and older | No                        |

## Security properties of `.K2F` packages

K2F treats compiled packages as **trusted data**, not programs. Implementations should preserve these properties:

### No executable content

`.K2F` archives must not carry JavaScript, WASM, macros, or other executable payloads. Viewers and SDKs must **not** evaluate or sandbox-run code from package contents.

### Viewers paint the lock only

Rendering uses the published `document.K2F.lock` (and embedded fonts/assets). Semantic edits require recompilation through the engine; viewers must not silently re-layout from `content/` alone.

### Integrity banners

Every open must surface integrity status. Codes `BROKEN_INTEGRITY` and `SIGNED_BUT_BROKEN` must **never** be presented as signed or trusted.

Hash verification covers semantic content (`content_hash`) and appearance binding (`appearance_hash`, including the lock’s recorded `engine_version` and `engine_commit_sha`). When the hash chain matches the lock but the reader was built from a different commit, verification reports `ENGINE_MISMATCH` — that is **provenance**, not tamper. Do not map `ENGINE_MISMATCH` to `BROKEN_INTEGRITY` or `SIGNED_BUT_BROKEN`. See [Compatibility](COMPATIBILITY.md).

Full code lists and banner UI tiers: [Integrity](/integrity) and [Integrity verification](docs/spec/k2f-v0.3.md#integrity-verification) in the spec.

### Signing

Cryptographic signing is a deliberate human or organizational step. AI agents and automated pipelines produce **`UNSIGNED`** packages by default. A **`SIGNED`** banner requires a valid hash chain **and** a verified Ed25519 signature over the canonical lock JSON (`signatures/v1.json`). Key handling: [Keys reference](docs/reference/keys.md).

### Exports are not sources

PDF, PPTX, DOCX, and IDML exports are one-way drawings of the published lock. They cannot be opened, edited, or re-imported as K2F sources (`PDF_IS_NOT_A_SOURCE` and sibling codes). Treat exports as read-only renderings, not authoritative document state.

### Other format prohibitions

The spec also forbids inline CSS on semantic nodes, arbitrary vector paths in State A, and reliance on system fonts (embedded faces only; missing glyphs fail closed). Details: [Prohibitions](docs/spec/k2f-v0.3.md#prohibitions).

## Related

- [Integrity](/integrity) — banner rules and verify hash codes
- [Verify](/verify) — in-browser package check
- [Compatibility](COMPATIBILITY.md) — engine version matching
- [Format spec](docs/spec/k2f-v0.3.md) — normative integrity and signing contract
- [Contributing](CONTRIBUTING.md) — code contributions (not for undisclosed security issues)

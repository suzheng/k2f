# Security Policy

## Reporting a vulnerability

Please report security issues through [GitHub Security Advisories](https://github.com/suzheng/k2f/security/advisories/new). Do not open public issues for undisclosed vulnerabilities.

We aim to acknowledge reports within five business days.

## Threat model

### No executable content

`.K2F` packages must not contain scripts, WASM, or other executable payloads. Viewers and SDKs must not evaluate code from package contents.

### Integrity banners

Viewers must display an integrity banner on every open. Status codes `BROKEN_INTEGRITY` and `SIGNED_BUT_BROKEN` must **not** be presented to users as signed or trusted documents.

Hash verification covers semantic content (`content_hash`) and appearance binding (`appearance_hash`, including the lock's recorded `engine_version` and `engine_commit_sha`). A reader built from a different commit reports `hash_code` `ENGINE_MISMATCH` but must not present a self-consistent package as `BROKEN_INTEGRITY` or `SIGNED_BUT_BROKEN`.

### Signing

Cryptographic signing is a deliberate human or organizational step. AI agents and automated pipelines produce `UNSIGNED` packages by default. A valid `SIGNED` banner requires both hash-chain validity and a verified Ed25519 signature over the canonical lock JSON.

### PDF is not a source

PDF export draws the published lock. PDF files cannot be opened, edited, or re-imported as K2F sources (`PDF_IS_NOT_A_SOURCE`). Treat exported PDFs as read-only renderings, not authoritative document state.

## Supported versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | yes       |

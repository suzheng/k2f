# Compatibility policy

## Engine version matching

Every compiled lock records:

- `engine_version` — from `engine_version()` (`engine/k2f_core/src/engine_identity.rs`, matches crate semver)
- `engine_commit_sha` — from `engine_commit_sha()` (git HEAD at build time)

`appearance_hash` binds those fields as recorded **on the lock**. Verification recomputes hashes with that recorded identity. When the hashes match but the reader's build identity differs, the hash-chain code is `ENGINE_MISMATCH`. That is **provenance**, not a broken package: viewers stay `UNSIGNED` or `SIGNED` and paint the published lock.

Rewriting `engine_version` or `engine_commit_sha` on the lock without refreshing `appearance_hash` is `APPEARANCE_CHANGED` (tamper), not a quiet mismatch.

## Release builds

Release builds of `k2f_core` panic if `engine_commit_sha` would be `UNKNOWN` (see `engine/k2f_core/build.rs`). Set `K2F_ENGINE_COMMIT_SHA` in environments without a git checkout.

## Spec version

Format spec **0.1** tracks crate version **0.1.0**. Breaking schema or lock-format changes bump the spec version and engine version together in the same release.

## What readers should do on mismatch

| Code | Action |
|------|--------|
| `ENGINE_MISMATCH` | Quiet read (or compact Signed). Relock only when editing with the current engine. |
| `CONTENT_CHANGED` | Recompile from updated semantic tree |
| `APPEARANCE_CHANGED` | Recompile after theme, font, page, or unbound engine-field changes |
| `FONT_MISSING` | Repack with required embedded fonts |

## Cross-platform locks

Example locks committed under `examples/published/` are verified in CI on macOS (`lock-macos` job) to catch platform-specific layout differences.

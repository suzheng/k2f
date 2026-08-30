# Compatibility policy

## Engine version matching

Every compiled lock records:

- `engine_version` — from `engine_version()` (`engine/k2f_core/src/engine_identity.rs`, matches crate semver)
- `engine_commit_sha` — from `engine_commit_sha()` (git HEAD at build time)

A reader or compiler must match both fields. When they differ, verification returns `ENGINE_MISMATCH`. This is **intentional by design**, not a bug — it prevents silent layout drift across engine builds.

## Release builds

Release builds of `k2f_core` panic if `engine_commit_sha` would be `UNKNOWN` (see `engine/k2f_core/build.rs`). Set `K2F_ENGINE_COMMIT_SHA` in environments without a git checkout.

## Spec version

Format spec **0.1** tracks crate version **0.1.0**. Breaking schema or lock-format changes bump the spec version and engine version together in the same release.

## What readers should do on mismatch

| Code | Action |
|------|--------|
| `ENGINE_MISMATCH` | Recompile with the current engine, or use a reader built from the lock's recorded version |
| `CONTENT_CHANGED` | Recompile from updated semantic tree |
| `APPEARANCE_CHANGED` | Recompile after theme or font changes |
| `FONT_MISSING` | Repack with required embedded fonts |

## Cross-platform locks

Example locks committed under `examples/published/` are verified in CI on macOS (`lock-macos` job) to catch platform-specific layout differences.

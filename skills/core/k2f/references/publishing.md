# Publishing K2F Documents

## Overview

**Publish** stores a locked `.K2F` ZIP keyed by `appearance_hash` and returns a permanent public viewer URL. `/v/{hash}` paints the lock only (viewer WASM) — it does **not** recompile. This is **not** the Playground 24-hour `/p/{uuid}` share.

Requires **your own** K2F site with Postgres. **Embedding** with `<k2f-viewer>` does **not** require publish ([embedding-viewer.md](embedding-viewer.md)).

## Decision

| Need | Path |
|------|------|
| Permanent public link | This skill → `/v/{appearance_hash}` |
| 24h draft demo | Playground `POST /api/playground` → `/p/{uuid}` |
| Local / app embed only | [embedding-viewer.md](embedding-viewer.md) (no host required) |
| Cryptographic sign | Human-gated — not in agent workflows |
| New document | [writing.md](writing.md) — never republish Gallery lock bytes as user work |

## Default path

```bash
python3 scripts/publish.py path/to/file.K2F --origin https://your-k2f-site.example
python3 scripts/publish.py file.K2F --dry-run   # appearance_hash only; no POST
```

**Origin is required** for POST (via `--origin` or `K2F_PUBLISH_ORIGIN`). There is no default localhost URL.

The origin must run K2F site with Postgres `DATABASE_URL` and migrations; otherwise publish returns 503.

## Other surfaces

| Surface | Call | Notes |
|---------|------|-------|
| HTTP | Script above (or same POST) | Body = raw ZIP; `Content-Type: application/zip` |
| Site UI | Publish (permanent link) | Same `POST /api/publish` |
| CLI | **None** — do not invent `k2f publish` |

Response: `{ appearanceHash, url, iframe }`. `url` is **relative** (`/v/{hash}`). Absolute share link = `{origin}{url}` where `origin` is the host you POSTed to.

## Embed

| Embed | `src` |
|-------|-------|
| iframe (Notion/README) | Use response `iframe`, or `<iframe src="{origin}/v/{hash}">` |
| `<k2f-viewer>` | `src="{origin}/api/publish?hash={hash}"` — **not** `/v/{hash}` (that route is HTML) |

## Validation loop

1. Ensure package is locked (`pack_verify.py` / Editor `save()`).
2. Optionally `k2f verify` first — **server does not verify or sign**.
3. Publish with explicit `--origin`. Confirm response `appearanceHash` is 64 hex and matches lock (`--dry-run`).
4. Give the user `{origin}{url}` and `iframe`. Do not present a localhost URL as production unless that was the intended origin.

## Rules

1. Body must be a valid ZIP with `document.K2F.lock` containing `appearance_hash` (64 hex). Max **8 MiB**.
2. Same hash dedupes (`onConflictDoNothing`) — republishing identical bytes is idempotent.
3. Links are **public and non-expiring**. Private / password / expiring publish is not shipped.
4. Publish does **not** sign.

## Failure protocol

| Situation | Action |
|-----------|--------|
| Missing `--origin` / env | Pass `--origin` or set `K2F_PUBLISH_ORIGIN` |
| 400 `invalid package size` | Empty or > 8 MiB — shrink or re-save |
| 400 `expected ZIP package` | Body is not ZIP (`PK` magic) |
| 500 `zip missing document.K2F.lock` | Unlock / missing lock — `save` / compile first |
| 500 `package lock missing appearance_hash` | Corrupt or non-K2F ZIP |
| 503 / message mentions `DATABASE_URL` | Site DB unset or migrations missing |
| Viewer shows wrong doc | Hash mismatch — compare `appearanceHash` in response to lock |

## Common mistakes

| Mistake | Reality |
|---------|---------|
| `<k2f-viewer src="/v/{hash}">` | `/v/` is HTML; package bytes are `/api/publish?hash=` |
| Put `content_hash` in the path | URL key is **`appearance_hash`** |
| Treat `/p/{uuid}` as permanent | Playground expires in 24h |
| Share localhost as production | Absolute URL follows POST origin |
| Republish Gallery `.K2F` as user work | Generate a fresh doc ([writing.md](writing.md)), then publish |

## See also

- [writing.md](writing.md) — pack / relock before publish
- [embedding-viewer.md](embedding-viewer.md) — paint lock in an app

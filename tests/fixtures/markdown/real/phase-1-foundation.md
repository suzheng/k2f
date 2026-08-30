# Phase 1 — Foundation: scaffold `apps/tools` and centralized API keys

## Objective

Stand up **`apps/tools`** as a first-class product app (mirroring `apps/voice`) and implement **platform-owned API keys** usable across Tools today and other products later. No tool business logic beyond health/catalog stubs.

**Exit criteria:** `tools.localtest.me:3004` serves a placeholder home; OIDC login works; creating an API key on platform and calling a stub `POST /api/tools/v1/tools/ping/run` with that key returns 200.

---

## 1.1 Scaffold `apps/tools`

**Template:** `apps/voice` (closest match: subdomain product, OIDC, Off client, own Prisma).

### Files to create (checklist)

| Artifact | Pattern source |
|----------|----------------|
| `package.json` | `@pepkio/tools-app`, `dev -p 3004`, `build`: `prisma generate && next build` |
| `next.config.mjs` | `transpilePackages: ['@pepkio/platform']` |
| `tsconfig.json` | `@/*` → `./src/*` |
| `vercel.json` | Monorepo build from repo root |
| `prisma/schema.prisma` | Minimal: `ToolRun` stub optional in Phase 2 |
| `src/middleware.ts` | `localhostAs: 'tools'`, allow tRPC `platform` \| `tools` \| `hello` |
| `src/lib/routing/surfacePaths.ts` | Copy from voice |
| `src/lib/auth/oidc/*` | `OIDC_CLIENT_TOOLS_ID`, `OIDC_CLIENT_TOOLS_SECRET`, `TOOLS_ORIGIN` |
| `src/app/api/oidc/{login,callback,logout}/route.ts` | Copy voice |
| `src/app/api/trpc/[trpc]/route.ts` | Copy voice |
| `src/server/{trpc,router,db}.ts` | `tools` router stub |
| `src/modules/platform/off/client.ts` | Copy voice |

### Monorepo wiring

- Root `package.json`: add `apps/tools` to `build`, `postinstall` prisma generate, `db:*:tools` scripts.
- `scripts/db/config.ts`: register tools app.
- `packages/platform/src/routing/hostSurface.ts`: add `'tools'` to `HostSurface`; `tools.` hostname; `getSiblingHostname` target.
- `packages/platform/src/routing/origins.ts`: `LOCAL_TOOLS_PORT=3004`, `getProductOrigin('tools')`.
- `packages/platform/src/auth/oidc/product-redirect-uri.ts`: add `'tools'` to `FirstPartyProduct`.
- `dev/caddy/Caddyfile` + `dev/dev-gateway.sh`: `tools.localtest.me` → `:3004`.
- Platform: seed `OidcClient` for tools redirect URIs (staging + prod + local).

### Product enum

```prisma
// prisma/schema/90_enums.prisma
enum Product {
  // ...
  TOOLS
}
```

Mirror in `packages/platform/src/types/enums.ts`. Add `toolsProductDescriptor` and register in `apps/www/src/app/(public)/productsRegistry.ts` (marketing path `/tools` → console on `tools.*`).

---

## 1.2 Centralized API key system (platform only)

### Requirements

- User-scoped keys (org/team later).
- **Store hash only** (e.g. SHA-256 of secret + pepper, or bcrypt — match security review).
- Prefix for identification: `ahx_live_` / `ahx_test_` (show secret once at creation).
- **Scopes** as string array: `tools:run`, `tools:read`, `off:session` (read), future `voice:*`, `cro:*`.
- Revocation, last-used timestamp, optional name/label.
- Audit: `ApiKeyUsage` optional in v1 (log last_used_at minimum).

### Prisma models (platform `public` schema)

```prisma
model ApiKey {
  id           String   @id @default(uuid()) @db.Uuid
  userId       String   @db.Uuid
  name         String?
  prefix       String   // first 8 chars of secret for UI display
  keyHash      String
  scopes       String[] // e.g. ["tools:run", "tools:read"]
  environment  String   // "live" | "test"
  expiresAt    DateTime?
  revokedAt    DateTime?
  lastUsedAt   DateTime?
  createdAt    DateTime @default(now())
  updatedAt    DateTime @updatedAt

  user User @relation(fields: [userId], references: [id], onDelete: Cascade)

  @@index([userId])
  @@index([prefix])
}
```

### Off API routes (`apps/platform`)

| Method | Path | Auth | Behavior |
|--------|------|------|----------|
| `GET` | `/api/off/v1/api-keys` | OIDC Bearer | List current user’s keys (no secret) |
| `POST` | `/api/off/v1/api-keys` | OIDC Bearer | Create key; body `{ name?, scopes, environment? }`; response includes **`secret` once** |
| `DELETE` | `/api/off/v1/api-keys/{id}` | OIDC Bearer | Revoke |
| `POST` | `/api/off/v1/api-keys/verify` | **Service** or tools-origin | Body `{ secret }` → `{ valid, userId, scopes }` or 401 |

**Verify endpoint security:**

- Restrict to server-to-server: `X-Pepkio-Internal-Secret` header + allowlist origins, **or**
- Implement verify only in `@pepkio/platform` package imported by tools (shared DB read) — prefer **shared package function** `verifyApiKey(prisma, secret)` to avoid extra HTTP hop.

Recommendation: **`packages/platform/src/apiKeys/verify.ts`** used by both platform (for admin) and tools app (direct Prisma read of platform DB is **not** allowed — tools has separate DB). Therefore:

- **Option A (chosen):** Tools calls `POST off.../api-keys/verify` with internal secret.
- **Option B:** Tools app gets read-only connection to platform DB (avoid — breaks app isolation).

### Shared package surface

```
packages/platform/src/apiKeys/
  types.ts       # ApiKeyScope constants
  hash.ts        # createHash, verifySecret
  scopes.ts      # hasScope(scopes, 'tools:run')
```

### UI for key management (minimal)

- **v1:** `www` account/settings page section “API keys” (OIDC session on www) calling off API via server action or tRPC proxy.
- **v1.1:** Same UI linked from `tools/settings`.
- Display: name, prefix, scopes, created, last used, revoke button.

### Documentation

- `docs/api/api-keys.md`: how to create key, header format, scopes, rate limits.

---

## 1.3 Tools app auth middleware

Implement `withToolsAuth` in `apps/tools/src/lib/api/auth.ts`:

1. Parse `Authorization: Bearer <token>`.
2. If token starts with `ahx_` → call platform verify → `ToolsAuthContext { type: 'api_key', userId, scopes }`.
3. Else → `getPlatformSessionFromOidcAccessToken` (same as `withPaper2VoiceApiAuth` in voice).
4. Else → `anonymous` context if route allows.

**Scope enforcement:** `POST .../run` requires `tools:run` or valid OIDC session (implicit full tools access for logged-in user). `GET .../runs` requires `tools:read` or ownership.

### Rate limiting (foundation)

| Identity | Limit (initial) |
|----------|-----------------|
| Anonymous IP | 20 runs / hour |
| Logged-in user | 200 runs / hour |
| API key | 500 runs / hour (configurable per key later) |

Use existing www rate-limit patterns if present (`apps/www/tests/lib/rate-limit.test.ts`); else Upstash or in-memory for dev.

---

## 1.4 Environment variables

**`apps/tools/.env.local` (document in DEPLOYMENT.md):**

| Variable | Purpose |
|----------|---------|
| `DATABASE_URL` | Tools Postgres |
| `APP_DOMAIN` | `localtest.me` / production domain |
| `TOOLS_ORIGIN` | `https://tools...` |
| `AUTH_ORIGIN`, `OFF_ORIGIN` | Platform surfaces |
| `OIDC_ISSUER`, `OIDC_CLIENT_TOOLS_*`, `OIDC_COOKIE_SECRET` | OIDC client |
| `TOOLS_INTERNAL_SECRET` | Server-to-server to off verify |
| `ANTHROPIC_API_KEY` | Phase 4 assistant (optional in Phase 1) |

**`apps/platform`:** no new public env beyond internal secret if using verify HTTP.

---

## 1.5 Testing (Phase 1)

| Test | Type |
|------|------|
| Middleware blocks `off` paths on tools host | Unit |
| OIDC login round-trip tools.* | Integration (existing `scripts/diagnose-oidc-login.mjs` extend `APP=tools`) |
| API key create → verify → revoke | Platform integration |
| Stub run with API key | Tools integration |

---

## 1.6 Out of scope for Phase 1

- MCP server
- Real bioinformatics handlers
- Pipelines
- www SEO tool pages (stub link only)

---

## Dependencies

- None (first phase).

## Unblocks

- Phase 2 REST + manifests
- Phase 3 MCP (needs auth)

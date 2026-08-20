---
id: ANTI-0009
title: A tenancy-blind cache key — a cached value keyed on less than its isolation boundary
type: anti-pattern
status: accepted
created: 2026-08-18
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, web, mobile]
applies_to: ["*"]
tags: [tenancy, cache, isolation, rsc, cache-components, nextjs, anti-pattern, adr-0024, std-0009]
breaking: false
enforced_by: []
sources:
  - url: https://nextjs.org/docs/app/guides/authentication-with-cache-components
    title: How to implement authentication with Cache Components — Next.js documentation (v16.3.1)
    retrieved: 2026-08-18
    supports: >
      THE DECIDING FACT. Next.js states that cache keys and tags are stored in plain text:
      "A cached function's arguments and captured variables are serialized into its cache key,
      and cacheTag values are stored as written. Neither is hashed: the default cache holds them
      as plain-text map keys and tag lists, and a remote cache receives them the same way."
      It further advises keeping secrets and sensitive personal data out of arguments and tags,
      and keeping the cached function unexported so a caller cannot request another subject's
      data by passing a different id.
  - url: https://nextjs.org/docs/app/guides/self-hosting
    title: How to self-host your Next.js application — Next.js documentation (v16.3.1)
    retrieved: 2026-08-18
    supports: >
      Establishes that the cache is per-instance by default and that a container orchestration
      platform gives each pod its own copy, so a shared cache handler is required for consistency.
      That shared cache is what turns a key collision from a single-pod anomaly into an
      estate-wide one.
---

# ANTI-0009 — A tenancy-blind cache key

## The shape

A cached value whose key does not carry every column of the isolation boundary the underlying
data sits behind.

```ts
// The framework's own example. Correct for a single-tenant application.
async function getNotesByUserId(userId: string) {
  'use cache'
  cacheTag(`notes:${userId}`)
  return db.query.notes.findMany({ where: eq(notes.userId, userId) })
}
```

Under ADR-0024 a row is behind **two** columns, not one. This key carries one. The same defect
appears in its smaller form, where a tag names a resource and no boundary at all:

```ts
cacheTag('notes')          // one tag for every tenant in the estate
cacheTag(`notes:${slug}`)  // a slug is not an isolation boundary
```

And in its most dangerous form, where the key is built from a value the caller supplied rather
than one the server resolved:

```ts
export async function getNotes(tenantId: string) {  // tenantId arrives from the client
  'use cache'
  cacheTag(`notes:${tenantId}`)
  ...
}
```

## Why it is forbidden

**1. Row-level security never sees it.** ADR-0038 sets session context per transaction and fails
closed, which protects *the query*. A cache hit does not reach the query. Every guarantee the
database layer provides is bypassed by construction, and the layer that bypassed it is the one
nobody thinks of as a data path.

**2. The collision is silent, and the key is legible.** Keys and tags are stored as written and
are not hashed. There is no type error, no failed assertion and no log line — one tenant is
served another tenant's rendered output, and the only evidence is a customer noticing.

**3. It defeats the assertion STD-0009 is built on.** STD-0009's rule is that *the denial is the
assertion*. A cache hit produces an allow with no query to deny, so a test suite that only
exercises the data path reports green against a boundary that is not being crossed in the test
and is being crossed in production.

**4. It is invisible locally.** One developer, one tenant, one warm cache never collides. This is
the same failure shape RFC-0016 records for its distance patterns: *"every one of these patterns
looks fine on a laptop with a database on the same machine."*

**5. `use cache: remote` publishes the tenancy graph.** A durable shared cache holding
plain-text keys means operator and tenant identifiers sit in storage that is neither the database
nor the audit log. That is a residency question under RFC-0014 and RFC-0017 before it is a
correctness one, and REQ-0003 owns the privacy half.

## The correct shape

**Every cache key and every tag carries the full tenancy pair**, and the values are resolved
server-side from the session — never accepted as arguments from a caller.

```ts
import 'server-only'
import { cacheLife, cacheTag } from 'next/cache'
import { verifySession } from '@/lib/auth/dal'
import { tenantScopedTag } from '@/lib/cache/keys'

// Exported. Resolves the boundary. Takes nothing the caller could forge.
export async function getNotes() {
  const { operatorId, tenantId, userId } = await verifySession()
  return getNotesScoped(operatorId, tenantId, userId)
}

// Unexported. Receives only resolved values.
async function getNotesScoped(operatorId: string, tenantId: string, userId: string) {
  'use cache'
  cacheTag(tenantScopedTag('notes', { operatorId, tenantId, userId }))
  cacheLife('minutes')
  ...
}
```

Three properties make that shape hold rather than merely look right:

- **The cached function is unexported.** If it is reachable, its arguments are an API, and its
  arguments are the isolation boundary.
- **No call site builds a key or tag by hand.** One helper constructs them, so adding a third
  tenancy column is one edit rather than a search. GLO-0004's nesting is exactly the kind of
  thing that acquires a third level.
- **Nothing sensitive enters a key or a tag.** Not a token, not a raw email, not a session id.
  They are stored as written.

Invalidation obeys the same rule. `updateTag` and `revalidateTag` on a tag that omits the
tenancy pair either invalidates every tenant's entry or none of them, and both are wrong.

## Where it is not this anti-pattern

- **Genuinely global data, keyed globally on purpose.** A currency list, a locale table,
  ADR-0048's registry values. These are identical for every tenant, and a tenancy-scoped key
  would be a cache with a hit rate of zero. Key it globally and say so at the call site, because
  the next reader's default assumption should be that a missing boundary is a bug.
- **`use cache: private`**, which never reaches the server. It is scoped to one browser, which is
  narrower than one tenant — but it is *not* a substitute for scoping, because a browser is not
  an authorization boundary and the data still had to be authorized before it got there.
- **Build-time static generation of pages containing no tenant data.** A marketing page has no
  boundary to carry.

## Why this is `accepted` rather than `locked`

Whether a key carries its isolation boundary requires knowing which values are tenancy columns
and whether a given cached function serves tenant data at all. Neither is a language construct.

A plausible heuristic exists and belongs to `tool-forge`: fail when a function containing a
`use cache` directive is reachable from outside its module and its enclosing scope does not
resolve tenancy through the named session helper. That is the same shape as
`web.server_actions_authorize` — heuristic by necessity, and needing a *named* helper to look
for before it has anything to match. That helper does not exist yet; establishing it is
RFC-0028's business, and this artifact stays `accepted` until it does.

Marking it `locked` today would claim an enforcement that nothing performs, which is the failure
ADR-0002 exists to prevent. Until the check lands, this rule is upheld by review, and the mistake
it prevents looks like a correct function with one argument missing.

## Related

- ADR-0024 — nested tenancy is two columns in one database; the boundary this artifact protects
- ADR-0038 — session context per transaction, fails closed; what a cache hit bypasses
- ADR-0046 — the RSC boundary and its anti-pattern; the precedent for a web-only hazard
- STD-0009 — tenant isolation testing, and why the denial is the assertion
- GLO-0004 — tenancy, and why it is nested
- RFC-0014, RFC-0017 — residency, which a shared plain-text cache touches
- RFC-0016 — the failure mode that only appears away from a laptop
- RFC-0028 — the template that must establish the named helper this rule needs
- REQ-0003 — data and privacy governance
- ADR-0002 — why this is `accepted`

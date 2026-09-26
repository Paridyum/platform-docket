---
id: ADR-0097
title: The docket CLI reaches a remote docket over its authenticated API; the database link is break-glass only
type: adr
status: accepted
created: 2026-09-14
locked_at: null
supersedes: []
superseded_by: null
scope: [tool, platform, org]
applies_to: ["*"]
tags: [docket, tool-docket, cli, api-client, oidc, device-flow, break-glass, security, adr-0092, adr-0093, req-0001]
breaking: false
enforced_by: []
---

# ADR-0097 — The docket CLI reaches a remote docket over its authenticated API

## Status

**Accepted**, 2026-09-14. Refines **ADR-0093 §3**, which said the CLI is "a break-glass client of
the same database." That is true but incomplete, and the incompleteness is a security hole in
disguise: it implied everyone who uses the CLI holds the production database URL. A human direction
(recorded per C-0004): most operators — HR included — must use docket without ever being handed the
database link.

## Context

docket now has a served, token-authenticated backend (ADR-0093, ADR-0092) and three ways its store
is reached, but only two were named, and the missing one is the one most people need:

- **File store** — offline/local, no network. Fine for a solo laptop, not for a shared platform.
- **Database store** (ADR-0093 §3, D-61) — the CLI as a direct Postgres client. It requires
  `DOCKET_DATABASE_URL`: the production database's host and credentials. That is a **prod secret**,
  and handing it out is exactly what RLS and per-operator authentication exist to avoid. HR cannot
  have it; nor should most engineers.
- **The served API** — the browser console (`console-docket`) already reaches docket this way: an
  OIDC operator token (ADR-0092), no database link. But there was **no CLI equivalent**, so a person
  who prefers the command line had only the two extremes: no access, or the whole database.

The operator's framing is the decision: the database link stays restricted, and reaching docket
without it must be a first-class, prod-ready path — for the CLI and for one docket (or an
integration) talking to another.

## Decision

**1. The CLI gains a third mode: an authenticated HTTP client of a remote docket's `/api/*`.** With
`DOCKET_API_URL` set (e.g. `https://docket.paridyum.com`), the CLI calls the served backend's
high-level endpoints (roster, person, onboard, sign, advance, enrol) and renders the response. It
does **not** run the application layer locally and does **not** touch the database — the server does,
behind ADR-0092 auth. This is the default networked path.

**2. The three modes are ordered by what the caller holds, and the API mode is the one to reach
for.** `DOCKET_API_URL` → API client (an operator token, no DB link — HR and most operators).
`DOCKET_DATABASE_URL` → direct Postgres (a prod secret — **break-glass, migrations, admin only**).
Neither → file store (offline). The CLI states which mode it is in on every run, and the database
mode says the word "break-glass" so nobody uses it by habit.

**3. The token is obtained by OIDC device authorization, not by pasting the database URL.**
`docket login` runs the device flow against the control-plane instance (`id.paridyum.com`): it prints
a URL and a code, the operator signs in in a browser, and the CLI receives and caches a short-lived
operator token (REQ-0001: the token is the credential, cached with restrictive permissions, never a
long-lived secret in a file). A **service** account — one docket automating against another — uses a
client-credentials or a supplied token instead of the interactive flow, the same `/api/*` behind the
same ADR-0092 verification.

**4. "docket connects to another docket" is this, not a second mechanism.** A docket instance, a
future integration, or a script reaching another docket does it over the authenticated API with a
token — never by sharing a database. The served backend is the one front door; the database is not a
door at all for anyone but a break-glass admin.

## Consequences

**Accepted costs.**
- **The API mode needs the backend deployed with a reachable ingress** — it is only usable once
  docket is served at a URL (the Helm chart exists; deployment is the gate). Until then the CLI
  falls back to file/DB mode.
- **A device flow is more moving parts than reading an env var** — a browser round-trip, a poll, a
  cached token with an expiry. That is the cost of not distributing the database, and it is the same
  cost the browser console already pays.
- **The CLI now speaks the wire contract** the console also speaks; a change to the served API's
  shape is a change two clients must track (the same deliberate duplication ADR-0093 already carries
  between docket and console-docket).

**Gained.** HR and every operator use docket — CLI or browser — with a login, not a database secret;
the production database URL stops being an access-distribution mechanism; and docket-to-docket has one
authenticated answer instead of an implicit "share the DB."

**Rejected alternatives.**
- **Give trusted operators the read-only database URL.** Rejected — RLS is per-tenant, not
  per-operator, so the URL is all-or-nothing within a tenant, and a credential handed to N people is
  a credential leaked; ADR-0092 exists precisely so access is a revocable token, not a shared secret.
- **A separate read-only reporting API.** Premature — the served `/api/*` already carries the reads,
  gated by the operator role; a second surface is more to secure for no capability the first lacks.

## Related

- **ADR-0093 §3** — refined; the CLI is a break-glass DB client *and* a first-class API client.
- **ADR-0092** — the operator token the API mode presents and the backend verifies.
- **REQ-0001** — the cached token is a runtime secret, short-lived, not a file credential.

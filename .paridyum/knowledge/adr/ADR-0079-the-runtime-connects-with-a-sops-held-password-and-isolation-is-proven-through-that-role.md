---
id: ADR-0079
title: The runtime connects with a SOPS-held password, and isolation is proven through that role
type: adr
status: accepted
created: 2026-09-12
locked_at: null
supersedes: null
superseded_by: null
scope: [platform, service]
applies_to: ["platform-*", "svc-*"]
tags: [database, credentials, sops, rls, tenancy, adr-0024, adr-0071, std-0009, std-0017, run-0004, req-0022]
breaking: false
enforced_by: []
---

# ADR-0079 — The runtime connects with a SOPS-held password, and isolation is proven through that role

## Status

**Accepted**, 2026-09-12. This decides the *credential and the connection*, not the role posture —
ADR-0024 decision 3 already fixed that the runtime connects as a non-owner, non-`BYPASSRLS` role
(`paridyum_app`). What was never written down is the question a re-implementer actually asks next:
*what password does that role use, where does it live, how is it rotated, and how do we know the
isolation is real and not just configured?* That gap is what this closes.

Not `locked`: the isolation proof it mandates is a runtime test whose gate (`STD-0009`'s
`test.tenant_isolation_suite_covers_owned_tables`) awaits tool-forge; it graduates when that check
runs the proof in CI rather than an operator running it by hand.

## Context

Three facts were already true and already governed, and they left exactly one hole between them:

- **ADR-0024 D3** — the application connects as `paridyum_app`, which owns no tables and cannot
  bypass RLS. The role exists live with precisely those properties.
- **RUN-0004** — a database password is an *Automation* secret: SOPS-encrypted in the consuming
  repository, never in the password manager, never memorised. The age key that decrypts it is the
  operator's one held secret.
- **ADR-0071 D3 / the web chart** — real applicant data is gated behind a rehearsed off-host
  restore, and the chart deliberately withholds `PARIDYUM_DATABASE_URL` until that rehearsal.

The hole: `paridyum_app` is `NOLOGIN` with no password, so nothing has ever connected as it, and
`verify-isolation.py` proves the *configuration* (RLS forced, no bypass, role is not the owner) but
not the *behaviour* — that a query run as that role with account A's context cannot see account B's
row. STD-0009 says in as many words that the configuration proof is the one that "every dashboard
reports as healthy" while an outer-only policy leaks. The behaviour has to be exercised, and it has
to be exercised **through the role the application actually uses**, or it proves nothing about the
application.

## Decision

1. **The runtime's database credential is an Automation secret in SOPS, per RUN-0004.** The
   `paridyum_app` password is generated (STD-0017 §5: by the password manager, alphanumeric, so a
   shell-based reset never becomes a quoting bug), set on the role, and the resulting
   `PARIDYUM_DATABASE_URL` DSN is stored SOPS-encrypted in `infra-kubernetes` next to the session
   secret. It is never placed in a migration file — a migration is committed plaintext, and a
   password in one is a secret in git (C-0004). The role's *capability* (LOGIN) may be set by a
   migration or a script; the *password* is set only by the credential script that reads and writes
   SOPS.

2. **The connection is wired only at the ADR-0071 gate, and the two moves are one moment.** Adding
   `PARIDYUM_DATABASE_URL` to the deployment and flipping `PARIDYUM_BACKUPS_CONFIRMED` to `1` happen
   together, after the restore round-trip is rehearsed — as the chart already documents. Preparing
   the credential (minting the password, storing the DSN ciphertext) is safe to do earlier because
   nothing references it and the tables enforce RLS and are empty; *using* it is what the gate holds.

3. **Isolation is proven behaviourally, through `paridyum_app`, and the denial is the assertion**
   (STD-0009). A runtime proof sets a session context, runs a read as the app role, and asserts the
   rows it must *not* see are absent — the account boundary (a signed-in account cannot see another
   account's rows), the tenant boundary (an account under one tenant cannot see another tenant's
   rows), the fail-closed default (an accessor with no context raises, it does not return NULL and
   filter), and the operator grant (operator scope sees the tenant's rows, and it is a set value,
   never an absence — GLO-0005). The proof runs against synthetic rows in a transaction that is
   rolled back, so it needs no credential (`SET ROLE`), touches no real data, and is not held by the
   ADR-0071 gate.

4. **The credential rotates by re-running the same script.** Rotation generates a new password, sets
   it on the role, rewrites the SOPS DSN, and the next deploy picks it up. There is no second path
   and no manual `ALTER ROLE` typed at a console — that is the forbidden production DDL REQ-0008
   item 1 names, and it also risks a plaintext password in a shell history file.

## Consequences

- `scripts/prove-isolation.py` exercises decision 3 now, live, through `SET ROLE paridyum_app` with
  synthetic accounts and tenants, asserting every denial and rolling back. It is the runtime half of
  ADR-0024 that STD-0009 requires, and it answers REQ-0022's isolation criterion behaviourally rather
  than only from the catalogue.
- `scripts/provision-app-login.py` is the credential script of decisions 1 and 4: generate, set,
  store, and it is what an operator runs at the ADR-0071 gate (and to rotate). It is built and
  documented now; it is executed at the gate, not before, because executing it earlier wires nothing.
- `verify-isolation.py` (configuration) and `prove-isolation.py` (behaviour) are both kept: the first
  catches a hand-disabled policy or a role that gained BYPASSRLS; the second catches a policy that is
  present but scoped to the wrong column. Neither subsumes the other, which is exactly STD-0009's point.
- The SOP for platform-tenancy documents both the gated wiring and the rotation path, so the next
  operator does not reconstruct this from the chart comments.

## Related

- **ADR-0024** — decision 3 is the role posture this credential attaches to.
- **ADR-0071** — the backup gate that the connection wiring is co-gated with.
- **STD-0009** — the runtime isolation testing this proof implements; the denial is the assertion.
- **RUN-0004 / STD-0017** — where the credential goes and how it is named and generated.
- **REQ-0022 / REQ-0008** — the tenancy and schema requirements this advances.

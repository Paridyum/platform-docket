---
id: RUN-0009
title: Taking down a tenant — an abuse report, from receipt to stopped host
type: runbook
status: accepted
created: 2026-10-09
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, infra]
applies_to: ["*"]
tags: [abuse, takedown, routing, zitadel, edge, adr-0138, rfc-0056]
breaking: false
enforced_by: []
---

# RUN-0009 — Taking down a tenant

**Status: accepted, 2026-10-09, after a rehearsal** on synthetic tenant tenant-b6 (infra-kubernetes
LOGBOOK, "ADR-0138 D6 checks"). Takedown, check, reinstate and X2's `check` all passed.

## When

A report arrives at `abuse@paridyum.xyz` or `abuse@paridyum.in` (ADR-0138 D4): phishing, malware,
illegal content, or impersonation on a tenant's default host (`<slug>.paridyum.xyz`) or own domain.

## What exists, and what does not

Tenants have no `suspended` state (ADR-0138 D5). A takedown therefore uses two controls that do exist:
- **routing's bindings.** Removing a host's binding stops it resolving at once. This is the saga's
  compensation call (`DELETE /v1/bindings/{host}`), made with the saga's identity.
- **the tenant's Zitadel organization.** Deactivating it stops every sign-in to the tenant.

Neither deletes data. ADR-0039 never hard-deletes, and a takedown must be reversible.

## Steps

1. **Confirm the report is about one of ours.** Resolve the host. A default host is ours if it sits
   under the app zone. An own domain is ours if routing has a binding for it. If it is not ours,
   answer the reporter and stop.
2. **Stop the host:** `scripts/takedown-tenant.py plan --host <host>` in infra-kubernetes. It prints the
   tenant, every host bound to it, and its organization, and changes nothing. Read the output, then
   run `apply` with the same arguments. Apply removes every binding of that tenant and records each
   host it removed in the state file, so reinstating can restore them. Then run infra-edge
   `make edge-cache-purge`, because the edge may hold a cached page for up to 60 s.
3. **Stop sign-in:** the same tool's apply also deactivates the tenant's organization on the tenant
   instance.
4. **Check:** each removed host answers 404 through the edge (follow redirects: the apps send `/` to
   `/en` first), and the tenant's `/dashboard` cannot be signed in to.
5. **Record and tell:** add a dated entry to infra-kubernetes' LOGBOOK with the report's reference, the
   hosts and the action, and no reporter's personal data. Then answer the reporter, and tell the
   tenant org's owner what was stopped and why.

## Reinstating

`scripts/takedown-tenant.py reinstate --tenant <id>` re-binds the recorded hosts and reactivates the
organization. Plan first, as above, then add `--apply`, then purge the edge cache. An own domain comes
back `awaiting_domain` and is verified again. In the rehearsal, `/dashboard` answered 404 for a few
seconds after `/` and `/careers` were back, then redirected to sign-in again, so check it last.

## Owed

A tenant `suspended` state that the saga and every app honour (RFC-0015 D4). When it exists, steps 2
and 3 become one state change, and this runbook is revised.

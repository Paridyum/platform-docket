---
id: STD-0022
title: Renaming or replacing a service renames it everywhere, in one change
type: standard
status: accepted
created: 2026-09-12
locked_at: null
supersedes: null
superseded_by: null
resolved_by: null
scope: [org, platform, service, web, infra, spec]
applies_to: ["*"]
tags: [naming, rename, replacement, supersession, c-0007, c-0002, adr-0070, adr-0086, adr-0035]
breaking: false
enforced_by: []
---

# STD-0022 — Renaming or replacing a service is a complete, atomic rename

## Status

**Accepted**, 2026-09-12. Written from a live instance: when hiring was split to `web-hiring`
(ADR-0086) and tenant administration to `web-admin`, both repositories were forked from
`web-paridyum` and carried its name in their `manifest.yaml`, `AGENTS.md`, `package.json` and docs
until each was corrected by hand. The correction should not depend on someone noticing; it is a
rule.

## Context

C-0002 and C-0007 fix how a thing is named. They do not say what happens when a thing is **renamed
or replaced** — and that is where names rot, because a name lives in many files and a rename that
updates the obvious ones (the directory, the repo) leaves the quiet ones (a redirect URI, a
manifest field, a doc header, a cross-reference in an ADR) pointing at something that no longer
exists. ADR-0035 makes the registry the source of truth; a stale former name is a second,
disagreeing source.

A replacement is a rename with a supersession attached: `web-paridyum`'s hiring client became the
`web-hiring` client (ADR-0086), and the old name must not survive in `applications.yaml`, in a
redirect allowlist, or in the consuming code.

## The standard

**When a repository, service, surface, OIDC client, VM, or any named runtime thing is renamed or
replaced, every occurrence of the old name is updated in the same change. A rename is not done
while the old name survives anywhere.**

The places to update, as a checklist — not exhaustive, but the ones that rot in practice:

1. **The directory and the repository name** (C-0002).
2. **The package identity** — `package.json` `name`, `Cargo.toml` `name`, the binary/crate name (C-0007).
3. **`manifest.yaml`** — `name`, `provides.capability`, `entrypoints`, and any `depends` that
   referenced the old name from another repo.
4. **Agent and human docs** — `AGENTS.md`/`CLAUDE.md` headers, `README.md`, `ARCHITECTURE.md`,
   `CHANGELOG.md`, `LOGBOOK.md`, `CONTENT.md`, `SOP.md`/`RUNBOOK.md`. A forked repo inherits the
   source's docs and therefore its name; the header is the first thing to fix, not the last.
5. **Identity and routing** — the Zitadel application in `applications.yaml`, its redirect and
   post-logout URIs, and the public origin it serves.
6. **Registries** — `address-registry.yaml`, `product-registry.yaml`, any `*-registry.yaml` that
   named it; the registry is the source of truth (ADR-0035), so a stale entry there is the worst
   stale name of all.
7. **Cross-references in knowledge** — an ADR, RFC or runbook that named the old thing. A
   superseding decision (ADR-0086 over ADR-0074) names the new thing; the superseded one keeps its
   history, but live artifacts point at the current name.

## What "replace" adds to "rename"

A replacement carries a supersession (ADR-0031 / `new-decision`): the old name does not merely
change, it **stops existing**, and something new stands where it stood. Record the supersession on
the governing artifact, and do not leave the old name reachable as if both still existed — two
clients with overlapping redirect URIs, two repos serving the same origin, two manifests claiming
one capability. ADR-0070 §1's test is the discriminator: if the two can be deployed, revoked or
rotated independently they are two things and both names are legitimate; if one replaced the other,
only the new name is.

## What this is not

Not a ban on keeping history. A superseded ADR keeps the old name in its own text — that is the
record of what was true (ADR-0031). The rule is about **live** artifacts: the ones a build, a
deploy, a login or a reader resolves against today.

## Related

- C-0002, C-0007 — how a thing is named in the first place.
- ADR-0035 — the registry is the source of truth, so a stale registry name is authoritative and wrong.
- ADR-0070 — one client per deployable; the test for whether two names are both legitimate.
- ADR-0086 — the live replacement (`web-paridyum` hiring → `web-hiring`) that occasioned this.
- ADR-0031 — supersession and errata; how the old name legitimately survives in history.

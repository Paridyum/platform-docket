---
id: C-0005
title: Naming rules, with the repository class list held in a registry
type: constitution
status: superseded
created: 2026-08-07
locked_at: 2026-08-07
supersedes: [C-0002]
superseded_by: C-0007
scope: [org]
applies_to: ["*"]
tags: [naming, conventions, api, classes, registry]
breaking: true
enforced_by:
  - check: repo.name_matches_scheme
    severity: error
  - check: cargo.package_name_matches_repo
    severity: error
  - check: cargo.lib_crates_have_no_binaries
    severity: error
    args: { applies_to: ["lib-*"] }
  - check: openapi.paths_are_plural_nouns
    severity: error
    rationale: Verb-in-path is the single most common drift in agent-generated APIs.
  - check: openapi.no_verbs_in_paths
    severity: error
  - check: db.tables_are_plural_snake_case
    severity: error
  - check: knowledge.class_registry_valid
    severity: error
    rationale: The class list moved out of this artifact, so something must guarantee the registry stays well-formed and internally unique — otherwise the list is less protected than when it was prose.
---

# Constitution C-0005 — Naming

Supersedes C-0002. **Every naming rule below is carried forward unchanged.** The single
substantive difference is that the repository class list is no longer a table here; it
lives in `class-registry.yaml`.

Names are the cheapest possible coordination mechanism and the most expensive thing to
change. Every rule here is mechanical so that no agent has to exercise taste.

## Why the class list moved

C-0002 held the classes in a locked prose table, so adding one meant superseding the
naming constitution. Six classes were foreseeable — web applications, React Native,
Flutter, generated SDKs, shared packages, workers — and six whole-artifact rewrites of a
constitution is not a plan, it is a prediction that the second one never happens.

The consequence of not fixing it is on record rather than hypothetical: ADR-0009 describes
the previous estate as *"predominantly Python with TypeScript frontends, none matching
C-0002."* The stack arrived without a class because the ceremony was too expensive.

This is the third time this repository has moved an enumeration out of prose into
validated data while keeping the rule in the artifact — after ADR-0014 for check ids and
ADR-0015 for scope resolution. **The grammar is constitutional. The list is data.**

## Repositories

`<class-prefix><name>`, lowercase, hyphen-separated, singular class prefix.

**The set of valid prefixes is `class-registry.yaml`**, validated against
`class-registry.schema.json`. A prefix not in the registry fails
`repo.name_matches_scheme`. Adding a class is a registry entry plus whatever standard,
template and workflow it needs — never a change to this artifact.

The knowledge base is **sui generis**: no prefix, no class, and deliberately absent from
the registry. It is governed by the `org`-scoped set (ADR-0015), and inventing a class to
satisfy the model is how models acquire vestigial concepts.

The `<name>` is a **noun naming the capability**, never a technology and never an
adjective. `svc-billing`, not `svc-rust-billing`, not `svc-fast-billing`.

Service names are **singular** (`svc-billing`, `svc-user`), because the repository is one
bounded context. Collections are plural only at the API and database layer, below.

## Rust crates and binaries

- Crate name = repository name with the class prefix replaced by `paridyum-`:
  `lib-core` → `paridyum-core`, `svc-billing` → `paridyum-billing`.
  This keeps crate names globally unambiguous without leaking the repo class into code.
- `lib-*` repositories publish **library targets only**. A `[[bin]]` in a `lib-*` crate is
  a hard CI failure — it means an executable escaped into a place nothing deploys.
- Binary name = the bare capability: `tool-forge` → binary `forge`.
- Modules and files: `snake_case`. Types: `PascalCase`. No abbreviations that are not
  already in the glossary.

These rules bind the `rust` **stack**, not a list of prefixes. A class whose registry entry
names a different stack is not governed by them — which is the point, and is why adding
`web` required no change here.

### Note on the `forge` binary name

`forge` collides with Foundry's Ethereum toolchain binary, which is common on developer
machines and would shadow — or be shadowed by — ours on `PATH`. The name is kept because it
is correct for the job, and the bootstrap runbook (RUN-0001) checks for a pre-existing
`forge` and warns. If the collision ever bites, the fallback is to install the binary as
`pdy` and offer `forge` as an opt-in shell alias; that is a one-line change in
`tool-forge`'s `Cargo.toml`, not a rename of anything else.

## HTTP APIs

- Resources are **plural nouns**: `/users`, `/invoices`, `/projects`.
- **Never verbs**: not `/getUser`, not `/createInvoice`, not `/users/search`.
  Actions that resist REST are subresources: `POST /invoices/{id}/refunds`.
- Path segments: `kebab-case`. JSON fields: `snake_case`. Both, always — no per-service
  exceptions.
- Every path is versioned at the root: `/v1/users`.
- Reserved, identical in every service, never versioned: `/health`, `/ready`, `/metrics`,
  `/docs`.

## Databases

- Tables: **plural**, `snake_case` — `users`, `billing_accounts`.
- Columns: `snake_case`. Primary key: `id`. Foreign keys: `<singular>_id`.
- Timestamps: `created_at`, `updated_at`, `deleted_at`, always `timestamptz`, always UTC.
- Migrations: `<seq>_<verb>_<subject>.sql` — `0003_add_invoices_table.sql`.

## Events

`<Subject><PastTenseVerb>` in PascalCase: `UserCreated`, `InvoiceSettled`. Never present
tense, never imperative — an event records what happened, not a command.

## Kubernetes and Helm

- Release name = repository name without the class prefix: `svc-billing` → `billing`.
- Namespace = class: `platform`, `services`.
- Labels: `app.kubernetes.io/name`, `app.kubernetes.io/part-of: paridyum`.

## Environments

Exactly three, named exactly: `dev`, `staging`, `prod`. Not `production`, not `stage`, not
`qa`.

## Consequences of the change

**Accepted costs**

- **The class list is now data, and data is easier to change than a constitution.** That is
  the intent, and it is also the risk: adding a class no longer requires the deliberation a
  supersession forced. `knowledge.class_registry_valid` guards well-formedness and
  uniqueness; it cannot guard judgement. A class still needs a standard, a template and a
  workflow, and those are the real cost — the registry entry was never the expensive part.
- **Two artifacts now read from the registry rather than naming prefixes** — ADR-0020 and
  ADR-0021, superseding ADR-0003 and ADR-0010. Three supersessions were paid at once so
  that none of them is paid again.
- **`scope: [org]` on this artifact** rather than C-0002's enumeration of every class. Under
  ADR-0015 those are equivalent, and the enumerated form would have needed editing every
  time the registry grew — which is the defect being fixed.

**Gained**

- Adding a repository class costs a registry entry, not a constitutional amendment.
- The prefix-to-scope map, the `scope` enum and the `class` enum are generated from one
  source rather than maintained in four places, which ADR-0015 already flagged as a hazard.
- A second language stack becomes possible without touching the constitution, which is the
  specific failure that produced the archived estate.

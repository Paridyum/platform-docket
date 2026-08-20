---
id: C-0002
title: Naming rules for repositories, crates, binaries, services and endpoints
type: constitution
status: superseded
created: 2026-08-06
locked_at: 2026-08-06
supersedes: null
superseded_by: C-0005
scope: [org, platform, service, library, tool, infra, spec]
applies_to: ["*"]
tags: [naming, conventions, api]
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
---

# Constitution C-0002 — Naming

Names are the cheapest possible coordination mechanism and the most expensive thing to
change. Every rule here is mechanical so that no agent has to exercise taste.

## Repositories

`<class-prefix>-<name>`, lowercase, hyphen-separated, singular class prefix.

| Prefix      | Contains                                   | Example            |
| ----------- | ------------------------------------------ | ------------------ |
| *(none)*    | The knowledge base — one repo, sui generis  | `knowledge`        |
| `platform-` | Platform services all products depend on    | `platform-gateway` |
| `svc-`      | Business capability services                | `svc-billing`      |
| `lib-`      | Shared Rust crates — no binaries            | `lib-core`         |
| `tool-`     | Developer-facing executables                | `tool-forge`       |
| `infra-`    | Deployment and cluster configuration        | `infra-kubernetes` |
| `spec-`     | Interface contracts                         | `spec-openapi`     |
| `template-` | Scaffolding sources consumed by forge       | `template-service` |

The `<name>` is a **noun naming the capability**, never a technology and never an
adjective. `svc-billing`, not `svc-rust-billing`, not `svc-fast-billing`.

Service names are **singular** (`svc-billing`, `svc-user`), because the repository is
one bounded context. Collections are plural only at the API and database layer, below.

## Rust crates and binaries

- Crate name = repository name with the class prefix replaced by `paridyum-`:
  `lib-core` → `paridyum-core`, `svc-billing` → `paridyum-billing`.
  This keeps crate names globally unambiguous without leaking the repo class into code.
- `lib-*` repositories publish **library targets only**. A `[[bin]]` in a `lib-*` crate
  is a hard CI failure — it means an executable escaped into a place nothing deploys.
- Binary name = the bare capability: `tool-forge` → binary `forge`.
- Modules and files: `snake_case`. Types: `PascalCase`. No abbreviations that are not
  already in the glossary.

### Note on the `forge` binary name

`forge` collides with Foundry's Ethereum toolchain binary, which is common on developer
machines and would shadow — or be shadowed by — ours on `PATH`. The name is kept because
it is correct for the job, and the bootstrap runbook (RUN-0001) checks for a pre-existing
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
- Reserved, identical in every service, never versioned: `/health`, `/ready`,
  `/metrics`, `/docs`.

## Databases

- Tables: **plural**, `snake_case` — `users`, `billing_accounts`.
- Columns: `snake_case`. Primary key: `id`. Foreign keys: `<singular>_id`.
- Timestamps: `created_at`, `updated_at`, `deleted_at`, always `timestamptz`, always UTC.
- Migrations: `<seq>_<verb>_<subject>.sql` — `0003_add_invoices_table.sql`.

## Events

`<Subject><PastTenseVerb>` in PascalCase: `UserCreated`, `InvoiceSettled`.
Never present tense, never imperative — an event records what happened, not a command.

## Kubernetes and Helm

- Release name = repository name without the class prefix: `svc-billing` → `billing`.
- Namespace = class: `platform`, `services`.
- Labels: `app.kubernetes.io/name`, `app.kubernetes.io/part-of: paridyum`.

## Environments

Exactly three, named exactly: `dev`, `staging`, `prod`. Not `production`, not `stage`,
not `qa`.

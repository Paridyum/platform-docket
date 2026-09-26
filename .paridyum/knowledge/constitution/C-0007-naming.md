---
id: C-0007
title: Naming rules, extended to abbreviations, environment variables, images and labels
type: constitution
status: accepted
created: 2026-08-09
locked_at: null
supersedes: [C-0005]
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [naming, conventions, api, classes, registry, configuration, containers, kubernetes]
breaking: true
verified_by: [TST-0043]
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
    rationale: >
      The class list lives outside this artifact, so something must guarantee the registry
      stays well-formed and internally unique — otherwise the list is less protected than
      when it was prose.
  - check: config.env_vars_match_scheme
    severity: error
    rationale: >
      The first deployable to set an environment variable sets the precedent for every
      other. Four names for one value is the observable outcome elsewhere, and renaming a
      variable a running service reads is a deployment, not an edit.
  - check: docker.image_name_matches_repo
    severity: error
    rationale: >
      The third derivation from the repository name, after the crate and the Helm release.
      Two already have checks; an image name left to taste is the one that ends up in every
      consumer's pull command.
  - check: helm.declares_recommended_labels
    severity: error
    rationale: >
      A label added after the fact is a rolling update of every workload, and one added to
      spec.selector cannot be added at all. The cheap moment is before the first chart.
---

# Constitution C-0007 — Naming

Supersedes C-0005. **Every naming rule in C-0005 is carried forward unchanged**, with one
exception stated plainly rather than buried: the abbreviation rule has been **moved out of the
Rust section**, because sitting there gave it a stack binding nobody chose and left five of six
stacks with no abbreviation policy at all. That is a widening, not a reversal — nothing legal
under C-0005 becomes illegal.

Four additions: abbreviations, environment variables, container image names, and the full
Kubernetes label set. RFC-0010 is the reasoning; this is the rule.

Names are the cheapest possible coordination mechanism and the most expensive thing to change.
Every rule here is mechanical so that no agent has to exercise taste.

## Why the class list is not here

C-0002 held the repository classes in a locked prose table, so adding one meant superseding the
naming constitution. Six classes were foreseeable — web applications, React Native, Flutter,
generated SDKs, shared packages, workers — and six whole-artifact rewrites of a constitution is
not a plan, it is a prediction that the second one never happens. ADR-0009 records the
consequence rather than predicting it: the previous estate was *"predominantly Python with
TypeScript frontends, none matching C-0002."*

**The grammar is constitutional. The list is data.** C-0005 made that move and this artifact
keeps it.

## What binds what

C-0005's sections bound unevenly, and one of them bound by accident. Stated explicitly here so
the next reader does not have to infer it:

| Section | Binds |
| --- | --- |
| Abbreviations | every stack, every class |
| Repositories | every class in `class-registry.yaml` |
| Rust crates and binaries | the `rust` **stack** only |
| HTTP APIs · Databases · Events | every repository that exposes one |
| Environment variables · Container images · Kubernetes and Helm | classes with `deployable: true` |
| Environments | org-wide |

Three of the four additions bind `deployable: true` from `class-registry.yaml` rather than a
prefix glob. That is deliberate and it is the trap RFC-0004 found in ADR-0003 and ADR-0010:
**a prefix glob in a locked artifact can never be extended to a new prefix.** Written against
the registry flag, none of these needs superseding when a class is added.

## Abbreviations

**This section binds every stack.** In C-0005 it was the last bullet of the Rust section, two
lines above a sentence reading *"these rules bind the `rust` stack"* — so HTTP paths, JSON
fields, database columns, event names and every TypeScript identifier had no abbreviation rule
at all.

- **Permitted** when widely recognised across the industry: `id`, `api`, `http`, `url`, `uri`,
  `db`, `io`, `ip`, `dns`, `tls`, `uuid`, `cpu`, `sql`, `ci`, `sdk`. This list is illustrative,
  not exhaustive — the test is recognition, not membership.
- **Permitted** when qualified by a namespace that removes the ambiguity: `otel.sdk.name` is
  clear where a bare `sdk` might not be.
- **Prohibited** when ambiguous, or invented to save characters: not `usr`, not `cfg`, not
  `mgr`, not `svc` in code, not `acct`.
- The **glossary** remains the authority for Paridyum-specific terms — `tenant`, `account`,
  `operator`, and the nested pair GLO-0004 defines. It is not, and was never populated to be,
  an allowlist of English.

C-0005's wording — *"no abbreviations that are not already in the glossary"* — is superseded.
Read literally it prohibited `id`, while the Databases section three paragraphs later **mandated**
a primary key named `id`. The formulation above is OpenTelemetry's, adopted because it is a rule
rather than a list and therefore needs no maintenance.

**No check enforces this**, and that is deliberate rather than an omission. "Widely recognised"
and "ambiguous" are judgement. A check approximating them with a wordlist would be the
false-confidence failure ADR-0002 exists to prevent — worse than no check, because it would look
like enforcement. This rule is upheld at review.

## Repositories

`<class-prefix><name>`, lowercase, hyphen-separated, singular class prefix.

**The set of valid prefixes is `class-registry.yaml`**, validated against
`class-registry.schema.json`. A prefix not in the registry fails `repo.name_matches_scheme`.
Adding a class is a registry entry plus whatever standard, template and workflow it needs —
never a change to this artifact.

The knowledge base is **sui generis**: no prefix, no class, and deliberately absent from the
registry. It is governed by the `org`-scoped set (ADR-0015), and inventing a class to satisfy
the model is how models acquire vestigial concepts.

The `<name>` is a **noun naming the capability**, never a technology and never an adjective.
`svc-billing`, not `svc-rust-billing`, not `svc-fast-billing`.

Service names are **singular** (`svc-billing`, `svc-user`), because the repository is one
bounded context. Collections are plural only at the API and database layer, below.

### One exception, narrow and written against the registry (ADR-0107, 2026-09-24)

**Where a repository's class declares more than one `stacks` entry in `class-registry.yaml`, the
name carries the stack as a trailing segment — and must, as soon as a second stack implements the
same capability.** `lib-auth-rust` and `lib-auth-typescript`. The segment is the stack name exactly
as the registry declares it, so it is derived rather than chosen.

This is the one place the rule above yields, and only because the reason behind it inverts. For a
service the stack is invisible to every consumer — a caller sends HTTP and cannot tell what
answered — so `svc-rust-billing` encodes nothing. For a **library** the stack is *what may depend on
it at all*: a Rust service cannot import an npm package. The segment names the set of consumers, not
the implementation, and there is no capability-noun that separates two repositories doing the same
job for two toolchains.

**The artifact the repository publishes drops the segment**, because its namespace is already
per-stack: `lib-auth-rust` publishes the crate `paridyum-auth`, `lib-auth-typescript` publishes
`@paridyum/auth`. So the derivation in the section below gains one step and stays mechanical —
**repository name, minus the class prefix, minus the stack segment.** `manifest.yaml`'s `name` keeps
the segment, because it identifies the repository rather than the artifact.

Written against `stacks` rather than the `lib-` prefix, for the reason given above about prefix
globs: `mobile` is the estate's other multi-stack class and is governed by this sentence without it
being rewritten. ADR-0107 has the argument, the rejected alternatives, and the rename it occasioned.

## Rust crates and binaries

- Crate name = repository name with the class prefix replaced by `paridyum-`:
  `lib-core` → `paridyum-core`, `svc-billing` → `paridyum-billing`. Where the repository name
  carries a stack segment under the exception above, the segment is dropped as well:
  `lib-auth-rust` → `paridyum-auth`, because cargo cannot install anything but a crate.
  This keeps crate names globally unambiguous without leaking the repo class into code.
- `lib-*` repositories publish **library targets only**. A `[[bin]]` in a `lib-*` crate is a
  hard CI failure — it means an executable escaped into a place nothing deploys.
- Binary name = the bare capability: `tool-forge` → binary `forge`.
- Modules and files: `snake_case`. Types: `PascalCase`.

These rules bind the `rust` **stack**, not a list of prefixes. A class whose registry entry
names a different stack is not governed by them — which is the point, and is why adding `web`
required no change here.

### Note on the `forge` binary name

`forge` collides with Foundry's Ethereum toolchain binary, which is common on developer machines
and would shadow — or be shadowed by — ours on `PATH`. The name is kept because it is correct for
the job, and the bootstrap runbook (RUN-0001) checks for a pre-existing `forge` and warns. If the
collision ever bites, the fallback is to install the binary as `pdy` and offer `forge` as an
opt-in shell alias; that is a one-line change in `tool-forge`'s `Cargo.toml`, not a rename of
anything else.

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

`<Subject><PastTenseVerb>` in PascalCase: `UserCreated`, `InvoiceSettled`. Never present tense,
never imperative — an event records what happened, not a command.

## Environment variables

**Binds classes with `deployable: true`.** An SDK is a published artifact and a mobile app bakes
its configuration at build time; neither reads a process environment, and a rule that pretended
otherwise would be satisfied by ignoring it.

```
PARIDYUM_<CAPABILITY>_<SETTING>
```

- `SCREAMING_SNAKE_CASE`. Uppercase letters, digits and underscore only — the POSIX portable
  set, so nothing surprising happens in a shell, a `Dockerfile` or a CI runner.
- `PARIDYUM_` on every variable this platform defines. The prefix is what stops a collision with
  the environment a process inherits, and it is why `DATABASE_URL` is not a legal name here.
- `<CAPABILITY>` is the bare capability, matching the Helm release and binary rules:
  `svc-billing` → `PARIDYUM_BILLING_DATABASE_URL`.
- **No service invents an unprefixed name for a value another service also has.** Four names for
  one connection string is the failure this rule exists to prevent.
- Lowercase names are reserved for process-local shell variables and are never Paridyum
  configuration.

The full word rather than `PDY_`: the prefix is typed by tooling far more often than by people,
and an abbreviation on the one string that appears in every deployment buys nothing the
abbreviation rules above would permit. `pdy` remains the contemplated fallback for the *binary*
name, which is a different question.

## Container images

**Binds classes with `deployable: true`.** Derived from the repository name the way the crate
name and Helm release name already are — the third instance of one rule, not a new one:

```
svc-billing  →  ghcr.io/paridyum/billing
```

- Registry namespace `paridyum`, matching the GitHub organisation, because `ghcr.io` derives the
  namespace from the organisation and offers no choice.
- Image name = repository name with the class prefix removed, identical to the Helm release name.
- **Tag and digest policy is not here.** REQ-0004 §2 owns the container baseline including
  `no :latest`, and REQ-0014 owns provenance. A naming constitution restating a deployment
  control would be the two-sources-of-truth failure ADR-0002 exists to prevent.

Accepted cost, recorded because it is a durable identity choice rather than a formatting one: an
organisation rename transfers packages to the new namespace but **breaks every consumer holding
the old reference**, and GitHub says so directly. The mitigation is not renaming the
organisation.

## Kubernetes and Helm

**Binds classes with `deployable: true`.** `mobile` ships to a store and `infra` defines the
cluster rather than running in it (ADR-0037); neither is governed here.

- Release name = repository name without the class prefix: `svc-billing` → `billing`.
- Namespace = class: `platform`, `services`.
- **All six recommended labels**, values derived rather than chosen:

| Label | Value |
| --- | --- |
| `app.kubernetes.io/name` | the release name — `billing` |
| `app.kubernetes.io/instance` | the Helm release instance |
| `app.kubernetes.io/version` | the chart's `appVersion` |
| `app.kubernetes.io/component` | the repository class — `service`, `platform` |
| `app.kubernetes.io/part-of` | `paridyum`, always |
| `app.kubernetes.io/managed-by` | `helm` |

C-0005 named the first and fifth. The other four are what cost attribution, fleet queries and
incident tooling group by, and adding a label later is a rolling update of every workload —
or, if it belongs in `spec.selector`, impossible, because that field is immutable on an
`apps/v1` Deployment and takes a delete and recreate.

The set stays in this artifact rather than moving to a registry, which is the one place this
constitution deliberately does **not** apply its own list-versus-grammar rule. The six are
defined upstream by Kubernetes and are a closed set; a registry for a list that does not grow
is the vestigial concept the sui-generis note above warns about.

## Environments

Exactly three, named exactly: `dev`, `staging`, `prod`. Not `production`, not `stage`, not `qa`.

## Consequences

**Accepted costs**

- **A second supersession of the naming constitution in three days.** C-0005 restated every rule
  from C-0002 and this restates every rule from C-0005. Nothing checks that a restatement is
  faithful, so each one is an unguarded transcription of the most-cited artifact in the
  repository. Batching four additions into one supersession is what keeps the count at two
  rather than five.
- **The abbreviation rule widens what is governed and has no check.** It now binds every stack
  and is upheld by review alone. Under ADR-0030 that makes it a rule with **neither** bridge —
  not defended at the diff, not verified at runtime. Visible rather than smoothed over, on the
  same principle that prints the enforcement backlog.
- **Three new checks on a backlog that is already the larger half**, all owned by `tool-forge`,
  which does not exist. Until it does, these are upheld by compliance.
- **The image rule couples the registry namespace to the GitHub organisation name.** `ghcr.io`
  leaves no alternative, and the consequence is recorded above rather than discovered later.
- **`config.` is a new check namespace.** Cheap under ADR-0014, but it is a twenty-ninth
  namespace and the value of a namespace falls as the count rises.

**Gained**

- Five of six stacks acquire an abbreviation rule they did not have.
- The first deployable cannot invent its own environment variable convention.
- An image name is derivable rather than chosen, closing the last of the three derivations from
  a repository name.
- Four missing Kubernetes labels land before the first chart exists, when the cost is zero.
- Three of the four additions bind registry data rather than prefix globs, so a new repository
  class needs no further supersession of this artifact.

**Rejected alternatives**

**Four separate amendments as each gap became urgent.** Four full restatements, four
transcription risks, and the predictable outcome that the second never happens — the failure
ADR-0009 records as having already come true once.

**A naming-extensions standard, leaving C-0005 untouched.** Naming would have two homes at two
precedence levels, and a standard cannot extend a constitution — it would be legal only where
C-0005 is silent. The abbreviation defect was *inside* C-0005's text and unreachable from a
standard.

**A separate `governance/naming/` tree with its own control IDs.** Rejected on four counts: it
duplicates the glossary, `class-registry.yaml`, STD-0004 and the supersession protocol; its
control-ID namespace competes with `check-registry.yaml`; three of its rules contradicted locked
C-0005 (`production`, `user.created`, unprefixed repository names); and its registries for
products, projects, teams and DNS describe an organisation that does not exist.

## Related

- C-0005 — superseded by this artifact; every rule carried forward, the abbreviation rule widened
- RFC-0010 — the reasoning, the audit of what already existed, and the five open questions this
  artifact resolves
- ADR-0030 — why the abbreviation rule's lack of a check is stated rather than hidden
- ADR-0037 — why `infra` is not bound by the Kubernetes and image sections
- REQ-0001 — consumes the environment variable shape; a predictable name is what a
  secret-detection rule keys on
- REQ-0004 §2 · REQ-0014 — own the container tag, digest and provenance policy this defers to

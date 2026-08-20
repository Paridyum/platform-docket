---
id: STD-0002
title: Documentation standard — what every repository must carry and prove
type: standard
status: locked
created: 2026-08-06
locked_at: 2026-08-06
supersedes: null
superseded_by: null
scope: [platform, service, library, tool, infra, spec]
applies_to: ["*"]
tags: [documentation, rustdoc, runbooks, api]
breaking: true
enforced_by:
  - check: docs.required_files_present
    severity: error
    args: { files: ["README.md", "ARCHITECTURE.md", "CHANGELOG.md", "manifest.yaml"] }
  - check: docs.service_files_present
    severity: error
    args: { applies_to: ["svc-*", "platform-*"], files: ["API.md", "RUNBOOK.md"] }
  - check: rustdoc.missing_docs_denied
    severity: error
    args: { applies_to: ["lib-*"] }
  - check: rustdoc.builds_without_warnings
    severity: error
    args: { flags: ["-D", "warnings", "-D", "rustdoc::broken_intra_doc_links"] }
  - check: docs.doc_tests_pass
    severity: error
  - check: docs.manifest_schema_valid
    severity: error
  - check: docs.no_stale_readme_commands
    severity: warn
    rationale: A README command that no longer exists is worse than no README.
---

# STD-0002 — Documentation

Documentation that is not executed goes stale silently. Every rule here either forces
the docs to run (doc tests, verified commands) or forces them to exist before the code
can merge. Prose nobody checks is the failure mode this standard exists to prevent.

## Required in every repository

| File | Answers | Audience |
| ---- | ------- | -------- |
| `README.md` | What is this, how do I run it in 60 seconds | someone who just cloned it |
| `ARCHITECTURE.md` | How is it built, and why that way | someone about to change it |
| `CHANGELOG.md` | What changed and when | someone debugging a regression |
| `manifest.yaml` | Machine-readable identity and dependencies | tooling, `forge graph`, the catalog |

Services and platform components add:

| File | Answers |
| ---- | ------- |
| `API.md` | The contract — generated from OpenAPI, never hand-maintained |
| `RUNBOOK.md` | It is 3am and this is paging. What do I do? |

Libraries add nothing, but carry `#![deny(missing_docs)]`. A library's documentation
is its rustdoc; a separate `API.md` would immediately diverge.

**Not every folder.** Per-directory README files produce stubs nobody reads. The
repository is the unit of documentation.

## `manifest.yaml`

The machine-readable half. `forge` reads it to build the dependency graph, the service
catalog, and ownership reports.

```yaml
name: billing
class: service                 # service | platform | library | tool | infra | spec
owner: platform                # a team, never a person — people leave
language: rust
runtime: kubernetes
lifecycle: production          # experimental | beta | production | deprecated

exposes:
  http:  { port: 8080, openapi: specs/openapi.yaml }
  events: [InvoiceIssued, InvoiceSettled]

depends:
  services: [auth]
  infra:    [postgres, redis]
  events:   [UserCreated]

slo:
  availability: 99.9
  latency_p99_ms: 250
```

`owner` is a team even when the team is one person. A manifest naming an individual is
wrong the day they change role, and the whole point is that it stays true.

## Rust documentation

```rust
#![deny(missing_docs)]                        // lib-* — mandatory
#![warn(clippy::missing_errors_doc)]
#![warn(clippy::missing_panics_doc)]
```

Every public item documents:

1. **One-line summary.** Imperative, ends with a period. This is what appears in the
   index, so it must stand alone.
2. **`# Errors`** — every variant the caller can receive and what causes it. A
   `Result` whose error cases are undocumented forces the reader into the source.
3. **`# Panics`** — every panic, or state that it cannot panic. ADR-0005 forbids
   panics in libraries, so this section is usually the sentence saying so.
4. **`# Examples`** — a compiling example for every non-trivial public item.

````rust
/// Settles an invoice against a payment intent.
///
/// # Errors
///
/// - [`BillingError::AlreadySettled`] if the invoice is not in `Issued`.
/// - [`BillingError::AmountMismatch`] if the intent total differs from the invoice.
///
/// # Panics
///
/// Never. All failure modes are returned as [`BillingError`].
///
/// # Examples
///
/// ```
/// # use paridyum_billing::{Invoice, PaymentIntent};
/// let invoice = Invoice::issued(1_000);
/// let settled = invoice.settle(&PaymentIntent::of(1_000))?;
/// assert!(settled.is_paid());
/// # Ok::<(), paridyum_billing::BillingError>(())
/// ```
````

**Examples are tests.** This is the reason the rule earns its cost: a doc example that
stops compiling fails CI, so the documentation cannot drift from the API. Prose can
lie; a doc test cannot.

## README

Fixed structure, so a reader lands in the same place in every repository:

```markdown
# billing

One sentence: what capability this owns.

| | |
|-|-|
| Owner | platform |
| Lifecycle | production |
| Health | `/health` `/ready` `/metrics` |
| API | `/docs` |

## Run locally
    docker compose up -d        # postgres, redis
    just dev

## Test
    just test

## Deploy
    Helm chart in `helm/`. See RUNBOOK.md.

## Documentation
    ARCHITECTURE.md · API.md · RUNBOOK.md · CHANGELOG.md
```

Every command in a README is verified in CI. A README whose first command fails is the
worst artifact in the repository, because it burns the goodwill of someone who was
trying to help.

## RUNBOOK

Written for someone woken by a page who did not build this. Not a design document.

Required sections: **Symptoms** (what the alert looks like) · **Immediate mitigation**
(stop the bleeding, before diagnosis) · **Diagnosis** (the actual commands, copy-paste
ready) · **Escalation** (who, and when to stop trying) · **Recovery verification**
(how you know it is over).

Mitigation comes before diagnosis deliberately. At 3am, restoring service beats
understanding it.

## ARCHITECTURE

What a new contributor needs before their first change: the boundary this service
owns, its data model, its dependencies and why each exists, the failure modes and
what happens on each, and — most valuable — **what was deliberately not built, and
why.** That last section prevents the same rejected idea being re-proposed annually.

Include one diagram. Mermaid, in the file, so it renders on GitHub and diffs as text.

## Generated documentation

`API.md` is generated from `specs/openapi.yaml`. Hand-editing it is a CI failure —
the file carries the `@generated` marker, and C-0004 denies edits to generated files.
The contract lives in the spec; the markdown is a rendering.

`forge docs generate` assembles the portal across repositories: rustdoc, manifests,
ADRs, and the dependency graph. Per-repository documentation is the input, which is
why these files are mandatory rather than encouraged.

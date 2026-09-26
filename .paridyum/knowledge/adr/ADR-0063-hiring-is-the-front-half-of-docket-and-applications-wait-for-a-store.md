---
id: ADR-0063
title: Hiring is the front half of docket, and applications wait for a store that exists
type: adr
status: accepted
created: 2026-09-05
locked_at: null
supersedes: null
superseded_by: null
scope: [org, web, console, tool]
applies_to: ["*"]
tags: [hiring, careers, docket, dpdp, personal-data, tenancy, adr-0023, adr-0038, adr-0055, c-0006, std-0019]
breaking: false
enforced_by: []
---

# ADR-0063 — Hiring is the front half of docket, and applications wait for a store that exists

## Status

**Accepted**, 2026-09-05.

## Context

`tool-docket` handles a person from the moment they have an offer: its lifecycle is
`offered → preboarding → active → offboarding → alumni`. **Its first stage is `offered`**, and
`role` is a free-text field somebody types in. So everything before the offer — a job
description, a candidate, an application, an interview — has no home in this estate, and the
job title itself is not modelled anywhere.

That gap was found while looking for the opposite. The premise going in was that a hiring
surface mostly needed wiring up to things that already existed; what exists is the back half of
the pipeline, built carefully, with nothing in front of it.

Two decisions were taken by the operator when the scope was put to them:

1. **Ours now, product later.** Built for Paridyum's own hiring, shaped so it could become
   multi-tenant, not sold to anyone yet. C-0006 anti-goal 6 forbids a second vertical before the
   first has a renewed paying customer; internal tooling is not a vertical, and this ADR does
   not authorise selling it.
2. **Applications are accepted in the product**, not by email.

The second is the expensive one, and the estate cannot honour it today.

## The blocking fact

**There is no persistence in the cluster.** No StorageClass, no PersistentVolumeClaim, no
StatefulSet, no database. `paridyum-web` runs two stateless pods and that is all that has ever
run there.

An application is name, email, a CV, and whatever someone chose to write about themselves. It is
personal data in the DPDP Act's sense, and this estate already holds itself to a standard for
that: `tool-docket` gitignores `people/` because git history cannot be erased and an
unerasable record makes the erasure right unhonourable; every field a person record may hold is
declared in `registry/fields.yaml` with a stated purpose, and an undeclared field is rejected.

Accepting applications into a system with no durable store, no backup, no encryption at rest and
no erasure path would be collecting personal data we cannot protect, cannot return and cannot
delete. **That is not a smaller version of the feature. It is the feature done in the one way
this estate has already decided is unacceptable.**

## Decision

### 1. The work splits at the personal-data line, and the line is where the store is

**Phase A — job descriptions.** A JD is not personal data. It is content Paridyum publishes
about itself, and it needs no store, no account and no encryption. It ships now.

**Phase B — applications.** Blocked on a store, and the block is named rather than worked
around: a StorageClass, PostgreSQL, backup and restore rehearsed, and the session-context
pattern ADR-0038 already specifies. Nothing in Phase A may be shaped to make Phase B feel
closer than it is.

### 2. A job description is declared data, and the repository is the admin surface for Phase A

JDs live as declared files, reviewed and merged like every other registry in this estate. That
is not a placeholder for a real admin UI — it is the same mechanism `address-registry.yaml`,
`fields.yaml` and `product-registry.yaml` already use, and it gives a JD a diff, an author, a
review and a history for free.

**An admin UI is Phase B's, not Phase A's**, because an admin UI that writes to a repository is
a worse version of a pull request, and one that writes to a database needs the database.

### 3. The public surface is `web-paridyum`; anything operator-facing is a console

ADR-0055's class boundary decides this and it is not negotiable by convenience: `web` is served
from the cluster, `console` runs where the operator is. A careers page is public and belongs to
the deployed web surface. Reviewing candidates is operator work and belongs to a console, next
to `console-docket`, not served to the internet.

### 4. `role` becomes a reference, and that is the seam docket is joined on

Today `people/*.yaml` carries `role: FILL-ME` as free text. A JD has an identifier; a person's
role should name it. That single change is what makes "who did we hire for this posting" a
question with an answer, and it is the whole integration — Phase A does not perform it, but the
JD identifier is designed to be referenced.

### 5. Shaped for tenancy, not built for it

ADR-0023 and ADR-0024 already fix how multi-tenant data works here: row-level in one database,
two columns, session context set per transaction and every accessor failing closed. Phase B
inherits that rather than inventing a scheme, which is what "product later" costs — **a design
that would be wrong for a tenant is not written now**, and no tenancy machinery is built for a
tenant who does not exist.

## What this does not decide

**Which database, or how storage is provisioned.** ADR-0020 puts workloads on Kubernetes and
ADR-0023 says one shared database; neither says what runs it here, and a single-node hypervisor
with no StorageClass makes that a real decision rather than a default. It is the next artifact
and it blocks Phase B.

**Whether applications are ever sold as a product.** C-0006 anti-goal 6 stands.

**Retention.** How long an unsuccessful application is kept is a DPDP purpose-limitation
decision, and it belongs with the field registry Phase B needs, not here.

## Alternatives considered

- **Accept applications by email now, in-app later.** What was offered and not what was chosen.
  It ships this week and stores no personal data — but the operator's call was in-app, and the
  honest consequence of that is Phase B waits rather than being approximated.

- **Applications into a file, or into git.** Immediate, and it is precisely the shape
  `tool-docket` gitignores `people/` to prevent. An unerasable record of a person who asked to
  be erased is worse than not collecting it.

- **A managed third-party ATS.** Cheap, works today, and puts candidate data in a vendor under
  REQ-0012 assessment with no residency guarantee — while the argument this company makes to its
  own customers is that isolation is provable and data stays where it was promised. Using an ATS
  we would not accept from a supplier is a claim we could not repeat.

- **JDs in a database from the start**, so Phase A and B share a store. Cleaner in the end state,
  and it makes content that is not personal data wait for infrastructure that only personal data
  requires. The repository is a better home for reviewed public text regardless.

## Consequences

**Accepted costs.**

- **Phase B is blocked and will feel arbitrary**, because the form is the easy part and the
  store is not. That is the trade every honest personal-data system makes.
- **Publishing a JD needs a merge.** A pull request to post a job is friction, and it buys review
  of public claims about pay, duration and what the work is.
- **The careers page will exist before applications do**, so it must say how to apply in the
  meantime and must not render a form that does not work.

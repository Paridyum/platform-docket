---
id: ADR-0055
title: An operator console runs where the operator is, and is a class of its own
type: adr
status: accepted
created: 2026-08-21
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [classes, naming, web, typescript, deployment, adr-0020, adr-0046, c-0005]
breaking: false
enforced_by: []
---

# ADR-0055 — An operator console runs where the operator is

## Status

**ACCEPTED** as of 2026-08-21. Not locked. What is mechanically checkable here
is already checked by `knowledge.class_registry_valid` and
`docs.manifest_schema_valid` once the class exists; the judgement this ADR adds
— whether a given interface faces operators or tenants — is not something a
check can make, and ADR-0002 reserves `locked` for what can be rejected
mechanically.

## Context

Paridyum's HR staff need a graphical interface to `tool-docket`, run on their
own machines rather than in a cluster. Two classes in `class-registry.yaml`
come close and neither fits; the registry is the count.

**`web`** is *"A user-facing web application, served from the cluster"*, with
`stacks: [typescript]` and `deployable: true`. That last field is the problem,
because **ADR-0020 is `locked`, scoped `org`, and applies to `["*"]`**:
Kubernetes is the deployment target for every class declared deployable. A
`web-` repository therefore owes probes, resource limits and a Helm chart. An
interface that runs on an HR workstation owes none of those, and there is no
cluster to owe them to.

**`tool`** is *"Developer-facing executables"*, `stacks: [rust]`,
`deployable: false`. Everything fits except who it faces. HR is not a developer,
and the class says what a repository IS — stretching "developer-facing" to mean
"anyone internal" would make the word carry no information.

So the estate can describe software that runs in the cluster and software that
runs on a developer's machine, and has no word for software that runs on an
operator's machine. That gap is not hypothetical: it blocked a decision.

**The competing pressure, stated honestly.** A class is not free. `class`
generates the manifest enum, `prefix` generates the naming rule, `scope`
generates the value artifacts bind against, and every existing artifact's scope
list is now a list that could have included this class and does not.
`artifact-registry.yaml` records this repository having already paid for a type
declared before its first instance. Adding a class for one repository is close
to that mistake.

Two things distinguish this from it. The first instance exists and is specified
(`tool-docket`'s served HR platform). And `class-registry.yaml`'s own header
argues for exactly this: *"The consequence of not fixing that is on record —
ADR-0009 describes the previous estate as 'predominantly Python with TypeScript
frontends, none matching C-0002'. The stack arrived without a class because the
ceremony was too expensive."*

## Decision

**1. A new class, `console`, prefix `console-`, scope `console`.**

    class:      console
    prefix:     console-
    scope:      console
    stacks:     [typescript]
    deployable: false
    ci:         true

**2. The test for the class is who reaches it and from where.** A `console-`
repository is an interface for **Paridyum's own operators**, run where the
operator is rather than served from the cluster. If tenant traffic can reach it,
it is not a console: C-0008 makes it a product and it belongs in `web-`.

**3. `deployable: false`, and that is the whole point.** ADR-0020 does not bind
it. A console is not deployed; it is run. Nothing here weakens ADR-0020 for the
classes it does bind.

**4. `stacks: [typescript]`, and it adopts ADR-0046's decisions by reference.**
Next.js as the framework, pnpm as the package manager, strict TypeScript, and
RFC-0003's supply-chain rules including lifecycle install scripts disabled.
ADR-0046's `applies_to` does not list `console-*` and is not amended here; this
ADR adopts its content for this class rather than widening a decision written
about something else. **The RSC server/client trust boundary ADR-0046 names as
having "no Rust analogue" applies in full**, and it remains undocumented as an
ANTI- artifact, which is a gap this decision inherits rather than creates.

**5. A console has no identity system of its own.** ADR-0047 binds every
repository, so a bespoke login is not available. A console authenticates by
being unreachable without existing credentials — bound to loopback, reached over
an SSH tunnel, so the operating system is the identity. When
`platform-identity` exists, OIDC against the staff instance ADR-0047 point 1
describes is added beside that, never in place of it (ADR-0054).

**6. A console may not hold business rules.** It calls a `tool-` or a
`platform-` for them. A console with a rule in it is ANTI-0004 with a different
file extension: the rule then exists in two places, and the copy in the
interface is the one nobody tests.

## Consequences

**Accepted costs.**

- **Another class**, and every artifact scope list in the repository is now a
  list that could name it and does not. Whether an existing artifact should
  bind `console` is a question each one now has, and this ADR answers it for
  none of them.
- **Two schema enums must be edited by hand** — `knowledge.schema.json`'s
  `scope` and `manifest.schema.json`'s `class`. `class-registry.yaml`'s header
  says it *generates* them; no generator exists, so the header describes an
  intent and the edit is manual. That is a C-0004 tension: hand-editing what is
  described as generated output. Recorded rather than resolved, because writing
  the generator is tool-forge's work and not this decision's.
- **A second stack for a second reason.** TypeScript already exists for `web`
  and `mobile`; `console` adds a third consumer of a supply chain RFC-0003
  specifies and nothing has yet implemented.
- **No template.** RFC-0028 is `proposed` and `templates/` holds `infra-ci` and
  `rust-service` only, so the first `console-` repository is scaffolded by hand
  and becomes the de facto template without having been reviewed as one.
- **The judgement is contestable.** "Faces operators" and "faces tenants" are
  clear at the extremes and not in the middle. A support tool a franchisee logs
  into is genuinely arguable, and point 2 decides it by reachability rather than
  by intent because reachability is the fact you can check.

**Gained.**

- An interface for internal operators is describable, and is not forced to
  either claim it deploys to a cluster it will never see or claim its users are
  developers.
- ADR-0020 stays literal. Its `deployable` test keeps meaning what it says,
  rather than acquiring an exception for things that are deployable-but-not-really.
- The estate has one web stack rather than two. Without this class, an internal
  interface would have been built in whatever its host repository was written
  in, which for `tool-docket` means a second way of producing HTML.

**Rejected alternatives.**

- **Stretch `tool` to cover it.** Cheapest, and it costs the word
  "developer-facing" its meaning. It also forces the interface into `stacks:
  [rust]`, producing the second web technology this decision exists to avoid.
- **Stretch `web` and set `deployable: false` for some `web-` repositories.**
  Makes `deployable` a per-repository property rather than a class property,
  which is where ADR-0020's test lives. A locked artifact's test should not
  become conditional to accommodate a new case.
- **Wait for the cluster and build it as a `web-` app.** Correct eventually and
  no use now. It also assumes an operator console should be cluster-served,
  which point 2 argues against on its own merits: an interface to records held
  on one machine is better run beside them.
- **No interface — the CLI is enough.** Genuinely considered. It is enough for
  the person who built it and not for the HR staff who did not, and "learn the
  terminal" is a cost imposed on them to avoid a decision here.
## Clarification, 2026-08-21

**`console` means Paridyum's own operators, not the HR department.**

The first console was `console-docket`, which HR uses, and the wording here leaned
on that example enough that the next case — a testing dashboard for engineers
(RFC-0034) — read as though it might need a different class.

It does not, and the test in this ADR already said so: the class turns on
**reachability**, not on who the operator is. If tenant traffic can reach it, it is
not a console and C-0008 makes it a product. Nothing in that test mentions HR.

The distinction that was nearly drawn — engineers are "developer-facing", which is
`tool` — is the wrong cut. `tool` is a Rust binary a developer runs on their own
machine; a console is a served interface an operator reaches, and an engineer
reading a dashboard is an operator. Splitting on the operator's job title would
mean a second class describing the same shape, and duplicating the CI workflow, the
supply-chain posture and the access model to serve the duplicate.


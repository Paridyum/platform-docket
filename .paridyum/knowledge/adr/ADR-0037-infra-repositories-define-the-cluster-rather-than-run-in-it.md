---
id: ADR-0037
title: Infrastructure repositories define the cluster rather than run in it, so they are not deployable
type: adr
status: accepted
created: 2026-08-08
locked_at: null
supersedes: null
superseded_by: null
scope: [org, infra]
applies_to: ["infra-*"]
tags: [class-registry, kubernetes, ci, bootstrap, adr-0020, adr-0021]
breaking: false
enforced_by:
  - check: knowledge.class_registry_valid
    severity: error
    rationale: >
      The correction is registry data, which is the mechanism class-registry.yaml exists to
      provide — `deployable` and `ci` replaced the prefix globs ADR-0003 and ADR-0010 carried
      precisely so a binding could be changed without superseding a locked artifact. The check
      that guarantees the registry stays well-formed is therefore the one that guarantees this.
---

# ADR-0037 — Infrastructure repositories are not deployable

## Status

**ACCEPTED** as of 2026-08-08.

## Context

Two locked artifacts, taken together, forbid the first repository this organisation needs to
create.

**ADR-0021 requires `Paridyum/infra-ci` by name.** Its decision is that CI *is* a reusable
workflow, called and never copied, and it fixes the caller verbatim:

```yaml
uses: Paridyum/infra-ci/.github/workflows/rust-ci.yml@v1
```

That repository takes the `infra-` prefix, which `class-registry.yaml` maps to `class: infra`.

**`class: infra` is `deployable: true`, and ADR-0020 binds every such class.** ADR-0020's own
words: *"This binds every class declared `deployable: true` in `class-registry.yaml`, and nothing
else"* — and what it binds a repository to is a multi-stage `Dockerfile`, a `helm/` chart with
Deployment, Service, Ingress, HPA and ServiceAccount, liveness and readiness probes, resource
limits, and `terraform/`.

So a repository containing nothing but reusable GitHub Actions YAML would be required to ship a
container image and a Helm chart, and to declare a Terraform stack it does not use. Nobody
decided that. It is the interaction of two decisions that were each correct in isolation.

**The premise is wrong for every `infra-` repository, not just this one.** ADR-0020 line 88 places
cluster-level infrastructure in `infra-kubernetes` and `infra-terraform`, and line 109 states the
sequencing: *"The roadmap must sequence `infra-kubernetes` before any deployable repository goes
live."* That sentence treats `infra-kubernetes` as the **prerequisite for** deployable
repositories rather than as one of them. An infra repository *describes* the cluster. It does not
run inside it, so probes and resource limits are meaningless for it — the same argument
`class-registry.yaml` already accepted for `mobile`, where *"`deployable: false` is the important
value and it is not an oversight."*

`class-registry.yaml` anticipated this correction in the `infra` entry itself, about the adjacent
field: *"A real gap, preserved rather than fixed during supersession because changing behaviour
inside one is how detail gets lost. Correcting it is a one-line change here and a separate
decision."* This is that separate decision, for `deployable` rather than `ci`.

**Nothing reads the field today.** `deployable` appears in no script, schema or workflow — only in
the registry. The checks that would act on it (`repo.has_helm_chart`, the `helm.*` and `docker.*`
namespaces) are all owned by `tool-forge`, which does not exist. So this correction is free to
make now and expensive to make after those checks are built and a real `infra-kubernetes` is
failing them.

## Decision

**1. `class: infra` becomes `deployable: false`.** ADR-0020 therefore does not bind `infra-*`.
An infrastructure repository is the definition of the runtime, not a workload in it.

**2. `class: infra` accepts `stacks: [terraform, none]`.** `infra-ci` holds workflow YAML and
shell, `infra-kubernetes` holds charts and manifests, and neither is Terraform. `none` is already
the declared value for `spec` and `template`, so no new vocabulary is introduced.

**3. No new class is created.** `infra-ci` is an infrastructure repository under the existing
prefix, which is what ADR-0021 already assumed when it wrote the name.

**4. Deployment governance for `infra-*` is now absent, and that is stated rather than implied.**
ADR-0020 was the only artifact providing it. How a Terraform plan is reviewed and applied, and how
a cluster change is rolled back, are unowned — the same shape as the gap `class-registry.yaml`
records for `mobile`, and it belongs to REQ-0004.

## Consequences

**Accepted costs**

1. **`infra-*` loses its only release governance**, and gains nothing in its place. A Terraform
   apply against a live cluster is a higher-consequence action than a Helm upgrade, and after this
   decision no artifact governs it. Decision 4 names it; REQ-0004 owns it; today it is genuinely
   ungoverned and saying so is the point.

2. **A repository can now hold cluster configuration with no chart, no probes and no limits, and
   pass every check.** That is correct for `infra-ci` and unproven for `infra-kubernetes`, which
   does not exist yet and may want some of ADR-0020's discipline back in a different form.

3. **`stacks: [terraform, none]` makes the class heterogeneous.** Two repositories under one
   prefix with different toolchains and no shared standard between them.

4. **This changes what a locked ADR binds, without superseding it.** Legitimate — it is the
   mechanism the registry exists for — and it means ADR-0020's binding set can now be read only by
   reading two files. The alternative was superseding a locked ADR to change one boolean.

**Gained**

- `infra-ci` becomes creatable, which unblocks the first step of the bootstrap sequence: every
  Rust repository's CI calls a workflow that cannot exist today.
- `infra-kubernetes` and `infra-terraform` stop being required to deploy themselves.
- The correction lands while the field is inert. After `tool-forge` implements `repo.has_helm_chart`
  it would be a live failure on a production cluster repository.

**Rejected alternatives**

- **Add a new `ci` class with its own prefix.** My first proposal, and wrong. ADR-0021 is locked
  and names `Paridyum/infra-ci`, so a `ci-` prefix would contradict it, and a second class sharing
  the `infra-` prefix is not expressible — `repo.name_matches_scheme` maps prefix to class. It also
  leaves the underlying modelling error in place for `infra-kubernetes` and `infra-terraform`.

- **Supersede ADR-0020 to exclude `infra-`.** Rewrites a locked ADR to change one boolean that the
  registry was built to hold. `class-registry.yaml`'s header rejects exactly this: *"a prefix glob
  in a locked artifact can never be extended to a new prefix."*

- **Grant `infra-ci` an exception.** Exceptions accumulate against a rule nobody revisits — the
  failure ADR-0026 names about its own budget. The rule is wrong here, not inconvenient.

- **Leave it and hand-wave the first repository through.** The checks are unimplemented, so this
  would work and would be discovered by `tool-forge` later, against a live cluster repository.

## Related

- ADR-0021 — requires `Paridyum/infra-ci` by name; unblocked by this
- ADR-0020 — Kubernetes as deployment target; its binding set changes, its text does not
- C-0005, class-registry.yaml — classes are registry data so a binding can change without a
  supersession, which is the whole mechanism this uses
- REQ-0004 — runtime and operations governance, which now owns the gap decision 4 names
- ADR-0036 — the same shape yesterday: a conflict resolved by noticing which value was actually
  declared, rather than by rewriting a locked artifact

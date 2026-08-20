---
id: ADR-0020
title: Kubernetes is the deployment target for every class declared deployable
type: adr
status: locked
created: 2026-08-07
locked_at: 2026-08-07
supersedes: [ADR-0003]
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [kubernetes, deployment, helm, docker, terraform, registry-driven]
breaking: false
enforced_by:
  - check: repo.has_helm_chart
    severity: error
    args: { path: "helm/" }
  - check: helm.chart_lints
    severity: error
  - check: helm.declares_probes
    severity: error
    args: { required: ["livenessProbe", "readinessProbe"] }
    rationale: A workload without probes cannot be safely rolled by Kubernetes.
  - check: helm.declares_resource_limits
    severity: error
    rationale: An unbounded pod can evict its neighbours on a small cluster.
  - check: docker.image_is_distroless_or_slim
    severity: warn
  - check: terraform.fmt_and_validate
    severity: error
---

# ADR-0020 — Kubernetes is the deployment target for every class declared deployable

## Status

**LOCKED** as of 2026-08-07. Supersedes ADR-0003.

**The decision is unchanged.** Kubernetes remains the deployment target, and every
generated artefact ADR-0003 named is still generated. What changes is *which repositories
this binds*: `applies_to: ["platform-*", "svc-*", "infra-*"]` becomes **every class whose
`class-registry.yaml` entry declares `deployable: true`**.

That is provably the same set today — `platform`, `service` and `infra` are exactly the
classes marked deployable — and it is the reason this supersession should be the last.
A prefix glob in a locked artifact can never be extended to a new prefix, so under
ADR-0003 every future deployable class would have cost another supersession. Under this
one, `web` was added by a registry entry and `mobile` will be too.

## Context

Every Paridyum service needs a production runtime. The realistic options were a managed
PaaS (Fly.io, Railway, Render), Kubernetes, or a single VM running Docker Compose.

The trade was put explicitly: Kubernetes from day one means several weeks of
infrastructure work — cluster, ingress, secrets, observability stack, Terraform state —
ahead of the first service serving traffic, and every generated Helm template needs
maintaining before it is load-bearing. A PaaS defers all of that.

**Kubernetes was chosen with that cost understood and accepted.**

The reasoning that carries it: Paridyum is an internal developer platform, not a single
product. `tool-forge`'s value proposition is that every service arrives with a complete,
identical, production-shaped deployment. Generating Fly.io configs now and Helm charts at
v0.4 would mean writing the deployment layer twice and migrating every service that already
exists. The target that services are scaffolded *for* is the most expensive thing to change
later, which is precisely the kind of decision that belongs in a locked ADR made
deliberately at the start.

## Decision

Kubernetes is the deployment target. **This binds every class declared `deployable: true`
in `class-registry.yaml`**, and nothing else.

`forge new` generates, from day one:

- a multi-stage `Dockerfile` producing a slim or distroless runtime image. For the `rust`
  stack this uses `cargo-chef` for dependency-layer caching; other stacks use the
  equivalent for their toolchain, which is a stack concern rather than a deployment one
- `helm/` — `Chart.yaml`, `values.yaml`, deployment, service, ingress, HPA, ServiceAccount,
  and ConfigMap/Secret wiring
- liveness, readiness and startup probes bound to the standard health endpoints fixed by
  the observability standard
- resource requests and limits, with no unbounded default
- `terraform/` — a module skeleton for the service's own cloud resources (database, bucket,
  queue), not for the cluster itself

Cluster-level infrastructure lives in `infra-kubernetes` and `infra-terraform`, versioned
separately from services, per ADR-0001.

`docker-compose.yml` is still generated, scoped strictly to **local development** —
Postgres, Redis, and the service's dependencies. It is never a deployment path.

### The Dockerfile is stack-shaped; everything else is not

ADR-0003 said `cargo-chef` because Rust was the only stack. Generalised: the *build* stage
belongs to the stack, the *runtime* contract belongs here. Probes, limits, the Helm chart
shape and the image being slim or distroless are identical whatever produced the binary.

This matters for the `web` class specifically. A Next.js application self-hosts via
`output: "standalone"` and needs `sharp` for image optimisation — both stack concerns for
RFC-0003 to settle — but its probes, limits and chart are governed here, unchanged.

## Consequences

**Accepted costs**

- Meaningful infrastructure work precedes the first service in production. The roadmap must
  sequence `infra-kubernetes` before any deployable repository goes live.
- Helm templates are maintained from the moment they are generated, including for services
  that are not yet deployed anywhere.
- Local development is heavier. Mitigated by keeping `docker compose up` — not a local
  cluster — the default inner loop, with `kind`/`k3d` reserved for verifying charts.
- `terraform` is not installed on the current workstation; the bootstrap runbook adds it
  (RUN-0001).
- **`deployable` is now a registry boolean rather than a prose list**, so marking a class
  deployable is a one-line change. That is the intent and it is also the risk: the
  deliberation a supersession forced is gone, and nothing checks that a class marked
  deployable actually has a chart until a repository of that class exists.

**Gained**

- One deployment shape, learned once, identical across every service.
- Probes, limits and observability are structurally impossible to omit — they are generated
  and gate-checked, not remembered.
- No second deployment migration later.
- **A new deployable class costs a registry entry**, where under ADR-0003 it cost a
  supersession of this artifact.

**Deferred, not decided here**

- Which cloud and which managed Kubernetes offering (EKS / GKE / DOKS / Hetzner + k3s).
  This ADR fixes the *interface* — Helm charts against a conformant cluster — precisely so
  the provider stays swappable. That choice gets its own ADR before `infra-kubernetes` is
  written.
- **ISR and other multi-replica cache coherence.** A Next.js `web` repository defaults to a
  per-instance cache, so with more than one pod users see different content depending on
  which answers. That is a stack concern and RFC-0003 records it; it is named here because
  it is invisible until it is a production incident.

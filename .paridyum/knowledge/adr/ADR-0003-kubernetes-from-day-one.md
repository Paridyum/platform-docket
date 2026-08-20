---
id: ADR-0003
title: Kubernetes is the deployment target from day one
type: adr
status: superseded
created: 2026-08-06
locked_at: 2026-08-06
supersedes: null
superseded_by: ADR-0020
scope: [platform, service, infra]
applies_to: ["platform-*", "svc-*", "infra-*"]
tags: [deployment, kubernetes, helm, terraform]
breaking: true
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

# ADR-0003 — Kubernetes is the deployment target from day one

## Status

**LOCKED** as of 2026-08-06.

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
identical, production-shaped deployment. Generating Fly.io configs now and Helm charts
at v0.4 would mean writing the deployment layer twice and migrating every service that
already exists. The target that services are scaffolded *for* is the most expensive
thing to change later, which is precisely the kind of decision that belongs in a
locked ADR made deliberately at the start.

## Decision

Kubernetes is the deployment target. `forge new` generates, from day one:

- a multi-stage `Dockerfile` using `cargo-chef` for dependency-layer caching, producing
  a slim or distroless runtime image
- `helm/` — `Chart.yaml`, `values.yaml`, deployment, service, ingress, HPA,
  ServiceAccount, and ConfigMap/Secret wiring
- liveness, readiness and startup probes bound to the standard health endpoints
  fixed by the observability standard
- resource requests and limits, with no unbounded default
- `terraform/` — a module skeleton for the service's own cloud resources (database,
  bucket, queue), not for the cluster itself

Cluster-level infrastructure lives in `infra-kubernetes` and `infra-terraform`, versioned
separately from services, per ADR-0001.

`docker-compose.yml` is still generated, scoped strictly to **local development** —
Postgres, Redis, and the service's dependencies. It is never a deployment path.

## Consequences

**Accepted costs**

- Meaningful infrastructure work precedes the first service in production. The
  roadmap must sequence `infra-kubernetes` before any `svc-*` goes live.
- Helm templates are maintained from the moment they are generated, including for
  services that are not yet deployed anywhere.
- Local development is heavier. Mitigated by keeping `docker compose up` — not a local
  cluster — the default inner loop, with `kind`/`k3d` reserved for verifying charts.
- `terraform` is not installed on the current workstation; the bootstrap runbook adds
  it (RUN-0001).

**Gained**

- One deployment shape, learned once, identical across every service.
- Probes, limits and observability are structurally impossible to omit — they are
  generated and gate-checked, not remembered.
- No second deployment migration later.

**Deferred, not decided here**

- Which cloud and which managed Kubernetes offering (EKS / GKE / DOKS / Hetzner + k3s).
  This ADR fixes the *interface* — Helm charts against a conformant cluster — precisely
  so the provider stays swappable. That choice gets its own ADR before
  `infra-kubernetes` is written.

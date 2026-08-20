---
id: REQ-0004
title: A deployed service is tiered, observable, reversible and bounded in cost
type: requirement
status: deferred
created: 2026-08-07
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, infra]
applies_to: ["*"]
tags: [runtime, deployment, observability, reliability, cost, progressive-delivery]
breaking: false
trigger: >
  The first service is deployed to a cluster and serves a request from anything other
  than a test.
review_date: 2027-02-01
enforced_by: []
---

# REQ-0004 — A deployed service is tiered, observable, reversible and bounded in cost

## Status

**DEFERRED** as of 2026-08-07. There is no cluster, no deployed service and no traffic.

## Requirement

**Before a service takes real traffic it shall declare its risk tier, expose the signals
needed to know it is healthy, be reversible without a code change, and carry a cost
ceiling — and the rigour applied to it shall scale with its tier rather than being
uniform.**

## Numbered items

1. **Risk tiering.** Every service is Tier 0–3. Approval depth, coverage thresholds,
   rollout strategy, on-call and review requirements scale with tier. **This is the single
   most important item here**, and the reason is in the source research: applying identical
   rigour everywhere is how rules get ignored. An internal script and the payments service
   must not face the same gate. `manifest.yaml`'s `lifecycle` field is the closest existing
   thing and is about maturity, not blast radius — they are different axes.
2. **Container baseline.** Distroless or slim base, no `:latest`, non-root, read-only
   filesystem, dropped capabilities, CPU and memory requests *and* limits. ADR-0003 already
   declares `helm.declares_resource_limits` and `docker.image_is_distroless_or_slim`.
3. **Image provenance.** Signed images, admission control rejecting unsigned. Vulnerability
   scan as a deploy gate.
4. **IaC only, no ClickOps.** Zero manual console changes in production. `plan` reviewed,
   drift detected, no `-auto-approve`. Policy-as-code gates. ADR-0003 declares
   `terraform.fmt_and_validate`; this is the governance around it.
5. **Environment promotion.** `dev → staging → prod`, no skipping, config schema-validated,
   fail fast on missing config. Environment parity stated, and its deliberate exceptions.
6. **Progressive delivery by tier.** Canary or percentage rollout for Tier 0–1, automated
   rollback on error-rate triggers. Not required for Tier 3.
7. **Feature flags with expiry.** Owner, default state, kill switch, and a **mandatory
   expiry date**. Stale flags are among the largest sources of silent complexity, because
   each one doubles the paths under test and nothing ever forces their removal.
8. **Observability baseline.** Structured logs with a correlation id propagated across
   services, RED/USE metrics, one dashboard per service, trace sampling policy. Metric
   cardinality limits — cardinality is where observability bills detonate.
9. **Alerts that are actionable.** Every alert has an owner and a runbook link. An alert
   nobody acts on trains people to ignore the ones that matter, which is worse than no
   alert.
10. **SLOs and error budgets.** Per service, with an exhausted budget blocking feature
    work. Only meaningful once there is traffic to measure.
11. **Incidents.** Severity definitions, escalation, blameless postmortems above SevN with
    action items that are actually scheduled.
12. **Backup and restore.** RPO/RTO stated, and **restores drilled**. An unverified backup
    is a belief.
13. **Cost.** Per-service budget with an owner, cost visible in the PR that causes it,
    idle-resource reaping, anomaly alerts. Token and rate budgets for anything calling an
    LLM.
14. **Change windows.** Freeze periods, deploy hours, and an emergency path with mandatory
    retrospective review.

## Why this is not startable today

Every item measures, limits or reverses something in production. There is no production.
Item 1 is the exception in principle — tiering could be defined now — but tiers with one
repository and zero services would be a taxonomy with no members, and the tier boundaries
would be guesses that harden before any evidence arrives.

## Related

- ADR-0003 — Kubernetes from day one; declares several of these checks already
- ADR-0010 — CI/CD architecture and release flow
- ADR-0004 — plan limits: several controls here need a paid tier
- REQ-0001 — secret delivery, a prerequisite for most of this
- ANTI-0005 — compensating paths, the reliability concern at design time

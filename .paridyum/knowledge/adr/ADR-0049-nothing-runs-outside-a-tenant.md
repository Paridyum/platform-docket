---
id: ADR-0049
title: Nothing in the runtime resolves without a tenant, and no capability ships without a buyer
type: adr
status: superseded
created: 2026-08-17
locked_at: 2026-08-17
supersedes: null
superseded_by: ADR-0116
scope: [platform, service, library, spec]
applies_to: ["platform-*", "svc-*", "lib-*", "spec-*"]
tags: [tenancy, dogfooding, resale, self-preferencing, request-path, manifest]
breaking: true
enforced_by:
  - check: domain.no_tenantless_request_path
    severity: error
    rationale: An operator bypass never looks like an exception at the time; it looks like the obvious way to get the first thing working.
  - check: domain.no_operator_only_capability
    severity: error
  - check: domain.operator_tenant_holds_no_extra_grant
    severity: error
    rationale: GLO-0005 makes scope a grant; this is the one principal for whom nobody would think to check.
---

# ADR-0049 — Nothing runs outside a tenant

## Status

**LOCKED** as of 2026-08-17. C-0008 states the principle; this carries the rules a check can
reject.

⚠️ **All three named checks are declared and unbuilt.** This artifact therefore joins the backlog
`check-knowledge.sh` prints on every run as *"enforced by nothing that runs today"*. That was
accepted deliberately: the rule binds judgement now, and ADR-0014's declared-versus-implemented
split exists so that stating a commitment before building it is honest rather than a lie. **Until
those checks run, this is enforced by your compliance.** Behave as though they were live.

## Context

BET-0005 commits Paridyum to operating the catalogue it franchises, and the decision recorded there
declines every commercial boundary — no vertical exclusivity, no territory, no exit. RFC-0022 D1
and D5 are the whole remaining defence, and both rest on an assumption nothing had written down:
**that Paridyum has no mechanism its tenants lack.**

That assumption is false by default. The cheapest way to build any first capability is to build the
half Paridyum needs and defer the tenant-facing half, and every such shortcut is invisible
afterwards — it does not look like a bypass, it looks like the code. C-0001 principle 6 names the
shape (*"a service that needs an exception is a signal that the platform is missing a capability"*)
and an operator bypass is the most attractive exception available, taken before anyone thinks to
record it.

Recording it after the fact is not possible. A capability shipped without a tenant path is not
missing a feature; it is shaped around one consumer, and adding the second consumer later is the
rewrite C-0001 principle 1 exists to avoid.

## Decision

**1. No runtime operation resolves without a tenant context.** Every request path in a
`platform-*`, `svc-*` or catalogue product derives `tenant_id` from the verified session, per
ADR-0038, and refuses when it cannot. There is no internal mode, no admin path, and no
configuration that disables the requirement. `domain.no_tenantless_request_path`.

**2. Paridyum's own use is a tenant.** Provisioned through ADR-0039's saga, stored in the same
silos, subject to the same entitlement and quota, and **granted no scope a customer could not
also be granted**. GLO-0005 already makes operator scope a grant rather than an absence; this names
the one principal for whom nobody would otherwise think to check.
`domain.operator_tenant_holds_no_extra_grant`.

**3. No deployable capability ships without a declared tenant-facing consumer.** A service's
`manifest.yaml` names how a tenant buys what it does. A capability that cannot answer *which tenant
would pay for this* is not a platform capability — it is either a product nobody has scoped or an
internal tool misfiled. `domain.no_operator_only_capability`.

**4. The boundary is the request path.** `tool-*` and `infra-*` are exempt: they run at build time
or in a pipeline, no tenant traffic reaches them, and ADR-0042 already classes such things
`deployable: false`. **If tenant data can reach it, it is in a request path and this ADR binds** —
including anything shaped like a support console or an internal dashboard, which is the case most
likely to be argued and most likely to be wrong.

**5. A new runtime service names its buyer before it is created.** ADR-0019 already requires
decomposition before implementation and escalates repository creation to a human; this adds one
question to that gate — *which tenant buys this, and through what interface?* An answer of "only
us, for now" is a decision to build a `tool-` instead.

## Consequences

**Accepted costs**

1. **The first capability is slower by the width of a tenant model.** Nothing can be demonstrated
   until tenancy, identity and entitlement exist, because there is no path that skips them. This
   is C-0001 principle 1's cost arriving where it is actually felt, and it is why RFC-0023's
   substrate and the first wave are sequenced the way they are.

2. **Paridyum inherits every limitation it sells**, and the pressure to grant an exception peaks
   during an incident affecting its own customers. Decision 2 is written to be checkable precisely
   because that is the moment judgement is worst.

3. **Decision 3 will reject work that is genuinely useful.** Something will be needed internally,
   have no tenant buyer, and not be build-time tooling. The rule forces it to become a product, be
   deferred, or be honestly reclassified — and sometimes all three will feel wrong.

4. **Three checks declared, none built.** The unenforced backlog grows by one artifact. Stated in
   Status rather than discovered in a gate run.

## Alternatives considered

**Keep it as a practice.** Dogfooding by intention, reviewed in code review. Rejected because the
failure mode is silent and cumulative: no single bypass looks wrong, and by the time the pattern is
visible the tenant-facing half is a rewrite. It is also the version that fails exactly when BET-0005
needs it most — under deadline, with our own revenue on the other side.

**Bind only decision 2.** Paridyum's tenant holds no extra grant, and everything else is judgement.
Genuinely cheaper, and it satisfies RFC-0022's conflict story. Rejected because it protects against
*misuse* of a privileged path and not against *building* one — the bypass would still exist, unused
by policy, which is a control that expires the first time someone is in a hurry.

**Exempt the first product.** Build product one directly, retrofit the tenant path once it works.
The strongest temptation and the one this ADR is most likely to be asked to permit. Rejected on
consequence 1's own reasoning: retrofitting is the rewrite, and product one is precisely where the
seams get set for everything after it.

**Extend it to `tool-*` and `infra-*` as well.** Purer, and no boundary to argue over. Rejected as
ceremony with no customer at the end of it — and a rule that is routinely ignored teaches that
rules are optional, which is more expensive than the exemption.

## Related

- **C-0008** — the principle this enforces; read it first
- BET-0005 — operating what we franchise, which is why this must be structural
- RFC-0022 — D1 and D5, whose credibility depends on decision 2
- C-0001 — principle 1 and principle 6
- ADR-0024, ADR-0038, GLO-0005 — nested tenancy, identifiers from the session, scope as a grant
- ADR-0039 — the provisioning saga decision 2 requires Paridyum to use
- ADR-0042 — `deployable: false`, the classification decision 4 leans on
- ADR-0019 — the design gate decision 5 extends
- ADR-0014 — why declared-and-unbuilt checks are honest rather than a lie

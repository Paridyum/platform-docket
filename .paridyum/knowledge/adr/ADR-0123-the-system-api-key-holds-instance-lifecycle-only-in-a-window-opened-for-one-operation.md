---
id: ADR-0123
title: The System-API key holds instance lifecycle only in a window opened for one operation
type: adr
status: accepted
created: 2026-09-28
locked_at: null
supersedes: null
superseded_by: null
scope: [platform, infra]
applies_to: ["*"]
product: [tenancy]
tags: [identity, zitadel, system-api, credential, least-privilege, control-plane, instance-lifecycle, adr-0045, adr-0090, adr-0119, rfc-0012]
breaking: false
enforced_by: []
sources:
  - url: https://raw.githubusercontent.com/zitadel/zitadel/v4.15.3/cmd/defaults.yaml
    title: "zitadel/zitadel v4.15.3 — cmd/defaults.yaml, SystemAPIUsers"
    retrieved: 2026-09-28
---

# ADR-0123 — The System-API key holds instance lifecycle only in a window opened for one operation

## Status

**Accepted** 2026-09-28 by the operator, who chose option (a) of the two put to them when the estate's
rebuild from empty had to create the `id.paridyum.com` control-plane instance.

It is not `locked`. The rule is mechanically checkable: every `SystemAPIUsers` entry in the committed
values carries an explicit membership list, and none of those memberships is `MemberType: System`.
But no check asserts it yet. It graduates when one does.

## Context

**The capability.** The System API creates and destroys Zitadel instances. It is never delegated
(RFC-0012 D5). ADR-0045 names its credential as threat T5: *"The highest-consequence credential: it
can destroy every tenant in a region."* The estate's one System-API user is `paridyum-system`:

- its public key sits in infra-kubernetes `zitadel/values.yaml`;
- its private key is in SOPS and never enters the cluster.

**What was believed.** On 2026-09-13 the key was given an `IAM_OWNER` membership scoped to the
control-plane instance, so the same key could automate that instance — branding, the operations
project — with no console step and no hand-copied PAT. The debt entry recording this said the key
*"already creates and deletes instances via the System API, so IAM_OWNER on one instance is not a new
class of power."*

**What was true.** Zitadel's `cmd/defaults.yaml`: *"If no memberships are specified, the user has a
membership of type System with the role SYSTEM_OWNER."* An explicit list **replaces** that default; it
does not add to it. So from 2026-09-13 the estate could not create or destroy an instance without
editing its configuration, and nothing said so, because nothing tried. The rebuild tried, on
2026-09-28, and `/system/v1/instances/_search` answered `403 No matching permissions found
(AUTH-5mWD2)`. The posture had to be decided rather than discovered.

**The pressures.**

- **Automation** wants the key always able to do whatever provisioning needs.
- **Least privilege** wants the most dangerous capability in the estate to exist only while it is
  being used.
- **Instance creation is rare.** The control plane once; one portal instance per region
  (ADR-0119 D1); a recreate after a configuration change. A standing capability buys convenience for
  an operation counted in times per year.

## Decision

1. **At rest, `paridyum-system` holds no System membership.**
   - Its committed memberships in `values.yaml` are explicit and instance-scoped. Today that is
     `IAM_OWNER` on the control-plane instance.
   - The list is mandatory. An empty or absent one is Zitadel's default `SYSTEM_OWNER` — the power
     this decision withholds, granted by omission.

2. **Lifecycle is a window, opened for one operation and closed at once.**
   - `make zitadel-system-lifecycle-open` upgrades the release with an override under which the key
     has the default System/`SYSTEM_OWNER`. The override is generated from `values.yaml` at run time
     and never written to the repository.
   - The operation then runs: `make zitadel-instance`, or `zitadel-instance-recreate`.
   - `make zitadel-install` closes the window by restoring the committed values.

3. **Closing is verified, not assumed.** After the close, `make zitadel-instance-plan` must answer
   `403` again. A `200` means the window is still open, and closing it comes before anything else.

4. **A window is recorded.** Every open and close is an entry in infra-kubernetes's `LOGBOOK.md` that
   names the operation it served. A standing credential leaves no trace of when its power was used; a
   window does.

5. **A new instance's scoped membership lands with the close.** The same change that closes the
   window commits the new instance's id as the `AggregateID` of the key's `IAM_OWNER` membership, so
   the automation reach it needs is declared rather than implied.

## Consequences

**Accepted costs.**

- **Two Helm upgrades per instance created**, each of which restarts Zitadel — minutes of degraded
  login per window. Acceptable at this frequency. A regional rollout would batch its instances into
  one window.
- **A window left open is the standing risk this decision exists to avoid.** Decision 3 and the
  logbook mitigate it; nothing closes it automatically.
- **During a window the key can destroy every instance**, every tenant's included — T5 at full
  strength, for minutes.
- **Whoever can run `helm upgrade` can open a window.** That boundary is not new: they could already
  rewrite the memberships. This makes the step explicit, named and logged.

**Gained.**

- The estate's most dangerous capability is off unless an operator turns it on for a named operation.
- The false belief is corrected at its source. The debt entry, the SOP and this ADR agree on what
  the key can do.

**Rejected alternatives.**

- **(b) A standing System membership beside the scoped one** — the posture the debt entry believed
  existed. A leaked age key, or a mistaken script, could then delete every instance, tenants
  included, at any moment, to save two upgrades a year.
- **A second System-API user kept only for lifecycle, always configured.** A different key with the
  same standing power: option (b) with more key management.
- **Creating instances by hand.** The System API is the only interface that creates an instance, so
  there is nothing to click; and a hand-run call leaves no reviewable trace.

## Related

- RFC-0012 D5 — the System API is never delegated.
- ADR-0045 T5 — the credential this narrows.
- ADR-0090, ADR-0091 — what the key automates on the control-plane instance.
- ADR-0119 D1 — a portal instance per region is the next instance this governs.
- infra-kubernetes: `zitadel/values.yaml`, `SOP.md` (*Five: the control-plane instance*), and
  DECISION-DEBT (*paridyum-system holds IAM_OWNER on the control-plane instance*).

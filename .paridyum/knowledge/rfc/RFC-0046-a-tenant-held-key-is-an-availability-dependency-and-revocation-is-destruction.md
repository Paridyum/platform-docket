---
id: RFC-0046
title: A tenant-held key is an availability dependency, and revocation is destruction
type: rfc
status: proposed
created: 2026-09-15
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, infra, service]
applies_to: ["*"]
product: [tenancy]
tags: [byok, cmk, encryption, key-management, kms, crypto-erasure, revocation, rotation, escrow, dr, adr-0024, adr-0041, adr-0057, adr-0064, adr-0071, adr-0094, adr-0103, rfc-0045]
breaking: false
sources:
  - url: https://learn.microsoft.com/en-us/azure/architecture/guide/multitenant/approaches/storage-data
    title: "Architectural Approaches for Storage and Data in Multitenant Solutions — Azure Architecture Center"
    retrieved: 2026-09-15
  - url: https://learn.microsoft.com/en-us/azure/architecture/guide/multitenant/approaches/governance-compliance
    title: "Architectural Approaches for Governance and Compliance in Multitenant Solutions — Azure Architecture Center"
    retrieved: 2026-09-15
---

# RFC-0046 — A tenant-held key is an availability dependency, and revocation is destruction

> **BUILD GATE.** This RFC is a design, deliberately completed before implementation on explicit
> human direction (2026-09-15). **No part of it is built until that direction is given.** ADR-0103
> decision 7 and ADR-0041 decision 4 both forbid listing BYOK as available before it is real and
> rehearsed; this document existing is not that.

## Problem

ADR-0103 decision 7 names customer-managed keys as a capability at containment class C2 and states
that they are not available. RFC-0045 open question 4 asked whether BYOK is a real capability or a
statement of intent, and observed that key custody, rotation and revocation are undesigned.

The gap is not "we have not built encryption." The estate encrypts: ADR-0057 holds secrets under
SOPS and age, ADR-0071 keeps the off-host store ciphertext-only, and storage is encrypted at rest
under a platform-held key. **What does not exist is any mechanism by which a key is held, or
controllably destroyed, by someone other than us** — and that, not encryption, is what a tenant
asking for BYOK is asking for.

Three specific things are undecided, and each one is the kind that cannot be retrofitted:

1. **Where the key lives and who can reach it.** A key we hold, labelled as the tenant's, is theatre.
   A key only the tenant holds makes their data unreadable to us by construction — including during
   an incident we are contractually obliged to resolve.
2. **What revocation means.** A tenant revoking its key is not a permissions change. It is
   irreversible destruction of every byte encrypted under it, including backups, WAL and off-host
   copies. There is currently no artifact that says whether that is a supported operation, how it
   is confirmed, or what survives it.
3. **What happens when a tenant loses its key.** Not a hypothetical: it is the single most common
   BYOK incident in practice. Without a decided answer, the answer defaults to whatever the engineer
   on call improvises, under pressure, on the worst day of a customer relationship.

**Who experiences it.** Nobody today, because no tenant is at C2. The cost of not deciding is paid
at the moment a contract names BYOK — a sales conversation where the honest answer is unknown, and
the pressure to say yes is maximal.

**What happens if nothing is done.** ADR-0103 decision 3 requires a named rung to be operable, so an
undesigned BYOK simply blocks C2 from ever being named. That is a safe failure, and it is also a
permanent ceiling on the deals the platform can take.

**How we will know it is fixed.** A tenant can generate a key we never see, encrypt their data under
it, rotate it without downtime, and destroy it with a confirmed, irreversible, evidenced operation —
and every one of those four is rehearsed before any page names it.

## Constraints

- **ADR-0103 (proposed)** — decision 7 defers to this document; decision 3 means nothing here is
  named publicly before it is built and rehearsed; decision 2 keeps C2 contract-gated.
- **ADR-0041 (accepted)** — decision 3: *"Key management has a policy from the first deployment. The
  Zitadel masterkey is immutable once initialised, per region — a compliance artifact, not an
  operational note."* This RFC is that policy extended to tenant data. Decision 4 forbids claiming
  the capability before an audit or a rehearsal says it is real.
- **ADR-0057 (accepted)** — secrets are SOPS + age. The estate already has an asymmetric key habit
  and a recipient model; this RFC should extend it rather than introduce a second cryptosystem.
- **ADR-0071 (accepted)** — the off-host store only ever holds ciphertext. A BYOK tenant's off-host
  copy is therefore doubly encrypted, which is fine, and its recoverability now depends on a key we
  may not hold, which is the whole problem.
- **ADR-0094 (accepted)** — Garage is the in-cluster S3 and WAL endpoint. WAL is a stream of the
  same bytes and is covered by any decision made about the data.
- **ADR-0064 (accepted)** — one Postgres instance on one hypervisor. C2's dedicated cluster presumes
  a second failure domain that does not currently exist (RFC-0045 open question 6), so this design
  may be deliverable only after a hardware decision.
- **ADR-0024 (locked)** — RLS is forced at every containment class. Encryption is defence *added*;
  a C2 tenant's tables still carry `tenant_id` and still force RLS.
- **ANTI-0007** — regulated data still waits for its control framework. BYOK is not a substitute for
  the controls HIPAA or PCI require, and must never be sold as one.
- **C-0004** — key destruction is the most irreversible operation the platform will ever offer.
- **RFC-0045** — §Proposal E's attestation gains key-custody fields; §Proposal F's export and restore
  paths must work under a key we may not hold.

## Proposal

### A. Envelope encryption, and the tenant holds only the outer key

Full end-to-end encryption under a tenant key is incompatible with a relational database: the server
must compare, index and join plaintext. What BYOK actually means everywhere it is sold is **envelope
encryption** — and saying so plainly is the first honest thing this design must do, because a tenant
who believes we cannot read their rows is a tenant we have misled.

```
tenant root key (KEK)          held by the tenant, never by us
        │  wraps
        ▼
data encryption key (DEK)      per tenant, per cluster; unwrapped into memory at startup
        │  encrypts
        ▼
volume + WAL + backups         at rest, at C2's dedicated cluster
```

**What this gives the tenant, stated precisely** — the sentence that belongs in the contract:

- Data at rest is unreadable without their key. Detached volumes, stolen disks, backup archives and
  the off-host copy are all inert.
- They can make their data permanently unreadable, by us and by anyone, at a time of their choosing,
  without asking us.

**What it does not give them, stated equally precisely:**

- It does not prevent us reading their rows while the cluster is running. The DEK is in memory; the
  database serves plaintext. An operator with break-glass access reads data, and RFC-0045
  §Proposal H's logging — not the key — is what constrains that.
- It is not per-row, per-account or per-field encryption. The unit is the cluster.

### B. Custody: the tenant's KMS, or an age recipient, and nothing in between

Two supported custody models, chosen by the tenant at provisioning and recorded on the placement:

**B1 — external KMS.** The tenant names a KMS they control; we hold an identity permitted to call
*unwrap* and nothing else. Revocation is the tenant removing that permission, which is instantaneous
and needs no cooperation from us. This is the model enterprises expect and the stronger of the two.

**B2 — age recipient.** The tenant supplies an age public key; the DEK is wrapped to it, and
unwrapping requires the tenant's private key to be presented at cluster start. This extends ADR-0057's
existing cryptosystem rather than introducing a second, needs no external dependency, and is
materially weaker operationally — an unattended restart cannot unwrap without a held secret, which
pushes it toward a cached-DEK compromise that erodes the guarantee.

**Rejected as a third option: keys we hold on the tenant's behalf.** It is the model most
platforms ship and label BYOK, and it provides no property the platform key does not already
provide. Offering it would breach ADR-0041 decision 4 in spirit — claiming a capability by naming it
rather than by having it.

### C. Rotation is routine; revocation is not

**Rotation** re-wraps the DEK under a new KEK. The data is untouched, there is no re-encryption, and
there is no downtime. Tenant-initiated at any time, scheduled by policy, and recorded to the audit
stream. This must be genuinely routine — a rotation path that is feared is a rotation path that is
never exercised, and then it does not work when a compromise requires it.

**Revocation is crypto-erasure and is irreversible.** When the KEK becomes unavailable — by tenant
action or by loss — every byte under that DEK is permanently unreadable: live volume, WAL, backups,
off-host copy. There is no recovery, no support escalation and no exception.

The controls around it, and none of them is optional:

1. **Named in the contract before C2 is provisioned.** The tenant signs having read what revocation
   destroys.
2. **Confirmed out of band**, by a second party on the tenant's side, against a challenge we issue.
3. **Delayed by a stated, contractual quarantine period** during which the key is marked revoked,
   access is refused, and the action is reversible by the tenant. This is the single most valuable
   control in the design, because the most common cause of key destruction is error rather than
   intent.
4. **Evidenced.** A destruction certificate, written to an audit stream that survives the data —
   RFC-0045 §Proposal D.4's rule applied to the most extreme case.
5. **Never initiated by us.** Not for non-payment, not for termination, not for abuse. Offboarding
   deletion (RFC-0045 §Proposal F) is our path and is separate.

### D. Key loss — decided in advance, because it will happen

**The default is: the data is gone, and we help with nothing but evidence.** Anything softer is a
key we effectively hold, which destroys the guarantee for every other C2 tenant.

Two mitigations offered at provisioning, both the tenant's choice and both recorded:

- **Tenant-side escrow.** They hold a second KEK in a separate custody path. Entirely theirs; we
  verify at provisioning that a second wrap exists and never learn what it is.
- **Split escrow with a third party.** The tenant names an escrow agent. We never hold a share.

**We do not offer an escrow we can open.** The recovery it would provide is exactly the access the
tenant paid to remove.

### E. What BYOK does to everything else

The parts most likely to be discovered late, listed because each one silently breaks:

| Surface | Effect |
|---|---|
| **Backups (ADR-0094)** | Ciphertext under the tenant DEK. Restorable only while the KEK is reachable. A backup is no longer proof of recoverability |
| **Off-host copy (ADR-0071)** | Doubly encrypted. Our DR rehearsal cannot verify a C2 tenant's restore without tenant cooperation — so the rehearsal becomes a joint exercise, scheduled, in the contract |
| **Attestation (RFC-0045 §E)** | Gains `key_custody`, `key_last_rotated_at`, `key_status`. `key_status: unreachable` must surface *before* it becomes an outage |
| **Export (RFC-0045 §F)** | Runs against a live cluster, so it works while the key does. A tenant that revokes first cannot then export |
| **Support** | We cannot reproduce a C2 tenant's issue from a backup without them. Response times differ and must say so |
| **Startup** | An unreachable KEK means the cluster does not start. Key availability is now an availability dependency with someone else's uptime inside ours |
| **RPO/RTO** | Both worsen at C2. The attestation must state the worse numbers, not the C0 ones |

### F. The rehearsal that gates the capability

Per ADR-0103 decision 3, C2 is not named publicly until all five are performed against a real
cluster with Paridyum as the tenant (ADR-0049 D2), and recorded:

1. Provision C2 with a tenant-held KEK; confirm we never hold it.
2. Rotate the KEK with the cluster serving traffic; confirm no downtime and no re-encryption.
3. Make the KEK unreachable; confirm the cluster refuses to start and the attestation says why.
4. Restore it; confirm recovery and that the quarantine window behaved as specified.
5. Destroy a KEK for real, on throwaway data; confirm the data is unrecoverable and the certificate
   is written.

**Step 5 is the one that will be skipped, and it is the only one that proves the product.**

## Alternatives considered

1. **Do not offer BYOK; state plainly that encryption is platform-managed.** Honest, cheap, and
   closes a recurring enterprise question with a no. Rejected by the human direction on 2026-09-15,
   which put BYOK in scope. Recorded because it remains the correct answer if §F's rehearsal proves
   too expensive — a capability withdrawn before it is named costs nothing.

2. **Application-level or column-level encryption under a tenant key.** Stronger on paper: we could
   not read the rows at all. Rejected — it breaks indexing, joins, sorting, collation (STD-0010) and
   RLS predicates on any encrypted column, and would fork the data model for a minority of tenants.
   It also does not remove operator access to anything the application decrypts to serve a request.

3. **Per-tenant keys at C0 or C1, in the shared cluster.** Rejected: a shared cluster has one storage
   layer and one WAL stream, so a per-tenant key there encrypts nothing distinct and cannot be
   independently destroyed. Crypto-erasure needs a dedicated cluster to be meaningful, which is why
   BYOK sits at C2 and not lower.

4. **Keys we hold, labelled per tenant.** The most common shipped model. Rejected in §B — it adds a
   label and no property, and a capability claimed by naming is what ADR-0041 decision 4 forbids.

5. **Escrow we can open.** Rejected in §D — it is the access the tenant paid to remove.

## Consequences

**What this makes worse.**

1. **Someone else's uptime is now inside ours.** A tenant's KMS outage is our outage for that tenant,
   and the incident review is a conversation about their infrastructure. This is inherent to BYOK and
   is the cost most underestimated at signing.
2. **Our DR rehearsal stops being unilateral.** ADR-0064 D3's gate requires a *rehearsed* restore;
   for a C2 tenant that rehearsal needs their participation, on a schedule, in the contract.
3. **Support is materially worse at C2**, and honestly so. We cannot open a backup to diagnose.
4. **A destructive operation now exists, and it works.** §C is a correctly-built path to permanent
   data loss. Its controls are the product; a bug in them is unrecoverable by definition.
5. **C2 presumes a dedicated cluster that one hypervisor cannot honestly provide** (ADR-0064). This
   design may be complete and undeliverable until that is answered.
6. **A second cryptosystem, if B1 is chosen.** ADR-0057's SOPS/age habit is extended by B2 and
   bypassed by B1. Two custody models is more surface than one, and B1 is the one enterprises want.

**What it does not change.** RLS is forced at C2 exactly as at C0. ADR-0038's session contract is
untouched. Encryption is defence added, never defence substituted — a point worth keeping in front of
anyone who assumes BYOK makes tenancy correctness less important.

## Rollout

**Gated. Nothing below starts without explicit human direction** (ADR-0103 D7).

| Slice | What lands | Depends on |
|---|---|---|
| 0 | This design accepted; ADR-0064's hypervisor question answered enough to say whether C2 is deliverable | — |
| 1 | Envelope scheme + DEK lifecycle at a dedicated cluster, **platform-held KEK** — C2 without BYOK | RFC-0045 slice 3 |
| 2 | Custody model B1 (external KMS); provisioning captures the KEK reference | slice 1 |
| 3 | Rotation, tenant-initiated and scheduled, no downtime | slice 2 |
| 4 | Revocation: quarantine window, out-of-band confirmation, destruction certificate | slice 3 |
| 5 | Attestation fields; RPO/RTO restated for C2 | RFC-0045 slice 7 |
| 6 | §F rehearsal, all five steps, recorded | all |
| 7 | **Only now** may any public artifact name BYOK | slice 6 |
| 8 | Custody model B2 (age recipient), if wanted | slice 4 |

**Slice 1 is worth noticing**: a dedicated cluster with a platform-held key delivers most of C2's
failure-domain isolation with none of BYOK's operational burden, and it is a defensible rung on its
own. If §F proves too expensive, stopping after slice 1 is a real outcome rather than a failure.

**How it is undone.** Slices 1–3 are removable. Slice 4 is not — a destruction path that has run
cannot be un-run, which is why it is fourth and not first.

## Open questions

1. **Which KMS platforms do we support at B1, and does each one's revocation actually behave as §C
   assumes?** Vendor-specific and the kind of fact that decays; needs `sources` and dates when
   answered.
2. **What is the quarantine period?** Long enough to catch an error, short enough to be a real
   guarantee. It is a contractual number and a human decision.
3. **Is C2 deliverable on one hypervisor at all?** Inherited from RFC-0045 open question 6 and now
   blocking rather than theoretical.
4. **Does a C2 tenant's data still go off-host under ADR-0071?** A doubly-encrypted copy we cannot
   open is arguably not a backup. If it does not, C2's DR story is materially different from C0's and
   the attestation must say so.
5. **What does the contract say about our access?** §A is explicit that BYOK does not prevent
   operator reads. A tenant who assumed otherwise has been misled, and the correction belongs in the
   contract and the docs, not in a footnote.
6. **Does DPDP or GDPR treat crypto-erasure as erasure?** Widely assumed, jurisdiction-specific, and
   it decides whether §C can serve an Art. 17 request. Needs a cited, dated source (RFC-0020).

## Related

- **ADR-0103** — decision 7 defers to this; decision 3 gates naming on the §F rehearsal
- **ADR-0041** — decision 3 (a key policy from the first deployment) and decision 4 (nothing claimed
  before it is real)
- **ADR-0057** — the SOPS/age cryptosystem B2 extends and B1 bypasses
- **ADR-0071, ADR-0094** — the off-host and WAL paths §E changes
- **ADR-0064** — the hypervisor constraint that may make C2 undeliverable
- **ADR-0024** — RLS forced at every class, BYOK included
- **RFC-0045** — the ladder, the attestation and the export path this modifies
- **ANTI-0007** — BYOK is not a substitute for a regulated-data control framework
- **C-0004** — key destruction, the most irreversible operation in the estate

---
id: ADR-0129
title: Signup opens before a real off-host backup exists, and the operator accepts that risk until one does
type: adr
status: accepted
created: 2026-09-29
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, infra]
applies_to: ["*"]
product: [paridyum, identity, tenancy]
tags: [risk-acceptance, backup, off-host, personal-data, signup, rfc-0056, adr-0064, adr-0071, adr-0105, t-g-7, dpdp]
breaking: false
enforced_by: []
---

# ADR-0129 — Signup opens before a real off-host backup exists, and the operator accepts that risk until one does

## Status

**Accepted** 2026-09-29 by the operator, who chose *"risk-acceptance ADR now"* over a real off-host
destination when asked. This is the risk-acceptance ADR that RFC-0056 §F names as the alternative to
T-G.7. It is written to be closed rather than kept: see decision 4.

## Context

**RFC-0056 §F:** *"Signup stores real people's addresses, so the public route opens only after one of:
T-G.7 — a rehearsed off-host restore; a risk-acceptance ADR, signed by the operator, with T-G.7 as its
closing trigger."*

**What exists, stated exactly:**

- WAL archiving and nightly base backups to Garage. A restore was rehearsed and matched live on
  2026-09-24 and again after the rebuild on 2026-09-28.
- **The only "off-host" copy is a second Garage bucket on the same machine.** T-G.7 is marked done in
  TASKS.md, but its own text says the copy rehearses *"the off-host process, not yet a real off-host
  destination"*, and ADR-0105 D4 says a real one is still owed.
- **On 2026-09-28 the host ran out of memory and its hypervisor guest hung.** Nothing was lost, but it
  showed that every copy of the data shares one machine's fate.

## Decision

1. **RFC-0056 signup may open to real people,** in region `in`, before a real off-host backup exists.
   This covers the portal instance's people — their addresses and credentials — and their orgs' rows:
   `orgs`, `org_profiles`, `org_members` and `org_invitations`.

2. **It covers nothing else.** In particular, it does **not** open the tenant data gate:
   - careers applications, which carry candidates' personal data;
   - docket's personnel records;
   - any `PARIDYUM_BACKUPS_CONFIRMED` flag.

   Those stay closed until a real off-host restore exists, or until the operator extends this ADR
   explicitly, in writing.

3. **What is accepted, in plain terms:** if the one machine and its disks are lost, the backups go with
   it, and every signed-up person's account and every org would have to be recreated. People would need
   to re-register, and nothing in the portal could be recovered from Paridyum's side.

4. **It closes on a trigger, and is reviewed by a date.**
   - **Closing trigger:** a backup copy on a real off-host destination, outside the machine that runs
     the estate, and a restore from that copy that matches live (ADR-0071 §3, ADR-0064 D3). When that
     is true, this ADR is superseded by the record of it.
   - **Review date: 2026-10-29.** If it is still open then, the operator either renews it with a
     reason or signup closes.

5. **Nothing claims otherwise while it is open.** No attestation, page or copy may say or imply that
   signup data is backed up off-host (ADR-0105 D3, ADR-0041 D4).

## Consequences

**Accepted costs.**

- **A host loss erases every account and org.** That is decision 3, accepted knowingly.
- **A deadline.** The review date is a commitment, not a reminder, and it is recorded in
  DECISION-DEBT.md.

**Gained.**

- **The sprint can finish:** the operator's own signup, and paridyum.com created through paridyum.in
  (ADR-0128), without waiting on a purchase.
- **The acceptance is narrow and visible.** The more sensitive tenant data stays gated.

**Rejected alternatives.**

- **AWS S3 in Mumbai.** It is off the host and still in India, and it is the recommended way to close
  this ADR. It was not chosen for now.
- **Cloudflare R2.** It offers no India-only jurisdiction, so it would need a data-residency reading
  first.
- **Open the tenant data gate too.** Candidates' and employees' records are more sensitive than a
  signup address, and nothing in the sprint needs them yet.

## Related

- RFC-0056 §F — the gate this accepts a risk against.
- ADR-0071, ADR-0064 D3 — the off-host copy and the rehearsed restore that close it.
- ADR-0105 — one failure domain, and decision 4's "off-host is still owed".
- ADR-0128 — the sprint this unblocks.

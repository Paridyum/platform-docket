# Review brief — RFC-0015 decision 3, sealed off-jurisdiction copies

**Written 2026-09-24.** Prose in `docs/`, not a knowledge artifact. It binds nothing; where it and
a knowledge artifact disagree, the artifact wins. Written to be handed to an independent reviewer —
human or agent — that has no context of the session that produced it.

---

## What is under review

A dated revision to **`rfc/RFC-0015-a-fact-has-one-home.md`**, decision 3:

- **The revision block** — `### Sealed copies are not replication — REVISED 2026-09-24`, inside
  decision 3 (search the file for `REVISED 2026-09-24`). This is the substantive change.
- **Frontmatter** — a new `sources:` block (three entries, all retrieved 2026-09-24).
- **Status line** — notes the revision; status remains `proposed`.
- **Consequences, accepted cost 3** — a dated note narrowing what remains accepted.
- **Related** — six added references (ADR-0071, ADR-0094, ADR-0105, RFC-0046, MKT-0003, T-G.7).

Nothing else was changed. **RFC-0015 is `status: proposed`**, so revising it in place is legal and
a supersession is neither needed nor being proposed. Do not demand one. (`artifact-registry.yaml`
records `immutable_when_locked: false` for `rfc` in any case.)

## The estate in five sentences

Paridyum is a pre-incorporation, single-operator platform whose product is provable data isolation,
starting in India (DPDP). One region is live: `in` — the control region — containing two declared
datacentres, `in-mum-1` and `in-hyd-1`. The running estate is actually **one host** (`sitar`); no
second datacentre, standby or replica exists, deliberately (`ADR-0064`, `ADR-0105`). The only
off-host copy is a manual, age-encrypted dump to Sync.com, which stores in Canada (`ADR-0071`), and
the database leg of that copy is **not yet configured** (`TASKS.md` T-G.7, measured 2026-09-21).

## The change in one paragraph

Original D3: *"Nothing crosses a region boundary — not a standby, not a replica, not a backup."*
The revision splits that: a copy with **operational consequences** (serves a request, resolves an
entitlement, holds a session, is kept in sync) is still refused across a boundary — the D1/D3/D4
reasons (staleness, revocation lag, deletion reach, suspended-tenant coordination) are untouched.
A **sealed copy** — restore-only, encrypted before egress, keys not held by the destination, opaque
paths, no automated restore path, named in the DPA/attestation, honest RPO — **may** cross. This
reconciles D3 with ADR-0071, which already crosses the boundary with ciphertext. It does not create
a second failure domain, a standby or a second region; `ADR-0105` is unaffected.

## Minimum reading (in this order)

1. `rfc/RFC-0015-a-fact-has-one-home.md` — decisions **1, 3, 4**; the revision; consequences;
   reversal conditions. Read decision 3's original accepted-risk paragraph _before_ the revision.
2. `adr/ADR-0071-the-off-host-store-is-sync-and-it-only-ever-holds-ciphertext.md` — the accepted
   posture being reconciled with. Note decisions 2, 3 and 4.
3. `adr/ADR-0064-postgres-runs-under-an-operator-and-one-instance-is-the-honest-shape.md` — D3, the
   gate ("a backup on the same hypervisor as its source is not a backup"; "an unverified backup is
   a belief").
4. `adr/ADR-0094-garage-is-the-in-cluster-s3-object-store-and-the-estates-backup-endpoint.md`
   — decision 4 (the whole Garage store is itself backed up off-host, manually, as ciphertext).
5. `adr/ADR-0105-a-second-failure-domain-is-declined-for-now-and-the-rpo-says-so.md` — the second
   failure domain is declined; attestations state the honest RPO.
6. `rfc/RFC-0046-a-tenant-held-key-is-an-availability-dependency-and-revocation-is-destruction.md`
   — what a tenant-held key does to a backup's meaning.
7. `rfc/RFC-0045-the-isolation-envelope-is-a-placement-a-containment-class-and-an-attestation.md`
   — §E (the attestation) and the rollout table (slice 4).
8. `markets/MKT-0003-india-data-residency-under-dpdp.md` — the bound: no blanket localisation in
   India; the transfer list is a negative list, not yet published.
9. `TASKS.md` — T-G.7 (search `T-G.7`), the measured current state.
10. `CLAUDE.md` and `constitution/C-0004-protected-operations.md` — the router's rules; locked
    artifacts must not be contradicted.

## Claims to attack

Each claim is falsifiable. For each, either confirm with a citation or produce a counterexample.

1. **No locked artifact is contradicted.** Check every locked ADR in this repository's scope, in
   particular anything about residency, tenancy, deployment or standards supremacy (e.g. `STD-0006`
   is the one RFC-0014 proposes to supersede; RFC-0015 does not touch it). If one is contradicted,
   name it — the revision is then not a revision but a supersession, and the brief is wrong.
2. **The sealed/live distinction is operatively decidable.** "Restore-only", "encrypted before it
   leaves" and "on no request path" must each be examinable by something other than the author's
   intent. Say what would decide each, and where it is silent it is weak.
3. **Erasure reaches the copy.** The revision leans on destruction as the deletion path. Attack it:
   does deleting the key destroy the copy? What about the vendor's version history, deleted-file
   retention, and the difference between "our file is gone" and "no recoverable copy exists"?
   Compare with `adr/ADR-0069-the-vault-is-shreddable-and-the-register-is-not.md`.
4. **The legal reading holds.** The revision's conditions implement the position that encryption
   makes a transfer defensible rather than exempt. Verify against the EDPB source and ask what its
   conditions require of **key custody and location** — ours live in a password manager whose
   jurisdiction is not stated anywhere. A finding here is valuable.
5. **The AWS evidence is apt, not borrowed.** The claim is: physical destruction exceeded multi-zone
   design; copies outside the damaged facility survived; that is the class a sealed copy covers.
   Check the two sources, and check the revision does not overclaim that this makes recovery real.
6. **Nothing is silently overtaken.** The original paragraphs still stand as written and are
   explicitly reconciled by a dated block. If any sentence elsewhere in RFC-0015 (or in RFC-0014,
   RFC-0016, RFC-0017, RFC-0045, RFC-0051) still contradicts the revision without acknowledging it,
   name the file and line.
7. **Scope creep.** The revision should not: weaken D4 (local suspension), permit a standby/replica
   by implication, or change RFC-0015's second-region trigger (decision 2, signed contract). If any
   reading of the new text permits more than a sealed copy, that is a blocking finding.
8. **The four conditions are enforceable later.** If they cannot be checked, the rule is a
   preference. Say what a check could assert and against what.
9. **Honest accounting.** Consequence 3's dated note says recovery is a cold restore under a
   contract amendment, never a failover, and depends on a copy that does not yet exist for the
   database leg. Verify each half against `T-G.7` and `ADR-0105`.

## Highest-yield questions for a reviewer

- **Restore-only could become a de facto availability dependency.** If the only surviving copy is
  in a foreign consumer cloud provider, has the platform acquired a dependency it refuses to
  acknowledge? Is there a difference between a dependency during normal operation and one during
  recovery, and does the revision say which it means?
- **Operator personal data.** RFC-0017 holds that an operator's personal data lives in the
  operator's region. A whole-database dump includes operators' personal data. Does the revision
  address that class explicitly, or does it treat all tenant data as one thing?
- **The tenant's contract.** `tenant.region` is contractual and immutable (ADR-0041). If a
  jurisdiction is lost, restoring elsewhere is a migration plus a contract amendment — is that path
  actually described anywhere, or is the revision promising a copy with nowhere lawful to restore
  it to?
- **Counts and metadata.** The revision requires opaque paths but says nothing about file sizes,
  counts, timing and object keys — all of which an observer at the destination can see. Is that a
  gap or an acceptable residue? MKT-0003's reading and the EDPB source both bear on it.
- **DPDP's unpublished blacklist.** The permission is the estate's default. If a destination is
  later blacklisted, what has to change, and does the revision make that a configuration or a
  rewrite?
- **The wrong comparison.** Does the revision accidentally compare itself to the *replication*
  alternative it actually beats, while never engaging the *in-region off-host* alternative
  (a second machine in India)? The revision asserts that alternative loses on recorded facts; test
  whether "wait for a second host" was genuinely considered and genuinely rejected, or merely
  dismissed.

## Known gate state (do not attribute to this change)

`./scripts/check-knowledge.sh` reports **1 error, 44 warnings**. The error is pre-existing and
unrelated: `ADR-0105` declares `docs.attestation_rpo_matches_failure_domain`, which is not yet
registered in `check-registry.yaml` (ADR-0014). Baseline before the revision was identical.

## Known adjacent inconsistency (also not caused by this change)

`region-registry.yaml` (declared `PROPOSED`, not binding — its `regions:` block) and item 3 of the
problem section in `rfc/RFC-0051-a-tenant-chooses-a-jurisdiction-and-nothing-below-it.md` both
assert that `in-mum-1` and `in-hyd-1` are `status: live`, while `ADR-0105` (accepted 2026-09-24)
states the estate remains one failure domain with no second hypervisor. Both cannot be true. It is
flagged here because it is the same class of defect the revision addresses — a record that outruns
the runtime — and because a reviewer checking claim 6 will find it.

## What a useful review returns

- Findings by severity — **blocking / should-fix / note** — each citing an artifact id and a quote
  or line reference. "The text feels wrong" is not a finding; name the sentence and the rule.
- For each numbered claim above: confirmed or a counterexample.
- Any external fact that has changed since the cited retrieval dates, with a URL and date.
- Explicitly: what you could not verify from the repository alone.
- **Read-only.** Do not edit, commit or reformat anything. Return markdown.

## Sources to verify (all retrieved 2026-09-24)

- https://www.datacenterdynamics.com/en/news/aws-unable-to-restore-access-to-data-centers-hit-by-iran-strikes/
- https://www.chosun.com/english/industry-en/2026/09/16/OQE6F6MFLNASVAG2YRPVOUABAA/
- https://www.edps.europa.eu/sites/default/files/publication/edpb_recommendations_on_supplementary_measures_-_dpo_meeting_11_dec_2020_en.pdf
- MKT-0003 carries the DPDP sources already; verify they still say what MKT-0003 says they say.

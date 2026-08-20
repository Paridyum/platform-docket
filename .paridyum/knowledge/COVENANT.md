# Paridyum — The Veer Covenant ledger

**The disbursement record required by [C-0009](constitution/C-0009-the-veer-covenant.md).**

This file lives in `knowledge/` root beside `ROADMAP.md` and `DECISION-DEBT.md`, **not** in a typed
artifact directory, for the reason ROADMAP.md gives about itself: it is a hand-maintained record
rather than a rule, and `scripts/check-knowledge.sh` scans only the directories named in
`artifact-registry.yaml`. It is not an artifact and it binds nothing.

**It is not, however, unwatched.** `covenant.disbursement_record_current` reads the most recent
`declared:` date in the ledger below and fails the gate when it is more than twelve months old.
That check is **declared and not yet implemented** — see `check-registry.yaml` — and it has no
subject until Paridyum has revenue.

---

## What this file can and cannot show

Stated at the top so nobody reads more into a green gate than is there.

**It can show** that a dated entry exists, that it is recent, and what was claimed.

**It cannot show** that the amount was correct, that the revenue base was computed honestly, that
the recipients were eligible under [GLO-0010](glossary/GLO-0010-covenant-beneficiary.md), or that
obligation III was honoured for anyone who asked. Those are upheld by judgement, exactly as C-0006
says of a market position. A false entry here is a lie rather than a drift, and that is the entire
improvement this file offers over silence.

---

## Entry format

One block per financial year, newest first. Every field is required; `nil` is a legitimate value
and an omitted field is not.

```
### FY <YYYY-YY>
declared: <YYYY-MM-DD>
revenue_base: <amount, ex-GST, or `nil` with the reason>
obligation_i_due: <1% of revenue_base>
obligation_i_paid: <amount>
routed_to: <recipient, per RFC-0026>
reference: <acknowledgement or transaction reference, or `none — see notes`>
obligation_iii_tenants: <count of active covenant tenants, or `nil`>
obligation_iii_value: <estimated unbilled capacity, or `not metered`>
notes: >
  Anything a reader needs in order to judge the numbers above, including anything
  that went wrong.
```

`revenue_base` is revenue excluding GST and other statutory amounts collected on the government's
behalf, per C-0009 obligation I. It is **revenue, never profit**, and the argument for that is in
C-0009 rather than repeated here.

`obligation_iii_value` may read `not metered` only while REQ-0020 is unimplemented. Once tenancy
exists, unbilled consumption is measured even though it is never billed — an accepted cost that
nobody measures is an accepted cost nobody can later decide to bound.

---

## Ledger

### FY 2026-27

```
declared: 2026-08-18
revenue_base: nil — Paridyum is not yet incorporated and has recognised no revenue
obligation_i_due: nil
obligation_i_paid: nil
routed_to: not yet established — RFC-0026 open
reference: none — no disbursement made
obligation_iii_tenants: nil — no runtime exists
obligation_iii_value: not metered — REQ-0020 deferred
notes: >
  THE OPENING ENTRY, written on the day C-0009 was accepted and before there is anything to
  record. It exists so that the ledger's first entry is not also its first excuse.

  Every number here is nil, and each is nil for a reason stated rather than left blank. Paridyum
  is at planning stage and incorporating soon; no revenue has been recognised, so obligation I is
  not yet payable. No runtime exists, so obligation III has no tenants to serve.

  OBLIGATION II WAS CONSIDERED AND NOT UNDERTAKEN, decided 2026-08-18, before incorporation. No
  founding equity is committed to the covenant and the incorporation documents carry no covenant
  clause. C-0009 slot II holds the decision, the argument for the obligation, the cost of deferring
  it, and the trigger that returns it — at every cap-table event, and no later than 2027-08-18.
  RECORDED HERE RATHER THAN OMITTED, because a ledger that shows only what was given is a
  promotional document. The consequence a reader should take from this entry: on an acquisition
  today the covenant receives nothing, since obligation I follows revenue and revenue stops being
  Paridyum's.

  NEXT ENTRY IS DUE BY 2027-08-18 whether or not anything has changed. "Nothing happened" is a
  valid entry. Not writing one is not.
```

---

## Related

- [C-0009](constitution/C-0009-the-veer-covenant.md) — the covenant, its four obligations, and the amendment barrier
- [GLO-0010](glossary/GLO-0010-covenant-beneficiary.md) — who a beneficiary is, and the priority order
- [MKT-0004](markets/MKT-0004-the-indian-armed-forces-welfare-estate-and-where-it-fails.md) — the evidence
- [REQ-0020](requirements/REQ-0020-covenant-eligibility-is-a-tenancy-attribute.md) — why obligation III is a tenancy attribute
- [RFC-0026](rfc/RFC-0026-the-covenant-vehicle.md) — where the money goes, and what still needs counsel
- `check-registry.yaml` — `covenant.disbursement_record_current`

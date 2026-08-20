# HR checklist — onboarding an intern

One page. Print it, or work down it on screen. Every step is a command whose
output tells you whether it worked; nothing here asks you to remember anything.

`SOP.md` has the same steps with full output and a failure table for each.
Where they disagree, `SOP.md` is right and this file is stale — say so.

---

## Before you start

- [ ] `docket check` exits 0. If it does not, fix that first: it is telling you
      the registry or an existing record is wrong, and everything below builds
      on both.

---

## 1 · Offer

- [ ] `docket new <slug> intern` — allocates their permanent identifier
- [ ] Fill in `people/<slug>.yaml`
      - [ ] `identity.date_of_birth` — **required.** It decides whether they
            are under 18, which changes what the law requires
      - [ ] `education.currently_enrolled: true` — the unpaid internship rests
            on this
      - [ ] `engagement.end_date` — an unpaid internship must be time-bound
      - [ ] `education.institution_requires_noc` — does their college want a
            letter?
- [ ] `docket check` exits 0
- [ ] `docket issue <slug> OFR` — the offer
- [ ] `docket emails <slug> --only offer` — copy the text, attach the offer, send
- [ ] **Wait for the signed acceptance.** Do not send the pack yet.
- [ ] `docket sign PAR/OFR/… --method wet-ink --on <date> --scan <file>`

## 2 · Under 18 only

Skip if they are 18 or older. `docket check` tells you which applies — it
refuses to issue for a child with no guardian recorded.

- [ ] Add the `guardian:` block to `people/<slug>.yaml`
- [ ] `docket emails <slug> --only guardian` — send to the guardian, not the intern
- [ ] Signed consent back before anything else is sent

## 3 · The documentation pack

- [ ] `docket onboard <slug>` — issues everything the stage owes
- [ ] `docket pack <slug>` — builds the folder
- [ ] Check the folder: `cd store/packs/PAR-P-… && shasum -a 256 -c MANIFEST-sha256.txt`
      — every line must say `OK`
- [ ] `docket emails <slug> --only pack` — send it, attaching the folder
- [ ] If their college wants a letter: `docket emails <slug> --only institution`

## 4 · As signed copies come back

One command per document, on the day it arrives — not in a batch later.

- [ ] `docket sign <serial> --method wet-ink --on <date> --scan <file>`
      (for an eSign, drop `--scan`; use `--method aadhaar-esign`)
- [ ] `docket status <slug>` — watch documents move from `[~]` to `[x]`

## 5 · Starting work

- [ ] `docket advance <slug>` — **this refuses** until the agreements, the
      security policy, the code of conduct and the grievance policy are all
      signed. That refusal is the point: it is what stops access being granted
      before the NDA is back.
- [ ] Create their accounts
- [ ] `docket event <slug> access_granted --detail "GitHub — Paridyum org"`
      — one per system, as you grant it
- [ ] `docket emails <slug> --only access` — the welcome message
- [ ] `docket index`

---

## Ending an internship

- [ ] Set `engagement.stage: offboarding` in their record
- [ ] `docket onboard <slug>` — issues the checklist and the feedback form
- [ ] `docket pack <slug>` and `docket emails <slug> --only offboarding`
- [ ] Remove each account, then
      `docket event <slug> access_revoked --detail "<same text as the grant>"`
      — the text must match the grant exactly, or the grant stays open
- [ ] `docket separate <slug>` — on their last working day
- [ ] Set `engagement.stage: alumni`, then `docket onboard <slug>` for the
      certificate and reference
- [ ] `docket emails <slug> --only completion`
- [ ] `docket index`

## Periodically

- [ ] `docket check` — after any change to a record or the registry
- [ ] `docket retention` — what is past its retention period. It reports; it
      never deletes. Deleting is a decision.
- [ ] `docket roster` — everyone, and where they stand

---

## Four things not to get wrong

**Do not send the pack before the offer is accepted.** Eleven documents landing
at once invites signing them all unread, which is the failure the cover sheet
is written to prevent.

**Do not grant access before `docket advance` lets you.** If you genuinely must,
`--force` exists, says it skipped the gate, and records the move either way.

**Do not edit anything under `store/`.** The register, the ledgers and the index
are generated or append-only. `docket check` will tell you if something was
edited, which is not the same as being able to undo it.

**Do not put a person's data anywhere this tool does not know about.** If it
goes in a cloud folder, record the link as `access.cloud_folder` — otherwise
`docket erase` cannot tell you it needs deleting, and an erasure request cannot
actually be honoured.

---

## What this tool will not do for you

It does not send mail, sign anything, create or remove accounts, delete on a
retention deadline, or reach a cloud folder. Each of those is a person's job,
and `SOP.md` says so at every step where it matters.

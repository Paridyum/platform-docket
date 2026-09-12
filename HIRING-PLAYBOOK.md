# Hiring playbook — from an empty seat to a signed offer

One page. `HR-CHECKLIST.md` starts at the offer; **this is everything before it.**
Where the two overlap, the offer is the seam and each says so.

`SOP.md` has the docket half in full, with output and a failure table for every
step. Where this file and `SOP.md` disagree, `SOP.md` is right and this one is
stale — say so.

---

## Read this before you start

**Two repositories, and the split is deliberate.** A job posting lives in
`web-paridyum/jobs/`. A person lives in `tool-docket/people/`. Nothing joins
them until you make an offer, because a candidate is not yet a person — that is
ADR-0068, and it is why there is no `docket apply` command to look for.

**Applications arrive by email today.** The web form exists and is deliberately
switched off. `api/apply` returns:

```
503  not_yet_accepting_applications
     "Applications open once off-host backups are confirmed (ADR-0071).
      Until then, apply by email."
```

That is not a bug and not a missing feature. **No real applicant's data is
stored until an off-host backup destination exists and a restore has been
rehearsed**, at which point an operator sets `PARIDYUM_BACKUPS_CONFIRMED=1` and
the form starts accepting. Until then `apply.email` on the posting is the only
intake, and this playbook describes that path because it is the one that works.

**Nothing here is automated, and that is the accepted state.** Sections 1–4
are a person's judgement, recorded by hand. docket takes over at section 5.

---

## 1 · Write the posting

A posting is a JSON file in `web-paridyum/jobs/<id>.json`, reviewed and merged
like any other registry in this estate. Copy the nearest existing one; do not
start from an empty file.

- [ ] Choose the id: `<slug>-<year>`, lowercase, e.g. `rust-platform-intern-2026`
      - [ ] **The year is not decoration.** Re-advertising next season is a new
            posting with new terms, not an edit of this one (RFC-0041)
      - [ ] Check no file of that name has ever existed — an id that has been
            `open` may never be renamed or reused, because a person record's
            `engagement.job_id` points at it years later
- [ ] `status: "draft"` while you write it
- [ ] `engagement.class` — `intern`, `employee` or `contractor`. **This must
      match the class you will later pass to `docket new`**
- [ ] `compensation.kind` — `unpaid` or `stipend`
      - [ ] If `unpaid`, the person you hire **must be currently enrolled**.
            docket will refuse the record later if they are not, so do not
            advertise an unpaid role you intend to fill with a graduate
- [ ] `compensation.provides` — only things actually backed by something.
      `CRT`, `REF` and `EQP` are document codes docket really issues
- [ ] `apply.email` — where applications land while the form is off
- [ ] `copy.en` **and** `copy.ar` — both. A posting with one locale does not
      render, and machine-translated Arabic is worse than none; get it read
- [ ] Open a pull request. **Someone else reads it before it merges** — a public
      claim about pay and duration gets a diff, an author and a review, which is
      the whole reason postings are files (ADR-0063)

## 2 · Publish it

- [ ] Change `status` to `"open"`, in its own commit
- [ ] Merge, and let `web-paridyum` rebuild
- [ ] Confirm it renders at `paridyum.com/en/careers` **and** `/ar/careers`
- [ ] **From here the id is frozen.** It has been public.

## 3 · While it is open

- [ ] Applications arrive at `apply.email`. Keep them in that mailbox — do not
      copy them into a spreadsheet, a notes app or a cloud folder
      - [ ] If you do put candidate material anywhere else, you have created
            personal data this estate cannot find, cannot age out and cannot
            delete on request. There is no command that will rescue that
- [ ] Acknowledge every applicant. Nothing sends this for you
- [ ] Keep the shortlist in the mailbox too, as a label or folder

## 4 · Decide

- [ ] Interview and decide. This playbook has nothing to add — it is a person's
      job and no part of it is recorded by a tool yet
- [ ] Tell the unsuccessful applicants. Do it before section 5, not after
- [ ] **Delete the material for candidates you did not hire**, once the
      posting closes and any re-application window you promised has passed.
      `docket retention` does not see this mailbox

## 5 · The offer — docket takes over

The moment a candidate accepts an offer they stop being an applicant and become
a person in docket. This is the seam ADR-0068 §3 calls the crossing, and it is
an operator action.

- [ ] `docket check` exits 0 before you touch anything
- [ ] `docket new <slug> <class>` — same class as the posting
- [ ] In `people/<slug>.yaml`, set `engagement.job_id` to the **posting id**.
      This is the join between the two repositories, and the only one
- [ ] Continue in **`HR-CHECKLIST.md` §1 · Offer**, from "Fill in
      `people/<slug>.yaml`"

## 6 · Close the posting

- [ ] Change `status` to `"closed"` in `web-paridyum/jobs/<id>.json`, PR, merge
- [ ] **Do not delete the file.** A closed posting is the record of what was
      promised to someone now employed under it
- [ ] Do not edit a closed posting's terms. If they were wrong, that is a
      conversation with the person, not a file edit

---

## Four things not to get wrong

**Do not advertise what docket will refuse to issue.** `compensation.provides`
is a promise. If you list `CRT` for a role whose stage never reaches a
certificate, you have committed to something the tool will not produce and
someone will have to explain why.

**Do not reuse a posting id.** Ever, including one that was drafted, opened for
a day and withdrawn. A person record may point at it for years.

**Do not turn the application form on early.** `PARIDYUM_BACKUPS_CONFIRMED=1`
is not a feature flag to be flipped so a demo works. It asserts that a restore
has actually been rehearsed. Setting it without that is how the first real
applicant's date of birth ends up in a database with no proven way back.

**Do not let candidate material leave the mailbox.** Same rule as
`HR-CHECKLIST.md`'s fourth: data this estate does not know about cannot be
aged out, cannot be erased on request, and will not appear in any report that
says it is time to.

---

## What this does not do for you

It does not post to a job board, screen anyone, schedule an interview, send an
acknowledgement or a rejection, or track a pipeline. There is no `docket post`,
no `docket apply` and no `docket shortlist`, and this file does not pretend
otherwise — sections 1 through 4 are entirely manual and the commands begin at
section 5.

## What changes when applications open

When the off-host backup gate clears and `PARIDYUM_BACKUPS_CONFIRMED=1` is set,
sections 3 and 4 change and nothing else does. Applications land in the tenanted
store instead of a mailbox, section 3's "do not copy it anywhere" becomes
enforced rather than advised, and the retention clock in ADR-0073 starts running
on its own instead of depending on someone remembering.

Sections 1, 2, 5 and 6 are unaffected. **This playbook does not describe that
future state anywhere else, deliberately** — a checklist that mixes what works
with what is planned is one a person cannot follow.

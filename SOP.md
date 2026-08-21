# SOP — tool-docket

Covers onboarding a person, issuing and verifying their documents, moving them
through the lifecycle, and archiving or erasing what they leave behind.

Every command takes `--root <repo>` if you are not running from the repository.
`DOCKET_STORE` and `DOCKET_PEOPLE` move the records; they default to `store/`
and `people/` beside the binary.

---

## Task: enrol a new intern, start to finish

**When:** someone has accepted an internship. This is the whole enrolment; the
tasks below are the individual steps if you need them separately.
**Preconditions:** the repository checks out clean — verify with `docket check`

    # 1. Open a record. Allocates their permanent identifier.
    docket new asha-verma intern

    # 2. Fill it in. Every field is declared in registry/fields.yaml with a
    #    stated purpose; anything not declared there is rejected.
    $EDITOR people/asha-verma.yaml

    # 3. Check it BEFORE issuing. This is where a missing date of birth, an
    #    unevidenced unpaid basis or an over-collected field is caught.
    docket check

    # 4. Issue everything their stage owes.
    docket onboard asha-verma

    # 5. Build the folder you actually hand over.
    docket pack asha-verma

    # 6. Send them store/packs/PAR-P-.../ — or just 00_Read_Me_First.pdf and
    #    the folders. Then, as each signed document comes back:
    docket sign PAR/NDA/2026/0001 --method wet-ink --on 2026-08-25 \
      --scan ~/scans/nda-signed.pdf

    # 7. Once the agreements and policies are back, move them to active.
    #    This REFUSES while any of them is merely issued.
    docket advance asha-verma

    # 8. Record access as you grant it, not at offboarding.
    docket event asha-verma access_granted --detail "GitHub -- Paridyum org"

    # 9. Refresh the archive index.
    docket index

**Correct output** at step 5:

    store/packs/PAR-P-000001-X
      00_Read_Me_First.pdf                    PAR/PCK/2026/0001
      01_Sign_and_return/
        01_Internship_Offer.pdf               PAR/OFR/2026/0001
        ... nine documents ...
      02_Complete_and_return/
        01_Intern_Onboarding_Form.pdf         PAR/ONB/2026/0001
        02_Emergency_Contact_Form.pdf         PAR/EMC/2026/0001
      03_For_your_records/
      04_Optional/
      MANIFEST-sha256.txt

**Verify:** `docket status asha-verma` at each stage, and from inside the pack
`shasum -a 256 -c MANIFEST-sha256.txt`.

**When it goes wrong:** see the individual tasks below — each step has its own
failure table.

---

## Task: create a record and issue the pack

**When:** you want the steps separately rather than the whole enrolment.
**Preconditions:** the repository checks out clean — verify with `docket check`

    docket new asha-verma intern
    $EDITOR people/asha-verma.yaml
    docket check
    docket onboard asha-verma

**Correct output** from `docket new`:

    PAR-P-000001-X  asha-verma  class intern
    record: /path/to/people/asha-verma.yaml

    Fill it in, then: docket check && docket onboard asha-verma

and from `docket onboard`:

    asha-verma -- stage "preboarding" -- issuing 9: OFR AGR NDA IPA ISP PSH DPN ONB EMC

      OFR  PAR/OFR/2026/0001      issued/PAR-P-000001-X/OFR-PAR-OFR-2026-0001.pdf
      ...
    9 issued. Register: /path/to/store/register/issued.jsonl

**Verify:** `docket status asha-verma` — must end `nothing outstanding` and exit 0.

**When it goes wrong:**

| What you see | What it means | What to do |
|---|---|---|
| `slug "x" is already allocated` | the slug was used before, possibly by someone since erased. Slugs are reserved forever so a previous person's documents can never reattach | choose another slug |
| `identity.date_of_birth is required and is not set` | the record is incomplete. DPDP s2(f) turns on age, so this is not optional | fill it in |
| `this person is under eighteen ... DPDP s9 requires verifiable guardian consent` | they are a child; guardian consent is owed before anything is processed | add the `guardian:` block, then re-run |
| `org.jurisdiction.forum is unset` | the governing-law clause would name no court | set it in `registry/org.yaml` |
| `statutory.pan must not be collected` | a field was recorded that this class forbids | delete it; no statute requires it for an unpaid intern |

---

## Task: check a certificate someone has presented

**When:** a university or employer asks whether a document is genuine.
**Preconditions:** you have the reference and the verification code from the document footer

    docket verify PAR/CRT/2026/0001

**Correct output:**

    serial     PAR/CRT/2026/0001
    document   Certificate of Internship
    issued_on  2026-08-19
    code       53E1C9D2CD03   <- must match the code printed on the document
    file       /path/to/store/issued/PAR-P-000001-X/CRT-PAR-CRT-2026-0001.pdf
    expected   c72d09af...
    actual     c72d09af...

    MATCH: this file is the document that was issued.

To check a copy someone sent you rather than the stored one, add
`--file /path/to/their.pdf`.

**Verify:** exit code 0 means match; 1 means the file is not what was issued.

**When it goes wrong:**

| What you see | What it means | What to do |
|---|---|---|
| `PAR/... is not in the register` | no such document was ever issued | the document is not genuine, or the reference was mistyped |
| `MISMATCH` | the file differs from the one issued under that serial | do not accept it; ask for the original |
| the printed code differs from `code` | the document was altered or fabricated | as above |

---

## Task: an intern is hired as an employee

**When:** an internship ends and the same person joins in another capacity.
**Preconditions:** their current engagement is closed — `docket status <slug>`

    docket separate asha-verma
    docket reengage asha-verma employee
    $EDITOR people/asha-verma.yaml     # the employee class requires more fields
    docket check
    docket onboard asha-verma

**Correct output:**

    person     PAR-P-000001-X  (unchanged -- the archive stays whole)
    engagement PAR-EMP-2027-0001  class employee

**Verify:** `docket record asha-verma` — one person, the new engagement, and
every document from the internship still listed against them.

**When it goes wrong:**

| What you see | What it means | What to do |
|---|---|---|
| `is still in an open engagement` | they were never separated | `docket separate` first |
| `compensation.basis is required` | the employee class requires fields the intern class did not | fill them in |
| `is erased; their data was deleted on request` | nothing can be re-engaged | this is final |

---

## Task: record a system being granted or removed

**When:** an account is issued or revoked. Do it at the time, not at offboarding.
**Preconditions:** the person has an identifier — check with `docket record <slug>`

    docket event asha-verma access_granted --detail "GitHub -- Paridyum org"
    docket event asha-verma access_revoked --detail "GitHub -- Paridyum org"

**Correct output:**

    2026-09-01  PAR-P-000001-X  access_granted  GitHub -- Paridyum org

**Verify:** `docket record asha-verma` — the "access still open" list is derived
from these events, and the offboarding checklist prints it.

**When it goes wrong:**

| What you see | What it means | What to do |
|---|---|---|
| `unknown event "x"; known: ...` | the event kind is not one the log declares | use one of the names it lists |
| `x has no identifier` | the record was created by hand rather than by `docket new` | nothing ties it to a person; recreate it properly |
| `is erased; no further events may be recorded` | their data was deleted on request | correct — do not record against a tombstone |
| the detail differs slightly from a previous grant | `access_revoked` matches on the detail string exactly, so a typo leaves the grant open | re-run the revocation with the detail copied from `docket record` |

---

## Task: work out what to send, and to whom

**When:** at every stage. This is the "what do I actually email them" question.
**Preconditions:** the person has a record — `docket record <slug>`

    docket emails asha-verma                 # everything due at their stage
    docket emails asha-verma --only pack     # just one

**Correct output:** per message, the timing note, the address from the record,
the filled subject and body, and every attachment resolved to a serial:

    [pack]  stage preboarding  ->  intern
    WHEN     On receiving the signed acceptance. Not before: ...
    TO       asha.verma@example.com
    SUBJECT  Your Paridyum internship documents — please read before signing
    ATTACH   NDA  Non-Disclosure Agreement    PAR/NDA/2026/0001

**Verify:** exit 0 means every named attachment exists. Exit 1 means at least
one does not, and it says which.

**When it goes wrong:**

| What you see | What it means | What to do |
|---|---|---|
| `ATTACH  PCK  ...  NOT ISSUED` | a document the message names has not been issued | run `docket pack <slug>` (for PCK) or `docket onboard <slug>` |
| `[unresolved placeholders: end_date]` | the record has no value for it, so the message has a hole | fill the field in `people/<slug>.yaml` |
| `TO  (no address recorded)` | no email in the record for that recipient | add `contact.email`, `guardian.email`, or `education.email` |
| a message you expected is absent | it belongs to a later stage, or its condition is unmet | check `engagement.stage`; the guardian message needs a guardian, the institution one needs `institution_requires_noc` |
| `no message "x" is due` | wrong id, or wrong stage | run without `--only` to see what is due |

Messages are **cumulative**: reaching `active` still lists the preboarding
messages, because reaching a stage is not evidence they were sent.

---

## Task: assemble the folder to hand over

**When:** after `docket onboard`, and again whenever a document is reissued.
**Preconditions:** the person has documents issued — `docket status <slug>`

    docket pack asha-verma
    docket pack asha-verma --out ~/Desktop        # somewhere else

**Correct output:** the folder tree above, ending

    14 documents. Verify the folder with: cd <path> && shasum -a 256 -c MANIFEST-sha256.txt
    The store stays authoritative; this is a copy you can hand over or delete.

Folders are grouped by **what the person must do**, not by document type:

| Folder | Meaning |
|---|---|
| `00_Read_Me_First.pdf` | generated index: what is here and what comes back |
| `01_Sign_and_return` | both parties sign; a signed copy returns |
| `02_Complete_and_return` | fill in and send back |
| `03_For_your_records` | theirs to keep |
| `04_Optional` | genuinely optional; declining changes nothing |

The grouping is derived from `lifecycle.yaml`, so a new document lands in the
right folder without anyone editing a list.

**Verify:** from inside the pack, `shasum -a 256 -c MANIFEST-sha256.txt` — every
line must read `OK`.

**When it goes wrong:**

| What you see | What it means | What to do |
|---|---|---|
| `has no issued documents` | nothing to pack | `docket onboard <slug>` first |
| `issued PDF is missing from the store` | the store lost a file the register expects | restore it from backup, then re-pack |
| a document is in `04_Optional` you expected to be required | no stage owes it | that is deliberate for consent documents; otherwise add it to a stage's `owes` in `lifecycle.yaml` |
| a stale file from a previous pack | it cannot happen — the folder is cleared and rebuilt | if you see one, it was added by hand after packing |

---

## Task: record a document coming back signed

**When:** an executed copy is returned. Do it on receipt, not in a batch later.
**Preconditions:** the document was issued — `docket status <slug>` shows it `[~]`

    docket sign PAR/NDA/2026/0001 --method wet-ink --on 2026-08-25 \
      --received 2026-08-26 --scan ~/scans/nda-signed.pdf

For an eSign, no scan is needed:

    docket sign PAR/AGR/2026/0001 --method aadhaar-esign --on 2026-08-25

**Correct output:**

    PAR/NDA/2026/0001  Non-Disclosure Agreement
    signed on   2026-08-25  by wet-ink
    received on 2026-08-26
    scan        scans/PAR-P-000001-X/PAR-NDA-2026-0001-signed.pdf
                sha256 deb71980a7d94047df550e9fb451f2f44615cda2ac05e07ee7b1a459c4e4e3cd
    person      PAR-P-000001-X

**Verify:** `docket status <slug>` — the document moves from `[~]` to `[x]` and
drops out of "awaiting the executed copy back".

**When it goes wrong:**

| What you see | What it means | What to do |
|---|---|---|
| `method wet-ink leaves no evidence of its own` | wet ink has no trail but the scan | pass `--scan`, or record the method that was actually used |
| `was issued on X but the signing date given is Y` | a document cannot be signed before it exists | check the date on the returned copy |
| `the received date precedes the signing date` | the dates are the wrong way round | swap them |
| `is already recorded as executed` | a signature is recorded once | if the first record was wrong, record why in a note; do not overwrite it |
| `is not in the register` | nothing was issued under that serial | check the serial on the document |

---

## Task: move a person to the next stage

**When:** the current stage is finished — typically preboarding into active.
**Preconditions:** every document the stage requires executed is recorded as such

    docket advance asha-verma

**Correct output:**

    asha-verma: preboarding -> active

**Verify:** `docket status <slug>` reports the new stage, and `docket record
<slug>` shows a `stage_changed` event.

**When it goes wrong:**

| What you see | What it means | What to do |
|---|---|---|
| `refusing to move ... N document(s) are not recorded as executed` | the gate is holding. Access must not be granted before the agreements come back | `docket sign` each one it names |
| `... but not recorded as executed` | that document went out and has not returned | chase it |
| `is at "alumni", the last stage` | there is nowhere further to go | nothing to do |
| `is separated; only an active engagement advances` | the engagement is closed | use `docket reengage` if they are returning |
| you must proceed without a document | a decision, not a workflow problem | `--force`. It prints that the gate was not checked, and the event log records the move |

---

## Task: refresh the archive index

**When:** after issuing anything, and before handing the store to anyone.
**Preconditions:** the registers are intact — check with `docket check`

    docket index

**Correct output:**

    /path/to/store/index/roster.csv
    /path/to/store/index/documents.csv
    /path/to/store/index/README.txt
    /path/to/store/manifest-sha256.txt

    1 people, 9 documents indexed.
    The index is generated output: regenerate it, never edit it.

**Verify:** from inside the store, `shasum -a 256 -c manifest-sha256.txt` —
every line must read `OK`. This works with no Paridyum tooling at all, which is
the point of it.

**When it goes wrong:**

| What you see | What it means | What to do |
|---|---|---|
| `store/index was stale and has been regenerated` from `docket check` | the index no longer matched the registers — usually because it was edited, or documents were issued after the last `docket index` | it has already been rebuilt; never edit it by hand |
| `manifest: <file> does not match its manifest hash` | an issued PDF changed on disk after it was issued | restore it from backup; the register records what it should be |
| `manifest: <file> listed in the manifest but missing` | an issued PDF was deleted | restore it, or if the deletion was intended, note that the register still refers to it |
| `FAILED` from `shasum -c` | as above, detected without any Paridyum tooling | same |

---

## Task: serve the API for the HR console

**When:** an HR operator needs `console-docket` rather than the command line.
**Preconditions:** the store is readable, and nothing else holds the port —
check with `docket check` and `lsof -nP -iTCP:8080 -sTCP:LISTEN`

    docket serve --port 8080

**Correct output:**

    docket serving on http://127.0.0.1:8080
    reach it from another machine with:  ssh -L 8080:127.0.0.1:8080 <this-host>

It runs in the foreground until you stop it with Ctrl-C. Nothing is written to a
log file; connection errors print to stderr where you can see them.

**Verify** — from another terminal, and note this is a different command from the
one that started it, because a server reporting its own readiness is not evidence:

    curl -s http://127.0.0.1:8080/api/health

    {"ok":true,"version":"0.1.0"}

**Then verify it is NOT reachable from anywhere else.** This is the whole access
model, so check it rather than trusting it:

    lsof -nP -iTCP:8080 -sTCP:LISTEN

`127.0.0.1:8080` is correct. `*:8080` would mean anyone who can route to this
machine has full read access to every personnel record, and `docket` will not
produce that — but check, because the equivalent default in a Node dev server
does exactly that.

**How an operator reaches it from their own machine:**

    ssh -L 8080:127.0.0.1:8080 <this-host>

Then they open `http://127.0.0.1:8080` on their own machine. The SSH key is the
authentication and the operating system account is the identity; there is no
login in the application, deliberately (ADR-0047 — this tool has no identity
system of its own, so it must not be reachable without one).

**What it serves.** Reads: `/api/health`, `/api/roster` (optionally
`?standing=active`), `/api/person/{slug}`, `/api/check`, `/api/retention`,
`/api/emails/{slug}`, `/api/form/{class}`. Writes: `POST /api/person` (enrol),
`/api/person/{slug}/onboard`, `/sign`, `/advance`. Every route calls the same
`application::` function the CLI calls, so the console and the command line cannot
disagree about what a person owes.

**Every write needs an operator, or it is refused.** Send an `X-Docket-Operator`
header naming someone in `registry/operators.yaml`:

```
curl -X POST -H 'X-Docket-Operator: ruhani' -d '{}' \
  http://127.0.0.1:8080/api/person/asha/onboard
```

Without it the answer is 403 and nothing is written. This is the same attribution
the CLI's `--by` provides and no stronger: it names a real operator, and SSH is
what proves who is at the keyboard. It is also what stops a page in the operator's
browser forging a write, since a cross-site form cannot set a custom header.

**A 409 is a refusal, not a failure.** It means the request was understood and the
domain declined — a gate not met, a record not filled in. The body carries
`blockers` and `refusals` saying which. A 400 means the request was malformed and a
500 means something is broken; those are different problems.

**Writes cannot be forced from here.** `onboard` has no force at all: fix the
record, or use the CLI where `--force` is typed out and visible in shell history.

**When it goes wrong:**

| What you see | What it means | What to do |
|---|---|---|
| `could not bind 127.0.0.1:8080; is another docket already serving?` | the port is taken, often by a `docket serve` you forgot | `lsof -nP -iTCP:8080 -sTCP:LISTEN`, then stop it or pick another port |
| `refusing to bind <addr>: this server has no authentication of its own` | something tried to bind a routable interface | correct, and not overridable. Use the SSH forward |
| `HTTP/1.1 404` with `no person with slug X` | the slug is not in the ledger | `docket roster` for the real slugs; 404 means mistyped, 500 would mean broken |
| `HTTP/1.1 405` | right path, wrong method — e.g. a POST to a read route | no write routes exist yet; use the CLI for writes |
| `HTTP/1.1 400` and `Transfer-Encoding is not supported` | a client sent a chunked body | send `Content-Length`. This server refuses what it does not implement rather than guessing |
| `connection error: ...` on stderr, server still running | a client hung up mid-request | ordinary; a failed connection never stops the server |
| the console shows stale data | there is no cache here; every request reads the store | reload. If it persists, the console is caching, which it must not |

---

## Task: find documents past their retention period

**When:** periodically, and before any archive transfer.
**Preconditions:** the register is readable — check with `docket check`

    docket retention

**Correct output** when nothing is due:

    unevaluated  permanent (indefinitely)

    nothing past its retention period

**Verify:** exit 0 means nothing is due; exit 1 lists what is.

Classes reported as `unevaluated` have a period this cannot parse from prose.
They are shown rather than assumed compliant, because silently treating a
period as satisfied is how over-retention becomes invisible.

**When it goes wrong:**

| What you see | What it means | What to do |
|---|---|---|
| `OVERDUE ...` | a document is past the period its retention class states | decide whether to delete it; this tool will not |
| `unevaluated  <class> (<period>)` | the period is prose this cannot turn into a number | evaluate that class by hand, or restate the period in `registry/retention.yaml` as "N years" or "N days" |
| nothing listed but you expected something | the clock runs from the ISSUE date, not the last working day | check `registry/retention.yaml` states the basis you meant |

---

## What this cannot do

- **It does not sign anything.** Signing happens on paper or through an eSign
  service. `docket sign` records that it happened and holds the evidence; it
  does not produce a signature.
- **It does not grant or revoke access.** `docket advance` refuses to move a
  person out of preboarding until the agreements are executed, and `docket
  event` records what was granted — but issuing and removing accounts is done
  by hand in each system.
- **It does not delete anything on a retention deadline.** `docket retention`
  reports; deleting is a decision.
- **It does not reach a cloud folder, a mailbox, or a backup.** `docket erase`
  names what it could not reach; a person must do those.
- **It does not make a document legally sufficient.** The wording of the POSH
  policy, the Data Protection Notice and the guardian consent are drafts for
  qualified counsel.
- **It does not hold medical, disciplinary or POSH case material.** That store
  is declared in `registry/person-classes.yaml` and deliberately not written
  here: a complaint routed through a CLI anyone with repository access can run
  is a confidentiality failure.

## Destructive operations

**`docket erase <slug> --confirm`** — irreversible.

- **Destroys:** the person record (`people/<slug>.yaml`), and the detail of
  every event in their history.
- **Retains:** the person identifier, engagement identifier, class and dates,
  as a tombstone, so serials already issued still resolve to something. The
  issued PDFs are retained too; the register still refers to them.
- **Recoverable:** no. Restore from a backup of the store if you must.
- **Before running:** confirm the erasure request is genuine and that no legal
  obligation requires continued retention. Read the "did not reach" list it
  prints and act on every line, or the erasure is incomplete.

**`docket index`** overwrites the index and manifest in place. That is safe —
they are generated output (C-0004) and are rebuilt from the registers.

No command edits the register, the identity ledger or the event log in place,
except `erase`, which rewrites the event log to its skeletal events and
re-seals the chain. That is the one place the append-only rule is broken, and
it is broken because an append-only log that cannot forget is a log that cannot
honour an erasure request.

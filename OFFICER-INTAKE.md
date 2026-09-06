# Officer intake

What to send each officer to collect the data their record needs, and nothing else.

Four people, five documents each: the officer Non-Disclosure Agreement (`NDO`), the
Intellectual Property Assignment (`IPO`), the Code of Conduct (`COO`), the Information
Security & Acceptable Use Policy (`ISO`) and the Data Protection Notice (`DPO`).

---

## Why the form is this short

**Five questions, two of them optional.** That is not brevity for its own sake — under the
DPDP Act personal data may be processed for a stated purpose and no other, and
`registry/person-classes.yaml` refuses a field the officer class does not permit. The
template ships more `FILL-ME` placeholders than this form asks about, because it is shaped
for interns.

Between them the five templates read exactly two things from the record:
`identity.legal_name` and `pre_existing_ip`. Everything else below is needed by a rule
rather than by a document, and anything needed by neither is not asked for:

| Not asked | Why not |
| --- | --- |
| Phone, home address | Printed on no officer document, required by no class rule. Collecting them would be over-collection. |
| LinkedIn, GitHub, skills | Same. The public officers page reads `registry/team.yaml`, not a person record. |
| Education, guardian | **Forbidden** for the officer class. `docket check` rejects a record carrying them. |
| PAN, Aadhaar, bank, salary | **Forbidden.** No officer is paid, so nothing statutory attaches. |
| Their office | Already in `registry/signatories.yaml`. Asking again invites two answers. |

**Delete the placeholders you did not ask about.** Every `FILL-ME` blocks issuance, whether
or not the class requires that field. Two ways to clear one: fill it, or delete the line.
For phone, address and the professional block, **delete** — not collecting is better than
collecting something nobody needs.

---

## Sending it

Send it to each officer individually. Their reply contains personal data, so:

- **Do not paste a reply into a repository, an issue, or a chat.** `people/` is gitignored
  because git history cannot be erased, and an unerasable record makes the DPDP erasure
  right unhonourable. A reply pasted anywhere else defeats that.
- Type the answers straight into `people/<slug>.yaml` and delete the email afterwards.
- **Check the legal name against a government ID**, not against the email signature. It is
  printed on both documents and on every signature block.

---

## Mapping a reply into the record

| Answer | Field in `people/<slug>.yaml` | Why it is needed |
| --- | --- | --- |
| 1 Full legal name | `identity.legal_name` | Printed on both documents and on the signature block. |
| 2 Date of birth | `identity.date_of_birth` | Decides whether DPDP s9 applies at all. `docket` refuses to issue without it — not knowing is a different state from being an adult, and must not be silently treated as one. |
| 3 Email | `contact.email` | Required by the officer class; where documents are sent. |
| 4 Appointment date | `engagement.start_date` | The date the office took effect. No end date — an office is held until vacated. |
| 5 Pre-existing IP | `pre_existing_ip` | Section 4 of the `IPO` is what keeps it theirs. See below. |
| 6 Preferred name | `identity.preferred_name` | Optional. |
| 7 Pronouns | `identity.pronouns` | Optional; defaults to they/them. Never inferred from a name. |

Also set `engagement.role` yourself, from `registry/signatories.yaml`, so the record and the
signatory register cannot give two answers.

### Question 5 is the one that needs a real answer

`people/_TEMPLATE-officer.yaml` ships `pre_existing_ip: None`, and for a founder that is
very often **wrong**. Anything you wrote before Paridyum and have brought into it — a
library, a tool, a design system, a prototype — is yours, and section 4 of the `IPO` is what
keeps it yours. Signing `None` when it is not `None` assigns more than anybody intended, and
it is not fixed by editing the record afterwards: the document has been signed.

Nobody will volunteer this unprompted, which is why the question says so in plain words.

---

## The form

Copy everything between the rules into an email. One per person.

---

**Subject: Your Paridyum officer record — five questions**

Before you sign your officer documents, Paridyum needs a few details for your record. There
are five questions and two optional ones. Please reply to this email with your answers.

Five documents are prepared from this: a Non-Disclosure Agreement, an Intellectual Property
Assignment, a Code of Conduct, an Information Security & Acceptable Use Policy, and a Data
Protection Notice. All are signed on Paridyum's behalf by another officer — nobody signs
their own.

**1. Your full legal name, exactly as it appears on your government ID.**
This is printed on both documents and on the signature block. Not a short form, not the name
you use day to day — the one on the ID.

**2. Your date of birth.** (YYYY-MM-DD)
Required by law rather than by us: the Digital Personal Data Protection Act treats anyone
under eighteen differently, and Paridyum has to be able to tell.

**3. The email address to use for your records.**

**4. The date your appointment took effect.** (YYYY-MM-DD)
The day you started holding the office, which may be earlier than the day the paperwork
catches up. There is no end date — an office is held until it is vacated.

**5. Any intellectual property you already own that you have used, or expect to use, in
Paridyum work.**

Read this one properly. If you wrote code, built a tool, designed something, or created any
other material **before Paridyum, or outside your Paridyum work**, and it has gone into a
Paridyum project — list it here. Anything you list stays yours; the IP assignment carves it
out explicitly.

If you list nothing, you are stating there is nothing, and the assignment covers everything
you have made for Paridyum. That is fine if it is true. It often is not, and it cannot be
corrected after the document is signed.

Answer `None` only if you mean it. Otherwise a short list is enough — a name and one line
each.

**6. Preferred name** *(optional)* — what you would like to be called, if it differs from
your legal name.

**7. Pronouns** *(optional)* — we default to they/them rather than guessing from a name.

---

**What Paridyum does with this.** These five facts go into your personnel record, which is
used to prepare and issue your officer documents and to keep the register of what has been
issued to whom. They are not shared outside Paridyum, and are not published — the public
officers page carries only your name, your office and a summary you can review. Your record
is held on Paridyum's systems and deliberately kept out of version control, so that it can
actually be corrected or erased if you ask. To see, correct or erase what is held about you,
reply to this address.

*(The full Data Protection Notice for officers is one of the five documents you will
receive. This paragraph is what is owed at the point of collection, before that notice is
issued.)*

---

## After the replies come back

    export DOCKET_OPERATOR=rohan
    scripts/bootstrap-officers.sh          # opens four records
    $EDITOR people/*.yaml                  # fill from the replies, delete what you did not ask
    docket addresses <slug>                # per person: the two accounts to create
    scripts/bootstrap-officers.sh --issue  # checks, then issues all five for each

`docket addresses` needs the legal name filled in first — it refuses to derive an address from
a placeholder, because an address may never be renamed once issued.

Then get them signed and record each one:

    docket sign PAR/NDO/2026/0001 --method wet-ink --on <date> --scan <file>

`SOP.md` has the full sequence with a failure table per step.

# Logbook — tool-docket

How this tool got the way it is, recorded as it was built rather than
reconstructed afterwards (RFC-0033). The `CHANGELOG.md` says what changed;
this says what was decided, what was wrong on the way, and what it cost.

Started 2026-09-06, part-way through the tool's life. Everything before that
date lives in `CHANGELOG.md` and `DECISION-DEBT.md`, which were doing this
job less directly.

## 2026-09-06 — officers, and the four templates that could not be shared

The ask was job descriptions and job IDs for the openings, wired to docket.
Two gaps sat behind it, and the second was not the one anybody was looking for.

**The wiring did not exist.** ADR-0063 decision 4 names the seam — a person's
`role` should reference a posting — and explicitly does not perform it.
`engagement.role` was free prose whose stated purpose is what the agreement,
the certificate and the reference letter print. So `engagement.job_id` is a new
optional field beside it rather than a replacement: those are typeset documents
and must not depend on a JSON file in another repository resolving at render
time. RFC-0041 has the argument, including why the posting identifier had to be
frozen before anything could reference it.

**Then the officers.** CEO, CTO and Director existed in `operators.yaml` and
`signatories.yaml` and nowhere else — no person record, no NDA, no IP
assignment, while every intern will sign all three. The instinct behind the
question ("we will hire our existing through this pipeline?") was right about
the gap and wrong about the mechanism: an officer is appointed, not offered, and
running one through `docket onboard` would have issued an Internship Offer and
a Certificate of Internship to a director and had the CEO sign her own offer
letter.

**What implementation found that the design got wrong.** RFC-0042 first claimed
`NDA`, `COC`, `ISP` and `DPN` applied to an officer "unchanged". They do not.
`nda.tex` says "Intern" sixteen times and "internship" six;
`infosec-acceptable-use.tex` twenty-eight and eleven; `code-of-conduct.tex`
states "This internship is unpaid and part-time", which is false of a director.
The party term is one macro away — but "the internship" is the *arrangement*,
not the person, and generalising it would have edited the operative text of
documents interns have already been issued, for the convenience of a different
class of person. So officers got their own two documents and the RFC was
corrected rather than the code bent to match it. The cost is recorded as D-46:
confidentiality prose now exists twice and nothing checks that it agrees.

**Three things had to be class-aware, and the middle one is the one that would
have shipped broken.** Documents (`classes:`, so a false document is refused);
the **stage gate** (`preboarding` requires the internship agreement and intern
IP assignment executed, so an unfiltered gate would have held an officer
forever, waiting on a document nothing would ever produce); and the record
template (`_TEMPLATE.yaml` carries education and a guardian block, both of
which the officer class forbids, so copying it produces a record `docket check`
rejects the instant it is written — which reads as a product defect rather than
as the registry working).

**Nobody signs their own document.** `signed_by_class` routes officer documents
to the Director, and `docket issue` refuses where subject and signatory resolve
to the same person. Verified against the **rendered PDF text**, not the
register: the register can be right while the paper is wrong, and the paper is
what somebody signs. `\paridyumsignblock` was added so the block resolves from
the document rather than the template naming the CEO/CTO pair in twenty-one
places.

**On the IP question, the operator overruled the design twice and the second
version is simpler.** The first draft proposed a founders' agreement with an
assignment-on-incorporation undertaking, then assignment to the Director as
nominee and trustee — both answers to "you cannot assign to a company that does
not exist" (`org.yaml` has `cin: null`; Internship Agreement §13 records
Paridyum as operated by Priyanshu Sharma personally). The operator's decision:
write everything in the company's name and treat it as in effect from
signature, as it would be after incorporation. That is what every intern
document already does, so it adds no new exposure — it extends the existing one
to the founders' own work product. Copyright needs no filing to subsist, so
nothing is lost by not registering early. Recorded in RFC-0042 §3 as an
operator decision rather than a legal finding, and flagged for counsel at
incorporation.

**Left undone, deliberately.** `COC`, `ISP` and `DPN` do not exist as officer
documents (D-46) — an officer holds the most access in the estate and has
agreed to no acceptable-use policy. `engagement.job_id` is validated by nothing
(D-47). ADITYA SHRIVASTAVA is recorded as COO with `appointed_on` and
`instrument` both `null`, because the appointment was stated and the instrument
was not produced; C-0004 and RFC-0042 §5 both want one, and the legal-name
spelling has not been checked against it.

Verified with `scripts/gates.sh` — fmt, clippy, 313 tests, `docket check` —
all green, run through the script rather than by hand, because the changelog
entry for that script exists precisely because piping `cargo fmt --check` into
`head` returns the exit code of `head`.

Skills used: `paridyum-platform:design-change` for the change lifecycle.
RFC-0041 and RFC-0042 were written, reviewed and accepted before any code in
this repository was touched.

## 2026-09-06 — the bootstrap, and what running it found

Four officers to onboard, run by one operator. Written as `scripts/bootstrap-officers.sh` in two
passes: open the records, stop; then, once a human has filled them in, check and issue. It does
not invent personal details, for the reason `_TEMPLATE.yaml` carries placeholders at all.

**Running it in a sandbox found two defects that reading the code did not.**

`blockers` refused every officer for `education.currently_enrolled is false` and
`engagement.end_date is unset`. Both are the intern class's preconditions and say so in their own
message text; `person-classes.yaml` forbids `education` for an officer and requires no end date,
so the refusals were **unsatisfiable** — no edit clears them without breaking a different rule.
Scoped to the intern class, with the DPDP s9 child check deliberately left unscoped because it is
about processing a child's data rather than about internships.

And the refusal for a self-signed document, correct as a rule, left the Director with no
documents at all. Substitution replaced it: the first active signatory in registry order who is
not the subject. Registry order rather than anything cleverer, because the substitute has to be
the same person every time — who signed a legal document must not depend on hash order.

**The sandbox mattered for a second reason.** `docket new` allocates a person identifier that is
never reused (ADR-0051), so a test run against the real store would have burned four of them.
`DOCKET_PEOPLE` and `DOCKET_STORE` made a throwaway store possible; the real run is the
operator's.

Verified: all eight documents issued in the sandbox, the register recording `signed_by=['director']`
for three officers and `['ceo']` for the Director, and `pdftotext` confirming **RUHANI KHURANA,
Chief Executive Officer (CEO)** in the Paridyum block of the Director's own NDA with PRIYANSHU
SHARMA countersigning as Officer. Sandbox removed; nothing was written to the real store.

## 2026-09-06 — the three officer policies, and the two that were inverted

D-46's open half: `COC`, `ISP` and `DPN` did not exist for officers, so the four people with
the most access in the estate had agreed to no acceptable-use policy. Written as `COO`, `ISO`
and `DPO`.

**The interesting part is that a word-swap would have been wrong in both directions.** Going
through the intern text section by section, four differ in substance and two are inverted:

*Speaking publicly* — the intern code says they may not speak on Paridyum's behalf. An officer
may; that is part of the office. Copying the intern clause would have forbidden a director from
doing their job, so the officer version is about care instead: a claim must be true when made,
not merely intended, and the officers' names are on the public site, which is what makes it
matter more for them than for anyone else.

*Access* — the intern policy grants access to what an assignment requires. An officer's access
is administrative and broad by necessity, so the boundary cannot be drawn by somebody else. The
officer version makes least privilege a discipline they impose on themselves: **being able to
reach something is not a reason to.** Copying the intern clause would have described a
permission model the estate does not have.

Two routes could point at their own subject, which is the same defect the signature
substitution exists to prevent, in a different place. `org.yaml` routes security incidents to
the CTO — so an incident involving the CTO had nowhere to go. And the Grievance Policy is
`classes: [intern]`, so it cannot be the route for a concern about an officer. Both now name
another officer, and the ISO says why in one line.

*Breach* — an office is vacated or removed by the directors. It is not terminated, and there is
no Internship Agreement to point at, so the intern clause was simply describing a thing that
cannot happen.

**`DPO` differs in what it says is collected, not only in wording.** The intern notice lists
education details, emergency contact and a supervisor; the officer class forbids `education` and
`statutory` and does not permit `emergency_contact`. A data protection notice that overstates
what is held is a defect in the notice — its entire purpose is to be an accurate account of the
processing. So section 1 is derived from what the class actually permits, and it adds the one
thing an officer has that an intern does not: two facts about them are *published*, not merely
held.

Verified by rendering all five officer documents and reading the PDF text, not the source. The
remaining "intern" strings in the output are deliberate cross-references — "differs from the
copy given to an intern", "the Grievance Policy is an intern document" — plus one false positive
on "internal business information".

Left open and narrowed rather than closed: D-46 is no longer a coverage gap, it is a
duplication. Five officer documents now carry text adapted from five intern ones, and nothing
compares them. Diffing fails immediately because they are supposed to differ.

Gates green: fmt, clippy, tests, `docket check` — 26 documents, 0 errors.

## 2026-09-06 — addresses, and the subdomain that would have repeated a locked mistake

Four officers need email, interns follow, and nothing said what an address looks like. C-0007
governs repositories, crates, API paths, tables, environment variables and images — not
mailboxes. Three function addresses existed by custom: `legal@`, `hello@`, `security@`.

The shape proposed was `vivaswan.pdol001.paridyum@gmail.com`, with `intern.paridyum.com`
floated for the domain half.

**The subdomain is the part worth recording.** `intern.paridyum.com` is ADR-0051's locked
mistake in a worse place. That ADR rejected `PAR-INT-2026-0001` as a person identifier because
`INT` is a class and class is mutable for a person — "the predictable case is the one that
breaks it: an intern is hired." An address is a far worse place to make it than a folder name,
because most copies are not yours: they are in other people's address books, on signed
documents, in university records, and in commit authorship that C-0003 requires be real and
attributable. A folder can be renamed by one person. An address cannot.

The honest counter is sending reputation, and it has a standard answer that does not put a
mutable fact in someone's identity: isolate bulk and automated sending on its own subdomain.
That is about what sends, not who sends.

**`pdol001` could not be decoded and was replaced rather than guessed at.** It is not the
person identifier and not the engagement identifier. Whatever it meant, docket already mints
something with exactly the properties a Gmail disambiguator needs — unique across Paridyum
forever, never reissued, and carrying a check character. Adding a second hand-kept counter
would have been a second thing that must never be reused, which ADR-0051 singles out as the
failure that silently reattaches one person's records to another.

**The Gmail account name keeps the given name, and that is a deliberate departure from
ADR-0051's opacity rule.** That rule is argued from a directory listing disclosing who Paridyum
engages to anyone who can read it. A Gmail account name is not a listing anybody browses; what
it has to be is findable in an account switcher when an operator holds twelve of them.

**The operator chose free Gmail long-term rather than Workspace, and the cost lands on
something docket was built to do.** `access.company_email`'s stated purpose is "so offboarding
can prove it was removed", and a consumer account cannot be suspended, audited or recovered by
Paridyum. Rather than let the record imply a proof that does not exist, the field now means the
published alias specifically and `access.mailbox_email` is separate: **severing an alias is
provable; recovering a consumer mailbox is a request.** Splitting them is also the mitigation —
Paridyum can end someone's ability to act as Paridyum without needing access to their mailbox.

Derivation is a domain function with six tests rather than a documented convention, including
the two cases a convention gets wrong: a three-part name keeps every part, and a name with no
ASCII yields a visibly wrong local part instead of an invented romanisation — permanent, under
a rule that forbids renaming.

`gates.sh` earned itself again: `cargo build` was clean and clippy was not (`-D warnings`), so
building and eyeballing would have recorded a failing gate as green.

## 2026-09-07 — the collision form, and a check character that did not check

The operator proposed replacing the surname with the person identifier — `firstname.<id>` —
on the grounds that it would be shorter and easier to print.

**Measured, it is neither.** Of the four real officer names, three get *longer*:
`rohan.batra@` is 24 characters and `rohan.p000147c7@` is 28. And on printability the evidence
runs the other way and comes from ADR-0051 itself, which attached a check character to the
identifier because *"this string gets typed and read aloud"* and MOD 11-2 *"catches the error
class a hand-copied folder name actually produces."* The identifier needs error-detection
machinery because people mistype it. `batra` does not.

**But the instinct was right about the collision case, and the first design was wrong there.**
When two people collide they share the surname, so `rohan.batra.p000147c7@` carries a segment
that disambiguates nothing. Dropping it is strictly better.

That let a rule be deleted rather than added. The middle-name step is gone: it existed only to
push the collision back one place, it depended on the person having a middle name, and it gave a
third shape an outsider could not predict. **Two shapes now** — `firstname.lastname@` normally,
`firstname.<id>@` on collision — and the second matches the Gmail local part, so one derivation
serves both domains.

**The test caught a fabricated example.** `p000147c3` had been used throughout the ADR, the
changelog and the doc comments; the real check character for sequence 147 is `7`. A documented
identifier that fails its own check digit is precisely the string somebody copies. Corrected in
all five places, including `identity.rs`'s own doc comment, which had carried `PAR-P-000147-3`
since before this work — the module that implements the check character was illustrating it with
a value that fails it.

Verified with `scripts/gates.sh`: fmt, clippy, tests, `docket check` — all green.

## 2026-09-07 — anti-collision by default, which deleted a subsystem

The operator's call: make the collision-proof form the default rather than the fallback. Every
address is `firstname.<person id>`.

**It removed more than it added, which is why it is the right call and my two-shape design was
not.** I had said an address register was mandatory — an append-only list of every address ever
issued, never pruned, consulted before handing out the next one, retained for people who had
left and for people whose records had been erased, because reuse delivers one person's password
resets to another. That whole structure existed to detect a collision. Derive the address from
an identifier that ADR-0051 already guarantees is unique forever and never reissued, and the
collision cannot happen: there is nothing to detect and nothing to keep.

Two smaller things fell out of it. An address is now **self-validating** — a transposed digit
fails the MOD 11-2 check character rather than bouncing silently, so the check character
ADR-0051 added for folder names earns its keep on every address. And **nobody's address is worse
because they joined second**, which the fallback scheme could not say.

**The cost is real and is recorded rather than argued away.** The address is not guessable and is
harder to read aloud. What makes that survivable is that `hello@`, `legal@` and `security@`
already exist and are what an inbound stranger actually uses — a personal address is given out,
not deduced. I had weighted guessability too heavily by treating personal addresses as the
inbound path when the function addresses already are.

The measurement from the previous round still stands and is now moot: `firstname.<id>` is not
shorter than `firstname.lastname` — it is longer for three of the four officer names. It was
never chosen for length.

Verified with `scripts/gates.sh`: fmt, clippy, tests, `docket check` — all green.

## 2026-09-07 — D-46 researched, and the answer was not to rewrite

Asked to research whether the duplicated officer/intern templates could be rewritten from
scratch. **The research says no, and it contradicts what I recommended before doing it.**

I had measured 71% of lines identical across the pairs and concluded a shared-clause refactor
was worth a day. Broken into sections, the picture inverts: **5 of 51 sections are byte-identical
and 28 differ in substance.** The line-level figure was counting `\begin{itemize}` and shared
phrasing inside sections that genuinely differ. Two sections are inverted outright.

Even with a party-term macro, 45% would be shareable — and it would put `\dkPartyTerm` into prose
a person signs, so no reviewer could read the document without evaluating a substitution in their
head. For legal text that is a worse defect than duplication.

**Across the entire template set, only seven duplicate section copies existed.** Five were
`Governing Law`, appearing in five documents, two of which differed only in where the line
wrapped. That one is now `templates/clauses/governing-law.tex`. The remaining two were within a
single pair each and not worth the indirection.

**The order mattered more than the extraction.** `template_sha256` hashed one file, and it feeds
`verification_code`, which is *printed on the document*. Extracting a clause without changing
that would have left five documents recording a hash that no longer covered their own terms.
So the closure hashing went first, with tests, and only then did a clause move — and the register
held **zero entries** at the time, which is the only moment that change is free. After the first
issuance there would be two eras of hash in one register with no way to tell them apart.

Verified by rendering all 25 documents to text before and after and diffing: **identical, not one
word changed.** That is the check that matters for a refactor of legal text, and it is the reason
to render rather than reason.

What replaced the rewrite is an **adaptation pin** — `% adapted-from: <source>@<sha256>` in each
adapted template, recomputed by the suite. It does not compare content, because the two are
supposed to differ; it makes changing one half silently impossible. Proved by amending the intern
ISP and watching the suite name the officer file and the digest to paste.

D-49 and D-48 closed in the same pass. The D-49 guard needed a second attempt to verify: removing
the `.gitignore` negation did not fail the test, because the file was already *tracked* and
tracked files ignore `.gitignore`. The test was right and my mutation was invalid — proved
properly by adding a throwaway class with an untracked ignored template, which failed with the
fix in the message.

339 tests. `scripts/gates.sh` green.

## 2026-09-07 — the vault, and the property that made it possible

Permanence and erasure look irreconcilable, and for the same bytes they are: a key that always
opens a document makes it uneraseable, and no such key makes loss permanent. Sync keeps versions,
so deleting a file there is a gesture — and this estate gitignores `people/` precisely rather
than accept an unerasable record.

**What resolved it was reading `verify` before designing anything.** It compares a PRESENTED file
against the hash recorded at issuance; the stored copy is a convenience and the signature already
takes an optional path. So shredding the archive does not destroy verifiability — an erased
person can hand over their own certificate years later and have it check out against a register
entry that survives. Without that, one of the two requirements would have had to give.

**One recipient per object, and the single-ness is the security property.** Encrypting to a
person's key and an estate key would have been recoverable *and* granular, and it is escrow in
disguise: destroying the personal key shreds nothing while the estate key still opens the file.
`crypt::encrypt` takes one recipient so that adding a second has to be a change to that function,
where the argument against it is written down.

**Erase was worse than I thought before touching it.** It deleted the record and re-sealed the
event log, and left every issued PDF sitting in `store/issued/`. So the thing docket called
erasure left the person's documents on disk. It now destroys the key, the ciphertext, the
plaintext and the record, in that order, with the tombstone written *first* — a crash between
tombstone and deletion leaves a visible inconsistency rather than a silent one.

**A test wanted to set an environment variable and the crate forbids `unsafe`.** That turned out
to be the compiler pointing at a design flaw rather than an obstacle: `estate_identity()` was
being read deep inside the vault, where a test could only reach it by faking a process
environment. Passing the identity in from the one caller that knows it needs one made the
dependency visible in the signature and the tests trivial. The lint was right.

Verified end to end in a sandbox with its own estate key: seal, confirm the object is
`age-encryption.org/v1` and not `%PDF`, erase, then verify the person's kept copy — **MATCH**.
Six integration tests, skipping rather than failing where `age` or a typesetter is absent.

`docket vault sync` prints what to upload and what must never be, because docket does not talk to
Sync: the vendor's client is Windows and Mac only (RFC-0038), and a tool that shelled out to it
would be pretending to own something it does not.

## 2026-09-07 — the half the vault had left out

The operator asked about periodic encrypted bundles to Sync, and about backing textual data into
Bitwarden with the `bw` CLI. Answering the first found a hole in what had just been built.

**`vault seal` covered issued PDFs and nothing else.** The register, ledger, event log,
signatures and tombstones — the half that is *never* erased and that makes every document
verifiable decades later — were in no uploaded path at all. The vault was protecting the
erasable half and leaving the permanent half on one laptop. `docket vault backup` fixes it,
encrypted to the estate key rather than a person's, which is also what makes it safe in a
versioned store: there is no erasure promise there for a retained old version to break.

**A whole-store bundle was rejected**, which is the shape the question implied. You cannot erase
one person from a tarball that is already uploaded and versioned, and encrypting it to one key
makes everyone's data recoverable with that key forever — the escrow ADR-0069 turned down.

**Bitwarden for textual data was rejected on RUN-0004's own stated property:** *"There is exactly
one thing to lose and exactly one thing to back up."* Backing arbitrary data there turns one
thing to protect into many. Worse, the password manager holds the key — putting the data beside
it means one compromise gets both. And Bitwarden has a trash and sync history, so it is not a
medium you can reliably delete from, which is the same flaw that keeps keys out of Sync.

**The bundle lists its five files individually rather than globbing `store/`.** A glob would have
taken `keys/`, and a key in a versioned store cannot be destroyed — after which no erasure this
system performs is real. There is a test asserting nothing under `keys/` reaches the uploaded
path, because that is the mistake that would be silent.

Recovery is rehearsed rather than asserted: a test decrypts the bundle with nothing but the
estate key and checks the serial and hash come back. That is the whole of what `verify` needs, so
**the estate key alone restores verifiability for every document ever issued** — including for
people whose vaults have been shredded and who hold the only copy.

Still open and now sharper (D-50): the per-person keys have no backup anywhere. Losing the laptop
today would leave the register intact and every person's documents unopenable — you would know
exactly what was issued and be unable to produce any of it.

347 tests. Gates green.

## 2026-09-07 — D-50, and the thing that was actually unprotected

Asked to handle "the per-person keys have no backup". Two findings, and the first says I had
described the problem wrongly.

**The tension is structural, not a gap.** Crypto-shredding needs key material that can genuinely
be destroyed. Any copy of that material improves durability by exactly as much as it damages the
erasure promise. I went looking for a scheme that escapes it — deriving keys from the estate key
plus a salt, so nothing needs backing up — and it fails for the same reason in a different place:
the salt then has to be destroyable, so it cannot live anywhere that retains deletions either.
There is no design that gets both. What a design can do is make the choice explicit, which is
what `docket vault escrow` is, with both columns written into ADR-0069 D6.

**I also checked whether the registry already resolved it, and it half did — the other way.** The
`permanent` retention class says *"register metadata is retained even when the underlying personal
data is erased."* So I could not move permanent-class PDFs to the estate key to make them durable:
the registry authorises keeping the metadata, not the document. The split as built is right.

**The second finding is the one that mattered.** Chasing "what is actually irreplaceable" found
that **signed scans were protected by nothing at all.** `seal` walked the register, which lists
issued PDFs; scans live in `store/scans/` and were in neither the vault nor the register bundle,
so they were in nothing that ever left the laptop. And `erase` deleted the issued PDFs while
leaving them — **erasing the copies the person already holds and keeping the one they do not**,
which is the inverse of what erasure is for.

They are the half that cannot be recovered from the counterparty. A document was handed over in a
pack; a returned scan exists nowhere else, and `sign.rs` says why: *"wet ink leaves no trail of
its own, so the scan IS the evidence."*

**A bug the test found, not me.** `vault status` reported "every issued document is sealed" when
nothing was sealed. The guard that skips a shredded person's documents keyed on the key file being
absent — and a person who has never been sealed also has no key file, so the two states were
indistinguishable. The tombstone is the discriminator: it is the record that erasure actually
happened.

**And a process note worth keeping.** Three separate edits silently did nothing this session
because `cargo fmt` had rewrapped the text I was matching against. The last two carry an assert on
the match, which turned a silent no-op into an immediate failure. A replace without one is a
change you have to remember to verify.

351 tests. Gates green.

## 2026-09-07 — Multiple nodes, and the two questions hiding inside one

**The question was how to run docket on more than one machine so that losing a laptop does not
hurt. Measuring first turned it into a different question, twice.**

**The key was never the exposure.** RUN-0004 already keeps the one age key in a password manager
and installs it per machine, so a lost laptop loses no key. What lived on exactly one machine was
the *store* — four hash chains, the issued PDFs, the signed scans, the per-person keys.

**Then multi-writer turned out to be wrong rather than hard.** A person identifier comes from
`next_person_seq(&ledger)`, read locally. Two machines allocate the same identifier to two
different human beings, and ADR-0051 is locked on sequential identifiers, so the sequence cannot
absorb concurrency — the concurrency has to go. That is the whole argument for a cold spare, and
it is stronger than "a spare is simpler".

**And docket was already multi-user, which settled the shape.** The server binds loopback,
refuses a routable address, and runs one connection at a time because *"this process is the only
writer to four append-only hash chains"*. Many operators, one machine, one process, one writer.
The single-writer invariant a second node would have broken was already being relied upon and
already argued for on its own merits.

**The hazard was live, on the machine in use.** That serial loop covers the server and not the
CLI, and there was no file locking anywhere. Ten concurrent `docket new` runs, five pairs:
duplicate identifiers in **five of five**, forked chains in four. Not a narrow race — the common
case, because `append_chain` is read, seal-onto-last, append with nothing in between.

**The sharpest finding came from reading the failure rather than the summary.** `docket check`
reported the forks and *not* the duplicates. One pair had appended cleanly — two stale reads that
happened to serialise — producing **the same person identifier twice with a correct chain, which
check passed in silence.** `docket.identifiers_are_never_reused` had been declared against
ADR-0051 on 2026-08-19 and never implemented; detection was an accident of the torn chain that
usually accompanies a race. A property that load-bearing has to be asserted.

**Three defects of my own, each found by running the thing rather than reading it.**

The uniqueness check's first version counted *rows* and called an erasure's appended row a
duplicate — caught by `every_chain_still_verifies_after_an_erasure`, which is the test that
exists for exactly that append. Reuse is one identifier across two **slugs**.

The restore guard tested `into.register()` — `register/issued.jsonl` — which does not exist until
something has been issued. A store holding three people and no documents therefore looked empty,
so the second restore overwrote `people.jsonl` and **silently deleted a person allocated after
the backup**. Found by running the restore twice in the simulation, not by reading the guard. A
guard has to be about the files being written, not about a file that stands for them.

And I reported `exit=0` for a refusal that had actually exited 2, because `$?` after a pipe is
the exit code of `tail`. The same defect `CLAUDE.md` already warns about for `gates.sh`, made in
a shell one-liner minutes after reading it.

**What I did not build, and why it is debt rather than an omission.** ADR-0072 decides five
things; code enforces four. The node registry and the off-authority write refusal (D1, D4) are
prose, because which machine is authoritative is estate inventory and RFC-0021 is still
`proposed` — building against a proposed naming scheme would bind docket to a name that may
change. D-52 records it, including the consequence: nothing yet stops a second docket started
against a restored store on the spare.

**What the trade still is.** Replication is safe at all only because ADR-0069 made erasure
key-destruction rather than file-deletion, so ciphertext can be copied anywhere. The per-person
keys are the one row of the table with no cell to fill: a key you can restore is a key you cannot
prove you destroyed. Multi-node sharpens that; it does not soften it, and the ADR says so rather
than implying a spare closed it.

**Skills and tools used.** `age` and `age-keygen` for the real encryption path; `SOPS_AGE_KEY_FILE`
to rehearse with a throwaway key so the operator's own was never touched; `tar` into a scratch
root to build a disposable second machine; a Python `assert old in s` on every text replace, after
last session's silent no-ops; two deliberate mutations (`if false &&` on the merge guard,
`keys` reclassified `Replicate`) to confirm the new tests fail when the guards are removed.

371 tests. Gates green. `check-knowledge.sh` passes.

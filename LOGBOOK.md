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

## 2026-09-07 — Two containers, and the one thing they could not prove

**The operator asked to make it production-ready by running two nodes connected over a network.
Half of that was the missing piece and half of it was the thing the morning's work had just ruled
out**, so it was worth separating rather than building.

Two docket binaries reachable over a network is multi-writer — the ADR-0051 collision closed
hours earlier — and `require_loopback` refuses a routable bind anyway, deliberately, because the
server has no authentication of its own. A spare that *runs* docket is not a spare.

**What was right is the isolation.** Two environments with separate filesystems, the bundle
crossing a network as bytes, is exactly the leg `tests/node.rs` cannot reach: those tests share a
kernel and a disk, so they prove the cryptography and not that a spare needs nothing from the
machine it is replacing. So the network carries the *bundle*, not docket traffic.

Node A creates three people, backs up, pushes the bundle to an off-host container, and is then
deleted along with its volume. Node B pulls it, restores, verifies, and allocates
`PAR-P-000004-4`. Thirteen assertions, four negative.

**Three failures on the way, and each one was informative.**

A glibc version error that read like a docket bug: `rust:1.97.1-slim` is trixie and the runtime
was bookworm. Both stages are now pinned, with the reason in the Dockerfile, because the symptom
names no cause.

**Two assertions were vacuous when first written, which is the failure mode a harness is most
prone to.** The plaintext check used `grep -v`, which is line-based, so it flagged every
encrypted file — the check that was meant to prove ciphertext had left the machine was instead
reporting a leak that did not exist. And the reachability check curled a container that was not
serving at all: the failure it recorded proved only that nothing was listening. Both are fixed,
and the ciphertext one was confirmed by injecting a plaintext file and watching it fail. A green
assertion nobody has made fail is not evidence.

**And one failure was the system working.** The reachability test first ran on the node sharing a
store with a running server, so `serve` was refused by the lock and the API never came up. I had
written a check that reported the lock as a defect. It now runs on its own store.

**What the containers cannot prove, stated in D-53 rather than implied away.** The Sync.com leg.
RFC-0038 records the vendor's client as Windows and Mac only, so it can never run in a Linux
container; the off-host container carries the same bytes over the same kind of hop and is a
stand-in, not the vendor. Exactly one manual rehearsal remains.

**Skills and tools used.** Docker with a private bridge network and per-node volumes so no two
nodes could see one filesystem; `age-keygen` inside the image for a real estate key and a real
wrong key; `tar` streamed through `docker exec` so the bundle moved without either node mounting
the other; `python3 -m http.server` as the off-host stand-in; a deliberate plaintext injection to
prove one assertion could fail.

371 tests plus 13 container assertions. Gates green.

---

## 2026-09-08 — the front half gets a playbook, and it is honest about being manual

`HR-CHECKLIST.md` begins at "1 · Offer". Everything before that — writing a posting, publishing
it, taking applications, deciding — was undocumented, and the CLI has no `post`, `apply` or
`shortlist` subcommand to document. ADR-0063 named this the front half of docket and left it for
Phase B. `HIRING-PLAYBOOK.md` covers it now.

**Capability was retrieved before it was written, per STD-0013.** `HR-CHECKLIST.md` (123 lines)
and `SOP.md` (837) already cover offer through alumni, so the playbook does not restate them: it
ends at section 5 by handing over to `HR-CHECKLIST.md` §1 at the exact line where docket takes
over. Two documents meeting at one seam, not two overlapping ones.

**The load-bearing decision was to describe the manual path rather than the designed one.**
`api/apply` is built and deliberately inert — gate 1 returns 503 with *"Until then, apply by
email"* unless `PARIDYUM_BACKUPS_CONFIRMED=1`, which asserts a rehearsed off-host restore
(ADR-0071 D3, ADR-0068 §5). So intake is a mailbox today, and sections 3 and 4 of the playbook
have no commands in them at all. Writing them as though the store existed would have produced a
checklist nobody could follow. The future state is confined to one closing section that says
exactly which sections change and which do not.

**Two joins between repositories are stated because neither is enforced.** `engagement.class` on
the posting must match the class passed to `docket new`, and `engagement.job_id` in the person
record must be the posting id — the only join between `web-paridyum/jobs/` and
`tool-docket/people/`. Both are checkboxes because nothing checks them; `jobs.ts` already carries
a hand-copied mirror of `ENGAGEMENT_CLASSES` and `DOCUMENT_PROVIDES` with a comment admitting a
copy goes stale silently.

**Skills and tools used.** `paridyum-platform:new-decision` for ADR-0074 and
`paridyum-platform:design-change` for RFC-0043 earlier in the session, which is where the
surface split this playbook assumes was decided; `check-knowledge.sh` after each; grep and sed
over `src/main.rs` to establish that no hiring subcommand exists rather than assuming it, and
over `api/apply/route.ts` to read the three gates in the order the code applies them.

No code changed. Documentation only.

**Later the same day — `RUNNING.md`, because the README answered the wrong question.**
`README.md` is 422 lines across 20 headings and had no install step, no first command and no
statement of which machine to run on. STD-0002 requires a fixed README shape with "Run locally"
and "Test"; this repository had neither, so the entry path was 400 lines of capability with no
door. The setup facts existed but were scattered across ADR-0072 (one authoritative node, the
`O_EXCL` lock), ADR-0068 D1 (loopback, SSH forwarding as the authentication), ADR-0071 (`.age`
before anything leaves), `scripts/serve.sh`'s header comment (build-first, refuse-occupied-port,
both written after real faults) and `config.rs` (`DOCKET_PEOPLE`, `DOCKET_STORE`). `RUNNING.md`
collects them; the README gains four lines and a pointer rather than a rewrite.

**The section that matters is §1, and it is first for that reason.** "Which machine" is the
question that goes wrong irreversibly — two docket instances fork a hash chain, and a forked
chain reads as tampering later. Everything else in the file is recoverable.

## 2026-09-09 — Four real people, and the guard D-51 had put in the wrong place

**Four officers are now recorded**, and the register stopped being a demonstration. That changed
which open debt was worth taking: not the node registry, but the one standing between four real
people and a single laptop.

**D-51 had reasoned itself into the wrong boundary, and its own words contained the correction.**
The entry said the check could not live in docket, because docket cannot see what Sync received —
true — and that what could exist is *"a guard on the other side, in whatever performs the
upload"*. The thing performing the upload is a person copying files into a watched folder. Filling
that folder is an operation docket can perform, and therefore one it can guard. The vendor's leg
was never the part that needed checking.

**The refusal worth building was the one that looks backwards.** Four of the five inspect what is
being sent. Only the never-list check inspects what is *already at the destination*, and it is the
only thing that can catch a `store/keys/` uploaded once, by hand, months ago — after which no
erasure the system performs is real and nothing would ever have said so.

**Two defects, both found by tests, both producing a refusal that lied.** The plaintext check
bailed inside the copy loop while printing "Nothing was uploaded", which was false for every file
the walk had already reached; a refusal that misreports what it did is worse than no refusal.
And the inside-the-store guard used `starts_with`, which is purely lexical — a relative
`store/uploads` walked straight past it and wrote into the directory it existed to protect. I
found that one by running the command against the real store and then having to delete what it
had created. The first fix absolutised the path and a `..`-laden path still defeated it, so
destinations are now normalised lexically.

**On the officer records.** A date of birth reading 27/01/2026 made the record structurally
impossible rather than merely wrong: DPDP s9 demanded a guardian and the officer class forbids
holding one, commented *"an officer is not a minor"*. Two rules disagreeing was the signal the
data was wrong. I asked rather than assuming 2006, because the year is printed on both documents.

**And the identifiers were reassigned, which is only possible because nothing has been issued.**
The request was to reorder four people across four identifiers, which locked ADR-0051 forbids as
reuse. Checking first showed what the prohibition is actually about: no document, no serial, no
signature, no tombstone, and every address field still null. ADR-0051's harm is a certificate that
stops naming the same person, and no certificate existed. So the ledger was reset and re-allocated
rather than edited — editing an append-only chain would have invalidated every digest after the
change — and the filled records survived untouched, because they are keyed by slug and a slug is a
filename, never an identity. That window is now closed: the first issuance ends it.

**`pre_existing_ip` is held as FILL-ME on all four deliberately**, which keeps `docket check` red.
That red is accurate — the records are incomplete — and it converts "remember not to issue" into
"cannot issue", with `--force` explicitly not applying.

**Skills and tools used.** `docket check` as the arbiter rather than my own reading of the
registry; deliberate mutations to confirm new tests fail; the real store as a test subject once,
which is how the relative-path defect surfaced and also why it had to be cleaned up by hand;
`check-knowledge.sh` to catch that citing a check in a new ADR left its `blocks:` list stale.

378 tests. fmt, clippy and test green; `check` red on four held records.

## 2026-09-09 (later) — A second company, and the decision that had already been taken

**Paridyum Cyber arrived, and the first useful thing was finding out how much of this was already
decided.** ADR-0051 labels the first segment of `PAR-P-000147-3` as *organisation* and states that
*"one person may hold many engagements; the archive stays whole across all of them."* Both
sentences were written for a single-company estate and both turned out to be the answer. Nothing
locked needed superseding, and the engagement identifier's shape did not change at all — only the
value of a segment that already existed.

**The obvious design was the wrong one, and its wrongness was checkable.** One docket store per
company is what anybody would reach for. It means two ledgers, so a person in both is allocated
two person identifiers — Rohan's archive splits down the middle, and the check that identifiers
are never reused cannot see across the boundary it would most need to. That is the exact harm
ADR-0051 exists to prevent, arrived at by convenience.

**The real work was not the registries.** `reengage` refused any second engagement while one was
open — *"separate them first"* — which reads as a rule about sequence and is actually an assertion
that a person works for one entity. That assertion was never decided and is now false. Narrowing it
to one open engagement **per company** keeps the protection it was actually providing (two open
internships at one company is a record nobody can reason about) and drops the part nobody had
chosen.

**Two collapses that resolved themselves into data.** Priyanshu holds CEO and Director at once, so
RFC-0042's rule — the CEO signs the Director's own documents — had nothing left to point at. Rather
than branch on it, he gets both entries in `signatories.yaml`; for his own documents both are the
subject, and the substitution that already existed falls through to the CTO. The operator's
decision and the code's behaviour are the same object. A test asserts the NDO resolves to `cto`.

**And the roster was quietly lying within minutes of the change working.** `identity::current`
keeps the last ledger row per person, which is correct for an intern who becomes an employee and
wrong for a man holding two offices: it showed Rohan at Paridyum Cyber and omitted Paridyum
entirely. Found by reading the roster after the ledger was already right — the ledger had five
correct rows and the view showed four. The summary then said "5 people" for four officers, which
is the same defect one line further down.

**On resetting the ledger a second time.** Ruhani's engagement had been opened at Paridyum before
she moved to Cyber, and an engagement's company is immutable by decision. Nothing is issued, no
address is provisioned, and every appointment date is still in the future, so the reset cost
nothing — and the person identifiers the operator chose were preserved exactly, which is why no
address changed. That window closes at the first issuance, and I said so rather than leaving it to
be discovered.

**What I did not do.** The officers page still names one company, and re-exporting now would
publish six officers of two companies as one flat list — worse than a stale page. D-55 holds it.
And a person record still has one `engagement:` block: Rohan being CTO of both is true of both, so
it works by luck, and D-54 names the schema choice rather than guessing at it.

**Skills and tools used.** `grep` against the locked ADR before designing anything, which is what
turned a supersede into a reading; `docket check` as the arbiter after every registry edit; four
new tests in `tests/officers.rs` including the two-companies-one-identifier case; and a careful
note that zsh does not word-split unquoted parameters, after a loop silently passed an empty
`--company` four times.

382 tests. fmt, clippy and test green; `check` red on four held records, plus the D-55 warning.

## 2026-09-09 (later still) — "Paridyum owns every other company"

**Said as an aside, and it is the sentence that decides who may sign.** ADR-0077 decision 4
resolves a subsidiary's countersignature through ownership — Paridyum's Director signs for the
holding company — so a company whose ownership does not reach the root has no answer to the
question of who may execute its documents. It would find that out at its first issuance, months
after somebody added the entry.

So it became an invariant rather than a comment: exactly one company declares no `owned_by`, every
other reaches it by following the chain, no chain loops, and no owner names a company that is not
declared. Four tests, each swapping `org.yaml` for a registry that breaks one rule.

**`owned_by` names a company rather than asserting `PAR` everywhere.** A company owned by Paridyum
Cyber is still owned by Paridyum, and a rule that demanded the root be named directly would be
wrong the first time the group is two levels deep.

**Widening the check found a second collision nobody had asked about.** `organisation` had only
ever validated the default company — so a subsidiary with no jurisdiction would have produced a
governing-law clause naming no court, visible only on that company's first document. Validating
every company made the shared-serial-format problem obvious: each company keeps its own per-kind
counter, so two companies with `PAR/{kind}/{year}/{seq}` issue `PAR/NDA/2026/0001` twice, to
different people. The serial is the string a third party verifies a document by. That check exists
now because the loop made it visible, not because anybody predicted it.

**A shell lesson, twice in one day.** Backticks inside a double-quoted `python -c` are executed by
zsh before python sees them, which silently ate a doc comment's `` `check::run` `` and left the
sentence starting "Test seam:  is the front door". Heredocs for anything containing backticks;
`python -c` only for the trivial case.

387 tests. fmt, clippy and test green; `check` red on the four held records.

## 2026-09-12 — Asked for a timeline, found a live correctness hole instead

**The question was whether docket should have a timeline or graph view. Checking before answering
is what made the answer easy, and it was not the answer the question expected.**

`docket record` already prints a dated history — the hash chain IS a timeline. ADR-0065 already
assigns structural diagrams to archify, with hand-authored `lifecycle` and `dataflow` IR explicitly
legal, so a renderer here would duplicate a tool pinned four weeks ago. And a graph of two
companies and four people is a screenshot, not a feature. The estate's own idiom applies: a planned
region is not a region, it is a sales conversation.

**But running the per-person commands to check what a view would even show surfaced this.**

    docket status rohan-batra
      engagement PCY-OFC-2026-0002    <- Paridyum Cyber

`identify` took the last ledger row for a slug. That is correct while a person holds one engagement
at a time — an intern who becomes an employee should resolve to the employee row — and it was
silently wrong from the moment concurrent engagements landed, an hour earlier, in code I had
written. Every per-person command reads through it, so Rohan's Paridyum office was unreachable, and
**`issue` derives the company from the engagement**: his Paridyum NDA would have carried Cyber's
letterhead, Cyber's jurisdiction and a `PCY/` serial. An issued document is corrected only by
reissuing under a new serial.

**I had already fixed this exact defect once, in `current()`, for the roster — and missed that
`identify` is a separate path doing the same thing.** One fix, one sibling left behind. The lesson
is not "be careful"; it is that a recency-based lookup is a *pattern*, and finding one instance
should have prompted a grep for the rest.

**Refusing beat resolving.** Ambiguity now errors and names both engagements rather than picking
one, because which company a command concerns decides the letterhead, the jurisdiction and the
serial. A global `--company` answers it — one flag, reused by `new` and `reengage` for allocation
and by everything else for disambiguation, rather than a per-command flag repeated eleven times.
A company the person holds no office at is an error, never a quiet fallback to the other.

**On the view question, the honest position is recorded rather than implied:** no graph, no general
timeline, with a named trigger — a person holding engagements at three or more companies, or the
group going two ownership levels deep. The one view I would still argue for is a forward
obligations report, because `retention` answers what is past due and nothing answers what falls due
next, and those are DPDP deadlines with legal consequence. Even that is a sorted list.

**Process note.** Threading one parameter through nineteen call sites with regexes double-patched
several and mis-inserted others into the middle of function bodies. Reading the compiler's error
list as a worklist rather than trusting the scripts is what kept it correct; three signatures had
to be repaired by hand after a script put `company: Option<&str>,` on the line after a closing
brace.

388 tests. fmt, clippy and test green; `check` red on the four held records.

## 2026-09-12 (later) — Working the debt list, and the first thing it found was a red CI

**Asked to implement the open engineering debt, the first useful act was `gh run list`.** CI had
been failing for five days — three consecutive red runs, including one from that morning — and
nothing in this checkout says so. `gates.sh` runs four local gates and reports them green, which
is exactly the trap D-28 had written down in 2026-08-20 and which nobody had come back to read:
*"do not treat a green local gate as evidence that CI passes."* The entry's status line was stale
and its warning was the live part.

**The failure was a fixture, not a test.** `the_keystore_is_never_in_what_gets_uploaded` reads two
paths and asserts about a layout, and it failed on every runner because `Sandbox::new` called
`age-keygen` unconditionally. Green everywhere `age` is installed, red everywhere it is not. That
is D-45's defect a second time, and the second occurrence is what made the general fix obvious:
`scripts/ci-parity.sh`, which runs the suite with `PATH` stripped to the Rust toolchain. Both
instances had been found by pushing and reading a log.

**Then D-30 closed by being proved true.** Tectonic had never been run against this repository —
no machine on the estate had it — so I installed it, and found two defects stacked.

`TEXINPUTS` was set for both engines and **Tectonic does not read it**. Every render failed with
`File 'paridyum.cls' not found`. The engine has `-Z search-path`, documented as "like TEXINPUTS",
and nothing had ever passed it. Fixed.

With that fixed, the real one: `paridyum.cls` uses `pdfx` for PDF/A, and under XeTeX it fails with
`CreationDate is not properly supported`. Seven of thirteen issuance tests pass; six cannot render
at all. **So the engine docket advertises as its supported alternative has never worked**, and
`detect` still offers it — which is now D-57 rather than a surprise for whoever first runs docket
on a machine without TeX Live.

**That finding chose the CI design.** I had written the render job around Tectonic for the obvious
reason: a 10MB checksummed binary against a multi-hundred-megabyte apt install. Having to throw it
away produced a better argument than the one I started with — **pdflatex is what every developer
machine runs and what has produced every document this estate has issued, so a CI job rendering
with a different engine would be verifying something nobody ships.** The registry entry says
`texlive`, not `tectonic`, and its exit path cites the experiment rather than asserting a
preference.

**A correction I nearly did not make.** Closing D-35 made a comment in `ci.yml` wrong: it promised
coverage would rise from 60 to 70 "when D-35 is closed and CI can render". D-35 is closed, CI does
render, and coverage cannot move — because **rendering and coverage are two different jobs**, and
only one of them is in docket's own workflow. The other belongs to the shared `infra-ci`
workflow in another repository. The comment is corrected in place so the next reader does not
repeat the inference.

**On D-49, a distinction worth keeping.** The check asserts against the `.gitignore` TEXT rather
than shelling out to `git check-ignore`, because those answer different questions: a file already
tracked is reported as not ignored whether or not the negation line exists. That difference had
already wasted one mutation test in an earlier session, which passed while proving nothing.

**Closed: D-28, D-30, D-35, D-45, D-49. Narrowed: D-36, D-41. Opened: D-57.**

**Skills and tools used.** `gh run list` / `gh run view --log-failed` to read a CI failure without
pushing; `gh api` to find a real pinned release rather than inventing a version; a stripped-`PATH`
harness to reproduce a runner locally; an actual Tectonic install to test a claim rather than
reason about it; and a mutation of `.gitignore` to prove the new check fails when it should.

389 tests. fmt, clippy and test green; `check` red on the four held records.

## 2026-09-13 — How a CEO is fired, and an entry I got wrong by not measuring it

**The question had no answer, and finding that out was most of the value.** RFC-0042 decided how an
officer is appointed — instrument, registry entry, human under C-0004 — and said nothing about
removal beyond the record's shape. The asymmetry is what made it a defect rather than a gap: an
appointment without an instrument is a `docket check` finding, and a removal without one was
indistinguishable from a correct removal.

**Simulating the firing found two live defects that reading would not have.** I set Priyanshu
`active: false` and resolved a document:

    resolve_signatories(...) -> ["director"]

He would still have been printed as the signatory on every document issued afterwards. Routing
names an OFFICE; only the register knows who holds it — and the substitution only ever asked "is
this the subject of the document", never "is this office vacant". `check` was silent because
`signed_by` is validated against every signatory rather than the active ones.

And `ceased_on` did not exist at all. RFC-0042 §5 says ceasing is `active: false` **plus** a date;
the struct had no such field, so half the rule had been unimplementable since August.

**Then the decision separated two things I could have conflated.** The operator's answer — Rohan
absolute, Priyanshu second — is about who may DECIDE. Rohan is `cto` in the signatory register and
third in Paridyum's fallback order. The tempting move was to reorder that list to match the
authority, and it would have been wrong: a document is executed by whoever holds the office it
routes to, because that is what lets the register say who held an office on a date. Reordering
would have made Rohan sign everything and changed every document Paridyum issues. ADR-0087 keeps
them apart and says why.

**And I had to retract D-59.** I had written — and put in a summary to the operator — that removing
Priyanshu would leave "nobody to countersign Ruhani's own documents". Measured:

    PCY, Priyanshu removed — ruhani: ["cyber-cto"]
    PCY, Priyanshu removed — rohan:  ["cyber-ceo"]

The chain is two deep and survives losing any one of three. I had reasoned from the registry's
ORDER — he is first in both lists — without following what the substitution does when the first
entry is vacant. That is a specific way to be wrong: reading a list's shape instead of the code
that reads it. The entry is kept with the correction rather than deleted, and the property it was
groping for is now a check — fewer than two PEOPLE among a company's active signatories is an
error, counted by person because Priyanshu is two rows and one human.

**A note on what I did not touch.** `check-knowledge.sh` reports three errors in `ADR-0086` and
`STD-0022`, both untracked files from another session and possibly mid-edit. Not mine to fix
silently.

**Skills and tools used.** Editing the real registry to simulate a removal and restoring it from a
backup; an existing test as the probe — `the_ceo_and_director_being_one_person_falls_through_to_
the_cto` failing with `["director"]` was the proof; a throwaway printing test to measure the Cyber
chain before writing a claim about it, which is what caught D-59; and `mem::forget` being denied by
STD-0003, which was right — the `Registry` is fully parsed and the temp directory can simply drop.

396 tests. fmt, clippy and test green; `check` red on the four held records.

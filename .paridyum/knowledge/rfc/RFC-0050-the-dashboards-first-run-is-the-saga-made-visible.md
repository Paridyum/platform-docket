---
id: RFC-0050
title: The dashboard's first run is the provisioning saga made visible
type: rfc
status: proposed
created: 2026-09-15
locked_at: null
supersedes: null
superseded_by: null
scope: [org, web, console, library, platform]
applies_to: ["*"]
product: [paridyum, tenancy]
tags: [dashboard, first-run, onboarding, provisioning, design-system, palette, theming, vocabulary, std-0015, std-0019, std-0021, glo-0002, adr-0039, adr-0100, rfc-0049]
breaking: false
---

# RFC-0050 — The dashboard's first run is the provisioning saga made visible

## Problem

Seven HTML screens were produced for the platform dashboard and placed in `scratch/`. They
describe a flow the estate genuinely needs and has never drawn: **how a person gets from a cold
visit to a running tenant.**

| Draft | What it shows |
| --- | --- |
| `ours_sign_in_operator_identity_first_time_visitor` | sign-in / identity, first visit |
| `ours_guided_workspace_genesis_first_time_post_login` | guided setup, immediately after login |
| `ours_guided_workspace_genesis_interactive_animated` | the same, animated |
| `ours_create_workspace_first_time` | naming and creating the first tenant |
| `ours_setting_up_acme_provisioning` | provisioning in flight |
| `ours_workspace_ready_open_acme` | the ready state, and the way in |
| `ours_workspaces_error_recovery` | several tenants, one of them failed |

**Read as a flow, this is ADR-0039's saga with a face on it** — `requested → provisioning →
active`, with `failed` given a recovery path. Nothing in the estate has ever shown that lifecycle
to a human, and RFC-0049 §Proposal C says the dashboard must.

**Read as code, none of it can ship.** Every screen is built on a stack, a palette, a brand and a
vocabulary that this estate's own governance rejects — not marginally, but on eight separate
counts, four of which have named checks behind them:

1. **The brand is "OURS", not Paridyum.** A different name, a different mark — an aperture circle
   drawn inline rather than `mark.svg`. `ui.mark_is_not_redrawn` exists for exactly this.
2. **The palette is Material Design 3**, primary `#6750a4` / `#4f378a`, with the full MD3 token set
   (`surface-container-*`, `on-primary-fixed-variant`, `tertiary-fixed-dim`) declared inline in a
   `tailwind.config`. STD-0015 §1 fixes the identity at three constants and
   `ui.tokens_are_not_locally_defined` refuses a local redefinition.
3. **There is no dark theme.** One draft declares `darkMode:"class"` and then defines light values
   only; the other six declare nothing. STD-0015 §5a requires three states, and
   `ui.theme_blocks_agree` now enforces the shape.
4. **"Workspace" is a fourth word for `tenant`**, and the drafts use both — sometimes in one
   screen, sometimes in one sentence (*"Synthesizing isolated tenant"* under a heading about a
   workspace). GLO-0002 warns in its own text that *"the word tenant is doing double duty, and
   that is the hazard"*; adding a synonym makes a naming problem the glossary already flags worse.
5. **Credibility is manufactured, and one claim is false.** `TLS 1.3 / E2EE`, `Identity Gateway //
   SEC-01`, `CORPORATE SSO LINKED` are decoration. **`E2EE` is not true** — RFC-0046 §A establishes
   that even under BYOK the database serves plaintext and an operator with break-glass access reads
   it. STD-0021 exists because a guide followed literally *"would put things on `paridyum.com` that
   are not true"*; a dashboard is not exempt from that.
6. **Self-service signup is drawn as shipping.** A "Create account" tab sits beside "Sign in".
   ADR-0100 decision 5 defers self-service signup and ships operator-driven creation first; ADR-0103
   decision 2 forbids a self-serve path to anything above C0.
7. **Tailwind is loaded from a CDN**, with the config inline in a `<script>`. ADR-0046 fixes the
   stack; `lib-ui` is the token pipeline; a CDN in a request path is a third-party dependency in
   the serving path of the control plane.
8. **A typeface is chosen.** `Public Sans` via Google Fonts. STD-0015 §Consequences 5 says plainly
   *"the typeface is not decided here"* and records it as decision debt. A mockup is not where that
   decision gets made.

One more, subtler and worth naming: the copy says **"Synthesizing isolated tenant"**. Synthesis is
BET-0003's vocabulary — generating an application per customer — and BET-0003 was **withdrawn** by
BET-0004 on 2026-08-08. We deploy catalogue products; we do not synthesise them. The word would
re-announce a strategy the company has left.

**What happens if nothing is done.** Either the drafts are implemented as drawn — in which case the
estate acquires a second palette, a second brand and a second word for `tenant`, all of which
`check.py` and the knowledge gate will then fight — or they are discarded, and the one thing nobody
has designed (the path from cold visit to running tenant) stays undesigned.

**How we will know it is fixed.** The flow ships, on our identity, in both themes, with `tenant`
as the only word for a tenant, and with no claim on screen that is not true.

## Constraints

- **STD-0015** — three identity constants; the signal is a ground and a graphic, never type;
  §5a's three theme states. `ui.brand_accent_is_not_a_text_colour`,
  `ui.tokens_are_not_locally_defined`, `ui.mark_is_not_redrawn`, `ui.theme_blocks_agree`.
- **STD-0019** — writing a shared component; the tier model this must land in.
- **STD-0021** — the marketing surface may not manufacture credibility it has not earned. Its
  adopted source also supplies *"dark-mode parity by default"* and *"audit-first redesign with
  preservation rules"*, which is the method §Proposal A follows.
- **GLO-0002, GLO-0004** — `tenant` is the outer silo and the only word for it; `account` is the
  inner one and is never called a tenant.
- **C-0007** — naming.
- **ADR-0039** — the saga these screens render, and its exact state names, lowercase per ADR-0032.
- **ADR-0100 (revised 2026-09-15)** — the dashboard at `/dashboard`; it is a **surface** that owns
  no entity; decision 5 defers self-service signup.
- **ADR-0049** — nothing runs outside a tenant; Paridyum is tenant zero and holds no scope a
  customer could not hold; no capability ships without a tenant-facing consumer.
- **ADR-0103** — no self-serve path to a containment rung above C0.
- **RFC-0049** — a tenant is a set of Releases; the dashboard is the view of them.
- **RFC-0046** — what BYOK does and does not give, which is why `E2EE` is refused rather than
  reworded.
- **BET-0004** — the catalogue is the product; BET-0003's generation wager is withdrawn.
- **ADR-0046, ADR-0020** — the web stack and the deployment target.
- **ADR-0019 decision 5** — creating a service escalates to a human. Where this code lives is
  §Open questions 1, not a decision this RFC takes.
- **ADR-0096** — localized content is a per-locale keyed object; REQ-0009 item 9 — right-to-left is
  designed for, not retrofitted. The drafts are English-only and use physical `left`/`right`.

## Proposal

### A. Adopt the flow. Rebuild the code. Keep the audit visible.

The drafts are **specification, not implementation** — and treating them as the first is what makes
them worth having. STD-0021's adopted source calls this audit-first redesign with preservation
rules: state what is kept before changing anything, so the redesign cannot quietly discard the
thing it was asked to improve.

**Preserved from the drafts, deliberately:**

- The **flow itself**, and its ordering: identity → name the tenant → watch it provision → enter it.
- **Provisioning is a first-class screen, not a spinner.** The drafts give in-flight state its own
  surface with per-step progress. That is the correct reading of ADR-0039 — the saga is
  asynchronous and its lifecycle is public, so the UI that hides it is the one that is wrong.
- **Failure is a screen too**, with recovery. `ours_workspaces_error_recovery` shows a failed tenant
  alongside healthy ones rather than an error dialog. ADR-0039 compensates to `failed` and never
  hard-deletes, so a failed tenant is a row that persists and must be visible.
- **A tenant list is the home**, not a single-tenant view. Correct under RFC-0049 §B.
- The **card-and-panel layout**, the segmented control, and the step-progress pattern.

**Rebuilt, because it cannot be otherwise:**

| Draft | Becomes |
| --- | --- |
| MD3 palette in `tailwind.config` | `lib-ui` tokens; §Proposal D extends them where the drafts prove a gap |
| "OURS" name and inline aperture mark | Paridyum, `Mark` from `@paridyum/ui` |
| Tailwind CDN + inline config | the estate's build (ADR-0046) |
| `Public Sans`, Material Symbols | `--font-sans` as it stands; icons are §Open questions 3 |
| Light only | three theme states (STD-0015 §5a) |
| `left`/`right` | logical properties (REQ-0009 item 9) |
| English strings inline | per-locale keyed (ADR-0096) |

### B. Refused outright, with the reason attached to each

Not "changed" — **refused**, so that a future draft carrying them is rejected on sight.

1. **`E2EE`.** False. RFC-0046 §A: the database serves plaintext and an operator can read it. No
   rewording — a weaker true claim ("encrypted at rest") may be made *only* when it is true and
   there is somewhere for a reader to verify it, which is what RFC-0045's attestation is for.
2. **`TLS 1.3`, `SEC-01`, `CORPORATE SSO LINKED`.** Decoration dressed as assurance. STD-0021's
   whole subject. If a security property is worth stating, it is worth stating where it can be
   checked.
3. **"Create account" as a shipping tab.** ADR-0100 decision 5. Operator-driven creation first.
4. **"Synthesizing".** BET-0003's withdrawn vocabulary. The tenant is *provisioned*; its products
   are *deployed*.
5. **"Workspace".** §Proposal C.

### C. One word: `tenant`

The thing a person creates on this screen is a **tenant**, at every layer — the URL, the heading,
the button, the toast, the error, the docs. GLO-0002 already carries the warning that `tenant` is
ambiguous across levels and resolves it by fixing the word to the outer silo; a synonym re-opens
what the glossary closed.

**The objection is real and is answered rather than dismissed.** "Tenant" is landlord vocabulary
and reads coldly to someone creating their own company's space. "Workspace" is friendlier. But the
friendliness is bought by having two words for one thing — and the estate's own record shows where
that leads: RFC-0047 §Finding could not be written until the `operator` ambiguity between GLO-0001,
GLO-0004 and BET-0005 was named, and that ambiguity is *the same mistake one level up*, already
costing a task (T-G.5) to repair.

If the word must soften for a reader, that is a **copy** decision inside one locale file — never a
second identifier in a URL, a column, an API or a component name.

### D. The palette extension the drafts prove is needed

Evidence-led: each token below is here because a specific element in the drafts has nothing to
resolve to. Nothing is added speculatively — §Open questions 2 keeps the rest out.

**Surfaces.** The drafts use four levels (page → card → input well → raised control). `lib-ui` has
two. Proposed, as *derived* values in the STD-0015 sense, never new identity constants:

| Token | Role | Draft element that needs it |
| --- | --- | --- |
| `--color-paper-sunken` | below the page — inputs, wells | the email field's `surface-container-low` |
| `--color-paper-raised` | *(exists)* cards, panels | the auth card |
| `--color-paper-overlay` | above everything — menus, dialogs | the tenant switcher |
| `--color-paper-hover` / `-active` / `-selected` | interactive surface states | segmented control, list rows |
| `--color-ink-disabled` | disabled text | the submit button while provisioning |
| `--color-info` / `--color-info-wash` | neutral informational | provisioning-in-progress notices |

**`--color-info` is the one that is genuinely missing rather than merely thin.** `status.css` has
refusal, outstanding and settled — red, amber, green. A provisioning tenant is none of those; it is
*in flight*, which is information, not warning. Without the token the drafts reach for the accent,
and the accent may not be type.

**The primary button needs no new colour, and this is worth stating because it looks like it
does.** The drafts' filled purple button maps exactly onto STD-0015 §2's sanctioned use of the
signal: **teal ground, obsidian type, 14.2:1.** That is the one place the brand accent is allowed
to be spent at full strength, and the dashboard's most important control is the right place to
spend it.

**Every token is defined in all three theme states**, or it does not exist. `ui.theme_blocks_agree`
will say so.

### E. Tier placement

Per STD-0019: the surfaces, states and `info` token are **tier 1** (`lib-ui` tokens). The card,
step-progress and segmented control are **tier 2/3** primitives and blocks — they know nothing
about tenants. Anything that knows what a saga state means, or what a Release is, is **tier 4** and
belongs to the dashboard, not the library. ANTI-0004's test decides each one.

## Alternatives considered

1. **Implement the drafts as drawn, then reconcile later.** Fastest to something on screen.
   **Rejected**: it lands a second palette, a second brand and a second word for `tenant` in the
   estate at once, each of which then has to be removed against a working surface people like. The
   estate has already paid this exact cost — STD-0015's context describes `console-docket` shipping
   a copied palette whose defect was *"found by looking at a screenshot"* rather than by any gate.

2. **Keep MD3 as the dashboard's system and treat Paridyum's identity as marketing-only.** Defensible
   — MD3 is complete, accessible and free, and STD-0015 §4 once said a console takes only the mark.
   **Rejected on the estate's own revision of that paragraph**: §4 was reversed within a day
   precisely because *"an operator console that shared a mark with the marketing surface and nothing
   else looked like a different product wearing the same badge."* That is what this alternative
   rebuilds.

3. **Adopt "workspace" and update the glossary.** Honest if the word is genuinely better.
   **Rejected**: it would supersede GLO-0002 and GLO-0004, re-key `tenant_id` conceptually across
   every artifact and migration, and the drafts themselves show the confusion it causes by using
   both words within single screens. If the word is wanted, it is a locale string (§Proposal C).

4. **Ship light-only, add dark later.** What the drafts do. **Rejected**: STD-0015 §5a exists as of
   today, `ui.theme_blocks_agree` enforces it, and dark-mode parity is adopted-by-default under
   STD-0021. Retrofitting a theme is how the `--color-ink-faint` asymmetry (4.04:1 on bone, 5.65:1
   on obsidian) becomes invisible.

## Consequences

**What this makes worse.**

1. **The drafts will look better than the first rebuild.** They are polished, dense and confident,
   and a faithful reimplementation on a two-surface palette with a system font will feel plainer.
   That gap is the argument for §Proposal D and against copying §B's decorations to close it.
2. **The palette grows, and every added token is permanent.** `ui.tokens_are_not_locally_defined`
   means a token that turns out to be unnecessary cannot be worked around locally — it has to be
   removed centrally. §Proposal D is deliberately short for this reason.
3. **`tenant` in the interface will be argued about again.** §Proposal C answers it once; it will be
   raised every time a new person reads the first screen.
4. **Refusing the security decorations makes the dashboard look less serious** to a viewer who
   expects them, and every competitor's screen has them. That is exactly the trade STD-0021 already
   took for the landing page.
5. **The typeface stays undecided**, so the rebuild ships in a system grotesque and will be
   revisited when the operator decides. Better than deciding it here by accident.

**What it does not change.** No new identity constants. The signal stays a ground. The mark stays
byte-identical to its board.

## Rollout

| Slice | What lands |
| --- | --- |
| **1** | §Proposal D's tokens in `lib-ui`, all three theme states, contrast asserted in both |
| **2** | Tier 2/3 primitives the flow needs — card, step progress, segmented control, status chip |
| **3** | Sign-in screen, rebuilt — the smallest, and it proves the token set |
| **4** | Tenant list, including a `failed` tenant, since failure is a first-class state |
| **5** | Create-tenant and the provisioning screen, driven by ADR-0039's real lifecycle names |
| **6** | Ready state and the way into a tenant |
| **7** | Locale extraction (ADR-0096) and logical-property sweep (REQ-0009 item 9) |

Slice 1 blocks the rest. Slices 3–6 are one screen each and independently reviewable (STD-0007).

**Where this code lives is not decided here** — §Open questions 1.

## Open questions

1. **Which repository?** `/dashboard` is a `web-` deployable that does not exist. Creating it
   escalates to a human (ADR-0019 D5), and the alternative — growing it inside `web-admin`, which
   declares `product: [identity]` — would put the control plane inside a product that is not it.
   **Blocking slice 3.**
2. **Does the dashboard need a categorical palette?** Not proven by these drafts, which show no
   charts. Deferred until a screen needs one, rather than invented now.
3. **Icons.** The drafts use Material Symbols, a Google-hosted font. An icon set is a design-system
   decision with a licence and a delivery mechanism attached, and it is not made here.
4. **Is `--color-ink-faint` allowed on this surface at all?** 4.04:1 on bone is below AA. The drafts
   set small monospace labels in its equivalent constantly. Either those labels get a compliant
   token or they stop being that small.
5. **What does the first screen say about self-service?** ADR-0100 D5 defers signup, so the
   first-run flow presumes an operator-created identity. Whether the sign-in screen says so, and how,
   is a copy decision with a commercial edge.

## Related

- **STD-0015** — the identity, and §5a's three theme states
- **STD-0019** — the tier model §Proposal E places this in
- **STD-0021** — manufactured credibility, and the audit-first method §Proposal A uses
- **GLO-0002, GLO-0004** — the vocabulary §Proposal C defends
- **ADR-0039** — the saga these screens make visible
- **ADR-0100** — the dashboard, and its deferral of self-service signup
- **RFC-0049** — a tenant is a set of Releases; this is the view
- **RFC-0046** — why `E2EE` is refused rather than reworded
- **BET-0004** — the catalogue is the product; "synthesizing" is withdrawn vocabulary

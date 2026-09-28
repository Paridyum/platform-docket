# Research — the org model, across regions and languages

> **Superseded in part, 2026-09-28 — ADR-0113 to ADR-0122.** A research
> record, kept as written. **Its org model was not adopted as drafted**: it stored an org as an
> account of a tenant that ran the portal, and the operator chose a **first-class org** instead — its
> own `org` scope and `paridyum.current_org()`, a personless registry row, ownership as
> `tenants.owner_org_id`, and no tenant for the portal (ADR-0114, ADR-0115, ADR-0118). Its region,
> identity, grant, routing and locale findings were adopted in ADR-0117 and ADR-0119 to ADR-0122.
> **Follow the ADRs, not this record.**

**Written 2026-09-28.** Prose in `docs/`, not a knowledge artifact. It binds nothing; where it and a
knowledge artifact disagree, the artifact wins. Written for the operator, and for an independent
reviewer — human or agent — who has none of the sessions' context.

This is the full research record behind the org-model drafts. The drafts are reviewed
personal-first in `personal/paridyum-org-model-2026-09-28/` (a snapshot sits beside this file in
`docs/drafts/2026-09-28-org-model/`). It continues the 2026-09-27 brief,
`docs/2026-09-27-rfc-0055-consume-serve-operate-research-brief.md`, and does not repeat it.

---

## The questions, in the operator's words

**2026-09-27:**

> *"If someone wants to run docket as a business and also use docket at the same time this flow is
> a bit not clear… deploy docket as a service and then create a tenant inside that, or… docket being
> a simple app they can enable and consume under their tenant and a separate business store for
> enabling businesses."*

**2026-09-28, first message:**

> *"Our main portal is the business as a service at paridyum.in and using which we are deploying
> paridyum.com so we can easily follow everything in the same manner as anyone else would receive
> them… people join as individuals or as an org/team… then deploy their first main business which
> looks like paridyum.com with an ecosystem of consumer end products. IdP and business level
> configuration happens here at paridyum.in… this model helps us wire up Play Store… my idea is
> introducing org on top of this."*

**2026-09-28, second message:**

> *"Now plug in how we run a multi region cloud — would that model still persist? All the orgs can
> spin up in all those different regions. All of the localisation and i18n multilingual stuff. See
> the gaps and complete everything."*

## The answers, in brief

1. **Separate the flows; a client is its own tenant.** A company that uses docket and also serves
   clients with it keeps its own tenant, and reaches each client's tenant through a grant the client
   can see and revoke — not by holding clients as accounts. (2026-09-27 brief, findings F-A to F-I.)
2. **The org model is right.**
   - paridyum.in is the product; an **org** is what a person joins, as an individual or a team.
   - An org owns businesses.
   - Paridyum's business at paridyum.com is one such business, received like anyone's.
   - The operator chose "org", with a rename of the old usage; a separate app zone for default
     hosts; and identity split by plane.
3. **It persists across regions — and the estate had already designed most of it.**
   - **RFC-0017 (proposed) already splits** Paridyum's staff (one directory, control region) from
     *operators* — in its 2026-08-11 vocabulary, exactly the org — whose personal data lives *"in
     their own region, beside the tenants they run."*
   - **RFC-0014 already keeps** *"which operator owns which tenant, in which region, on which
     plan"* as a person-free skeleton in the control plane.
   - **Four things in the first drafts change as a result:**
     - the portal's tenant exists in every region, not once;
     - ownership moves to the control-plane skeleton;
     - staff and customers are not merged into one identity store;
     - the org's language defaults seed a business at creation rather than being read across
       regions.
4. **Languages need two sets the estate currently conflates.**
   - **The languages Paridyum's software ships in**, today `en` and `ar`. They are a security floor
     and a right-to-left check, not a market claim.
   - **The languages a business publishes its content in**, declared by that business.
   - The live schema forces every localized row to carry `en` and `ar` for every business, and
     `tenants` has no declared locales at all.
5. **The gap register below lists 24 gaps.** Each names where it is closed — which draft — or who
   decides.

---

## Part A — the org model

### A1. What the estate already had (measured, not remembered)

| Fact | Where it was verified |
| --- | --- |
| The party above a tenant was load-bearing and unmodelled: self-serve signup (PLAN-TENANT-FLOW D5) and `max_tenants` (D6) need it; ADR-0101 names it `Operator` and "three per operator"; ADR-0109 D3 declined to model it | the artifacts, read 2026-09-27/28 |
| `GET /v1/tenants` returns every tenancy to any control-plane token, defined as *"Paridyum's own staff"* | `platform-tenancy/src/handlers.rs` (`list_tenants`, `require_control`) |
| `web-dashboard` lists every tenant | `web-dashboard/src/lib/tenancy.ts` (`fetchTenants`) |
| GLO-0003 already designed individual vs team: *"Upgrading a personal account to a shared one is adding members, not migrating a model"* | `glossary/GLO-0003-account.md` |
| `paridyum.resolve_account()` mints a personal account on first sign-in | `platform-tenancy/migrations/0002` |
| `accounts` is a table: one row per product instance per tenant | `migrations/0012` |
| `operator_grants` records grants with provenance, expiry and revocation, readable by the tenant it is over | `migrations/0010` |
| docket resolves only the `default` account, as an interim | `tool-docket/src/adapters/pg.rs` (`enter()`) |
| docket's four sealed chains refuse `UPDATE` and `DELETE` | `migrations/0006` |
| Both auth libraries drop the organization a role was granted in, and take the tenant from `resource_owner` | `lib-auth-rust/src/claims.rs` (`roles_from_claims`), `lib-auth-typescript/src/verify.ts` (`Object.keys(rolesClaim)`) |

### A2. The word "org" is taken three times

Measured 2026-09-28.

1. **The Zitadel object that mirrors a tenant** (ADR-0047 D2). Paridyum-authored identifiers across
   about 20 locations: at most ~490 matching lines, some of them prose. The heaviest are
   `infra-kubernetes/zitadel` (112), `web-hiring` (62), `tool-docket` (54 + 9 + 5 + 3),
   `platform-identity` (54 + 9) and `lib-auth-rust` (46).
   - **Published API:** `lib-auth-rust`'s `organization_id()`, `require_organization()`,
     `is_missing_organization()`, `Reason::NoOrganization` and `ORGANIZATION_CLAIM`, plus
     `spec-identity`.
   - **`web-hiring#5` (merged 2026-09-27)** added `servedOrgIdOrNull`, `orgForRequest`,
     `wrong_organization` and `no_organization_claim`. Its author renamed them on 2026-09-28.
2. **`scope: org`** — ADR-0015's org-wide scope. **34 locked artifacts carry it.**
3. **The `org.*` check namespace** — the GitHub organisation. It blocks locked ADR-0004, ADR-0009
   and ADR-0012.

**The operator chose "org" plus a rename of meaning 1.** Meanings 2 and 3 are unrenameable —
locked — and sit in syntactic positions no runtime value occupies, so the vocabulary draft fences
them.

### A3. Outside practice (all retrieved 2026-09-27/28)

| Source | What it shows | Used for |
| --- | --- | --- |
| AWS Well-Architected | *"In AWS, accounts are a hard boundary"*; Organizations manage accounts centrally | the org is a management and billing layer; the boundary stays below it |
| Vercel | every workspace is a team; an individual has a *"Hobby team"*; a person switches between *"the list of teams you belong to"* | individual = a personal org |
| QuickBooks Online Accountant | the firm's own books are separate; a client moved off the firm's plan: *"It doesn't impact their accounting data in any way"* | a client is its own container; ownership and billing are relations |
| Microsoft GDAP | *"Customers must explicitly grant the least-privileged access to their partners"*; access is time-bound; customers remove it | provider and support access as grants |
| Microsoft Entra B2B | partners *"use their own credentials"*; once admitted *"they're represented in your directory as a user object"*; guests are *"redirected to their own organization's sign-in page"* | the linked-user pattern, across orgs and regions |
| HighLevel | route 1 as a product (agency → client sub-accounts); even so, *"Use an Agency Sub-Account for your own business instead of mixing agency operations with a client account"* | the white-label franchise shape, not the provider shape |
| ZITADEL | IdPs linkable *"exclusively to a specific organization"*; generic OIDC; automatic creation and linking; roles map to the organizations they were granted in; external role assignments exist | federation into each business; the F-H defect |
| Apple App Review Guidelines 4.2.6 | template services *"should not submit apps on behalf of their clients"*; *"submitted directly by the provider of the app's content"* | each business publishes under its own developer account |
| Google Play Developer API | owner invites a service account under *"Users & Permissions… Invite new users"* | Paridyum as a grantee inside the business's store account |
| Heroku | `herokuapp.com` on the Public Suffix List so apps *"are prevented from setting cookies for `*.herokuapp.com`"* | the app zone must be a public suffix |
| Auth0 | region selection *"control[s] the assigned tenant domain and the region where your data will be hosted"* (e.g. `example-co.au.auth0.com`) | a residency unit carries its region in its login host |

**A dead end, recorded so it is not re-researched:** Google Play's current Spam policy page carries
no clause on apps submitted by a service on behalf of others. Apple 4.2.6 alone establishes the
principle.

---

## Part B — does the org model persist across regions?

### B1. The estate's region model

All of these are `proposed` except where marked.

- **RFC-0014 — a region is a residency boundary, nothing else.**
  - It defines three planes: **edge** (global, rented, nothing durable); **control plane**
    (control region, one); **data plane** (each region).
  - The control plane holds *"the global skeleton — which operator owns which tenant, in which
    region, on which plan — plus the region registry, domain bindings, and staff identity. **No
    operator personal data** (RFC-0017)."*
  - *"No plane writes into another."*
  - Every table declares its plane (`-- paridyum:plane control | regional`), and
    `db.tenant_scoped_tables_are_regional` is **implemented**.
- **RFC-0015 — one fact, one home; nothing is replicated into a region.**
  - Login and business data never need the control plane. Plan- and quota-gated actions do.
  - Provisioning is *"the one operation that spans both planes."*
  - D2: **one region live, the model built for many.** A second region needs *"a signed contract
    whose residency requirement the control region cannot satisfy."*
  - D5: *"The operator chooses, explicitly, from the regions that exist… Placement is never
    inferred from the address a signup arrives from."* Its worked example is GLO-0006's traveller:
    `en` / `en-IN` / `America/New_York` / `USD`.
- **RFC-0017 — the one that decides this question.**
  - | Population | Where their personal data lives |
    | --- | --- |
    | **Staff** — Paridyum's own people | one directory, control region |
    | **Operators** — our customers | *"their own region, beside the tenants they run"* |
  - What stays central is *"a skeleton with no person in it: operator id, which tenants it owns,
    which region each is in, which plan applies."*
  - It already imagines *"an operator with businesses on two continents."*
  - **In RFC-0017's vocabulary, "operator" is the org.** ADR-0109 later reassigned the word.
- **RFC-0051 — a tenant chooses a jurisdiction, and nothing below it.**
  - Jurisdiction constrains; latency only orders, through `suggest_for`, and is never a placement.
  - Options are labelled by regime ("India · DPDP"); nothing is preselected.
  - Unbuilt jurisdictions show "coming soon" or "on request"; a datacentre is never offered.
- **GLO-0007 — "home region" is retired.** There are only the *control region* (ours, exactly one)
  and a tenant's *region*. **An org has a *region*; it never has a "home region".**
- **ADR-0047 D1** (accepted): one Zitadel deployment per region, plus a staff instance in the
  control region. **ADR-0090** (accepted) already runs two virtual instances: tenant
  (`auth.paridyum.com`) and control-plane (`id.paridyum.com`).
- **State:**
  - `region-registry.yaml` has one live region, `in`: `control: true`, datacentres `in-mum-1` and
    `in-hyd-1`, `suggest_for: [IN, AE, ID]`. Fourteen other jurisdictions are `on_request`.
  - Migration `0011` makes `tenants.region` `NOT NULL`, with no default, immutable.

### B2. The verdict

**The org model persists across regions. It is, in fact, RFC-0017's model with the word changed.**

What survives unchanged:

- an org owns businesses;
- a business is a tenant with one immutable region;
- accounts live inside businesses;
- grants are the only way in;
- ADR-0024 D9 is not engaged — no third discriminator on business data;
- Paridyum's business is ordinary.

### B3. What changes in the first drafts, and why

| # | First draft said | Region-correct form | Forced by |
| --- | --- | --- | --- |
| 1 | the portal runs in **one** platform tenant | a **platform tenant in every region**. An org is an account of *its region's* platform tenant; the portal is one global host fronting regional backends | RFC-0014 D3 (`db.tenant_scoped_tables_are_regional`, implemented); RFC-0017 |
| 2 | ownership is an account-scoped row inside the platform tenant | ownership is part of the **control-plane skeleton**: org id, region, plan, owned tenant ids — identifiers only | RFC-0014 D2; RFC-0017 |
| 3 | the control-plane instance becomes the platform instance at `id.paridyum.in`, **staff and customers in one Zitadel organization** | **staff stay in the control region's staff instance**; each region gets a **platform instance** for its orgs' people | RFC-0017 (staff vs operators); ADR-0047 D1's deprovisioning argument |
| 4 | a business's locale chain includes the org's default at request time | the org's defaults **seed** the business at creation; the business owns its declarations thereafter | RFC-0015 D1 (a request never reads another region); ADR-0034 amended |

### B4. The cross-region flows, enumerated — and there are only five

**RFC-0015's rule is that nothing is copied into a region.** The design keeps the crossings to
this list, and each is necessary:

1. **Gated actions read the control plane** — entitlement, `max_tenants`, plan (RFC-0015 D1).
2. **Provisioning spans the control plane and the target region** — the saga (RFC-0015 D1). An org
   in `in` creating a business in `eu` writes to the control plane and to `eu`, and **nothing to
   `in`** except the ownership skeleton, which is central anyway.
3. **Login discovery.** A person's credential lives in one region's platform instance. The portal's
   sign-in asks which region: ordered by `suggest_for` against the visitor's country, never
   inferred, remembered on the device. Email links name their region.
   - **No global email→region index exists**, because a hashed email is still personal data.
   - Auth0 solves the same problem by putting the region in the tenant's login host.
4. **A person working in an org or business outside their credential's region.** The admitting org
   or business holds its **own record** of that person — a linked user plus the grant — in its own
   region.
   - This is the Entra B2B shape: *"represented in your directory as a user object"*, credentials
     staying at home.
   - It is the admitting side's fact: who it let in, and what they did. It is not a copy of the
     person's account.
   - It is minimal: subject, display name, a contact address, and locale and timezone preferences.
     It is consented at the grant.
5. **Portal views across regions.** A member's session sees their org's businesses from the
   control-plane skeleton (identifiers and regions), and opening one moves them into that
   business's region under a grant.

### B5. What multi-region does NOT require now

**RFC-0015 D2 stands: one region is live.** The model must be *N-region-correct in shape* — region
columns, plane declarations, region-scoped identity hosts, the skeleton/person split — and
nothing more is built until a second region's trigger fires.

The first drafts violated the shape in places, which is why they change. The build plan does not
grow.

---

## Part C — localisation and languages

### C1. The estate's i18n model

- **GLO-0006 — locked.**
  - A locale is one of **four independent fields**: `language`, `locale`, `timezone`, `currency`.
  - They exist at the platform, operator, tenant, account and user levels, and a request may
    override: six steps in ADR-0034's chain.
  - Its explicit examples of why nothing may be derived from a country: the traveller, the Hindi
    reader, the British-spelling user.
- **ADR-0034 (accepted)** — the chain: request override → user preference → account default →
  tenant default → *"operator default — who pays us"* → platform default.
  - D2: the platform default is a declaration, not a fallback.
  - D3: `Accept-Language` is a hint, never a preference.
  - D4: a locale in a token is a cache.
  - D5: a reader's locale renders and never reinterprets stored data.
  - D6: out-of-band messages resolve against the recipient at send time.
  - D7: every tenant declares `supported_locales`.
  - **The "operator default" is the org.** It was already in the chain, under the old word.
- **ADR-0096 (accepted)** — localized content is `jsonb` keyed by locale, and it *"MUST carry every
  active **estate** locale… Today that is `{"en": …, "ar": …}`"*. Enforced by `CHECK`; widening the
  set is a migration.
- **STD-0010 (accepted)** — the four fields as separate columns, each `text` except currency
  (`char(3)`); `timestamptz` UTC; UTF-8; non-Latin test fixtures; *"A tenant additionally carries
  `supported_locales`."*
- **ADR-0032 / ADR-0033 (the latter locked)** — identifiers are locale-independent; money is minor
  units plus an ISO 4217 code.
- **REQ-0009 (deferred)** — accessible, translatable, locale-correct UI. **Trigger:** *"The first
  user-facing interface is written — the first screen… a person outside the organisation will look
  at."* **Fired** — careers and the landing are public.
- **REQ-0019 (deferred)** — language-aware search, sort and collation. **Trigger:** *"The first
  feature that searches user-supplied text, or orders it for display, across more than one row."*
  It fires with the portal's first sorted list, if it has not already.

### C2. What the code and schema actually do

- **`LOCALES = ["en", "ar"]`** in `web-dashboard/src/lib/locale.ts` and `web-hiring/src/lib/locale.ts`.
  The file's own comment: *"TWO IS A SECURITY FLOOR, NOT A PRODUCT DECISION"* — a CVE bypasses
  proxy checks with exactly one locale — and *"`ar` is the right-to-left cut. It is not a market
  claim."*
- **`job_postings` (`0007`, applied) and the landing sections (`0016`, pending)** hard-code a
  `CHECK` requiring **both `en` and `ar`** in every localized field.
- **`tenants` (`0001`) has `id`, `slug`, `name`, `created_at` and, from `0011`, `region`.** There is
  no `supported_locales` and no locale defaults. STD-0010 §1 and ADR-0034 D7 are **unimplemented**
  at the tenant level.
- **`applications.locale`** (`0001`) exists — the one per-row locale column.

### C3. The gap: two locale sets are being treated as one

| Set | What it is | Whose | Today |
| --- | --- | --- | --- |
| **Software locales** | the languages a product's UI chrome ships in (`messages/*.json`) | a product release — the platform's | `en`, `ar` for every web product |
| **Content locales** | the languages a business publishes its own content in | a business — declared by its owner | not modelled; forced to `en` + `ar` by `CHECK` |

**For Paridyum's business, the two happen to coincide.** For any other business they do not: a
Hindi-and-English business could not store a job posting, and an English-only one would be forced
to write Arabic. ADR-0096 was written when postings were Paridyum's alone; RFC-0048 made them every
tenant's.

### C4. The design the locales draft carries

1. **Two sets, named.** A product declares its software locales; a business declares its content
   locales at the portal. Neither is derived from the other, from the region, or from the
   jurisdiction.
2. **Activation gate.** A business may activate a content locale only when every product it runs
   ships software in it, and every localized row it owns has that locale.
3. **Enforcement moves from an estate `CHECK` to a per-business check.** A function in the business's
   regional database compares a localized field's keys to that business's declared set. `0016`
   should take this shape before it is applied; `0007`'s `CHECK` is replaced before a second
   business posts a job.
4. **Two chains.**
   - **Portal:** request → member → org → platform default.
   - **Business:** request → user → account → business → platform default.
   - **The org's defaults seed a business at creation.** The business owns them afterwards; no
     request reads across regions.
5. **All four fields are declared per business, separately from its region.** Jurisdiction,
   language, locale, timezone and currency are five separate choices at creation.
6. **Out-of-band mail renders in the recipient's preference.** For a grantee from another region,
   that preference is held on the business's own record of them (B4 item 4).
7. **Identity texts** — Zitadel login and message texts — exist per business per content locale, set
   at the portal.
8. **Store listings exist per content locale** (store-accounts RFC).
9. **REQ-0009 and REQ-0019 move to `accepted`** — their triggers have fired.

---

## Part D — the gap register

Status keys: **Closed-by** names the draft that closes the gap; **Decide** means the operator
decides; **Build** means the gap closes in code after its artifact lands.

| # | Gap | Evidence | Status |
| --- | --- | --- | --- |
| 1 | The party above a tenant is unmodelled while self-serve needs it | ADR-0109 D3 vs PLAN D5/D6, ADR-0101 | Closed-by ADR-NEW-ORG |
| 2 | "Org" means three things | A2 | Closed-by ADR-NEW-VOCAB (+ fence) |
| 3 | `/v1/tenants` is a staff-only list of every tenant; a self-serve token must never reach it | `handlers.rs` | Closed-by ADR-NEW-ORG D7 · Build |
| 4 | The control plane runs inside Paridyum's business tenant | ADR-0100 D3 (revised), RFC-0049 §B | Closed-by ADR-NEW-SPLIT |
| 5 | Default hosts: `<tenant>.paridyum.com` puts businesses inside Paridyum's zone; own-domain-only blocks individuals | PLAN D8; the 2026-09-27 decision | Closed-by ADR-NEW-ROUTING · Decide (the app zone's name) |
| 6 | Identity re-home was "everything to `.in`" | the 2026-09-27 plan | Closed-by ADR-NEW-IDENTITY (by plane) |
| 7 | Roles lose their organization (F-H) | `claims.rs`, `verify.ts` | Closed-by ADR-NEW-IDENTITY D5 · Build (lib-auth stage 1) |
| 8 | Support access undesigned; C-0008 forbids a Paridyum-only path | GLO-0005; RFC-0022 D5 | Closed-by ADR-NEW-GRANTS |
| 9 | Per-instance grants undecided | ADR-0104 cost 4 | Closed-by ADR-NEW-GRANTS D6 |
| 10 | An access decision could exist only in Zitadel, which pgaudit does not audit | `infra-kubernetes#8`; RFC-0022 §D5a path 8 | Closed-by ADR-NEW-GRANTS D3 |
| 11 | The org's people would sit in the control region — against RFC-0017 | first-draft ADR-NEW-ORG and ADR-NEW-IDENTITY | Closed-by ADR-NEW-REGIONS + revisions |
| 12 | Ownership must be control-plane skeleton, not a regional row | RFC-0014 D2 | Closed-by ADR-NEW-REGIONS, revised ADR-NEW-ORG D4 |
| 13 | Staff and customers were merged into one identity store | first-draft ADR-NEW-IDENTITY D6 vs RFC-0017 | Closed-by revised ADR-NEW-IDENTITY |
| 14 | Login discovery across regions without a global index of people | RFC-0017 cost 1 | Closed-by ADR-NEW-REGIONS D6 |
| 15 | A person working across regions: whose record, where | Entra B2B pattern | Closed-by ADR-NEW-REGIONS D7, ADR-NEW-GRANTS |
| 16 | An org's region: how chosen, and can it move | RFC-0015 D5 applies to tenants only | Closed-by ADR-NEW-REGIONS D2 |
| 17 | The multi-region foundation is all `proposed` — RFC-0014, RFC-0015, RFC-0016, RFC-0017, RFC-0051, GLO-0007/0008 — and the org drafts rest on it | frontmatter | **Decide** — accept them, or the org ADRs cite them as direction only |
| 18 | Two locale sets conflated; the schema forces `en` + `ar` on every business | C2, C3 | Closed-by ADR-NEW-LOCALES · Build (`0016` before apply; `0007` replaced later) |
| 19 | Tenants declare no locales, against STD-0010 §1 / ADR-0034 D7 | `0001`, `0011` | Closed-by ADR-NEW-LOCALES · Build |
| 20 | ADR-0034's org level cannot be read across regions at request time | RFC-0015 D1 | Closed-by ADR-NEW-LOCALES (seed at creation) |
| 21 | REQ-0009's trigger has fired; REQ-0019's fires with the portal's first sorted list | triggers | Closed-by ADR-NEW-LOCALES (moves both to `accepted`) |
| 22 | Signup: verification, abuse controls, half-finished signups — owed since PLAN §2 | PLAN-TENANT-FLOW §2 | Closed-by RFC-NEW-SIGNUP |
| 23 | Apps-as-a-service: which apps a business's customers may use, and who installs | the other agent's missing-piece file | Closed-by RFC-NEW-APPS |
| 24 | App-store publishing per business; tenant-supplied credentials | Apple 4.2.6; Google service accounts | Closed-by RFC-NEW-STORES · Decide (later phase) |

**Known and not re-opened here:**

- billing's split between control plane and region (RFC-0017 open question 1 — waits for
  `platform-billing`);
- the public-launch risk ADR vs rehearse-first (T-G.7);
- docket's ADR-0112 kind;
- RFC-0047's stamp for a party above orgs.

---

## Part E — decisions only the operator can take

1. **Accept the multi-region foundation** (gap 17): RFC-0014, RFC-0015, RFC-0017, RFC-0051 and
   GLO-0007/0008. The org drafts are written to them. If any is rejected, the drafts that cite it
   change.
2. **Sign the D9 reading** (ADR-NEW-ORG D8).
3. **Name and buy the app zone.**
4. **Keep `auth.paridyum.com` or move to `login.paridyum.com`** for Paridyum's business.
5. **The order of landing.** Proposed:
   - vocabulary;
   - regions and org together;
   - split;
   - identity;
   - grants;
   - routing;
   - locales;
   - then the three RFCs and RFC-0055's revision.

---

## Sources — all retrieved 2026-09-27/28

**Microsoft**

- [Granular delegated admin privileges (GDAP) introduction — Microsoft Learn](https://learn.microsoft.com/en-us/partner-center/customers/gdap-introduction)
- [Customer-Led Removal of a Granular Admin Relationship — Microsoft Learn](https://learn.microsoft.com/en-us/partner-center/customers/gdap-customer-terminate)
- [What is Microsoft Entra B2B collaboration? — Microsoft Learn](https://learn.microsoft.com/en-us/entra/external-id/what-is-b2b)

**Accounts, workspaces and regions**

- [AWS account management and separation — AWS Well-Architected](https://docs.aws.amazon.com/wellarchitected/latest/security-pillar/aws-account-management-and-separation.html)
- [Account Management — Vercel](https://vercel.com/docs/accounts)
- [Create Tenants — Auth0 Docs](https://auth0.com/docs/get-started/auth0-overview/create-tenants)

**Agency and accountant precedents**

- [Do your firm's accounting in QuickBooks Online Accountant — Intuit](https://quickbooks.intuit.com/learn-support/en-us/help-article/company-file/firms-accounting-quickbooks-online-accountant/L8hCIQ77L_US_en_US)
- [Manage your ProAdvisor Preferred Pricing plan and ProAdvisor discount clients — Intuit](https://quickbooks.intuit.com/learn-support/en-us/help-article/manage-client/manage-wholesale-discount-plan-wholesale-discount/L13MoEwj6_US_en_US)
- [Agency Sub-Accounts in HighLevel Explained — HighLevel](https://help.gohighlevel.com/support/solutions/articles/155000007979-what-is-an-agency-sub-account-in-highlevel-)

**ZITADEL**

- [Identity providers — ZITADEL Docs](https://zitadel.com/docs/guides/integrate/identity-providers/introduction)
- [Retrieve User Roles — ZITADEL Docs](https://zitadel.com/docs/guides/integrate/retrieve-user-roles)
- [External User Role Assignments — ZITADEL Docs](https://zitadel.com/docs/concepts/features/external-user-grant)
- [Roles and Role Assignments — ZITADEL Docs](https://zitadel.com/docs/guides/manage/console/roles)

**App stores and hosting**

- [App Review Guidelines (4.2.6) — Apple](https://developer.apple.com/app-store/review/guidelines/)
- [Google Play Developer API: Getting Started — Google](https://developers.google.com/android-publisher/getting_started)
- [Cookies and the Public Suffix List — Heroku](https://devcenter.heroku.com/articles/cookies-and-herokuapp-com)

**Indian law**

- [DPDP Act 2023, s.2](https://indiankanoon.org/doc/20425670/), [s.7](https://indiankanoon.org/doc/62814281/), [s.8](https://indiankanoon.org/doc/186118625/) — Indian Kanoon

**A caution on quotations.** Quotations were taken through a fetch-and-extract tool; check each
against its page before an artifact rests on it.

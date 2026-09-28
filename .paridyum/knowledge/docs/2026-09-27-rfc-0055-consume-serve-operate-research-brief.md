# Research brief — use, serve, operate: the rung RFC-0055 is missing

> **Superseded in part, 2026-09-28 — ADR-0113 to ADR-0122.** A
> research record, kept as written. Its findings (F-A to F-I) were adopted; **its proposals were
> not all adopted**, and where it speaks of an **operator** owning tenants, of a client served as an
> account, or of an unnamed party above a tenant, the landed model is: an **org** (GLO-0012, ADR-0114)
> owns tenants; a client company is **its own tenant, reached by a grant** (RFC-0055 as revised,
> ADR-0120). RFC-0055 was retitled and its file renamed on 2026-09-28. **Follow the ADRs, not this
> brief.**

**Written 2026-09-27.** Prose in `docs/`, not a knowledge artifact. It binds nothing; where it and a
knowledge artifact disagree, the artifact wins. Written for the operator, and for an independent
reviewer — human or agent — that has no context of the session that produced it.

It answers the operator's question below, and reviews
**`rfc/RFC-0055-consuming-and-operating-are-separate-flows-and-a-client-is-an-account-before-it-is-a-tenant.md`**
(`proposed`, written earlier today by another session from the same question) against the schema,
the code, the accepted artifacts and outside practice. **RFC-0055 was not edited**: its author
session was still researching this area at 22:27, and two writers on one file lose work.

---

## The question

> *"the current flow is very janky as the tenant is getting connected and apps follow that tenant
> but if someone wants to run docket as a business and also use docket as the same time this flow
> is a bit not clear, there are two routes first they deploy docket as a service and then create a
> tenant inside that or they should have both of these separate as docket being a simple app they
> can enable and consume under their tenant and a separate business store for enabling
> businesses."*

## The answer

**Route 2 — separate the flows — is right, and route 1 should be refused at the product level.
RFC-0055 reaches both conclusions. It goes wrong on the rung between them.**

- A company that **uses** docket is a tenant that enables it. Unchanged.
- A company that **serves other companies** with docket — an HR bureau, a CA firm, a PEO — keeps its
  own tenant for its own staff, and works **inside each client's own tenant** through a grant the
  client gives, can see and can revoke. **It does not hold its clients as accounts.** RFC-0055 D2
  says the opposite (*"a client is an account before it is ever a tenant"*), and for docket that
  promotion is impossible: docket's sealed chains refuse `UPDATE` and `DELETE`, so records written
  into a bureau's tenant can never move to the client's (F-A).
- **Selling docket as a business** (franchise) and **renting the platform** (RFC-0047) stay gated,
  as RFC-0055 says.
- The **business store** is a surface on `paridyum.com/dashboard` over a party that holds or serves
  several tenants. That party is the one ADR-0109 declined to model on 2026-09-26, and the tenant
  flow's self-serve signup and `max_tenants` already depend on it (F-G).

So a company that uses docket *and* runs docket for clients is **one tenant of its own, plus
grants into its clients' tenants**. It is not one tenant holding clients as accounts. This is the
shape of Microsoft's partner model and of QuickBooks Online Accountant (F-E).

## What RFC-0055 gets right — keep it

- **F1's four verbs** — use, serve, sell the business, rent the platform. The jank really is four
  things sharing one phrase.
- **F2** — only the provisioning saga creates tenants, and a product never does. Route 1 ("deploy
  docket as a service, then create tenants inside it") would be a second tenant-creator; refusing
  it at the product level is correct.
- **D1's separation of surfaces**, and that the store is **not a marketplace** (C-0006's framing
  table).
- **D4** — RFC-0044's "docket as a service" means Paridyum selling to consuming tenants.
- **D5** — franchising waits on its gates. Nothing in this brief moves a gate.

---

## Findings

### F-A. For docket, "a client is an account before it is a tenant" is a one-way door

Migration `0006` puts `paridyum.forbid_mutation()` on `BEFORE UPDATE OR DELETE` of
`docket_identities`, `docket_register`, `docket_events` and `docket_signatures` (ADR-0093). `0012`
made `account_id` `NOT NULL` on all five person-bearing tables, beside the `tenant_id` they always
carried.

So a client whose records start as an account inside a bureau's tenant can never become a tenant
**with its history**. The rows cannot be re-keyed (no `UPDATE`) or removed from the bureau's tenant
(no `DELETE`). A copy inserted elsewhere leaves the original behind, permanently.

This is ADR-0104 decision 2's own argument — *"three of those five tables could not have been
retrofitted at all"* — applied to the tenant boundary instead of the account boundary. There it
justified retrofitting while the tables were empty. Here it means **the container a client's
records live in is final at the client's first record.**

**Accounts stay right for what ADR-0104 was written for**: *"Two dockets, for two divisions or two
jurisdictions, under one tenant"* — separate records, one legal owner.

### F-B. Under DPDP, the employer carries the obligation and a bureau is its processor

From the Digital Personal Data Protection Act, 2023:

- **s.2(i)** — a Data Fiduciary is *"any person who alone or in conjunction with other persons
  determines the purpose and means of processing of personal data"*.
- **s.2(k)** — a Data Processor is *"any person who processes personal data on behalf of a Data
  Fiduciary"*.
- **s.7(i)** — processing *"for the purposes of employment"* is a legitimate use.
- **s.8(1)** — the Fiduciary stays responsible, *"irrespective of any agreement to the contrary"*,
  for processing done *"on its behalf by a Data Processor"*.
- **s.8(2)** — a Processor is engaged *"only under a valid contract"*.
- **s.8(7)(b)** — the Fiduciary must *"cause its Data Processor to erase any personal data that was
  made available by the Data Fiduciary for processing"*.

An employer is the fiduciary for its staff's records, a bureau is its processor, and Paridyum sits
further down. **Client-as-account puts the fiduciary's records inside the processor's
container.** There the fiduciary has no tenancy and no relationship with Paridyum. Given F-A, it
also has no way to take the records back, or to have the bureau's copy erased.

**Client-as-tenant keeps the container with the fiduciary** and admits the processor by grant.
This is the Act's text set against the schema, not legal advice. A lawyer should confirm the
mapping before a contract relies on it.

### F-C. An account cannot stand for a company

- **One product per account.** `accounts` is `(tenant_id, product, slug, display_name)` (`0012`). A
  client using docket and careers would be two unrelated accounts, with nothing grouping them.
- **No identity of its own.** ADR-0047 D2: *"A tenant is a Zitadel organization. An account is not
  a Zitadel object."* A client held as an account has no users, login, branding, password policy
  or SSO of its own; its people become users of the bureau's organization.
- **docket's configuration is per tenant** (ADR-0099 D2, `0008`): companies, signatories, operators,
  templates. A client's letterhead and signatories are tenant-level facts, and so is branding
  (ADR-0084).

### F-D. "Serve clients as accounts — available today" is not true for docket

RFC-0055 F1 lists *Serve* as "ADR-0104; available". Its F3 says client-as-account, with "the
client's people as users with grants", is "available today, by decision, with no gate".
Measured against the code:

1. **docket serves exactly one account per tenant.** `tool-docket/src/adapters/pg.rs` `enter()`
   looks up `product = 'docket' AND slug = 'default'`. The module header calls this "an interim":
   "this repository has no such routing yet".
2. **Per-instance grants are undecided.** ADR-0104 cost 4, restated in ADR-0108 D5: *"An operator
   with the tenant's operator role still reaches every instance, on purpose and for now."*
3. **Every docket policy requires `has_operator_scope()`** (`0014`), and operator scope spans every
   account of the tenant (GLO-0005). No role exists that would admit a client's HR person to their
   own instance without also admitting them to the bureau's other clients.
4. **Addressing by path needs a grant to be legal.** ADR-0104 D5 puts the account in the URL
   (`/app/<product>/<account>/…`). **ADR-0024 D5 (locked)** takes both identifiers *"from the
   verified session only. Never a request body, query string, path segment or client-supplied
   header."* A path may only choose among accounts the session is already granted, which needs
   point 2.

### F-E. The missing rung already exists — in RFC-0047's "flat" reading and in outside practice

**RFC-0047 already weighed it.** It compared two readings of "tenant B serves tenant C":

- **Flat** — C is a sibling row; *"ADR-0024 decision 9 never fires"*; *"the hard part becomes
  authorisation"*.
- **Nested** — C lives inside B's own stamp.

RFC-0047's 2026-09-24 finding answers "nested" — prompted by the operator's case of a partner
whose customer needs another region, and still `proposed`. It answers for RFC-0047's own party: one
that wants to *"rent the whole business… brand it as their own"*.

**A bureau is not that party.** It does not want its own platform; it wants its clients on
Paridyum and its staff admitted to them. RFC-0055 D2 routes every client-as-tenant case to
RFC-0047, and so to its heaviest machinery: a separate cluster, database and Zitadel instance per
partner.

**The rung is lighter than even the flat reading, because nothing is owned.** The client's tenant
is the client's; the bureau holds a grant.

**It creates no cross-tenant scope.** Each grant lives in the client's tenant, and a session acts
in one tenant at a time. That is GLO-0005's own rule for a principal in several tenants: *"holds
operator scope in each, separately, and never simultaneously in one session"*.

Outside practice converges on it:

- **Microsoft (GDAP).** *"Customers must explicitly grant the least-privileged access to their
  partners"*. Access is *"granular and time-bound"*. A customer can *"Remove a partner's GDAP from
  their tenant"*, after which the partner's users *"no longer have access to administer
  services"*.
- **QuickBooks Online Accountant.** A firm's own books are separate — *"where you manage your firm's
  books, rather than your client's"*. A client can move off the firm's billing, and *"It doesn't
  impact their accounting data in any way."*
- **HighLevel is the counter-example**: route 1 as a product (agency → client sub-accounts,
  white-label, rebilling). Even there the agency's own work is kept apart: *"Use an Agency
  Sub-Account for your own business instead of mixing agency operations with a client account."*
  That is the franchise shape (BET-0005's white-label), not the service-provider one.
- **Zitadel supports the rung natively.** An external user role assignment lets an organization
  *"grant access to projects within your organization to users from other organizations"*, and
  *"A user account is always unique across a ZITADEL instance"* — one identity, no second login.
  The roles claim maps each role to the organization ids it was granted in.

### F-F. Two readings of "consumer" are both live, and product kind is the switch

**The account reading.** BET-0005: *"A small business or freelancer wanting invoicing software and
a founder wanting to run an invoicing business… the first is an `account` inside Paridyum's own
tenant, the second is a `tenant`."* BET-0006 and BET-0007 build on it, and C-0006 anti-goal 6's
2026-08-15 note repeats it.

**The tenant reading.**

- RFC-0048 (accepted): *"docket is becoming a multi-tenant HR platform and its tenants will post
  jobs."*
- RFC-0049: a tenant is a set of Releases.
- The tenant flow: a stranger provisions a tenant that runs docket, a landing and careers.
- RFC-0044, as qualified today.

**Product kind reconciles them.** A `capability` is consumed by tenants; a `catalogue` product
serves end customers as accounts of the tenant that operates it (`product-registry.yaml`, ADR-0112).

**RFC-0055 cites both readings and states that reconciliation nowhere.** It also prejudges docket's
owed ADR-0112 re-review: D1 calls the consumption flow *"RFC-0049, unchanged and correct"*, while
open question 3 says the RFC *"does not prejudge"* that re-review.

**For docket, F-A to F-C all point one way: a company using docket is a tenant.** If the ADR-0112
re-review makes docket a catalogue product, its end customers have to be tenants, which is not the
shape BET-0005's table draws. "Franchising" docket would then run through the provider rung, not
through accounts.

Two constraints bear on that decision:

- **Hosting is vertical one** (`catalogue`, `building`, gate passed 2026-08-23), so anti-goal 6
  applies to any second vertical.
- **Anti-goal 1 requires Paridyum to operate docket for real end customers** (the `operated`
  stage). Using docket for its own staff does not count.

### F-G. The business store needs a party the model removed

- **The tenant flow depends on it.** PLAN-TENANT-FLOW D5 (fully self-serve) and D6 (`max_tenants`,
  default 3) need a principal that holds several tenants. ADR-0101 (accepted) says *"D6 caps
  tenants at three per operator"* and *"an operator's plan carries `max_tenants`"*. Its decision 1
  gives `platform-tenancy` *"the operator → tenant → account hierarchy"*, owning `Operator`.
- **ADR-0109 (accepted 2026-09-26) retired that word and declined the entity.** Its D5 correction
  list is GLO-0001, GLO-0004, BET-0005 and BET-0002. ADR-0101 and the plan are not on it and still
  use the old reading, and GLO-0005 still says *"An operator owning several tenants"*.
- **The code has only the staff reading.** `platform-tenancy`'s `GET /v1/tenants` returns every
  tenancy to any control-plane token — *"Paridyum's own staff, `id.paridyum.com`"*. `web-dashboard`
  lists them all, and nothing scopes a tenant to whoever created it.
- **One person across several tenants is undesigned.** A person with three tenants has either three
  logins (against ADR-0089, *"one identity per person"*) or one identity holding roles in each —
  the same mechanism as the bureau. The plan's one sentence on it names `id.paridyum.com` for a
  tenant operator's sign-in, which `lib-auth` defines as the staff plane.

Whether a holding or billing relation on the control plane is ADR-0024 D9's "third level" is a
reading for the operator. It has the same shape as RFC-0047 open question 8.

This brief's view, offered and not decided: D9 guards a third discriminator in tenant-scoped data
(*"Two explicit columns can be migrated to a path; a path cannot easily be migrated back"*). A
relation on the control plane adds no discriminator, and rung 3 below needs no party at all.

### F-H. Latent defect: both auth libraries drop the organization a role was granted in

- `libs/lib-auth-rust/src/claims.rs` `roles_from_claims` returns `roles.keys()`.
- `libs/lib-auth-typescript/src/verify.ts` returns `Object.keys(rolesClaim)`.
- Both take the tenant from `urn:zitadel:iam:user:resourceowner:id`, the user's **home**
  organization.

Zitadel's roles claim maps each role to the organizations it was granted in. So under an external
role assignment, **"operator in client C" would be read as "operator in the user's own tenant B"
and would grant nothing in C** — the wrong answer on both sides.

- **Not live today.** Nothing in the estate creates a cross-organization assignment:
  `provision-org.py` grants roles to the users it creates in the organization it provisions, and
  `platform-identity` serves `GET /v1/roles` only.
- **Unverified:** whether a tenant's own managers can create an external assignment on a Paridyum
  product project, as grants are configured. Zitadel's console guide says users of other
  organizations can be added, not who may add them.
- **The fix is small and belongs before anything cross-organization exists.** Honour a role only
  for the organization it was granted in, and derive the acting tenant from that organization.
  This amends ADR-0047 D5's claims contract (tenant from `resourceowner`), which is accepted, not
  locked.
- **It is a precondition for rung 3, and for one person holding several tenants (F-G).**

### F-I. C-0008 makes the rung an obligation, not an extra

- **Support access is undesigned.** GLO-0005 calls it *"a different mechanism and a different
  decision (deferred…)"*.
- **RFC-0022 D5 (accepted today) promises visibility:** *"If a Paridyum operator accesses a
  tenant's data, that tenant can see it"*.
- **C-0008 forbids a Paridyum-only path:** *"Special-casing Paridyum's tenant anywhere —
  provisioning, entitlement, quota, billing or support — beyond what is published and available to
  any tenant on the same terms."*

So whatever lets Paridyum's staff into a tenant — consented, scoped, expiring and visible to the
tenant — must be a mechanism any tenant can use. **That is the bureau's grant**: one mechanism, two
users, built once.

Its storage half exists. `operator_grants` (`0010`) records subject, scope, granter, expiry and
revocation. It is readable by the tenant the grant is over, and its provenance is immutable.
`audit_events` carries the visibility.

---

## The ladder, corrected

| Rung | What it is | Boundary | Today | Waits on |
| --- | --- | --- | --- | --- |
| **1. Use** | A tenant enables a product for itself | tenant | available (RFC-0049) | — |
| **2. Divide** | One company, several instances — divisions, jurisdictions | account (ADR-0104) | schema yes; docket serves `default` only | per-instance grants (ADR-0104 cost 4) |
| **3. Serve** | A provider works inside clients' own tenants | tenant, plus a grant the client can see and revoke | **not built, and absent from RFC-0055** | F-H fix; one identity across tenants; the support-access design (F-I) |
| **4. Sell the business** | Franchise the playbook | per BET-0005 | gated | ADR-0112 re-review; `operated`; anti-goals 1 and 6; BET-0005 clause 1 |
| **5. Rent the platform** | Own brand, own control plane, own tenants | stamp (RFC-0047) | gated | D9 reading; ADR-0064; the `platform` dossier |

**The operator's scenario — use docket and run docket as a business — is rung 1 for the
business's own staff, plus rung 3 for its clients. A separate company is never rung 2.**

Route 1 ("deploy docket as a service, then create tenants inside it") is rung 5 if the things
inside are real tenants. If they are accounts, it misuses rung 2.

## What each surface would show

A proposal, not a decision:

- **`<tenant>.paridyum.com/dashboard`** (`web-console`) — this business's apps: enable, open, see
  health. A business that only uses Paridyum never needs anything else. This is the "app store".
- **`paridyum.com/dashboard`, for a signed-in customer**, shows three things:
  - the businesses they hold, and creating one (capped by `max_tenants`);
  - the clients that have granted them access (rung 3);
  - later, behind the gates, **"run a business"** — the business store.

**As written today, `paridyum.com/dashboard` is Paridyum's staff surface and lists every tenant.**
That must not be what a self-serve customer's token reaches (F-G).

## Decisions only the operator can take

1. **docket's kind** (ADR-0112 D4): capability, or catalogue? If catalogue, are its end customers
   tenants (F-A to F-C) or accounts (BET-0005's table)?
2. **The party above a tenant.** Should it be modelled on the control plane for self-serve and
   `max_tenants`? And does doing so engage ADR-0024 D9?
3. **Rung 3.** Should it be designed now, as the same mechanism as support access (F-I)?
4. **Timing.** C1/C2 (self-serve signup, `max_tenants`) are the tenant flow's next items. Should
   they wait for decision 2? At minimum, a self-serve token must not reach `GET /v1/tenants` as it
   is written.
5. **The auth libraries.** Should the role→organization binding (F-H) be fixed now?

## If this brief is accepted, what changes in RFC-0055

- **Title and D2:** a separate company is a tenant; an account is a division. "Before it is a
  tenant" goes.
- **F3:** the five rungs above replace the three; rung 3 is new.
- **F1:** *Serve* becomes "not built", and *Divide* becomes its own row.
- **D1:** take ADR-0112's docket decision, or stop calling the consumption flow "unchanged and
  correct" for docket.
- **Frontmatter:** a `sources:` block carrying the entries below. Until an artifact declares them,
  the Stop hook files each fetch as an open loop in `DECISION-DEBT.md`, as intended.

---

## Evidence index

| Claim | Where |
| --- | --- |
| Four sealed chains refuse UPDATE and DELETE | `platform-tenancy/migrations/0006_docket_records_and_the_sealed_chains.sql` |
| `accounts` is one product instance; `account_id` NOT NULL on docket's five tables | `…/0012_accounts_and_the_docket_instance_retrofit.sql` |
| Every docket policy requires operator scope and the current account | `…/0014_the_inner_boundary_the_retrofit_left_out.sql` |
| `operator_grants`: tenant-readable, provenance immutable, expiry and revocation | `…/0010_one_correlation_id_and_an_append_only_audit_stream.sql` |
| docket resolves only the `default` account, as an interim | `tool-docket/src/adapters/pg.rs` (module header, `enter()`) |
| The registry lists every tenant to any control-plane token ("Paridyum's own staff") | `platform-tenancy/src/handlers.rs` (`list_tenants`, `require_control`) |
| The dashboard lists every tenant | `web-dashboard/src/lib/tenancy.ts` (`fetchTenants`) |
| Roles lose their organization; tenant is `resourceowner` | `lib-auth-rust/src/claims.rs`; `lib-auth-typescript/src/verify.ts` |
| The flow is mostly unbuilt (measured 2026-09-27) | `PLAN-TENANT-FLOW.md`, "Measured 2026-09-27 — what exists between here and C8" |
| Hosting is vertical one; docket's re-review owed | `product-registry.yaml`, entries `hosting` and `docket` |
| Account vs tenant readings | BET-0005 "The three levels"; RFC-0048 Problem item 4; RFC-0049 §B; RFC-0055 D1 and open question 3 |
| The uncorrected "operator" | ADR-0101 context and decisions 1 and 3; ADR-0109 D5; GLO-0005 "What it is not" |
| Flat and nested readings; the nested answer | RFC-0047 "The crux this raises" and "open question 9 is answered" |
| Support access deferred; tenant-visible access; no special-casing | GLO-0005; RFC-0022 D5; C-0008 "What this forbids" |

## Sources — all retrieved 2026-09-27

Quotations were taken through a fetch-and-extract tool; check each against its page before an
artifact rests on it.

- Microsoft Learn — [Granular delegated admin privileges (GDAP) introduction](https://learn.microsoft.com/en-us/partner-center/customers/gdap-introduction)
- Microsoft Learn — [Customer-Led Removal of a Granular Admin Relationship](https://learn.microsoft.com/en-us/partner-center/customers/gdap-customer-terminate)
- Intuit — [Do your firm's accounting in QuickBooks Online Accountant](https://quickbooks.intuit.com/learn-support/en-us/help-article/company-file/firms-accounting-quickbooks-online-accountant/L8hCIQ77L_US_en_US)
- Intuit — [Manage your ProAdvisor Preferred Pricing plan and ProAdvisor discount clients](https://quickbooks.intuit.com/learn-support/en-us/help-article/manage-client/manage-wholesale-discount-plan-wholesale-discount/L13MoEwj6_US_en_US)
- HighLevel Support Portal — [Agency Sub-Accounts in HighLevel Explained](https://help.gohighlevel.com/support/solutions/articles/155000007979-what-is-an-agency-sub-account-in-highlevel-)
- ZITADEL Docs — [External User Role Assignments](https://zitadel.com/docs/concepts/features/external-user-grant)
- ZITADEL Docs — [Retrieve User Roles in ZITADEL](https://zitadel.com/docs/guides/integrate/retrieve-user-roles)
- ZITADEL Docs — [ZITADEL Roles and Role Assignments](https://zitadel.com/docs/guides/manage/console/roles)
- Indian Kanoon — [Section 2, The Digital Personal Data Protection Act, 2023](https://indiankanoon.org/doc/20425670/)
- Indian Kanoon — [Section 7, The Digital Personal Data Protection Act, 2023](https://indiankanoon.org/doc/62814281/)
- Indian Kanoon — [Section 8, The Digital Personal Data Protection Act, 2023](https://indiankanoon.org/doc/186118625/)

---
id: ADR-0113
title: Org, tenant and account are three words for three levels, and the vendor's organization is named as the vendor's
type: adr
status: accepted
created: 2026-09-28
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
product: [paridyum, tenancy, identity]
tags: [vocabulary, org, tenant, account, organization, zitadel, rename, retired-terms, lib-auth, adr-0004, adr-0015, adr-0047, adr-0054, adr-0082, adr-0109, adr-0114, glo-0002, glo-0003, glo-0012]
breaking: true
enforced_by:
  - check: knowledge.retired_terms_absent
    severity: error
    args:
      terms: ["platform tenant", "operator → tenant", "home region", "client is an account", "operator default"]
      exempt: [ADR-0113, GLO-0012, GLO-0007]
    rationale: >
      Decision 5 made mechanical. Every retired phrase on the list is one an agent would act on:
      "platform tenant" builds the portal inside a tenant, "operator → tenant" names the org
      "operator", "home region" re-creates the term GLO-0007 retired. A file that must keep one —
      history, or a body a dated note corrects — carries that note, and the check accepts the file
      only when the note names this ADR. It refuses the phrase, not the idea; review is the
      defence against a fresh phrasing of an old mistake.
  - check: repo.retired_org_identifiers_absent
    severity: warn
    args:
      retired: [require_organization, organization_id, is_missing_organization, NoOrganization, ORGANIZATION_CLAIM, servedOrgIdOrNull, orgForRequest, wrong_organization, no_organization_claim, provision-org]
    rationale: >
      Decision 4. The code rename is staged, so a retired identifier is legal until its stage-3
      removal and illegal after. `warn` while the sweep runs, raised to `error` in the change that
      removes the deprecated library names. A closed list held in `args` (ADR-0036): the check sees
      names, not meaning, and says so.
---

# ADR-0113 — Org, tenant and account are three words for three levels, and the vendor's organization is named as the vendor's

## Status

**ACCEPTED** 2026-09-28 by the operator, with the rest of the org model (ADR-0114 to ADR-0122).
The operator's instruction: *"get the vocab of main product org vs inner silo tenants — we cannot
afford to mix none of them."*

It lands first, so that the org arrives with its word already cleared rather than sharing it with
the vendor's object for a sprint.

## Context

**The estate already used "org/organization" three ways** (measured 2026-09-28):

1. **The Zitadel object that mirrors a tenant.** ADR-0047 D2 says *"a tenant is a Zitadel
   organization"*, and code across about 20 locations borrowed the word. Two of those locations are
   published API:
   - `lib-auth-rust` — `Principal::organization_id()`, `require_organization()`,
     `AuthError::is_missing_organization()`, `Reason::NoOrganization`, `ORGANIZATION_CLAIM`;
   - `spec-identity`'s contract.

   `web-hiring#5` added `servedOrgIdOrNull()`, `orgForRequest()` and the refusal codes
   `wrong_organization` / `no_organization_claim`. The only immutable instance is already
   vendor-qualified: `tenants.zitadel_org_id` (migration `0005`).
2. **`scope: [org]`** in artifact frontmatter — ADR-0015's *binds every repository*. Locked artifacts
   carry it.
3. **The `org.*` check namespace** — the GitHub organisation (ADR-0004), blocking locked ADR-0004,
   ADR-0009 and ADR-0012.

**And the estate used "operator" for the party this ADR now calls an org.** GLO-0001's reading A,
RFC-0014 D2's *"which operator owns which tenant"*, RFC-0017's *"operators"*, ADR-0101's
*"operator → tenant → account"*. ADR-0109 gave "operator" to its other reading — breadth within one
tenant — and left the party above a tenant unnamed. An agent reading the estate on 2026-09-28 met
three models at once, and a draft the same day stored the org *as an account of a "platform tenant"*:
two levels' words used for a third.

**ADR-0109 settled how to choose between readings:** *"when two readings disagree, the one already
written into something immutable wins, and the one that is only claimed gets corrected."* It also
rejected qualifying both terms: *"a term that needs an adjective every time is a term that failed,
and the adjective will be dropped in the document that matters."* So the qualifier goes on the
vendor's object, where it is true anyway.

## Decision

**1. The levels have one word each, and no word names two levels.**

| Term | What it is | Column | Never |
| --- | --- | --- | --- |
| **Paridyum** / **the platform** | our software and our estate | — | an org, a tenant |
| **Org** (GLO-0012) | the customer of paridyum.in — a person or a team; owns tenants; members, plan, billing, a region | `org_id` | a tenant, an account, a Zitadel organization, an operator |
| **Tenant** (GLO-0002) | a business: the outer silo for its data, domain, identity and branding; owned by exactly one org | `tenant_id` | an org; "platform tenant" |
| **Account** (GLO-0003) | a silo inside a tenant: a product instance, a customer of the business, a division | `tenant_id` + `account_id` | a tenant, an org |
| **Member** | a person in an org, with an org role | — | an operator |
| **Operator** (GLO-0001, GLO-0005) | a principal with breadth across the accounts of **one tenant** | — | the org, the party that pays |
| **the operator** — in a record of who decided | the person who takes the estate's decisions: Paridyum's founder, as in *"accepted by the operator"* | — | a runtime principal, a level, an org |
| **Staff** | Paridyum's own people, in the staff instance | — | members, operators |
| **The portal** | paridyum.in — the platform's own surface, where orgs work | — | a tenant, a console |
| **The console** | a tenant's own administration surface, on the tenant's host | — | the portal |
| **Tenant zero** (C-0008) | Paridyum's tenant — its business at paridyum.com, owned by the Paridyum org | — | the platform, the portal, the control plane |

- **"Business" is the tenant's name on screen and in conversation, and means exactly a tenant.** In
  governance and code the word is *tenant*. Nothing calls an org a business.
- **An org sits above the silos as their owner. It is not a silo** and holds no business data
  (ADR-0115 D9).

**2. The Zitadel object is always named as the vendor's.**

- `zitadel_org` / `zitadel_org_id` names the object; `resource_owner` names the claim Zitadel calls so
  (`urn:zitadel:iam:user:resourceowner:id`).
- Prose says **"the tenant's Zitadel organization"**. The staff and portal instances each hold one
  Zitadel organization that is a people **directory**; it mirrors neither an org nor a tenant, and
  prose calls it *the instance's directory*.
- Zitadel's own API paths, field names and console words are untouched — they are the vendor's.

**3. Three governance positions keep their older meanings, permanently, and are fenced.**

- **The frontmatter `scope:` value `org`** — ADR-0015's *binds every repository*.
- **The `org.*` check-id prefix** — the GitHub organisation (ADR-0004).
- **"The operator" as the decider** — in a Status line or a quotation, the person who accepted a
  decision. The estate has written *"accepted by the operator"* since August, and every such line
  is a record. It never names a level and never appears in code.

The first two are carried by locked artifacts and are syntactic positions no runtime value occupies;
the third is a person in a record. None may be used to mean anything else. **Every other position is runtime**, and there bare `org` means
GLO-0012 — including the migration directive `-- paridyum:scope org` (STD-0023), which names an
org's own rows. The fence is where the word sits, not what the artifact is about: an artifact about
orgs that binds everywhere still writes `scope: [org]`, as GLO-0012 does.

**4. The code rename is staged, add-then-remove (ADR-0054).**

| Stage | What lands | Where |
| --- | --- | --- |
| 1 | Vendor-qualified names beside the old ones; the old ones deprecated, not removed. ADR-0119 D5's claims-contract fix rides the same release | `lib-auth-rust`, `lib-auth-typescript` |
| 2 | Every consumer moves to the new names | `web-hiring`, `tool-docket`, `platform-identity`, `platform-tenancy`, `web-admin`, `web-dashboard`, `infra-kubernetes/zitadel` (`provision-org.py` → `provision-tenant.py`, `org-*` Make targets → `tenant-*`), `tool-forge`, `spec-identity` |
| 3 | Deprecated names removed, one change per library; `spec-identity` bumps its contract version; `repo.retired_org_identifiers_absent` raised to `error` | libraries, spec |

REQ-0010's rule for a published API applies in spirit although REQ-0010 is deferred: the break is
announced by the deprecation and is never silent.

**5. Retired phrases leave the knowledge base's prose.** `knowledge.retired_terms_absent` refuses,
in every scanned file, the phrases in this ADR's `args.terms`:

| Retired | Say instead | Why |
| --- | --- | --- |
| "platform tenant" | *the portal* (ADR-0118) | the portal runs in no tenant |
| "operator → tenant → account" | *org → tenant → account* | the party above a tenant is the org (GLO-0012) |
| "home region" | *the control region*, or *a tenant's / an org's region* | GLO-0007 retired it |
| "a client is an account" | *a client company is its own tenant, reached by a grant* (RFC-0055, ADR-0120) | an account cannot stand for a company |
| "operator default" | *the org default* (ADR-0122) | ADR-0034's operator is the org |

- **A file that must keep a retired phrase** — history, or a body a dated note corrects — carries a
  blockquote beginning `> **Superseded in part, <date>` that names this ADR. The check accepts such a
  file. Superseded artifacts, `CHANGELOG.md`, the generated `DECISION-DEBT.md`, research logs and
  fixtures are not scanned. The artifacts that define a retirement (`args.exempt`) are not scanned.
- **The list grows by amendment and never shrinks.**

**6. No new bare `org` for the vendor object, from today.** New code uses the vendor-qualified names
at once.

**7. Artifacts that are not locked get a dated note, not a rewrite.** ADR-0109's method for GLO-0001.
Every artifact in the 2026-09-28 inventory carries one (ADR-0114 §Consequences). Locked artifacts
are untouched.

## Consequences

**Accepted costs**

1. **A breaking change to two published libraries and one contract, for a vocabulary reason.**
   Staging means nothing breaks at once, but every consumer is touched, and the sweep competes with
   feature work for review time.
2. **Decision 3 is a permanent exception to decision 1.** A reader who meets `scope: [org]` must
   know it means *binds everywhere*. It is taught here, in GLO-0012 and in the CLAUDE.md router, and it
   cannot be removed without superseding every locked artifact that carries it.
3. **Both checks see words, not meaning.** A new identifier or a new phrase that carries an old
   mistake under a fresh name passes them; review is the only defence against that.
4. **Dated notes leave stale bodies in place.** A reader who skips the note at the top reads the old
   model. The note is placed first for that reason, and the check makes it impossible to keep a
   retired phrase without one.
5. **Vendor material keeps saying "organization" for tenants.** Zitadel's docs, console and API are
   the vendor's, and anyone reading them translates.

**Gained**

- **Four levels, four words, no overlap** — in conversation, code, UI and governance.
- **The vendor object is self-describing** wherever it appears; `zitadel_org_id` was always clearer
  than `organization_id`.
- **The collision is caught while it is small**: one merged PR and a published API with few
  consumers, rather than after the org model has spread through every service.
- **The old models cannot quietly return to the knowledge base**, because the phrases that carried
  them fail the gate.

**Rejected alternatives**

- **"Team".** Collides with nothing; the operator chose "org", with the rename.
- **"Org" on screen only, another noun in the schema.** Two words for one thing leak back into code —
  the failure ADR-0109 corrected for `operator`.
- **Keep "operator" for the paying party.** ADR-0109 D1 gave the word to breadth within a tenant,
  because twelve applied migrations and a published library carry that reading.
- **Renaming `scope: org` and the `org.*` checks as well.** It means superseding every locked
  artifact that carries them to move one word.
- **Rewriting every stale artifact.** Accepted artifacts are the record of what was decided when; a
  dated note corrects the reading without falsifying the record.

## Related

- **GLO-0012** — the term · **ADR-0114** — the entity · **ADR-0115** — the scope category
- **ADR-0119** D5 — the claims-contract change that rides stage 1
- **ADR-0109** — the one-word-one-level method this reuses; superseded by ADR-0114
- **ADR-0054** — add, never swap · **ADR-0036** — parameters from `args` · **STD-0022** — a rename is
  complete
- **ADR-0015** · **ADR-0004** — the two fenced meanings
- **GLO-0001**, **GLO-0002**, **GLO-0003**, **GLO-0005**, **GLO-0007** — the neighbouring terms

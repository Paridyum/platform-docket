---
id: ADR-0122
title: A product ships software locales, a tenant declares content locales, and neither is derived from the other
type: adr
status: accepted
created: 2026-09-28
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, web, library, mobile]
applies_to: ["*"]
product: [paridyum, tenancy]
tags: [i18n, l10n, locale, language, timezone, currency, supported-locales, content-locales, software-locales, rtl, collation, org, glo-0006, adr-0032, adr-0033, adr-0034, adr-0096, adr-0114, adr-0117, std-0010, req-0009, req-0019, rfc-0028, rfc-0048]
breaking: true
enforced_by:
  - check: db.localized_content_matches_its_tenant_locales
    severity: error
    rationale: >
      Decision 4 made mechanical. A completeness test that names fixed locale codes — `v ? 'en' AND
      v ? 'ar'`, as the helper functions of 0007 and 0016 do — encodes one tenant's languages as every
      tenant's. The check refuses a literal locale code tested with the jsonb key-exists operator
      anywhere in migration SQL, so the shared per-tenant function is the only way to write it.
  - check: db.tenant_declares_its_locale_fields
    severity: error
    rationale: >
      STD-0010 §1 and ADR-0034 D7 have required a tenant to carry `supported_locales` since August,
      and no migration gives it one. A declared rule with no column is the defect class ADR-0014
      names — it reads as a fact and is a plan.
verified_by:
  - TST-0059
  - TST-0060
  - TST-0050
---

# ADR-0122 — A product ships software locales, a tenant declares content locales, and neither is derived from the other

## Status

**ACCEPTED** 2026-09-28 by the operator, carrying *"all of the localisation and i18n multi-lingual
stuff"* into the org model.

- **Amends ADR-0096** D1 and D2: completeness is per tenant, not per estate.
- **Amends ADR-0034** D1: the chain splits into a portal chain and a tenant chain, and the org's
  defaults seed a tenant once.
- **Implements STD-0010 §1's tenant fields.**
- **Moves REQ-0009 and REQ-0019 to `accepted`** — both triggers have fired.

**GLO-0006 is `locked`, and it is relied on, never edited.** Its four independent fields and five
levels stand as written; its "operator" level is the org (ADR-0113 D1).

## Context

**The rules already fixed:**

- **GLO-0006 (locked).** A locale is one of four independent fields — `language`, `locale`, `timezone`,
  `currency` — *"Not one field with four meanings, and not one field the other three are derived
  from."*
- **ADR-0034 (accepted)** — the chain: request override → user preference → account default → tenant
  default → the default of *"who pays us"* → platform default. `Accept-Language` is a hint, never a
  preference (D3); a reader's locale renders and never reinterprets stored data (D5); mail resolves
  against the recipient at send time (D6); every tenant declares `supported_locales` (D7).
- **ADR-0096 (accepted).** A localized field is `jsonb` keyed by locale and *"MUST carry every active
  **estate** locale… Today that is `{"en": …, "ar": …}`"*, enforced by `CHECK`.
- **STD-0010 §1** — four columns, and *"A tenant additionally carries `supported_locales`."*

**What the code and schema do** (measured 2026-09-28):

- **`LOCALES = ["en", "ar"]`** in `web-dashboard` and `web-hiring`. The file's own comment: *"TWO IS A
  SECURITY FLOOR, NOT A PRODUCT DECISION"* — a CVE bypasses proxy checks with exactly one locale entry
  (RFC-0028) — and *"`ar` is the right-to-left cut. It is not a market claim."*
- **`job_postings` (`0007`) and the landing sections (`0016`)** require both `en` and `ar` in every
  localized field, by literal `CHECK`.
- **`tenants` carries no `supported_locales` and no locale defaults.**

**Two different sets have been treated as one:** the languages Paridyum's *software* ships in — a
property of a product release — and the languages a *tenant* publishes in — a property of that tenant.
For Paridyum's tenant they coincide, so nothing broke. For anyone else, a Hindi-and-English tenant
cannot store a job posting, and an English-only one must write Arabic.

**The org adds the level ADR-0034 anticipated — "who pays us" is the org — and a constraint it did
not.** An org may be in a different region from its tenants (ADR-0117 D2), and no tenant request may
read another region (RFC-0015 D1).

**The triggers.** REQ-0009 fires on *"The first user-facing interface … a person outside the
organisation will look at"* — careers and the landing are public. REQ-0019 fires on *"The first feature
that … orders [user text] for display, across more than one row"* — the portal lists tenants, members
and orgs.

## Decision

**1. There are two sets, and each has one owner.**

| Set | Owner | Declared where | Today |
| --- | --- | --- | --- |
| **Software locales** — the languages a product's interface ships in | the product | its release: the translation set, plus a proven right-to-left layout if any locale is right-to-left | `en` and `ar` for every web product — a security floor and the right-to-left check, not a market claim |
| **Content locales** — the languages a tenant publishes its own content in | the tenant | the portal: `supported_locales` plus a default `content_locale`, in the tenant's settings | Paridyum's tenant: `en` and `ar` |

**2. A tenant declares five separate things, and none is derived from another.** At creation the
portal asks for each on its own:

- region (jurisdiction, RFC-0051);
- content locales;
- default locale (rendering conventions);
- default timezone;
- default currency.

Nothing is derived from the region, the jurisdiction or the visitor's address. `Accept-Language` and
the visitor's country may *order* suggestions, never select one (ADR-0034 D3; RFC-0015 D5).

**3. The activation gate.** A tenant may activate a content locale only when:

- **every product it runs ships software in that locale**, so no visitor sees half-translated chrome;
- **every localized row it owns carries that locale**, so no page is half-translated;
- **its identity and mail texts exist in that locale** (decision 7).

Deactivating a locale is always allowed; reads then fall back deterministically (ADR-0034 D7: `de-AT`
→ `de` → the tenant default).

**4. Completeness is checked against the tenant, not the estate.** This amends ADR-0096 D1 and D2.

- **A localized field must carry every content locale of the tenant that owns the row.**
- **The tenant's declarations are one tenant-scoped, regional row** — `tenant_settings`, holding
  `supported_locales`, `content_locale`, `locale`, `timezone` and `currency`, each `NOT NULL` with no
  `DEFAULT` (a default here is a declaration nobody made, ADR-0034 D2).
- **One function in each regional database** reads that row, and every localized column is validated
  by it **in a row trigger, never in a `CHECK`.** A `CHECK` that reads another table breaks a restore:
  `pg_dump` creates `CHECK` constraints before it loads rows and triggers after, so a restore would
  refuse a posting whose tenant's settings had not loaded yet. A literal list of locale codes — in a
  constraint or in a function one calls — is refused.
- **`0007` and `0016` take this shape.** The estate is rebuilt from empty (ADR-0118), so both are
  corrected in place before they are applied again.
- Paridyum's tenant declares `en` and `ar`.
- **ADR-0096 D2's estate-wide active-locale set stops being a schema fact.**

**5. There are two chains, and neither reads across a region.** This amends ADR-0034 D1.

```
the portal (an org, in the org's region)       a tenant (in the tenant's region)
  request override                               request override
  member preference                              user preference
  org default                                    account default
  platform default                               tenant default
                                                 platform default
```

- **The org's defaults seed a tenant.** At creation the portal proposes them as the tenant's
  declarations, and the owner confirms or changes each.
- **After creation the tenant owns them.** Changing an org's defaults changes no tenant, and no tenant
  request ever reads its org's settings — they may live in another region.
- **GLO-0006's five levels are unchanged as levels.** This fixes only which chain reads which level at
  runtime.

**6. Readers and records keep ADR-0034 D5 and D6.** A grantee reading a tenant renders in their own
preferences, which the tenant holds on its record of the grantee (ADR-0117 D7). The record's own fields
never change for a reader. Mail about a tenant to a grantee renders in the grantee's preferences; mail
from the portal to a member renders in the member's.

**7. Identity and mail texts exist per content locale.** Each tenant's login and message texts
(Zitadel, per organization and per language; ADR-0084 D4 calls message text branding) and its
system-mail templates exist for every content locale it activates, configured at the portal, and part
of the activation gate.

**8. Software locales grow by release; content locales grow by declaration.** Adding a language to a
product is a release. A tenant adding a content locale is a setting, subject to decision 3, and a
tenant may never declare a content locale none of its products ships.

**9. REQ-0009 and REQ-0019 move to `accepted`, because their triggers have fired.** A sorted list of
user text is ordered by the reader's locale collation (ICU), never by bytes. Searching a tenant's
content uses that tenant's content locales.

**10. Money stays as ADR-0033 (locked) fixes it.** A tenant's default currency is the default for its
own transactions; Paridyum bills an org in that org's billing currency; neither follows the region,
and an amount's currency never follows its reader.

## Consequences

**Accepted costs**

1. **Tenants gain five declared fields, and the portal gains the screen that sets them.** Paridyum's
   tenant is created with `en` and `ar`, `en-IN`, `Asia/Kolkata` and `INR`, stated rather than assumed.
2. **One function per regional database, and a row trigger per table with localized columns.** A
   cross-row check is harder to reason about than a literal, and a trigger is one more thing a
   restore rehearsal must exercise (T-G.7).
3. **The activation gate is deliberate friction.** Translation debt becomes visible per tenant instead
   of silently absent.
4. **Login and message texts per content locale per tenant**, at scale.

**Gained**

- **A Hindi tenant can publish in Hindi**, and an English-only tenant writes no Arabic.
- **Paridyum can add a language to its software without forcing any tenant to translate.**
- **The four fields stay independent** (GLO-0006), and **no locale resolution crosses a region.**
- **REQ-0009 and REQ-0019 stop being deferrals whose triggers fired unnoticed.**

**Rejected alternatives**

- **Keep the estate-wide set.** Every tenant would publish in `en` and `ar`, forever, because
  Paridyum's software needed two entries.
- **Per-tenant locales with no activation gate.** Pages whose chrome and content disagree.
- **Inherit the org's locale at request time.** A cross-region read on every request (RFC-0015 D1), and
  a tenant silently changed when its org changes.
- **Derive locale, timezone or currency from the region.** GLO-0006 is locked against exactly that.

## Related

- **GLO-0006** (`locked`) · **ADR-0034** · **ADR-0096** · **STD-0010** · **ADR-0032** · **ADR-0033**
  (`locked`)
- **REQ-0009** · **REQ-0019** · **ADR-0084** D4 · **RFC-0048** §E · **RFC-0028**
- `platform-tenancy/migrations/0007` and `0016` · `web-dashboard/src/lib/locale.ts`
- **ADR-0114** · **ADR-0117** (D2, D7) · **RFC-0058** (listings per content locale)
- Research record `docs/2026-09-28-org-model-regions-and-locales-research.md`, Part C

---
id: MKT-0005
title: "Media custody for small production businesses: paid for today, lost anyway, and unverified in the cloud"
type: market
status: accepted
created: 2026-09-27
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [market, media, custody, creators, offload, sync, cloud, reel, rfc-0054, c-0006, bet-0006]
breaking: false
enforced_by: []
sources:
  - url: https://www.prnewswire.com/news-releases/creator-economy-market-to-reach-usd-1-345-54-billion-by-2033--driven-by-ai-powered-content-creation-direct-monetization-models-and-expanding-digital-entrepreneurship-302806507.html
    title: "Grand View Research — Creator Economy Market to Reach USD 1,345.54 Billion by 2033"
    retrieved: 2026-09-27
    supports: >
      The population and its direction: the global creator economy at USD 252.33B (2025) growing to
      USD 310.37B (2026), a 23.3% CAGR to USD 1,345.54B by 2033; **"Leading Creative Service:
      Photography & Videography"**; individual content creators as the largest end-use segment
      (57.2%); Asia-Pacific the fastest-growing region. Read via search extract on the release;
      a vendor-commissioned analyst forecast, cited as a direction rather than a measurement.
  - url: https://www.creatoriq.com/press/releases/creatoriq-state-of-creators-report-2026
    title: "CreatorIQ — The State of Creators 2026"
    retrieved: 2026-09-27
    supports: >
      The willingness-to-pay bound, and it is the most load-bearing number here: a survey of
      **5,095 creators across 100 regions** finds **67% earn less than USD 10,000 a year from
      content creation and 62% do not treat it as primary income**, while just under 5% earn more
      than USD 100,000. So the population is large and its paying head is small — the product must
       be priced and sold to the professional minority, not to "creators". Read via search extract.
  - url: https://www.prnewswire.com/news-releases/a-survey-from-handy-recovery-advisor-shows-over-70-of-us-users-have-faced-data-loss-302500629.html
    title: "Handy Recovery Advisor — survey: over 70% of U.S. users have faced data loss"
    retrieved: 2026-09-27
    supports: >
      That loss is the normal outcome rather than the rare one: **71% of 1,000 surveyed U.S. adults
      have lost data at least once**, and the two dominant causes are **accidental deletion (34%)
      and hardware failure (30%)** — together nearly two-thirds of incidents. Also the closest
      thing to a control: **~60% of users who never lost data actively use cloud storage.** A
       consumer survey by a recovery-tool vendor; cited for the pattern, not the decimals. Read via
       search extract.
  - url: https://hedge.co/products/offshoot
    title: "Hedge — OffShoot (the application formerly known as Hedge)"
    retrieved: 2026-09-27
    supports: >
      The spend that already exists in this exact workflow, and the shape of the tools it buys:
      OffShoot is a perpetual per-activation licence — **USD 169 base, USD 249 Pro, USD 49 Solo
      (iPadOS), Pro rentable at USD 49/30 days** — with **verification by design**, media hash
      lists that re-verify backups later, duplicate detection, reports, and Finder/Explorer
      integrations. **S3 destinations, ASC MHL and API/scripting are Pro-only features.** macOS and
      Windows; "trusted by over 50 thousand people". It copies **cards to disks and to S3** — it
      does not keep a cross-device custody ledger, and it does not speak a consumer cloud.
  - url: https://www.g2.com/products/shotput-pro/pricing
    title: "G2 — ShotPut Pro pricing (page updated October 2025)"
    retrieved: 2026-09-27
    supports: >
      The second established vendor at the same price point: **USD 169 perpetual with update plans
      from USD 59/year** for the industry's long-standing offload-and-verify tool. Two independent
      vendors charging near-identical perpetual licences is the clearest evidence that small
      production businesses pay for custody software. Read via search extract.
  - url: https://www.globenewswire.com/news-release/2026/06/02/3304965/28124/en/digital-asset-management-market-by-offering-application-asset-type-organization-size-vertical-and-region-global-forecast-to-2031.html
    title: "Research and Markets — Digital Asset Management market forecast to 2031"
    retrieved: 2026-09-27
    supports: >
      The adjacent, enterprise-heavy category as a bound rather than an invitation: Digital Asset
      Management is projected to grow **from USD 6.23B (2025) to USD 14.51B (2031), a 15.4% CAGR**,
      with media and entertainment the dominant vertical and the cloud segment growing fastest.
      reel is deliberately not this product — the evidence is cited so the boundary is visible.
      Read via search extract.
  - url: https://help.sync.com/hc/en-us/articles/38973855303571-Troubleshooting-slow-upload-and-download-speeds
    title: "Sync.com — Troubleshooting slow upload and download speeds"
    retrieved: 2026-09-27
    supports: >
      The vendor's own account of where its consumer cloud strains under a media workflow: the
      number and size of files dominate performance; every file is **integrity-checked and
      encrypted before upload**, during which it can appear that nothing is transferring; they
      recommend small batches, the desktop app for volumes, and keeping the computer awake
      **24 hours** for large transfers. This is the slowness the product exists to make visible and
      survivable, stated by the vendor rather than by a reviewer. Read via search extract.
      (RFC-0054 carries the hard limits
      — five devices, 500 MB web-panel degradation, silent skips, no Linux client — from the
      general-limits pages.)
  - url: https://apps.apple.com/us/app/sync-secure-cloud-storage/id917410266?platform=ipad&see-all=reviews
    title: "App Store review — Sync for iPad, February 2026"
    retrieved: 2026-09-27
    supports: >
      One customer's account of the failure class the product's reconciliation exists for: an
      upload "shows complete" while only **88 of 100 photos** arrive, and repeated attempts
      converge slowly. A single review is an anecdote, not a survey — cited because it names the
      exact property no consumer cloud offers (**proof of completion**) and because a vendor's own
      documentation already admits silently skipped files (RFC-0054's third source). Read via
      search extract.
---

# MKT-0005 — Media custody for small production businesses: paid for today, lost anyway, and unverified in the cloud

## Status

**ACCEPTED**, 2026-09-27. This is the evidence that selected the market `reel` is registered
against (`product-registry.yaml`, `kind: catalogue`), written because C-0006 requires a vertical's
selecting evidence to be recorded and **not tenant-derived**. Nothing here comes from a tenant —
there are none. What it is, plainly: **a direction, four facts and their bounds**, assembled from
dated external sources.

The rule this artifact exists to satisfy, quoted so it is not paraphrased away:

> *"Every vertical Paridyum enters must be recorded in an artifact naming the evidence that
> selected it, and that evidence may not be tenant-derived."* — C-0006

**The operator's own need is why this was attempted; it is not the evidence.** That distinction is
the whole point of the requirement, and a document that blurred it would be the exact failure the
control exists to prevent.

## The four facts

**1 · The population is large, and its paying head is small.** The creator economy is put at
USD 310.37B in 2026 growing 23.3% a year, with **photography and videography listed as the leading
creative service** — but 67% of surveyed creators earn under USD 10,000 a year from content, and
under 5% earn six figures. The product is therefore never sold "to creators". It is sold to the
**professional minority**: micro-studios, agencies, solo operators whose shoots are paid work and
whose media is their business record.

**2 · People already pay for exactly this job — at a price point that proves the category.** Two
established vendors sell media offload-and-verify software at nearly identical perpetual licences
(USD 169 base; OffShoot's Pro tier for S3 and API is USD 249; ShotPut Pro is USD 169 with updates
from USD 59/year). OffShoot says it is *"trusted by over 50 thousand people"*. The willingness to
pay is not a hypothesis; it is a market with incumbents.

**3 · Loss is normal, and the mitigations are manual.** 71% of surveyed U.S. adults have lost data
at least once; accidental deletion and hardware failure cause nearly two-thirds of incidents. The
industry's answer is a **discipline** — the 3-2-1-1-0 backup rule, quarterly restore tests, an
unplugged drive — which is to say: a checklist a busy operator is trusted to remember. Nothing in
the paid toolset *proves* the copy landed somewhere else; the proofs stop at the local disk or the
bucket.

**4 · The clouds in between do not custody, and the custodians do not cloud.** This is the gap, and
it is visible from both sides:

- The consumer zero-knowledge clouds are cheap, private and end-to-end encrypted — and they are
  genuinely bad at this workflow. Sync's own documentation says performance is dominated by file
  count and size, that every file is encrypted before upload (during which it looks stalled),
  recommends small batches, and asks the customer to keep a computer awake for **24 hours**. The
  hard limits RFC-0054 records — five devices, a 500 MB web-panel ceiling, **silently skipped
  filenames**, no Linux client — are the same story in numbers. A 2026 app-store review names the
  one property that matters and is absent: *"I want to know that if I upload to Sync, and it says
  I did it successfully, that it really did upload everything."*
- The professional offload tools verify beautifully — **on the way to a disk or a bucket they are
  pointed at**, with hash lists and re-verification. They are per-activation desktop licences
  (macOS/Windows; an iPad edition alone, no Linux), they keep no ledger across devices, and none
  of them speaks a consumer cloud as the off-host store.
- The enterprise category between them (DAM/MAM — USD 6.23B in 2025 growing 15.4% a year) is
  priced and shaped for content operations departments, not for a two-person studio's card dump.

**No one joins the parts**: your own machines, your own cloud account, a chain-of-custody ledger,
and a completion check that is not the uploader's own word. That unserved middle — *bring your own
storage, we make it honest* — is the market this artifact selects.

## Bounds, stated rather than discovered later

1. **The evidence is strongest in the U.S. and Europe.** No India-specific market measurement is
   cited here because none credible was found in this pass. Paridyum's own operation will be the
   first in-market observation — and under C-0006 it can **confirm** the selection, never select.
2. **Earnings concentrate at the top.** The long tail of creators will not buy anything; every
   judgement about price, plan and channel must survive that, and BET-0006's segment work is where
   it lands.
3. **The sharpest single datapoint is an anecdote.** The "88 of 100 photos" review is one
   customer. It is included because it names the property, not because it measures a rate; the
   vendor's own documented silent-skip behaviour is the corroborating fact.
4. **Prices and forecasts move.** Every figure here is a dated observation (retrieved
   2026-09-27). A re-read a year from now should expect the numbers to have changed and the shape
   to have held — which is exactly what RFC-0020 says a source block is for.
5. **What this evidence does not establish:** that a product can be built profitably, that
   Paridyum can operate the vertical (BET-0005's test), or that anyone franchises it. Those are
   falsifiable wagers, not findings, and they live in the bets.

## Why this is the selecting artifact and not another

- It is **external**: creator-economy forecasts, a cross-region creator survey, two vendors'
  published prices, a consumer data-loss survey, a vendor's own troubleshooting page, an adjacent
  market report. No tenant data exists; none was needed.
- It is **dated and bounded**: every source carries `retrieved`, and the limits section says what
  the evidence cannot do — the same posture MKT-0004 took for the entitlement estate.
- It selects **one market, narrowly**: paid media custody for small production businesses using
  their own storage. It deliberately does not select "the creator economy", which is too large,
  too poor on average, and already crowded at the layer above (DAM/MAM) and below (cloud drives).

## Related

- **RFC-0054** — the product this market selects, and the vendor limits it is shaped around
- **`product-registry.yaml`** — the registration (`reel`, `catalogue`, `proposed`) whose dossier
  obligation this artifact discharges
- **C-0006** — non-commodity item 4's rule, anti-goal 1's operate-then-franchise sequencing, and
  anti-goal 6's gate on building a second vertical
- **BET-0006** — the segment whose pricing and channel this evidence must survive
- **MKT-0001 – MKT-0004** — the precedent for dated, external, bounded evidence (this is the
  fifth; note it is the first selecting a *catalogue product* since MKT-0001–0003 chose hosting)
- **ADR-0071**, **REQ-0023** — the estate's existing Sync constraints and the custody obligations
  the product must discharge

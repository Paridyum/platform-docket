---
id: RFC-0016
title: The edge is rented, never holds tenant data, and never asserts identity
type: rfc
status: proposed
created: 2026-08-11
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, infra, web]
applies_to: ["*"]
tags: [edge, cdn, dns, latency, caching, residency, vendor, c-0006, adr-0046]
breaking: false
enforced_by: []
---

# RFC-0016 — The edge is rented

## Status

**PROPOSED** as of 2026-08-11. Feedback deadline **2026-09-10**.

**Third of four.** RFC-0014 defines the structure; RFC-0015 decides how the planes behave. This RFC
decides the tier in front of both — what it does, what it may hold, and what the application must do
so that distance is survivable. Depends on RFC-0014.

## The question

RFC-0014 pins a tenant's data to one region and RFC-0015 refuses to copy it anywhere. Both are
right, and together they guarantee that some users are far from their data: a person in the United
States working in an India-resident tenant pays an ocean crossing on every exchange.

The question is what can be recovered without moving the data — because moving it is the one thing
neither RFC will permit.

Most of the answer is not ours to build. Connection setup, static delivery and warm origin
connections are a mature vendor category, and C-0006 names CDN and web hosting as **commodity**:
*"We will not position on these, price on these, or describe them as our value. We depend on them."*
The archive did the opposite, running `cloud_provider: inhouse` across all eight regions, and its
own retrospective judged that a very large operational surface for a platform at this stage.

What *is* ours is narrower and more important: deciding what may be cached without breaking
residency, ensuring the tier never becomes an identity authority, and building applications whose
shape does not multiply the distance that remains.

## Decision 1 — the edge is rented, and it does five things

C-0006 names CDN and web hosting as commodity: *"We will not position on these, price on these, or
describe them as our value. We depend on them."* Building a point-of-presence network is therefore
not ours to build. The archive ran `cloud_provider: inhouse` across all eight regions and its own
retrospective judged that a very large operational surface for a platform at this stage.

The edge does exactly five things:

1. Terminates TLS, including per-hostname certificates for tenant custom domains.
2. Serves cached static responses (decision 2).
3. Resolves `Host` to a region via the generated routing table.
4. Fails closed when the resolved region is absent, disabled or unhealthy.
5. Originates the `traceparent` ADR-0044 requires, and never regenerates one.

**DNS is not the router.** One anycast address set is announced globally; region selection happens
on the `Host` header inside the edge. Adding a region therefore touches no DNS record, which is the
property that makes this shape work and the property the archive's per-region `endpoint` column
lacked.

**The edge never asserts identity.** Its resolution of `Host` to a tenant is a **transport hint**.
It is never forwarded as a trusted value — no `X-Tenant-Id`, no equivalent — because ADR-0038
decision 5 derives both tenancy identifiers from the verified session and *"never a request body,
query string, path segment or client-supplied header."* The moment a tenant id appears on the wire
as a header, something will read it.

The mismatch case is already handled elsewhere and needs no new machinery: with one issuer per
region (ADR-0047 decision 1) and mandatory `iss` validation (decision 5), a token minted in one
region and presented in another fails at the resource server.

**The edge does not infer locale.** GLO-0006 resolves locale through the tenancy chain. A
point-of-presence in Sydney serving a Frankfurt-resident tenant would guess wrong, and GLO-0006
already rejects deriving locale from a region field.

## Decision 2 — the cacheability rule

> **The edge may cache a response only if it is byte-identical for every anonymous visitor of that
> hostname.**

This is the rule that makes "serve static content near the user" decidable rather than a matter of
taste, and it is deliberately keyed on *hostname*, not on *tenant*.

Test it against the hard case. A premium tenant's branded login page at `login.theirbrand.com`
(ADR-0047 decision 3) carries the tenant's logo, name and theme. That is tenant-owned data, cached
at a point of presence outside the tenant's region — and it is **permitted**, because it is
identical for every anonymous visitor and the tenant published it deliberately on a public domain.

**The axis is published-public versus private, not tenant-owned versus not.** Anything varying by
user, session, account or locale falls outside the rule automatically.

**Rendering never happens at the edge.** ADR-0046 decision 4 makes the RSC server/client boundary a
trust boundary, and a React Server Component runs with server credentials. Rendering a tenant's page
at a point of presence puts server credentials and tenant data outside the tenant's region. Static
assets and prerendered public pages are cached; nothing is rendered. ADR-0046's Next.js pinning —
`output: "standalone"`, self-hosted, ADR-0020's charts and probes applying unchanged — is therefore
untouched by this RFC.

**This rule is not machine-checkable and that is stated rather than hidden.** Whether a response is
identical for every anonymous visitor is a property of running software, not of a repository.
ADR-0046 made the same admission about its own trust boundary; the attempt is not made rather than
made badly. `verified_by` at the integration tier is the closing mechanism, not a check.

## Decision 3 — distance-aware defaults live in the template, not in prose

Decision 3's edge removes the cost of *connecting* across a distance. It cannot remove the cost of
*talking* across one. A tenant in one region with users in another pays a full round trip on every
data exchange, permanently, and past first paint the application's own shape decides whether that is
felt as one wait or as five.

**Four patterns, scaffolded into `template-web` rather than written as a rule:**

- **Static shell, dynamic fill.** Layout, navigation and skeleton are cacheable under decision 2 and
  appear immediately; data arrives after.
- **One batched request, not several.** Five sequential calls over a 250ms link is 1.25 seconds of
  waiting that no infrastructure can recover.
- **No chained requests.** A call whose input is the previous call's output pays the distance twice.
- **Optimistic writes**, reconciled on confirmation.

**Why the template and not a standard.** ADR-0046 point 7 already made this argument for
accessibility and internationalisation: putting them in `template-web` *"is the difference between a
rule and a default."* A distance rule in prose is obeyed by the first application and forgotten by
the third, and the failure is invisible locally — every one of these patterns looks fine on a laptop
with a database on the same machine, and only degrades for users far from their region.

**This interacts with REQ-0016.** Performance budgets there are per endpoint or per workflow; this
decision says the budget for a workflow depends on the distance between a user and their tenant's
region, so one number cannot cover both a local tenant and a remote one.

**Like decision 2, this is not machine-checkable in general** and the claim is not made. Whether a
screen chains its requests is a property of running software. `verified_by` at the integration tier
is the closing mechanism; the template is what makes the default right before anyone thinks about it.


## Decision 4 — the vendor is Cloudflare, and the three criteria are met component by component

*Added 2026-08-19, answering open question 1.* This RFC set three narrow criteria and deliberately
declined to weigh features, because decision 1 keeps the tier dumb enough to be portable. Cloudflare
is selected against those criteria and nothing else.

**The footprint question has a named mechanism rather than an assurance.** Cloudflare's Data
Localization Suite is three separate controls, and they map onto this RFC's concerns one to one:

| The worry | The control |
| --- | --- |
| *"may process logs anywhere"* — open question 1's own words | **Customer Metadata Boundary** — restricts where logs and analytics are stored |
| RFC-0015 decision 3, nothing crosses a region boundary | **Regional Services** — restricts which Cloudflare datacentres may decrypt and inspect HTTPS traffic |
| Decision 1 item 1 terminates TLS, so the keys are in scope | **Geo Key Manager** — restricts where TLS private keys are stored |

The suite covers India. With Regional Services applied, Cloudflare's own design guidance states that
*"all processing, including TLS termination, occurs exclusively within the specified geographic
region"* — which is the property RFC-0015 decision 3 needs and the one a global anycast network
would otherwise make impossible to assert.

**The DPA exists, is versioned and is dated.** Customer Data Processing Addendum v6.4, 3 April 2026,
with sub-processor appointment under clauses 4 and 6.3 and a customer right to object under clause
4.4. That satisfies the criterion as this RFC worded it — *will the vendor sign a DPA naming its
sub-processors* — and it does not satisfy REQ-0012, which wants the DPA **signed** rather than
available. See the costs.

**The exit path is the one decision 1 was designed to preserve.** Five dumb functions, no
vendor-specific logic at the edge: exit is repointing DNS and terminating TLS at our own ingress,
losing the point-of-presence network and nothing else. The routing table is already generated output
we own.

**The architecture is documented by the vendor rather than inferred by us**, which matters because
decision 1 and RFC-0023 decision 5 together need an inbound path to a machine with no static
address. Cloudflare's reference architecture puts the fallback origin as a CNAME to a public hostname
exposed by Cloudflare Tunnel, with TLS terminating at the edge; Cloudflare for SaaS provisions and
manages per-hostname certificates for customer-owned domains, which is decision 1 item 1's tenant
custom domain requirement. The origin dials out, and RFC-0023 decision 5's *"no inbound ports, no
dynamic DNS, no static IP"* holds without a workaround.

**Three costs, recorded rather than deferred.**

1. **The Data Localization Suite is a paid add-on and this RFC states no price**, because RFC-0020
   forbids writing an external figure without a dated citation and none was obtained. The figure is
   owed, and it lands against Track B's stage B1 budget of roughly Rs 3,000-4,000 per month - so it is
   plausibly the largest single line in the early hosting cost model. **If the suite proves
   unaffordable, the three controls above go with it**, and the footprint criterion becomes unmet
   rather than met - which would reopen this decision rather than degrade it quietly.
2. **The tunnel-as-fallback-origin pattern is scoped by the vendor's own guidance** to providers
   *"that do not need granular load balancing, such as geo-based traffic steering, across multiple
   origin servers,"* with Cloudflare Load Balancer named as the answer when they do. That is
   correct at one cluster and becomes wrong on RFC-0014's multi-region axis, which is exactly where
   this platform is heading. **Not a blocker now; a known second decision at the second region.**
3. **This concentrates the entire inbound path in one vendor.** REQ-0012 item 6 already names the
   GitHub case - *"source control, CI, artifact storage, secret storage and identity... one failure
   domain"* - and Cloudflare would hold DNS, the edge, the tunnel and every certificate. A Cloudflare
   incident is then a total outage with no degraded mode, because the tunnel is also how the origin is
   reachable at all. The mitigation is not architectural at this scale; the honest response is to
   record it as the second such domain rather than to discover it during the first incident.

**What remains owed under REQ-0012**, which is `deferred` and therefore enforced by compliance rather
than machinery, as accepted cost 3 below already states: the DPA signed rather than merely published,
the sub-processor list read against RFC-0015 decision 3, the egress allowlist for `cloudflared`, and
the criticality tier - which on the strength of cost 3 is the highest in the system.

## Consequences

**Accepted costs**

1. **Global reach does not mean uniform speed.** A user in Sydney working in a Frankfurt-resident
   tenant gets a fast first paint from a nearby point of presence and a full round trip on every
   data exchange. Residency forbids fixing this. It must never be described, internally or
   externally, as "fast globally" — that is a claim the architecture cannot honour, and C-0006
   already warns about conflating two different quantities in a promise.

2. **The edge configuration lands in the one repository class with no release governance.** The
   routing table is generated output belonging to `infra-*`, and ADR-0037 decision 4 states that
   *"no artifact governs it… REQ-0004 owns it; today it is genuinely ungoverned."* This is the
   highest-blast-radius configuration in the system — one wrong line routes a region's traffic to
   another region — sitting in the least governed class. Naming it is not fixing it.

3. **The edge vendor becomes a GDPR sub-processor.** Edge access logs carry client IP addresses,
   which are personal data. REQ-0012 requires every external dependency be *"approved, scoped, and
   exitable"*, and REQ-0012 is `deferred`. This is a contractual obligation arriving before the
   requirement that governs it.

4. **Two of this RFC's three decisions are unenforceable by machine**, and the claim is not made for
   either. Decision 2's cacheability rule and decision 3's request shape are both properties of
   running software. Decision 2's failure mode is a residency breach, which is the most expensive
   one in the estate.

5. **A second deployment target for the web tier, in effect.** ADR-0046 pins Next.js to
   `output: "standalone"` under ADR-0020's charts and probes, and this RFC keeps that intact by
   caching assets rather than rendering — but `assetPrefix` and cache rules are still a second place
   the web application is configured.

**Gained**

- The largest latency cost — connection setup across an ocean — disappears for the price of a DNS
  change, without any data moving.
- Adding a region touches no DNS record, because the `Host` header rather than DNS selects the
  region. This is the property the archive's per-region `endpoint` column lacked.
- The cacheability rule gives a decidable answer to a question that would otherwise be settled case
  by case, and it settles the awkward case — a tenant's branded login page — correctly.
- The distance-aware patterns land as template defaults rather than prose, which is the difference
  between a rule and a default.

## Alternatives considered

- **Build the point-of-presence network.** What the archive did, with `cloud_provider: inhouse`
  across eight regions. Rejected by C-0006's commodity distinction, and by the archive's own
  outcome: all eight seeded regions pointed at one cluster-local endpoint, so the topology was
  modelled and never deployed.

- **Validate tokens at the edge**, rejecting unauthenticated traffic before it crosses a region.
  Rejected by ADR-0047 decision 4 — a proxy on the validation path is a hop on every authenticated
  request and a platform-wide single point of failure. It would also put every region's trust
  material at the outermost tier and create a second place the tenant question is answered, which is
  precisely what ADR-0038's design exists to prevent.

- **Render at the edge**, so pages paint before data arrives. Rejected because a React Server
  Component runs with server credentials (ADR-0046 decision 4), so rendering a tenant's page at a
  point of presence puts credentials and tenant data outside the tenant's region. The cacheable half
  of the benefit is available without it.

- **Cache tenant reads at the edge with a short TTL.** Rejected as a residency breach regardless of
  TTL. Recorded so it is refused on the record rather than revisited each time latency is discussed.

- **Let the edge call back to the control plane for an unknown hostname**, rather than shipping a
  generated routing table. Rejected because it turns unknown-hostname traffic into control-plane
  load, which is a free amplification path for anyone hitting random names. ADR-0039 step 5 already
  tolerates minutes for custom-domain activation, so generated configuration costs nothing a
  customer notices.

## Open questions

1. ~~**Which vendor, and does its data-processing footprint satisfy RFC-0015 decision 3?**~~
   **Answered 2026-08-19 by decision 4 — Cloudflare**, with the Data Localization Suite as the named
   mechanism for the footprint half rather than an assurance. **Selection is not approval:** REQ-0012
   still owns the approval step, it is `deferred`, and decision 4 names the four things still owed
   under it. The unpriced add-on in cost 1 is the one input that could reverse this.

2. **Does `template-web` exist to receive decision 3?** ADR-0046 point 7 puts accessibility and
   internationalisation scaffolding in the template; this RFC adds distance-aware defaults to the
   same place. If the template does not exist yet, this decision has no home and becomes the prose
   it argues against.

3. **What is the performance budget for a workflow whose user is far from their region?** REQ-0016
   makes budgets per endpoint or per workflow. Decision 3 says the number depends on the gap between
   user and region, which means one budget cannot cover both a local tenant and a remote one — and
   REQ-0016 does not currently have a way to express two.

## Decision 1 is partially reversed, 2026-09-04, by its own condition

*The original decision above is left intact. This section records what changed and why, rather
than editing the text that argued the other way — the reasoning is still the reasoning, and it
was right about everything except one fact nobody had checked.*

**The reversal condition below fired.** It reads:

> Decision 1 reverses if renting the edge is found to make a residency guarantee undeliverable —
> the only condition under which C-0006's commodity classification would be worth contesting.

**Regional Services is an Enterprise-only add-on.** Decision 4 names it as the control for
*"which datacentres may decrypt and inspect HTTPS traffic"*, which is the mechanism RFC-0015
decision 3 depends on. It cannot be purchased on any plan this estate can afford. So a rented
edge cannot deliver the residency guarantee for traffic where residency binds, and the condition
is met on its own terms.

**The reversal is PARTIAL, and the line is what the traffic carries.**

| Traffic | Edge | Why |
| --- | --- | --- |
| Public, no tenant data — the marketing surface | **rented, unchanged** | residency does not bind; C-0006's commodity classification stands and decision 1 holds |
| Anything where RFC-0015 D3 binds | **ours, in-region** | the rented tier cannot deliver it at a price we can pay |

**AN EDGE NODE OF OURS EXISTS FOR RESIDENCY AND CONTROL. NEVER FOR LATENCY.** One node per
residency region, never one per population centre. Latency is the rented tier's job and it is
better at it than this estate will ever be. The moment a node is added *because users are far
away*, we have begun building the point-of-presence network decision 1 correctly refuses — which
is what the archive did across eight regions, and what its own retrospective judged too large an
operational surface.

**What the free tier cannot do, listed against decision 1's five jobs.** Selection of Cloudflare
under decision 4 stands; what changed is knowing which of the five it performs on a plan we can
afford:

| | Job | Rented, free tier | Ours |
| --- | --- | --- | --- |
| 1 | Terminate TLS | yes | yes |
| 2 | Serve cached static responses | yes, subject to the vendor's own content-type terms | yes |
| 3 | Resolve `Host` to a region via the generated routing table | **yes** — see below | yes |
| 4 | Fail closed when the region is absent or unhealthy | **no** — health checks are Load Balancing, paid | yes |
| 5 | Originate the `traceparent` ADR-0044 requires | **no** — needs Workers or Enterprise | yes |

**Item 3 needs no paid product, and an earlier reading of this that said otherwise was wrong.**
One DNS record per hostname, pointing at the edge node of the region that hostname belongs to,
IS the routing table — which is why decision 3 calls it *generated*. Resolution happens in DNS
rather than inside the proxy, and the property decision 1 actually wants is preserved: adding a
region touches no DNS record a human writes, because the records are produced from the table.

What the paid product buys is item 4 — health checks, and failing over when a region stops
answering. Without it a region that is down is a region visitors still resolve to, and the
failure is a timeout rather than a redirect.

Item 4 is harmless while there is one region and becomes binding at the second. **Item 5
remains partly unmet even with our own node**, because the rented tier sits in front of it: the
trace begins at our edge and the leg before it is invisible. That is an improvement on nothing
and it is not compliance, and it is recorded here rather than discovered in an incident.

**THE TIER STAYS DUMB ENOUGH TO BE PORTABLE, which is decision 1's own phrase and is now a
rule.** The rented edge is used for proxying, DDoS absorption and DNS hosting, and for nothing
else — no Workers, no Page Rules, no Transform Rules, no vendor-issued origin certificates, no
bot management. Every feature used is a feature that must be rebuilt to leave, and the value of
this tier being commodity is entirely in being able to change who provides it.

**Our node holds a publicly-trusted certificate, never a vendor origin certificate.** A
Cloudflare Origin CA certificate is trusted only by Cloudflare, so an origin holding one cannot
serve traffic the moment the proxy is bypassed — which destroys the fallback that is the
strongest reason for the node to exist. ACME with the DNS-01 challenge, because DNS-01 issues a
certificate for a hostname the node is not yet receiving traffic for, and that is the property
that makes adding or replacing a node cheap.

## Reversal conditions

- **Decision 1 reverses** if renting the edge is found to make a residency guarantee undeliverable —
  the only condition under which C-0006's commodity classification would be worth contesting.
- **Decision 2 does not reverse toward more caching.** Every relaxation of the rule is a residency
  question, not a performance one, and belongs to REQ-0003 item 8 rather than to this RFC.
- **Decision 3 reverses** if the estate stops having users far from their region — which, under
  RFC-0015 decision 5, is the operator's choice rather than ours to predict.

## Sources

1. Cloudflare Data Localization Suite — the three controls and what each restricts: Geo Key Manager
   (TLS private key storage), Regional Services (which datacentres may decrypt and inspect HTTPS
   traffic), Customer Metadata Boundary (where logs and analytics are stored). Retrieved 2026-08-19.
2. Cloudflare press release, 21 September 2022 — Data Localization Suite extended to Australia, India
   and Japan. Retrieved 2026-08-19.
3. Cloudflare reference architecture, *Extending Cloudflare's benefits to SaaS providers' end
   customers* — fallback origin as a CNAME to a Cloudflare Tunnel public hostname, TLS terminating at
   the edge, the Regional Services processing statement, and the stated load-balancing limitation.
   Retrieved 2026-08-19.
4. Cloudflare Customer Data Processing Addendum v6.4, 3 April 2026 — sub-processor appointment under
   clauses 4 and 6.3, objection rights under clause 4.4. Retrieved 2026-08-19.

## Related

- RFC-0023 — decision 5's dial-out origin, which decision 4's tunnel is the inbound half of
- REQ-0012 — the approval decision 4 does not substitute for, and item 6's concentration risk
- RFC-0014 — the structure this RFC sits in front of
- RFC-0015 — the no-replication rule that makes distance unavoidable, and the routing table exception
- C-0006 — CDN and web hosting as commodity, and the claim about speed decision 1 must not make
- ADR-0047 — why the edge never validates a token, and the custom login domain decision 2 must cache
- ADR-0038 — why the edge's tenant resolution is a transport hint and never an identity claim
- ADR-0044 — the correlation id originating at the edge and never regenerated
- ADR-0046 — the RSC trust boundary, the Next.js pinning, and point 7's template-not-prose argument
- ADR-0039 — step 5, DNS and ACME as a saga step that fails in ways retry does not fix
- ADR-0037 — why the edge configuration lands in an ungoverned repository class
- ADR-0020 — the Kubernetes obligations this RFC deliberately leaves untouched
- REQ-0012 — the sub-processor approval the edge vendor needs
- REQ-0016 — performance budgets, which decision 3 makes distance-dependent
- REQ-0003 — item 8 owns residency, which is the axis decision 2's rule defends
- GLO-0006 — why the edge never infers locale from where a request arrives
- `archived/paridyum-docs/REGION-MODEL.md` — `apisix_upstream`, the unused column this RFC's routing table fills

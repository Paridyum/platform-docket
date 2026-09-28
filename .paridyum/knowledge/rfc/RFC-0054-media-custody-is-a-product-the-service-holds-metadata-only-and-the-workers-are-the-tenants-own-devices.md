---
id: RFC-0054
title: "Media custody is a product: the service holds metadata only, and the workers are the tenants' own devices"
type: rfc
status: proposed
created: 2026-09-27
locked_at: null
supersedes: null
superseded_by: null
scope: [org, service, web, tool, infra]
applies_to: ["*"]
tags: [media, shoot, ingest, archive, custody, checksum, mhl, equipment, sync, vault, webdav, playwright, product, capability, tenancy, workers, tenant-zero, c-0008, adr-0049, adr-0104, rfc-0049, bet-0005, rfc-0052, adr-0071, req-0021, req-0023]
breaking: false
resolved_by: null
verified_by: []
enforced_by: []
sources:
  - url: https://help.sync.com/hc/en-us/articles/38275627279379-General-limits-when-using-Sync
    title: "Sync.com — General limits when using Sync"
    retrieved: 2026-09-27
    supports: >
      The measured platform, the constraints the design is shaped around, and the numbers the
      pre-flight checks assert: desktop client Windows 8+/macOS 10.11+ (10+/13+ for CloudFiles),
      no Windows Server and no service mode, **five active devices per account**, the Sync folder
      may not be an SD card, network drive or NAS (and on macOS not an external drive at all),
      unlimited single-file size on the desktop app (slower above 40 GB), the web panel degrades
      above 500 MB per file and the vendor recommends the desktop app when uploading more than
      100 files or 500 MB, a hard limit of 50,000 files per folder, file-level de-duplication,
      and no Linux client — a WebDAV client is stated as in progress.
  - url: https://help.sync.com/hc/en-us/articles/38275630195347-What-is-the-Sync-Vault
    title: "Sync.com — What is the Sync Vault?"
    retrieved: 2026-09-27
    supports: >
      Vault is cloud-only storage: files placed there do not synchronise to any device, can be
      uploaded from anywhere on the computer **including external drives**, remain viewable from
      the web panel and mobile apps, and stay in the cloud until downloaded back. Also the
      constraint that pins the macOS build: **Copy to Sync Vault is unavailable in the macOS
      CloudFiles app (5.0.x)** for a file-system-level reason, while the web panel can still
      upload to Vault.
  - url: https://help.sync.com/hc/en-us/articles/45690132994835-How-do-I-confirm-that-the-files-in-my-Sync-folder-are-uploaded
    title: "Sync.com — How do I confirm that the files in my Sync folder are uploaded?"
    retrieved: 2026-09-27
    supports: >
      The confirmation surface this RFC refuses to trust as truth: the desktop app's own
      "In Sync" status is the uploading client's claim, while an independent read-back exists as
      (a) a **full listing export of Sync folder and Vault as CSV**, (b) the web panel events log,
      and (c) the progress tab, which is where files **silently skipped** for an incompatible
      name are visible. Reconciliation is built on (a) or a channel other than the uploader.
  - url: https://help.sync.com/hc/en-us/articles/51295382529043-WebDAV-Early-Access-Guide-for-Linux
    title: "Sync.com — WebDAV Early Access Guide for Linux"
    retrieved: 2026-09-27
    supports: >
      The Linux path exists but is gated: WebDAV for Linux runs a **local** server
      (`sync-webdav`) that preserves end-to-end encryption, exposes `http://127.0.0.1:4918/`,
      can be scoped to Vault (`--mount vault/`), works headless, and is the vendor's own
      documented way to use **rclone copy/sync** for large uploads and many files. As of
      2026-09-02 it is **invited-users-only early access** and the article says it will not
      work for accounts that have not been enrolled.
  - url: https://help.sync.com/hc/en-us/articles/45711521662227-How-does-Sync-work
    title: "Sync.com — How does Sync work?"
    retrieved: 2026-09-27
    supports: >
      The mobile app's Camera Uploads is **one-directional** — files go from the phone to the
      cloud and files added elsewhere do not sync back down to the phone — which is what makes a
      phone a governed *inbox* the pipeline drains rather than a peer of the archive. Read via
      search extract rather than a direct fetch; recorded rather than hidden.
  - url: https://playwright.dev/docs/input
    title: "Playwright — Actions: upload files"
    retrieved: 2026-09-27
    supports: >
      That browser automation can select local files for upload deterministically
      (`setInputFiles`, including a directory, and the `filechooser` event for dynamic inputs) —
      which is the capability D6 bounds rather than refuses.
  - url: https://github.com/microsoft/playwright/issues/34149
    title: "Playwright issue #34149 — Cannot transfer files larger than 50Mb to a browser not co-located with the server"
    retrieved: 2026-09-27
    supports: >
      The sharp edge that decides how the browser channel may be wired: a browser **not
      co-located** with the automating driver refuses uploads above 50 MB, while a browser the
      harness launches locally has no such limit. The rule in D6 — launch the browser, never
      attach to one over CDP — exists because of this failure mode.
  - url: https://github.com/ascmitc/mhl
    title: "ASC Media Hash List (ASC MHL) — reference implementation"
    retrieved: 2026-09-27
    supports: >
      The custody format this RFC adopts rather than invents: an XML manifest per copy, hashes
      per file and folder, a chain of custody across generations, a standard `ascmhl/` folder
      location, and hash support including **xxHash64 and XXH3**. The reference implementation is
      MIT-licensed, so it is adoptable and exitable (REQ-0012). Read via search extract.
  - url: https://theasc.com/mitc/asc-media-hash-list
    title: "The ASC — Media Hash List"
    retrieved: 2026-09-27
    supports: >
      That the format is a maintained industry standard rather than a personal convention: the
      ASC MHL specification was developed by the Advanced Data Management Subcommittee of the
      ASC Motion Imaging Technology Council, v1.0 dated 2022-03-15. Read via search extract.
  - url: https://zitadel.com/docs/guides/integrate/login/oidc/device-authorization
    title: "ZITADEL — Device Authorization Flow (RFC 8628)"
    retrieved: 2026-09-27
    supports: >
      That the identity provider the estate already runs supports the **OAuth 2.0 Device
      Authorization Grant (RFC 8628)** as a Native application with the Device Code auth
      method — which is the mechanism D3's station pairing uses: a code the user approves in a
      browser, then tokens on the device, so no secret is distributed to machines. Read via
      search extract.
---

# RFC-0054 — Media custody is a product: the service holds metadata only, and the workers are the tenants' own devices

> **Superseded in part, 2026-09-28 — ADR-0113, ADR-0118.** Where this RFC's worked example has tenant zero
> provision a tenant, read: the saga provisions it **for the org that will own it** (ADR-0118 D5),
> and that org's member reaches it by a per-tenant grant (ADR-0120).

## Status

**ACCEPTED, 2026-09-27** — proposed and revised the same day, then accepted by the operator on the
direction each revision recorded. **Registered as a `catalogue` product at `proposed`**: the
operator chose that route over the capability route explicitly (open question 1, resolved below),
accepting both consequences — MKT-0005 is the external evidence that selected the market, and
**C-0006 anti-goal 6 gates entry to `building`** until hosting (vertical one) has a paying customer
who renewed. Acceptance binds the design; it authorises no build, no purchase and no schedule.

The first draft refused the product framing — it proposed an operator's tool and said so in its
decision 1. **The operator's requirement is the opposite:** this must work for independent users,
with Paridyum as the first tenant. That requirement does not sit beside the draft's premise; it
corrects it, and the correction is one the estate's own rules had already made:

- **A service-shaped thing cannot be shipped as a tool.** ADR-0049 (locked) decision 5 — *"a new
  runtime service names its buyer before it is created… an answer of 'only us, for now' is a
  decision to build a `tool-` instead"* — cuts the other way here: the buyer is **not** only us. It
  is any media-producing business that wants its shoot media governed. C-0008's `tool-`/`infra-`
  exemption is for build-time things no tenant traffic reaches (ADR-0049 D4), and an agent whose
  ledger a tenant reads is not that.
- **C-0008 is the operator's sentence** — *every runtime capability is a product, and Paridyum is
  its first customer and never a privileged one* — which is exactly "I would be the first tenant".

**A second operator specification arrived before acceptance, and it is incorporated too:** the
destination is not hard-wired to Sync. A user connects an ingestion station to the portal, the
configuration lives in the portal, and the off-host store is either an **S3-compatible target** or
the account's own **Sync.com** — *"why not"* was the argument, and it is the right one: custody
should not care which store the bytes land in. D5 and D6 carry the destination model; D2 carries
the split it forces between configuration and secrets.

Acceptance binds the design; it authorises no building and no purchase (ADR-0087 for governance,
ADR-0013 for implementation). **REQ-0023 is unchanged by the revisions**: the custody obligations
are about media, not about the product that moves it, and they bind this design exactly as they
would any other. What was *decided in the first draft* — the archive grammar, the custody rule,
the vendor constraints, the transports — survives below, re-shaped by the product frame.

**The registration question is resolved: `catalogue`.** On 2026-09-27 the operator chose the
creator-market route over the capability route, with both consequences accepted rather than argued
away: reel owes an **MKT-** — MKT-0005, external and not tenant-derived — and **entry to
`building` is gated by C-0006 anti-goal 6** until hosting has a paying customer who renewed. What
the gate does not stop: the runbook, the evidence, and this design. The product build waits; the
operating discipline does not.

`enforced_by` is empty and honest: nothing CI runs can see a camera card or a consumer drive. What
can be mechanical lives in the agent's refusals and the service's constraints; REQ-0023 names the
one item to build first if any ever is.

## The question

The operator shoots across a DSLR, a mirrorless body, action cameras and phones, on cards that are
reformatted and reused. The media lands wherever it lands, and the destination it has today is
Sync.com — no API, no Linux client (WebDAV is early access), a five-device account cap, and a web
panel the vendor itself says degrades exactly where media is heaviest. The requirement is that this
becomes **a product independent users can run** — each with their own machines, their own
destination, their own media — with Paridyum's own media estate using it first.

How is that built so the thing the operator needs now is the same thing a stranger can use later —
without becoming a single-user script, a service only its authors can reach, or a cloud that holds
other people's footage?

## What is already decided, retrieved rather than invented

| Already decided | Where |
| --- | --- |
| **Every runtime capability is a product; Paridyum is tenant zero, never privileged** | **C-0008** |
| **No runtime operation resolves without a tenant; Paridyum's use is a tenant provisioned by the saga; no deployable capability ships without a declared tenant-facing consumer; the boundary is the request path; a new service names its buyer before it is created** | **ADR-0049 (locked)** |
| A product is a fourth governance axis; `capability` and `catalogue` owe different dossiers; **entry to `building` for a catalogue product is the C-0006 anti-goal 6 gate** | **ADR-0050**, `product-registry.yaml` |
| Three levels — platform, tenant, account — and **serving both a founder and a small business is one market at two depths, not a second market**; operate a vertical before franchising it | **BET-0005**, C-0006 anti-goals 1 and 6 |
| **A product instance is an account**, addressed `/app/<product>/<account>/…`, releases keyed on the account | **ADR-0104** |
| A tenant is a set of releases, and `/dashboard` is where they are deployed | **RFC-0049** (proposed) |
| Tenants are provisioned by one compensating saga; one Zitadel organization per tenant | **ADR-0039**, **ADR-0047** |
| Tenancy is **nested, two columns in one database**; operator scope is a **grant, never an absence** | **ADR-0024 (locked)**, GLO-0004, GLO-0005 |
| Session context is set per transaction and **fails closed**; every request carries a correlation id and audit is append-only | **ADR-0038**, **ADR-0044** |
| **The denial is the assertion** — tenant isolation tests assert what must NOT work | **STD-0009** |
| **"Control plane" is the platform's global-facts tier and "fleet" is a Proxmox cluster** — neither word may be reused for this product | **GLO-0007**, **GLO-0009** |
| A tool the estate adopts is registry data; a pin is exact and current; supply chain is governed | **ADR-0066**, **ADR-0053**, **REQ-0014** |
| Classified data has **no exit into a log**; leaving must be walkable — a credible export is a sales asset | **ADR-0080**, **REQ-0013** |
| Repositories are `<class-prefix><name>`; `svc-` is one bounded context per repo; `web-` is a user-facing app; `tool-` is a local executable | **C-0007**, `class-registry.yaml` |
| Media is PMD's (RFC-0052); the off-host store is Sync with `.age` rules for the classes ADR-0071 enumerates; a backup that has never been restored is a belief | **ADR-0071** |
| Bulk operator data leaves a governed host **read-only, enumerated, one uploader unless measured, reconciled — never against the uploader's own report** | **REQ-0021** |
| Decomposition precedes implementation; there is no prototype tier | **ADR-0019**, **C-0010** |
| A service that needs an exception signals a missing capability; one service owns its tables | **C-0001 principle 6**, **ANTI-0006**, **ANTI-0001** |

## What the vendor actually provides, dated 2026-09-27

Measured from the vendor's own documentation, because every Sync decision in this estate has turned
on exactly these facts. Full `sources` above; the sharp ones inline.

| Fact | Consequence here |
| --- | --- |
| Desktop app: **Windows and macOS only**. No Linux client; **WebDAV for Linux** is real but early access, invited users only | The Linux story is ranked, not guaranteed (D6) |
| **Five active devices per account**, phones included | A vendor ceiling on any "many devices upload in parallel" story, alongside the uplink |
| The **Sync folder must be local disk**: no SD cards, network drives, mapped drives, NAS — and on macOS no external drives at all | The archive cannot *be* the synced folder; the product must not depend on a mirror |
| **Vault** is cloud-only: no sync-down, uploadable from external drives, browsable from web and mobile | The destination shape for an archive that stays put |
| Files above 500 MB degrade in the web panel; the vendor recommends the desktop app above 100 files or 500 MB per transfer | Browser automation is bounded, never the bulk path (D6) |
| Any single file size on the desktop app, resume on partial transfers, **file-level de-duplication** across the account | Retries and re-uploads are cheap; batching is scheduling, not correctness |
| **50,000 files per hard folder limit**, slower above ~300,000 files; incompatible filename characters cause files to be **skipped, silently**, visible only under a progress tab | Pre-flight name validation is a correctness requirement (D9) |
| A full **CSV listing export of Sync folder and Vault** exists in the desktop app | The reconciliation primitive (D7) |
| macOS **CloudFiles 5.0.x cannot Copy to Sync Vault** | The macOS worker pins the build, or uses the web panel for small deltas |

## The proposal — decisions

| | Decision | Costs |
| --- | --- | --- |
| D1 | A product in three parts — `svc-reel`, `web-reel`, `agent-reel` — tenant zero first, buyer named | The tenancy-first cost ADR-0049 already imposes |
| D2 | **The service holds metadata, never media** | The platform cannot help a user whose devices are all offline |
| D3 | **Workers are the tenants' own devices**; the service schedules with jobs and leases | Distribution and update become product problems |
| D4 | The archive grammar and the custody rule (unchanged from the first draft) | Cards held in escrow; slugs are forever |
| D5 | The off-host store is the account's own — **Sync or S3-compatible, configured in the portal**; the platform never holds the secret | BYO-destination support matrix |
| D6 | Destination adapters ranked per worker capability; browser automation bounded | Linux bulk waits on early access |
| D7 | Completion is reconciliation, recorded as account evidence | One read-back rehearsal per adapter |
| D8 | Gear is account data with permanent, printed ids | Label discipline |
| D9 | Destination limits are pre-flight checks | Slightly slower ingest on odd filenames |
| D10 | What the product never does | Rules someone will occasionally want to break |
| D11 | Sequencing is forced: runbook, then agent, then service, then console | The product is slower to appear than a script |

### D1 — A product in three parts, with tenant zero first

**`svc-reel`** — a `svc-` repository (one bounded context: media custody within one account). It
owns the account-scoped records: devices, workers, gear, shoots, batches, manifests' metadata,
jobs, leases, custody events, reconciliation results. **`web-reel`** — the tenant-facing console: the
ledger, the queue, custody state, reconciliations, **and the account's configuration** —
destinations, workers, windows, rules; reached at `/app/reel/<account>/…` per ADR-0104, deployed
as a release per RFC-0049's direction. An ingestion station pairs to an account here by device
authorization (RFC 8628 — a code the user approves once in a browser, after which the station
holds its own revocable session), which is the whole onboarding story: connect the machine,
configure it in the portal, done. **`agent-reel`** — the agent and CLI
(binary `reel`): everything that touches media, and the worker mode the service schedules. The
class is new and deliberate — a distributed application on the tenant's own machines — because
`tool-` could not carry it (ADR-0111; open question 3, resolved below).

**The product boundary is custody and transport — never creative work.** It ingests, verifies,
archives, transports, reconciles and reports. It does not select, grade, edit, publish or deliver
design. The boundary matters twice: it keeps the product's promise specific — *your media,
governed* — rather than drifting into a creative application, and it keeps every downstream
creative decision where it belongs — the tenant's.

**The buyer is named** (ADR-0049 D5): any media-producing business that shoots on cards and needs
its media governed — an agency, a studio, a solo operator. As a `catalogue` product these are
Paridyum's own end customers first (the operated phase) and, if it ever comes, founder-tenants
after — C-0006 anti-goal 1's sequencing, not an accident. **Tenant zero is Paridyum itself** — the
media estate is the vertical's first operation — provisioned through ADR-0039's ordinary saga,
holding no scope a customer could not also be granted (ADR-0049 D2). "I would be the first
tenant" is C-0008 working as designed.

**What tenant zero actually needs**, in the order D11 fixes: the runbook (D11.0), the agent, the
service, the console. The first governed shoot runs by hand; the agent automates the sequence that
runbook proves; the service records custody; the console makes it visible.

### D2 — The service holds metadata, never media

**What the service stores:** account and instance records; users and roles through
`platform-identity`; devices and workers; gear; shoots and batches; manifest *metadata* (paths
relative to the shoot, file counts, byte totals, hashes, card and device ids); destination
configuration (class, endpoint, bucket or Vault, prefixes, schedules, windows); jobs and leases;
custody events; reconciliation results; audit. **What it never stores:** media bytes, previews,
EXIF, GPS, thumbnails — and **never a destination secret or session state**, because the
destination account is the user's own (D5).

**Why this is the decision the whole product hangs from, four times over:**

1. **The platform cannot read the media, ever** — and for a Sync destination the vendor cannot
   either (its zero-knowledge design); for an S3 destination the bucket is the tenant's and
   Paridyum holds no key to it. That is the strongest form of the promise RFC-0022 exists to make
   sense of: *we isolate tenants from us* — here the platform operator is **structurally
   incapable** of opening a tenant's footage, not restrained by policy.
2. **Residency shrinks to metadata.** Readable personal data on the platform is names, emails and
   opaque camera filenames. That is a classification surface a small team can actually govern.
3. **ANTI-0007 is not approached.** No regulated data, health data or payment data ever flows
   through tape — there is no tape.
4. **Storage economics stay the user's own.** The product never buys disk for other people's
   media; the user's destination account is the off-host store, and the user's own drives are the
   working store. This is also the honest answer to "where does the data live": with its owner.

**Configuration lives in the portal; the secret does not.** The console is where an account
configures its destinations — class, endpoint, bucket or Vault, prefixes, schedules, per-worker
windows — and an agent fetches that configuration when online and caches it. The *secret* that
opens a destination is entered on the worker and never leaves it: the portal holds no credential
it could leak, and a replacement worker re-enters it once. Delivering secrets **to** workers from
the platform is a real later capability and deliberately not this one — it needs REQ-0001
discharged (runtime secret management) and a tenant-key scheme, and it is deferred rather than
implied.

**The metadata is classified too.** Shoot slugs carry no person names (D4); camera filenames are
opaque device-generated strings; the service's classification is `internal`; ADR-0080 applies to
the service's logs *and* the agent's — counts, ids and outcomes, never filenames of classified
content and never EXIF. **Export is the ledger** (REQ-0013, C-0006 anti-goal 4): an account can
walk away with its manifests, and it never needed us for the media at all.

### D3 — Workers are the tenants' own devices, and the service schedules them

**A worker is one agent installation bound to one device and one account.** It declares its
capabilities at registration, and the scheduler matches jobs to them:

| Capability | What it means |
| --- | --- |
| `ingest` | A card reader is attached; this device can receive media |
| `adapter:s3` | An S3-compatible destination can be written from here (rclone/SDK) |
| `adapter:sync-client` | The vendor's desktop app is present to copy into Vault (with its build constraints) |
| `adapter:sync-webdav` | A local WebDAV server / rclone is available (early access) |
| `adapter:sync-browser` | The bounded browser channel may run here (D6) |
| `window` | When this worker may use the uplink (overnight, unmetered, etc.) |
| `store` | Where the working archive lives, and its free space |

**Jobs are account-scoped records with one state machine each**: `ingest`, `upload`, `reconcile`,
`seal`, `gc`. A worker claims a job by taking a **lease**; it heartbeats while working; it completes
with **evidence** (manifest ids, counts, byte totals, a digest of the listing it read back) or
fails with a reason. A lease that dies returns the job to the queue; **retry is safe by
construction** — a Sync destination deduplicates identical bytes, an S3 write is idempotent for
identical content, and reconciliation compares against the manifest, not against optimism.

**The scheduling rules, stated once:**

- **Ingest parallelizes across workers freely.** Hashing and copying are local I/O; this is the
  "multiple devices" the operator asked for — dump cards on two machines at once and both feed one
  account ledger.
- **Uploads default to one worker per destination account,** per REQ-0021 item 7. The bottleneck
  is the uplink (RFC-0023's residential reality) — and, where the destination is Sync, the
  vendor's five-device cap beside it; two ceilings, not preferences. Fan-out happens only after a
  measurement says the bottleneck is somewhere else, and the RFC re-asserts that rule rather than
  relaxing it because the product is multi-tenant.
- **Maintenance jobs are the service's own** — lease expiry, metering, retention — light,
  metadata-only, no media anywhere near them.
- **Degradation is designed, not discovered:** agents keep the full local pipeline — ingest,
  verify, manifest, archive, transport — when the service is unreachable, syncing job state when
  it returns. A service outage blocks scheduling and visibility; it never blocks custody.

**Security and isolation, at RFC depth:** a station authenticates through Zitadel's **device
authorization grant (RFC 8628)** as a Native-type application of the reel project (ADR-0111) — the
user approves the station once and the station holds a refresh session revocable per device,
so no secret is distributed to machines (ADR-0070 D5's reason); `svc-reel` validates tokens
locally (`iss` and `aud`, never signature alone) and derives the tenant from the resourceowner
claim with the account resolved from Paridyum's own tables, per ADR-0047 D5; context fails closed
per ADR-0038; correlation ids on every job (ADR-0044); append-only audit; and STD-0009's shape —
**the denial is the assertion**: account A's worker cannot see, claim or completion-tamper account
B's jobs; A's console cannot read B's ledger. Those denials are test suites, not sentences.

**Vocabulary, three words used precisely:** a **station** is a machine running the agent — the
ingestion station is one whose job is copying cards; a **worker** is that agent bound to one
device and one account, claiming jobs; the **ledger** is the account's record in the service. The
service is **not a control plane** (GLO-0007 reserves that) and a set of workers is **not a fleet**
(GLO-0009 reserves that). RFC-0038 already uses "worker" for uploader guests; this RFC keeps that
word and drops the other.

### D4 — The archive grammar and the custody rule (carried forward, re-scoped to an account)

The first draft's decisions 2 and 3 stand, unchanged in substance, now as product rules per account:

```
<archive>/shoots/<YYYY>/<YYYY-MM-DD>-<slug>/
    orig/<device-id>/<batch>/…      camera originals, structure preserved byte-for-byte
    manifest/                       ASC MHL generations + shoot.json
    derived/                        regenerable
    deliver/                        finished outputs; local until published
```

- Date-first (principal photography date), **slug assigned once and never renamed**, **no person's
  name in any path segment** (ADR-0071 §4's reason on media), originals preserved in their camera
  structure, `<batch>` disambiguates ingest sessions and the manifest binds the card.
- Ingest is **copy → hash (xxHash64, MHL-compatible) → manifest → verify by re-hash**; the write
  is not trusted until the read-back is equal; failures are loud and stop the batch.
- States are earned in order: `opened` → `archived` → `uploaded` → `clearable`. **A card is cleared
  only when its shoot is archived and present in one further governed place** — the account's
  reconciled off-host store, or a second governed local store — and clearing is a deliberate human
  act, never a pipeline step. `reel cards status` answers *what still depends on this card* at
  every moment. That sentence is REQ-0023's core.
- Custody events are service records when the service is reachable and local log entries when it
  is not; either way they reconcile into the account ledger.

### D5 — The off-host store is the account's own — Sync or S3-compatible — and the platform never holds the secret

**Bring your own destination.** An account configures one or more off-host stores in the portal,
and a shoot names the store it lands in (defaulting to the account's). Two production classes:

- **`s3`** — any S3-compatible endpoint (Backblaze B2, Cloudflare R2, MinIO, AWS): resumable,
  integrity-checkable, headless-friendly, no consumer apps anywhere. The production-grade class.
- **`sync`** — the account's own Sync.com, reached through the Vault: the consumer class, for the
  users who already have it and want their media where they can see it from a phone.

**The product does not rank the classes as better or worse; it is precise about what each can
prove.** S3-class stores publish object sizes and checksums, so reconciliation can compare them
(D7); a zero-knowledge Sync store cannot, so its floor is a listing comparison and its ceiling a
read-back rehearsal. Whoever wants verification strength in the loop picks the store that offers
it, and the product says which that is.

**The product does not resell storage.** The destination account is always the tenant's own; the
platform never buys, brokers or bills the bytes — which is what keeps this a capability and keeps
C-0006 anti-goal 2 untouched: nothing is resold at all. If the platform ever *provisions* a
destination for a tenant rather than pointing at one, REQ-0012 discharges the vendor first.

**For a Sync destination, the shape is the Vault, not the Sync folder.** The Sync folder is a
bidirectional mirror (delete-local deletes-remote), pulls trees to every device, and cannot live
where media lives (the vendor facts above). Vault is cloud-only, accepts uploads from external
drives, and is browsable from web and mobile when the user wants to look — the right shape for an
archive that stays put. The cloud tree mirrors the local grammar under `shoots/…`; `deliver/` goes
only to the shareable Files surface.

**Classification, stated openly:** originals are uploaded as they are **by default** — the point
is a browsable archive, and per-original `age` would make it un-previewable in either store;
**sealed shoots** exist for the exceptions: any shoot under a client contract, statutory material,
anything the operator marks sensitive, uploaded as `age`-encrypted bundle(s). Sealing is decided
**before the first upload, because uploading plaintext and sealing later does not un-expose
anything.** Statutory material never enters a plaintext tree. Names obey D4 regardless. ADR-0071's
rules for its own classes (documents, backups) remain the estate's own choice and are untouched —
this decision adds a class for media, it does not renegotiate the old one.

### D6 — Destination adapters are ranked per worker capability

An **adapter** is how one worker writes to one destination class. Workers declare the adapters they
can run (D3); the scheduler matches jobs accordingly. Ranked:

1. **`s3`** — rclone or an SDK on the worker, against any S3-compatible endpoint. Production-grade:
   resumable, integrity-checkable, headless, no vendor apps, no five-device cap, no OS matrix — the
   recommended adapter for every account that can choose its store. (rclone's adoption is owed to
   `tool-registry.yaml` per ADR-0066.)
2. **`sync-vault-client`** — the desktop client's Copy to Sync Vault: Windows, and the
   non-CloudFiles macOS build (CloudFiles 5.x on macOS cannot copy to Vault; small web-panel deltas
   remain). Any size, resume, de-duplication.
3. **`sync-webdav`** — the vendor's local WebDAV server with rclone, Vault-scoped, headless-
   capable, E2EE preserved. **Early access, invited accounts only**; adoption gated on an
   invitation or GA.
4. **`sync-windows-guest`** — RUN-0005's machinery, pointed at the archive; the interim headless
   path where no desktop OS with a client exists. Inherits RUN-0005's exposures and its credential
   discipline.
5. **`sync-browser`** — Playwright, bounded: small batches inside the vendor's web-panel comfort
   zone (≤ 100 files and ≤ 500 MB per transfer), the read-back half of reconciliation, and hosts
   where no other door opens. **Never the bulk path.** **The harness launches its own browser and
   never attaches to a remote one over CDP** — a browser not co-located with the driver refuses
   files above 50 MB. The browser profile is a credential: worker host only, never git, never an
   image (REQ-0021 item 4's shape).

One uploader per destination account by default; concurrency from measurement (REQ-0021 item 7).

### D7 — Completion is reconciliation, recorded as account evidence

The uploader's word is not completion — not the desktop client's "In Sync", not the agent's own
exit code. A fresh listing of the destination — an **S3 object list with sizes and checksums where
the store publishes them**, the Vault CSV export, an `rclone` listing, or a web-panel read-back —
is compared against the manifest: counts and byte totals per shoot, plus object hashes where the
destination offers them; where it does not (a zero-knowledge Sync store), the listing comparison is
the floor and the read-back rehearsal below is the ceiling. Mismatch means `archived`, not
`uploaded`; the job retries. The result is an account record like every other piece of evidence:

- **An adapter is not trusted until one shoot has round-tripped through it** — uploaded,
  downloaded on a different worker, re-hashed against the manifest. ADR-0071 §3, applied to media:
  *a backup that has never been restored is a belief, not a control.*
- **Re-trust after a major adapter or client update**, because the vendor's own release notes
  contain large-upload silent failures fixed — a channel whose failures are silent is re-proven,
  not assumed.
- **A skipped file is a reconciliation failure, not a warning** — D9 prevents it at the door.

### D8 — Gear is account data, and the identity rules are product rules

Every card, body, lens and reader gets a **permanent short id — assigned once, never reused, and
printed on the object**; the label is the id a human reads at 2 a.m. The account's gear records
live in the service (class, make/model, serial where printed, capacity, acquisition, status,
current custody) with the agent holding a local mirror so ingest works offline. Manifests bind
bytes → card → device → shoot; EXIF serials corroborate a body, never replace its id. Custody
events (a card cleared, a card retired) are recorded with who and when. Following
`device-registry.yaml`'s own split: the *rules* are product rules; the *list* is state, and state
never goes in a knowledge registry.

### D9 — The destination's limits are pre-flight checks

Before any batch is staged: for a **Sync** destination, every path segment is validated against the
vendor's current incompatible-character set (**the failure mode is a silent skip, not an error**),
files-per-folder counts are checked against the 50,000 cap (the D4 grammar makes this unreachable,
which is why it is shaped that way), and single files above 40 GB are noted as slower but
supported. For an **S3** destination the checks are the bucket's own — key length, policy,
checksum mode — and those fail loudly by design. The agent owns these checks, and they are the
mechanically checkable part of REQ-0023.

### D10 — What the product never does

- **The service never holds media, previews or EXIF** (D2), **never a vendor credential or
  session** (D5), and **never an operator bypass** — no internal mode, no admin path that a tenant
  could not also be granted (ADR-0049 D1/D2).
- **The agent never writes a source** — cards and phones are read-only in every step.
- **The pipeline never deletes, moves or renames an archived original.** Deletion is a human act,
  outside the tool; the pipeline's verbs are copy, verify, upload, reconcile, report.
- **Nothing trusts a status** — not the client's, not the agent's, not the service's own.
- **Logs carry counts and ids, never metadata content** (ADR-0080) — in the service and on the
  agent.
- **Local-only is a degradation mode, never the product.** The agent's offline mode exists so
  custody survives outages; it is not a way to dodge the tenant path. A shape that only its
  authors can use is the thing ADR-0049 exists to refuse.

### D11 — Sequencing, forced rather than chosen

1. **The runbook, first** — "the shoot comes home", executed by hand on the next shoot. Under
   C-0010 this is the production process at one shoot's scale, not a prototype; it is also what
   the agent will be written against (REQ-0023 item 10's runbook).
2. **`svc-reel` skeleton** — schema with two-column tenancy and its migration comments (ADR-0024,
   STD-0006), account/device/worker/job/lease records, the agent protocol. This is the step that
   makes it a product rather than a tool, and it lands **before** the agent ships anything.
3. **`agent-reel` agent** — ingest, manifests, destination adapters, reconciliation, worker mode; wired to the
   service from its first commit, with the offline mode as degradation (D10).
4. **`web-reel` console** — ledger, queue, custody, reconciliation; path-addressable per ADR-0104.

Registration and the dossier obligations are recorded in *What acceptance landed* (the MKT landed
with it); step 2 begins when the gate permits.
The order is ADR-0019's rule — decomposition precedes implementation — and ADR-0049's cost,
accepted rather than dodged: *"the first capability is slower by the width of a tenant model."*

## What acceptance landed

One operator act, a small exact footprint — no code, no spend:

1. **RFC-0054 and REQ-0023 moved `proposed` → `accepted`.** A status transition is the one
   frontmatter edit a knowledge artifact permits without supersession.
2. **`reel` is registered in `product-registry.yaml` as a `catalogue` product at `proposed`** —
   with the gate and the evidence named in the entry itself rather than discovered later.
3. **The dossier obligation is met at this stage: MKT-0005**, the external, non-tenant-derived
   evidence that selected the market, cited by the registry entry. Entry to `building`
   additionally owes a `REQ` (REQ-0023 serves until a product-specific one replaces it) and the
   `product-contract` slot; both wait on the gate.
4. **The gate is real and stated:** C-0006 anti-goal 6 bars `building` until hosting — vertical
   one, `building` since 2026-08-23 — has a paying customer who *renewed*. Nothing in this RFC
   softens it, and no work below is assigned a date around it.
5. **The first runbook is the next artifact** — *the shoot comes home* — so the operator's next
   shoot is governed by hand long before the product can be built.

Three of those are edits to files that already existed; the MKT is new. Nothing else moved.

## The run, in order — a shoot from card to clearable, with workers

0. **Before the shoot:** cards and gear registered (D8) — the habit that makes every later
   question answerable. For tenant zero, the first shoot of all is run by hand under the runbook.
1. **On return**, on any worker declaring `ingest`: `reel shoot open` for the date and slug; the
   tree is created and the slug is permanent.
2. **Per card:** the worker enumerates the real listing, hashes on read, copies into
   `orig/<device>/<batch>/`, re-hashes, compares. Failure stops that batch; nothing is cleared.
3. **Manifest** written (MHL generation + `shoot.json`); the card's custody state becomes
   *holding*; the shoot is `opened` until every batch verifies → `archived`.
4. **Upload:** a worker with a suitable adapter claims the upload job, takes a lease, writes the
   shoot into the destination's `shoots/<YYYY>/<date>-<slug>/` (the Vault for Sync; the bucket
   and prefix for S3), and completes with evidence.
5. **Reconcile:** a fresh listing of the destination — object list and checksums for S3, the Vault
   listing for Sync — is compared against the manifest (D7). Mismatch → retry. Match → `uploaded`.
6. **Clear:** the operator sees `reel cards status` and clears the cards; custody events recorded;
   the cards return to service. A shoot far behind on uploads can clear against the second local
   store instead — the purchase decision in open question 4.
7. **After:** `derived/` is worked locally; `deliver/` reaches a shareable surface only when
   something is being shared; the destination copy stays the off-host truth; the ledger answers
   where every byte is.

## The worked example — Acme Corp

A media company, eight people, one tenant, two accounts — the whole model on one page.

```
platform: Paridyum
└── tenant: acme        Zitadel org "acme" · acme.paridyum.com · Acme's own billing
    └── reel
        ├── account: commercial   its ledger, its stations, its destination
        └── account: weddings     isolated from commercial; different people granted
```

**Setup.** Tenant zero provisions Acme through the ordinary saga (tenant row, Zitadel org,
branding, grants, hostname, billing). With Release (RFC-0049) reel starts for Acme; today the
equivalent is a pipeline deployment. Priya (owner), Arjun (DIT) and Meera (editor) are invited —
roles from the reel project, membership in Acme's own tables, access by grant, and Meera is
granted `commercial` only. Arjun pairs the ingest-room Mac and a headless Linux box by device
code; the Linux box is marked `adapter:s3`, window 00:00–07:00. Acme adds its own B2 bucket in the
portal and types the key once on each station — Paridyum never sees it.

**A shoot.** 2026-10-03, the Zephyr commercial. Four cards, already labelled `C-014`…`C-017`.
Arjun opens the shoot in `commercial`; each card is enumerated, hashed, copied to
`orig/<device>/<batch>/`, re-hashed and manifested. The drone card is ingested on the second Mac at
the same time — ingest parallelises, one ledger. Every batch verifies → `archived`, cards held.
Overnight the Linux box leases the upload job and writes the shoot into Acme's bucket;
reconciliation compares the object listing and checksums against the manifest → `uploaded`. Priya
sees nothing depends on the cards and clears them.

**Who sees what.**

| Party | Sees | Cannot see |
| --- | --- | --- |
| Priya (owner) | both accounts, config, stations, custody | — |
| Meera (editor) | the `commercial` ledger | `weddings` |
| Arjun (DIT) | the stations he runs, the ingest queue | destination secrets unless granted |
| Paridyum | tenant rows, station registrations, job counts, hashes | **no footage, no EXIF, no Acme's key** |
| Borealis Films (the next tenant) | its own everything | that Acme exists at all |

**And tenant zero is this same script**, run by Paridyum for its own media estate — the first row,
not a special case.

## Costs, stated

- **The product is slower to appear than a script would be**, because nothing is demonstrated
  before the tenant path exists (ADR-0049's accepted cost, arriving where it is felt). The runbook
  is what makes this bearable: the operator's friction drops at step 1, not step 3.
- **A metadata-only platform cannot help a user whose devices are all offline.** Queues wait; the
  ledger is stale until a worker reports. That is the price of never being custodian of the media,
  and it is worth it.
- **Agent distribution is a new class of problem for this estate**: signed binaries across three
  OSes, updates, revocation, support. REQ-0014/ADR-0053 pins apply; the mechanism is owed (open
  question 6). This cost did not exist when the thing was a script on one person's laptop.
- **BYO destination means a support matrix** — two destination classes, several adapters, vendor
  client builds, OSes, WebDAV enrolment, and the S3-compatibility spectrum (checksum modes,
  multipart semantics, region quirks). The ranked adapters and per-worker capabilities are the
  containment, not an elimination, and each carries D7's rehearsal obligation.
- **The catalogue route's costs are live, by decision.** MKT-0005 is owed and written; entry to
  `building` is gated by C-0006 anti-goal 6 until hosting renews; anti-goal 1 sequences operating
  before franchising. None of that gates the design work or the runbook; it gates the build.
- **Plaintext-by-default at the destination has no undo** for a shoot once uploaded; sealing is
  decided before the first byte (D5, open question 5).
- **Cards are held in escrow** until uploads land — the custody rule's price at the busiest moment.
- **The manual legs rot**; the rehearsal obligation (D7) is the mitigation and it costs one
  download per adapter.

## Alternatives considered

**A — Keep the first draft's operator tool.** Rejected on the operator's requirement and
ADR-0049 decision 4/5: the thing has a buyer beyond us, so it is a service-shaped capability and
must be built as one. The agent survives; the framing does not.

**B — Server-side integration with the vendor.** Impossible, not merely rejected: the vendor
publishes no API, its store cannot receive an S3-style stream (ADR-0071's finding), and
zero-knowledge means the keys that could act on a user's behalf never leave the user's devices.
The agents *are* the API, and that is a constraint of the terrain.

**C — The platform stores and stages the media.** Rejected four times over: it would make
Paridyum the custodian of other people's footage (residency, deletion, breach surface), break the
zero-knowledge property end to end, hand the operator the very access RFC-0022's conflict is about,
and buy disk for media that already has a home. The tenants' devices plus their own cloud make it
unnecessary.

**D — The Sync folder as the archive tree.** Rejected: bidirectional deletes, device pull-down
against a five-device cap, macOS's external-drive ban. (First draft's alternative A.)

**E — `age`-encrypt every original by default.** Rejected as default; kept as the per-shoot seal
(D5). (First draft's alternative B.)

**F — Browser automation as the primary uploader.** Rejected on the vendor's own published
guidance; kept bounded (D6.4). (First draft's alternative C.)

**G — An S3-compatible vendor now (B2, R2) with rclone.** Rejected for now; recorded as the exit
path — the ledger and manifests are transport-agnostic, which is also REQ-0012's answer. (First
draft's alternative D, unchanged.)

**H — One service absorbing platform concerns.** Rejected: `svc-reel` owns only media-custody
tables and uses `platform-identity`, tenancy and routing like every other product; absorbing them
would be ANTI-0006's shape and ANTI-0001's ownership violation.

## Open questions

1. **RESOLVED 2026-09-27: `catalogue`.** The operator chose the creator-market route after the
   capability route was tabled and compared — reel is a vertical Paridyum operates and will later
   franchise, not platform toolbox. The consequences are accepted, not softened: MKT-0005 was
   written as the external, non-tenant-derived evidence, and **entry to `building` is gated by
   C-0006 anti-goal 6** until hosting has a paying customer who renewed. Re-takeable later only
   by a deliberate reclassification, which is itself a decision with an artifact.
2. **RESOLVED 2026-09-27: `reel`.** The operator chose the noun; the parts are `svc-reel`,
   `web-reel` and `agent-reel`, and the binary is `reel` (C-0007's derivation).
3. **RESOLVED 2026-09-27: a new `agent-` class, authorised.** `tool-` could not carry the station
   — ADR-0070 gives a tool no OIDC client and no internet identity, and the station needs both —
   so ADR-0111 adds the class and its Native device-authorization identity. The registry entry
   and the two generated enums land with it, before the agent's first commit.
4. **The hub, and the second copy.** Which machines hold each account's working archive, and
   whether a second local store is bought so cards can clear before uploads land. Per tenant
   configuration, not code. **Closure path:** the operator's machine-and-drive inventory; the
   answer is configuration, not design.
5. **RESOLVED 2026-09-27: plaintext at the destination stays the default**, with the per-shoot
   seal lever (D5). The operator confirmed; sealing remains a pre-upload decision, because there
   is no undo.
6. **Agent distribution and updates.** Signing, release channels, auto-update, revocation — a new
   domain for the estate; REQ-0014 and ADR-0053 bind, the mechanism is owed. **Closure path:** a
   design artifact before any station ships beyond tenant zero — the trigger is the first
   shipment, not a date.
7. **Phones.** Whether Camera Uploads is enabled per account as an inbox the worker drains, and
   the rehearsal question D7 leaves open: dedupe semantics when the same bytes exist in Files and
   Vault and the Files copy is removed. Checked, not assumed. **Closure path:** the first
   rehearsal (D7) against a real Sync account, recorded in the logbook (RFC-0033); a
   problem-registry entry if the behaviour surprises (ADR-0076).
8. **RESOLVED 2026-09-27: ledger and custody only for v0** — read-first: shoots, card status,
   queue states, reconciliation results. Configuration and queue control follow in later slices.

## What this does not decide

- **Pricing.** BET-0007's shape is org-level; a per-product BET- sets numbers if and when the
  product reaches `operated`.
- **Franchise timing** — when reel is offered to founder-tenants; C-0006 anti-goal 1 sequences it
  behind the operated phase, and the gate in the Status section stands.
- **Post-production.** NLE projects, proxies, grades — `derived/` is a place, not a pipeline.
- **Any change to ADR-0071's rules for its own classes.**
- **Purchases** — drives, readers, accounts, licences.
- **Entitlement specifics** — plan and quota contents; `platform-entitlement` owns the mechanism.
- **Portal-delivered destination secrets.** The portal configures; the secret stays on the worker
  (D2). Pushing secrets *to* workers from the platform is a later capability behind REQ-0001's
  discharge and a tenant-key scheme, and this RFC deliberately does not imply it.
- **The PMD relationship.** `reel` is Paridyum's capability; the media estate uses it as any
  account would, and RFC-0052's structure is untouched.

## Related

- **REQ-0023** — the custody obligations this design discharges; unchanged by this revision
- **C-0008**, **ADR-0049** — every capability is a product; nothing runs outside a tenant; the
  buyer is named
- **ADR-0050**, `product-registry.yaml` — the registration (`catalogue`, `proposed`) and the gate
  it states; **ADR-0111** — the `agent-` class and the station's Native identity; **MKT-0005** —
  the external evidence that selected the market
- **BET-0005**, **BET-0004**, **C-0006** — the levels, the catalogue, and the anti-goals the
  registration route must respect
- **ADR-0104**, **RFC-0049**, **ADR-0039**, **ADR-0047** — instance, release, saga, identity
- **ADR-0024**, **GLO-0004**, **GLO-0005**, **ADR-0038** — the tenancy this product sits in
- **STD-0009**, **ADR-0044**, **ADR-0080**, **REQ-0013** — the denial tests, the audit, the logs,
  and walking away
- **ADR-0071**, **REQ-0021**, **RFC-0038**, **RUN-0005** — the off-host store, egress discipline,
  and the Windows worker precedent
- **RFC-0052** — whose media this is; **ADR-0066**, **ADR-0053**, **REQ-0014**, **REQ-0012** —
  tools, pins, supply chain, and the vendor discharge any provisioned destination would owe
- **GLO-0007**, **GLO-0009** — the two words this document spends four paragraphs not using
- **C-0010**, **ADR-0019**, **ANTI-0001**, **ANTI-0006** — no prototype tier, decomposition first,
  one owner per table, no god service

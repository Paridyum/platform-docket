# Decision debt

Open loops in Paridyum's knowledge base. Maintained by the `capture-open-loops` Stop
hook (ADR-0011) and surfaced at the start of every session, so that a decision left
unrecorded is put back in front of the next person to work here — whether or not
anyone remembers it exists.

<!-- REGENERATED: 2026-09-24 — do not hand-edit below; close the loop and the entry disappears -->

## Open loops

### Undecided — draft or proposed

- **GLO-0007** `proposed` for 43d — Control region — where the platform's global facts live, not where a customer belongs
  `glossary/GLO-0007-control-region.md`
- **GLO-0008** `proposed` for 43d — Datacentre — a failure domain inside a region, never a residency boundary
  `glossary/GLO-0008-datacentre.md`
- **GLO-0009** `proposed` for 37d — Fleet manager — what provisions metal, and deliberately not called a control plane
  `glossary/GLO-0009-fleet-manager.md`
- **RFC-0014** `proposed` for 44d — A region is a residency boundary, and every table declares which plane it lives in
  `rfc/RFC-0014-a-region-is-a-residency-boundary.md`
- **RFC-0015** `proposed` for 44d — A fact has one home, and nothing crosses a region boundary
  `rfc/RFC-0015-a-fact-has-one-home.md`
- **RFC-0016** `proposed` for 44d — The edge is rented, never holds tenant data, and never asserts identity
  `rfc/RFC-0016-the-edge-is-rented.md`
- **RFC-0017** `proposed` for 44d — An operator's personal data lives in the operator's region, not the control region
  `rfc/RFC-0017-operator-personal-data-is-regional.md`
- **RFC-0018** `proposed` for 44d — The control plane is one repository, with a closed list and a split trigger
  `rfc/RFC-0018-the-control-plane-is-one-repository.md`
- **RFC-0019** `proposed` for 43d — A datacentre is one Proxmox cluster, and Kubernetes runs on virtual machines above it
  `rfc/RFC-0019-a-datacentre-is-one-proxmox-cluster.md`
- **RFC-0020** `proposed` for 43d — An external fact is cited and dated, because it changes somewhere else
  `rfc/RFC-0020-external-facts-are-cited-and-dated.md`
- **RFC-0021** `proposed` for 43d — A device hostname is family, country and sequence, and it is not a database
  `rfc/RFC-0021-a-device-hostname-is-family-country-and-sequence.md`
- **RFC-0022** `proposed` for 40d — What must be true for a founder to build on a platform whose operator competes with them
  `rfc/RFC-0022-the-operator-competes-with-its-tenants.md`
- **RFC-0023** `proposed` for 40d — The development substrate is production one level down, and the transition is a variable
  `rfc/RFC-0023-the-development-substrate-is-production-one-level-down.md`
- **RFC-0025** `proposed` for 37d — The fleet manager bootstraps itself, and Paridyum is customer zero
  `rfc/RFC-0025-the-fleet-manager-provisions-paridyum.md`
- **RFC-0026** `proposed` for 37d — The covenant vehicle — where the tithe goes, and what holds it when the sums are small
  `rfc/RFC-0026-the-covenant-vehicle.md`
- **RFC-0027** `proposed` for 37d — Research is captured where it happens, because provenance does not survive a memory
  `rfc/RFC-0027-research-is-captured-where-it-happens.md`
- **RFC-0028** `proposed` for 37d — The web template — what template-web contains, and the five decisions it needs first
  `rfc/RFC-0028-the-web-template.md`
- **RFC-0029** `proposed` for 36d — A derivation is an artifact, and an estimate is writable only if it is labelled
  `rfc/RFC-0029-a-derivation-is-an-artifact.md`
- **RFC-0030** `proposed` for 36d — A task id cited by a governed artifact must resolve, and two files must not disagree about it
  `rfc/RFC-0030-a-cited-task-id-must-resolve.md`
- **RFC-0031** `proposed` for 36d — The pull request is not the unit of work — stacks, a work-in-progress ceiling, and automatic draining
  `rfc/RFC-0031-the-pull-request-is-not-the-unit-of-work.md`
- **RFC-0032** `proposed` for 36d — Every repository carries an SOP, and every repository declares what it provides
  `rfc/RFC-0032-every-repository-carries-an-sop.md`
- **RFC-0033** `proposed` for 36d — Every deliverable carries a logbook, and how it was built is captured rather than remembered
  `rfc/RFC-0033-every-deliverable-carries-a-logbook.md`
- **RFC-0036** `proposed` for 24d — The substrate host's graphical session is reached over the tunnel, by a service that owns no port on the LAN
  `rfc/RFC-0036-the-substrate-hosts-graphical-session-is-reached-over-the-tunnel.md`
- **RFC-0038** `proposed` for 24d — The personal drive is read-only, its egress is exclusive, and it finishes before the substrate exists
  `rfc/RFC-0038-the-personal-drive-is-read-only-exclusive-and-off-the-host-before-the-substrate.md`
- **RFC-0039** `proposed` for 19d — The people platform is one program, and the order it must be built in is forced rather than chosen
  `rfc/RFC-0039-the-people-platform-is-one-program-and-its-order-is-forced.md`
- **RFC-0040** `proposed` for 19d — A diagram is a renderer, and the model stays where it already is
  `rfc/RFC-0040-a-diagram-is-a-renderer-and-the-model-stays-where-it-is.md`
- **RFC-0043** `proposed` for 16d — web-careers is carved out of web-paridyum in four independently revertable slices
  `rfc/RFC-0043-web-careers-is-carved-out-of-web-paridyum-in-four-slices.md`
- **RFC-0044** `proposed` for 9d — docket as a service completes the platform against ADR-0042, tenant zero first
  `rfc/RFC-0044-docket-as-a-service-completes-the-platform.md`
- **RFC-0045** `proposed` for 9d — The isolation envelope is a placement, a containment class, and an attestation
  `rfc/RFC-0045-the-isolation-envelope-is-a-placement-a-containment-class-and-an-attestation.md`
- **RFC-0046** `proposed` for 9d — A tenant-held key is an availability dependency, and revocation is destruction
  `rfc/RFC-0046-a-tenant-held-key-is-an-availability-dependency-and-revocation-is-destruction.md`
- **RFC-0047** `proposed` for 9d — Renting the platform is a stamp, not a third column
  `rfc/RFC-0047-renting-the-platform-is-a-stamp-not-a-third-column.md`
- **RFC-0048** `proposed` for 9d — A public surface is a tenant's surface, and Paridyum is its first tenant
  `rfc/RFC-0048-a-public-surface-is-a-tenants-surface-and-paridyum-is-its-first-tenant.md`
- **RFC-0049** `proposed` for 9d — A tenant is a set of releases, and /dashboard is where they are deployed
  `rfc/RFC-0049-a-tenant-is-a-set-of-releases-and-the-dashboard-is-where-they-are-deployed.md`
- **RFC-0050** `proposed` for 9d — The dashboard's first run is the provisioning saga made visible
  `rfc/RFC-0050-the-dashboards-first-run-is-the-saga-made-visible.md`
- **RFC-0051** `proposed` for 9d — A tenant chooses a jurisdiction, and nothing below it
  `rfc/RFC-0051-a-tenant-chooses-a-jurisdiction-and-nothing-below-it.md`

### Durability

- Uncommitted changes in the knowledge repo — governance edits that exist on one disk only


<!-- APPENDED: reviewed and cleared by a human; the hook never removes these -->

## Possible unrecorded decisions

- [ ] 2026-09-20 **`docket` and `hiring` are registered at `capability`/`building` and each owes a REQ-.** A capability at `building` owes `artifacts: [REQ]` and the `tenant-facing-interface` slot. The slot is filled for both — web-docket for docket, web-hiring for itself. The REQ- is not written for either. `product.stage_dossier_complete` is `owner: tool-forge` and therefore **not implemented**, so both entries pass the gate today while being incomplete. Registering at `proposed` would have made the gate honest and the *record* false — both are deployed and serving, and `proposed` reads "nothing is being built". The stage is correct and the dossier is owed; CLAUDE.md's rule for an unimplemented check is to behave as though it were running, which is what this line is for. Clear it by writing both REQ- artifacts, or by recording why a capability at `building` need not.

- [ ] 2026-08-08 session `6a39cd18` — 12 decision-shaped statements, no artifact created or changed. Was something decided that is not written down?
- [ ] 2026-08-08 **ADR-0045 risk R3 expires at the first deployment.** The Zitadel masterkey, database credentials and service-account keys all exist before REQ-0001 (runtime secret management) is discharged, and REQ-0001's own context notes GitHub Free has no org-level secrets for private repositories — so there is no secret-management story at all. Accepted only until something is deployed. Clear this by moving REQ-0001 from `deferred` to `accepted`, or by recording why the risk still stands.
- [ ] 2026-08-08 **C-0003 is enforced nowhere for checkpoint commits.** `git.conventional_commit_subject` and `git.no_ai_authored_as_human` are `severity: error` on a locked constitution, but `--no-verify` skips the pre-commit hook, the checkpoint hook calls `check-commits.sh --staged` (identity config only, never the message or trailers), CI is `continue-on-error` on push, and no PRs are opened. Three individually-justified bypasses leaving a constitutional rule with no execution path. Now inside REQ-0006's scope, which fired today.
- [ ] 2026-08-08 **Lock ADR-0032, ADR-0034 and STD-0010** when the first `spec-` repo exists or the first service publishes an error or event. Held because they encode designs we invented that nothing has consumed; ADR-0033/GLO-0006 locked instead because ISO 4217 and BCP 47 already froze theirs. Nothing else would remember: no stale-`accepted` check exists and an ADR cannot carry a `trigger`.
- [ ] 2026-08-17 session `1a7357ad` — 46 decision-shaped statements, no artifact created or changed. Was something decided that is not written down?
- [ ] 2026-08-17 session `6e66e6a9` — 83 decision-shaped statements, no artifact created or changed. Was something decided that is not written down?
- [ ] 2026-08-18 session `728efb92` — 15 decision-shaped statements, no artifact created or changed. Was something decided that is not written down?
- [ ] 2026-08-19 session `2f35e2c2` — 119 decision-shaped statements, no artifact created or changed. Was something decided that is not written down?
- [ ] 2026-08-22 session `a8266409` — 36 decision-shaped statements, no artifact created or changed. Was something decided that is not written down?

- [ ] 2026-08-22 **The Paridyum brand typeface is not chosen.** STD-0015 fixes the colour half of the visual identity because the logo boards fix it. They do not fix a face: the wordmark is drawn as outlines and board 02's annotations are Illustrator's default Arial, which is an export artefact rather than a choice. `libs/lib-ui/brand/brand.css` therefore ships a system grotesque and no brand face. The web-surface design (§3.3) expects the marketing theme to have "its own face", so this blocks that theme being complete. Close it by choosing a face and recording it as an erratum-free addition to STD-0015, or by recording that the system stack IS the decision.
- [ ] 2026-08-31 session `1d09d6be` — 15 decision-shaped statements, no artifact created or changed. Was something decided that is not written down?
- [ ] 2026-08-31 read `https://www.sync.com/help/the-sync-desktop-application/` (session `d1e099c1`) — no artifact declares it in `sources`. Cite it under RFC-0020, or tick this if it was a dead end.
- [ ] 2026-08-31 read `https://www.sync.com/help/what-are-the-system-requirements-for-sync/` (session `d1e099c1`) — no artifact declares it in `sources`. Cite it under RFC-0020, or tick this if it was a dead end.

- [ ] 2026-09-05 **Five deferred requirements have triggers that have already fired, and nothing
      detected it.** `knowledge.deferred_requirements_have_triggers` checks that a deferred
      requirement HAS a trigger. No check asks whether the trigger has GONE OFF, so a
      requirement deferred correctly stays deferred after the condition it was waiting for
      arrives. Found by reading them against what is now running, not by any gate.

  - **REQ-0001** runtime secret management — *"the first service needs a credential at runtime"*.
    The edge holds a Cloudflare DNS-edit token in `/etc/paridyum/lego.env`, the cluster holds a
    `ghcr` pull secret, and SOPS age keys sit on the workstation. ADR-0045 risk R3 says this
    expires at the first deployment; the first deployment happened on 2026-09-02.
  - **REQ-0004** runtime and operations governance — *"the first service is deployed to a cluster
    and serves a request from anything other than a test"*. `paridyum.com` serves the public.
  - **REQ-0009** accessibility and internationalisation — *"the first user-facing interface is
    written"*. A landing page and a careers page, in two locales, one right-to-left.
  - **REQ-0014** supply-chain integrity and provenance — *"the first artifact intended to run
    somewhere other than a developer machine"*. Container images with SBOM and provenance, and
    `@paridyum/ui` published twice today.
  - **REQ-0018** running software is tested from outside — *"the first HTTP surface reachable
    from anything other than a developer machine"*. Nothing tests the public site from outside;
    every check today runs against source or a local build.

  Close by moving each to `accepted` and doing the work, or by recording why the deferral still
  holds now that its condition is met. **The cheapest durable fix is a check** — a deferred
  requirement whose trigger has fired is a decision that has gone quietly overdue, which is
  exactly what this ledger exists to prevent.

- [ ] 2026-09-05 **There is no persistence decision, and it blocks more than it looks like.**
      ADR-0020 puts workloads on Kubernetes and ADR-0023/0024/0038 specify tenancy as row-level
      in one database with session context set per transaction. The cluster has no StorageClass,
      no PersistentVolumeClaim and no database, so none of that data architecture can be
      implemented. It blocks ADR-0063 Phase B (job applications), REQ-0008 (migrations),
      REQ-0003 (personal data), REQ-0013 and REQ-0020. Named in ADR-0063 as the next artifact.

- [ ] 2026-09-05 **`tool-docket` stores personal data and REQ-0003 is still deferred.**
      REQ-0003's trigger names "the first *service*" and docket is class `tool`, so the letter
      of the trigger has arguably not been met — while `people/*.yaml` holds names, addresses
      and engagement records for real people, which is why that directory is gitignored. Either
      the trigger's wording is too narrow or the deferral is wrong; both are decisions.

- [ ] 2026-09-06 **Officer documents name the company before the company exists.**
      Operator decision 2026-09-06 (RFC-0042 §3): everything is written in PARIDYUM's name and
      treated as in effect from signature, as it would be after incorporation — no separate
      pre-incorporation drafting, no assignment to an individual. It rests on the assumption that
      Internship Agreement §13's transfer clause carries these into the entity on registration.
      **Scope reduced 2026-09-06:** the operator confirmed incorporation completes BEFORE the
      first intern is hired, so no intern IPA is ever executed pre-incorporation. This applies to
      four officer documents, held by the four people who would be party to any correction and
      all still in office when the CIN issues — a smaller and more fixable exposure than the
      whole-estate one first described. Copyright needs no filing to subsist, so nothing is lost
      by not registering early. Still worth putting to counsel at incorporation, along with the
      founders' arrangement — equity, vesting, leaver terms — which RFC-0042 does not draft.
      Trigger: issue of the CIN.
- [ ] 2026-09-07 read `https://raw.githubusercontent.com/zitadel/zitadel/v4.15.3/internal/api/grpc/management/proto/zitadel/management.proto` (session `a8266409`) — no artifact declares it in `sources`. Cite it under RFC-0020, or tick this if it was a dead end.
- [ ] 2026-09-07 read `https://zitadel.com/docs/apis/resources/mgmt/management-service-add-oidc-app` (session `a8266409`) — no artifact declares it in `sources`. Cite it under RFC-0020, or tick this if it was a dead end.
- [ ] 2026-09-09 read `https://api.github.com/repos/charmbracelet/vhs` (session `e1bb2152`) — no artifact declares it in `sources`. Cite it under RFC-0020, or tick this if it was a dead end.
- [ ] 2026-09-09 read `https://github.com/mkdocs/catalog` (session `e1bb2152`) — no artifact declares it in `sources`. Cite it under RFC-0020, or tick this if it was a dead end.
- [ ] 2026-09-12 read `https://zitadel.com/docs/apis/resources/admin/admin-service-add-smtp-config` (session `a8266409`) — no artifact declares it in `sources`. Cite it under RFC-0020, or tick this if it was a dead end.
- [ ] 2026-09-12 read `https://pve.proxmox.com/wiki/Package_Repositories` (session `a8266409`) — no artifact declares it in `sources`. Cite it under RFC-0020, or tick this if it was a dead end.
- [ ] 2026-09-16 session `47909226` — 214 decision-shaped statements, no artifact created or changed. Was something decided that is not written down?

Tick an item to dismiss it, or record the decision as an ADR and delete the line.

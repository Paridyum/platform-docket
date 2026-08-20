# Decision debt

Open loops in Paridyum's knowledge base. Maintained by the `capture-open-loops` Stop
hook (ADR-0011) and surfaced at the start of every session, so that a decision left
unrecorded is put back in front of the next person to work here — whether or not
anyone remembers it exists.

<!-- REGENERATED: 2026-08-20 — do not hand-edit below; close the loop and the entry disappears -->

## Open loops

### Undecided — draft or proposed

- **RFC-0014** `proposed` for 9d — A region is a residency boundary, and every table declares which plane it lives in
  `rfc/RFC-0014-a-region-is-a-residency-boundary.md`
- **RFC-0015** `proposed` for 9d — A fact has one home, and nothing crosses a region boundary
  `rfc/RFC-0015-a-fact-has-one-home.md`
- **RFC-0016** `proposed` for 9d — The edge is rented, never holds tenant data, and never asserts identity
  `rfc/RFC-0016-the-edge-is-rented.md`
- **RFC-0017** `proposed` for 9d — An operator's personal data lives in the operator's region, not the control region
  `rfc/RFC-0017-operator-personal-data-is-regional.md`
- **RFC-0018** `proposed` for 9d — The control plane is one repository, with a closed list and a split trigger
  `rfc/RFC-0018-the-control-plane-is-one-repository.md`
- **RFC-0019** `proposed` for 8d — A datacentre is one Proxmox cluster, and Kubernetes runs on virtual machines above it
  `rfc/RFC-0019-a-datacentre-is-one-proxmox-cluster.md`
- **RFC-0020** `proposed` for 8d — An external fact is cited and dated, because it changes somewhere else
  `rfc/RFC-0020-external-facts-are-cited-and-dated.md`
- **RFC-0021** `proposed` for 8d — A device hostname is family, country and sequence, and it is not a database
  `rfc/RFC-0021-a-device-hostname-is-family-country-and-sequence.md`
- **RFC-0022** `proposed` for 5d — What must be true for a founder to build on a platform whose operator competes with them
  `rfc/RFC-0022-the-operator-competes-with-its-tenants.md`
- **RFC-0023** `proposed` for 5d — The development substrate is production one level down, and the transition is a variable
  `rfc/RFC-0023-the-development-substrate-is-production-one-level-down.md`
- **RFC-0025** `proposed` for 2d — The fleet manager bootstraps itself, and Paridyum is customer zero
  `rfc/RFC-0025-the-fleet-manager-provisions-paridyum.md`
- **RFC-0026** `proposed` for 2d — The covenant vehicle — where the tithe goes, and what holds it when the sums are small
  `rfc/RFC-0026-the-covenant-vehicle.md`
- **RFC-0027** `proposed` for 2d — Research is captured where it happens, because provenance does not survive a memory
  `rfc/RFC-0027-research-is-captured-where-it-happens.md`
- **RFC-0028** `proposed` for 2d — The web template — what template-web contains, and the five decisions it needs first
  `rfc/RFC-0028-the-web-template.md`
- **RFC-0029** `proposed` for 1d — A derivation is an artifact, and an estimate is writable only if it is labelled
  `rfc/RFC-0029-a-derivation-is-an-artifact.md`
- **RFC-0030** `proposed` for 1d — A task id cited by a governed artifact must resolve, and two files must not disagree about it
  `rfc/RFC-0030-a-cited-task-id-must-resolve.md`
- **RFC-0031** `proposed` for 1d — The pull request is not the unit of work — stacks, a work-in-progress ceiling, and automatic draining
  `rfc/RFC-0031-the-pull-request-is-not-the-unit-of-work.md`
- **RFC-0032** `proposed` for 1d — Every repository carries an SOP, and every repository declares what it provides
  `rfc/RFC-0032-every-repository-carries-an-sop.md`
- **RFC-0033** `proposed` for 1d — Every deliverable carries a logbook, and how it was built is captured rather than remembered
  `rfc/RFC-0033-every-deliverable-carries-a-logbook.md`


<!-- APPENDED: reviewed and cleared by a human; the hook never removes these -->

## Possible unrecorded decisions

- [ ] 2026-08-08 session `6a39cd18` — 12 decision-shaped statements, no artifact created or changed. Was something decided that is not written down?
- [ ] 2026-08-08 **ADR-0045 risk R3 expires at the first deployment.** The Zitadel masterkey, database credentials and service-account keys all exist before REQ-0001 (runtime secret management) is discharged, and REQ-0001's own context notes GitHub Free has no org-level secrets for private repositories — so there is no secret-management story at all. Accepted only until something is deployed. Clear this by moving REQ-0001 from `deferred` to `accepted`, or by recording why the risk still stands.
- [ ] 2026-08-08 **C-0003 is enforced nowhere for checkpoint commits.** `git.conventional_commit_subject` and `git.no_ai_authored_as_human` are `severity: error` on a locked constitution, but `--no-verify` skips the pre-commit hook, the checkpoint hook calls `check-commits.sh --staged` (identity config only, never the message or trailers), CI is `continue-on-error` on push, and no PRs are opened. Three individually-justified bypasses leaving a constitutional rule with no execution path. Now inside REQ-0006's scope, which fired today.
- [ ] 2026-08-08 **Lock ADR-0032, ADR-0034 and STD-0010** when the first `spec-` repo exists or the first service publishes an error or event. Held because they encode designs we invented that nothing has consumed; ADR-0033/GLO-0006 locked instead because ISO 4217 and BCP 47 already froze theirs. Nothing else would remember: no stale-`accepted` check exists and an ADR cannot carry a `trigger`.
- [ ] 2026-08-17 session `1a7357ad` — 46 decision-shaped statements, no artifact created or changed. Was something decided that is not written down?
- [ ] 2026-08-17 session `6e66e6a9` — 83 decision-shaped statements, no artifact created or changed. Was something decided that is not written down?
- [ ] 2026-08-18 session `728efb92` — 15 decision-shaped statements, no artifact created or changed. Was something decided that is not written down?
- [ ] 2026-08-19 session `2f35e2c2` — 119 decision-shaped statements, no artifact created or changed. Was something decided that is not written down?

Tick an item to dismiss it, or record the decision as an ADR and delete the line.

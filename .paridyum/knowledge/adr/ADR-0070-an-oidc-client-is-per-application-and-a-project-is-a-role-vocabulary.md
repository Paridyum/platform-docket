---
id: ADR-0070
title: Every application gets its own OIDC client, and a project is a role vocabulary rather than a company
type: adr
status: accepted
created: 2026-09-07
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, web, console, mobile, tool, spec]
applies_to: ["*"]
product: [identity]
tags: [identity, zitadel, oidc, auth, clients, audience, roles, adr-0047, adr-0045, adr-0049, adr-0055, req-0001, glo-0005]
breaking: false
sources:
  - url: https://zitadel.com/docs/concepts/structure/projects
    title: "ZITADEL — Projects"
    retrieved: 2026-09-07
    supports: >
      That a project is "a vessel for all components who are closely related to each other",
      that it contains applications, roles and role assignments, that ALL APPLICATIONS WITHIN A
      PROJECT SHARE THE SAME ROLES AND ASSIGNMENTS, and that a project may be granted to
      another organization whose admins then manage their own users' assignments. Decision 2
      rests on the shared-role-vocabulary property.
  - url: https://zitadel.com/docs/guides/manage/console/applications
    title: "ZITADEL — Applications"
    retrieved: 2026-09-07
    supports: >
      The five application types (Web, Native, User Agent, API, SAML) and their permitted auth
      methods; that Native "only supports code authentication type"; that redirect URIs,
      additional origins, token settings and client credentials are configured PER APPLICATION;
      and that Development Mode "enables non-HTTPS protocols and glob patterns". Decisions 3
      and 5 rest on these.
  - url: https://zitadel.com/docs/apis/openidoauth/scopes
    title: "ZITADEL — Reserved scopes"
    retrieved: 2026-09-07
    supports: >
      That `urn:zitadel:iam:org:project:id:{projectid}:aud` is how a project id enters a
      token's audience — "By adding this scope, the requested project id will be added to the
      audience of the access token" — that it is REQUESTED BY THE CLIENT rather than implied,
      and that `urn:zitadel:iam:org:projects:roles` asserts roles only for projects already in
      the audience. Decision 4 rests on the opt-in property.
  - url: https://zitadel.com/docs/apis/resources/mgmt/management-service-add-oidc-app
    title: "ZITADEL — Management API: AddOIDCApp"
    retrieved: 2026-09-06
    supports: >
      The fields an OIDC application is created with — app type, auth method, response and grant
      types, redirect URIs — which is the shape decisions 3 and 5 constrain when they fix PKCE and
      a public client per application.
  - url: https://raw.githubusercontent.com/zitadel/zitadel/v4.15.3/internal/api/grpc/management/proto/zitadel/management.proto
    title: "ZITADEL v4.15.3 — management.proto"
    retrieved: 2026-09-06
    supports: >
      The API surface at the exact version this estate runs, read because the published docs and
      the deployed version can disagree. Pinned to v4.15.3 in the URL for that reason: an
      unversioned doc page is a moving claim about a fixed deployment.
enforced_by: []
---

# ADR-0070 — An OIDC client is per application, and a project is a role vocabulary

## Status

**Accepted**, 2026-09-07. Not `locked`: most of this is checkable in principle — a repository's
manifest could declare its client and a check could refuse a second repository naming the same
one — but nothing checks it today and there is no registry of applications to check against.
It graduates when the application list is registry data, which is decision 6.

## Context

ADR-0047 decision 2 already maps the estate onto Zitadel:

| Paridyum | Zitadel |
| --- | --- |
| tenant | organization |
| account | *none — Paridyum's own tables* |
| product | project |
| interface | application |

**That is a mapping table, not a rule, and two things fall through it.**

The first is what "interface" means when the thing needing a login is not a product. There are
already two such surfaces: `web-paridyum`, which is marketing and careers, and `console-docket`,
which is operator work. Neither is a catalogue product. ADR-0047 gives them no project, so the
first person to wire auth chooses one, and the cheapest choice is whichever project already
exists.

The second is that nothing says a client may not be reused. Sharing one `client_id` across two
applications is a five-second convenience, and every property that makes OIDC clients useful is
per-client: the redirect URI allowlist, the secret, the token settings, and the ability to
revoke one thing without revoking another.

**Three facts about Zitadel decide the shape**, all retrieved 2026-09-07 and cited above.

**All applications within a project share the same roles and assignments.** A project is not a
folder. It is a role vocabulary and every application inside it inherits the whole thing.

**Audience is opt-in and client-requested.** A project id enters a token's `aud` because a client
asked for it with `urn:zitadel:iam:org:project:id:{projectid}:aud`. Nothing is in the audience by
accident — and nothing is in it by default either, so a resource server that validates only the
signature accepts tokens minted for something else entirely.

**Roles follow the audience.** `urn:zitadel:iam:org:projects:roles` asserts roles for the
projects already in the token's audience. Audience is therefore the primary boundary and the
role claim is downstream of it.

## Decision

### 1. One OIDC application per deployable surface, per environment. Never shared

Every application that authenticates a human or a service gets its **own** Zitadel application,
with its own `client_id`, its own redirect URIs and its own token settings.

**Staging and production are different applications**, not one application with both hostnames in
its redirect list. A redirect allowlist holding a staging hostname is a production client that
will send a production authorization code to a staging origin, and the mistake is one typo in a
`redirect_uri` parameter away.

Sharing is not permitted for convenience, for "it's the same app really", or because two
surfaces are in one repository. **If two things can be deployed, revoked or rotated
independently, they are two applications.**

### 2. A project is a role vocabulary and an audience — never "the company"

A project is created for **one population holding one set of roles**. It is not a container for
everything one organisation runs.

The consequence is the part that will be argued: **Paridyum's own surfaces do not share a
project.** Candidates applying for a job and operators running the estate are two populations
with two role vocabularies, and putting them in one project would mean every operator role is
defined in the same namespace a candidate's token draws from. Zitadel shares roles across every
application in a project; that is the documented behaviour, not a risk being hedged against.

So, today:

| Project | Population | Applications |
| --- | --- | --- |
| `paridyum-hiring` | candidates, and staff acting on applications | `web-paridyum` (Web) |
| `paridyum-operations` | operators only | `console-docket` (Native), and consoles after it |
| one per catalogue product | that product's tenants | that product's interfaces |

A product's project is granted to tenant organizations, per ADR-0047 decision 2. The two
Paridyum projects are not granted to anyone, and if that ever changes it is a decision with an
artifact, not a checkbox.

### 3. The application type follows the repository class, and a console is Native

| Class | Zitadel type | Auth method | Why |
| --- | --- | --- | --- |
| `web` | Web | Authorization Code + PKCE | Server-rendered Next.js (ADR-0046). PKCE even though a secret is possible — see decision 5. |
| `console` | **Native** | Code + PKCE, loopback redirect | **The load-bearing row.** ADR-0055 says a console runs where the operator is, so its binary is on operator machines. A confidential client distributes its secret to every one of them, and a secret held by N people is not a secret. Native has no secret; Zitadel's docs state Native "only supports code authentication type". |
| `mobile` | Native | Code + PKCE | Same argument, same reason. |
| `service`, `platform` | API | Private key JWT | Machine-to-machine. Private key JWT over Basic: a JWT assertion is per-request and short-lived, a shared password is neither. |
| `tool` | **none** | — | A tool binds loopback and resolves an operator locally (`tool-docket` already does). It is not on the internet and it has no OIDC client. Giving one a client is how a `tool-` becomes a service without an ADR. |
| `library`, `sdk`, `spec`, `template`, `infra` | none | — | Nothing to log into. |

### 4. Every resource server validates `iss` and `aud`, and requests only the audience it needs

ADR-0047 decision 5 already requires validating `iss` and `aud` rather than the signature alone.
This decision records **why that is load-bearing rather than belt-and-braces**: audience is
opt-in and client-requested, so a valid, unexpired, correctly-signed token from a different
application is a token this instance issued and would happily verify.

Two rules follow:

- **A resource server rejects a token whose `aud` does not contain its own project id.** Not
  logs-and-continues. Rejects.
- **A client requests `...:{projectid}:aud` only for projects it is contractually entitled to
  call.** Adding another project's audience scope because a call failed is escalation by
  copy-paste, and it succeeds silently.

### 5. PKCE by default; a client secret is a runtime secret and is the exception

Authorization Code with PKCE is the default for every interactive client, including Web
applications that *could* hold a secret.

Where a secret genuinely exists, it is a **runtime secret under REQ-0001** — SOPS-encrypted with
age (ADR-0057), never in a repository in plaintext, never in a Helm value, never in an
environment variable committed to a chart. REQ-0001 moved from `deferred` to `accepted` on
2026-09-07 and criterion 3 — rotate without editing a repository — is the one SOPS does not
meet, so **every client secret created here is a known future migration**, and creating fewer of
them is the cheapest way to make that migration smaller.

### 6. Development Mode is off, and redirect URIs are exact

Zitadel's Development Mode "enables non-HTTPS protocols and glob patterns". Both halves are
disqualifying for anything reachable: `http` on a redirect exposes an authorization code to the
network, and a glob in a redirect allowlist is an open redirect waiting for a subdomain.

Development Mode is permitted **only** on an application whose redirect URIs are all loopback,
and it is never enabled on an application any other person can reach.

### 7. Applications are registry data before the list gets long

Every Zitadel application the estate owns is recorded — project, type, owning repository,
environment, and whether it holds a secret. This file does not create that registry; it states
that the registry is where the list lives, so the list is not discovered by clicking through a
console. This is the same argument `tool-registry.yaml` and `product-registry.yaml` already
won, applied before there are three of these rather than after there are thirty.

## Consequences

**Accepted costs**

- **More applications to create and keep track of**, and the count multiplies by environment.
  Two surfaces × two environments is four clients before a single product exists. That is the
  price of independent revocation and it is paid on the day something must be revoked.
- **A second project for Paridyum's own surfaces looks like over-engineering today**, because
  the same two people are the operators and the reviewers of applications. It stops looking
  like it the first time a candidate holds a token and an operator role exists in the same
  vocabulary.
- **Consoles get no client secret and some libraries make that awkward.** Native + PKCE with a
  loopback redirect is well-trodden but it is not the copy-paste path most OIDC quickstarts
  document, and it will cost time in `console-docket`.
- **Per-application token settings can drift.** Four applications is four places a token
  lifetime can be set differently, and nothing checks that today. Decision 7's registry is
  where that check will eventually attach.

**Gained**

- One compromised or misconfigured client is one client. Rotating it does not log out the
  estate, and revoking it does not take down a surface that had nothing to do with the
  incident.
- The console's binary carries no secret, so distributing it to a new operator is not a
  credential distribution problem.
- The audience boundary is explicit and enforced at the resource server, which is what makes
  ADR-0045's threat T1 defensible rather than assumed.
- A candidate's token cannot carry an operator role, because the role does not exist in the
  vocabulary the token draws from.

**Rejected alternatives**

- **One client for everything Paridyum runs.** Simplest, and it fails the first rotation: the
  only way to revoke access for one surface is to break every surface. It also puts staging
  hostnames in a production redirect allowlist, which is the concrete mechanism by which an
  authorization code leaves the origin that was supposed to receive it.
- **One project called `paridyum` holding every application.** Tempting because there is one
  company. Rejected on the documented behaviour that all applications in a project share the
  same roles — it would define operator roles in the vocabulary a candidate's token is drawn
  from, and the mitigation would be remembering never to grant them.
- **Confidential clients for consoles, with the secret shipped in the binary.** What most
  desktop OIDC integrations do. The secret is extractable from any copy of the binary, so it
  authenticates the software rather than the operator, which is the property a client secret is
  supposed to have and would not have.
- **Trusting the signature and skipping `aud`.** Fast, and defensible only if audience were
  implicit. It is not: it is a scope the client asks for, so signature-only validation accepts
  tokens minted for a different application by the same issuer.
- **Deferring the application registry until there are more applications.** The position this
  estate has now rejected four times — C-0005's classes, ADR-0022's artifact types, ADR-0048's
  externally-cadenced values, ADR-0066's tools. Each time the defect was a list held somewhere
  extending it was expensive.

## Amendment — 2026-09-27

**Decision 3's mapping table gains one row, added by ADR-0111, and the `tool-` row is
untouched.** A distributed agent — a client on machines the estate does not control, acting for
one tenant — is class `agent-`, Zitadel type **Native**, auth **Device Code (device authorization,
RFC 8628)**, with a session revocable per station and no client secret.

This is deliberately **not** a widening of `tool-`. Giving a tool a client is still how a tool
becomes a service without an ADR (decision 3's own sentence), which is exactly why the station
got a class of its own. RFC-0054's `agent-reel` is the first instance; the class registry entry
landed with ADR-0111.

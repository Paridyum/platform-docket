---
id: ADR-0090
title: The Zitadel login UI is deployed once per virtual instance
type: adr
status: accepted
created: 2026-09-13
locked_at: null
supersedes: null
superseded_by: null
scope: [infra, platform]
applies_to: ["*"]
tags: [zitadel, login, identity, control-plane, id.paridyum, rfc-0012, adr-0047, adr-0089, multi-instance, edge, login-client]
breaking: false
enforced_by: []
---

# ADR-0090 — The Zitadel login UI is deployed once per virtual instance

> **Superseded in part, 2026-09-28 — ADR-0113, ADR-0119.** Read *the control region* wherever this ADR uses the
> name GLO-0007 retired. The `paridyum` instance at `id.paridyum.com` is the **staff instance**, and
> holds staff only (ADR-0119 D1). A **portal instance** per region, with its own login deployment on
> this ADR's pattern, is added with the portal (ADR-0117 D10, ADR-0118 D3).

## Status

**Accepted**, 2026-09-13. Refines the implementation of **RFC-0012 D2** (resolved by ADR-0047):
the home region runs a *second* Zitadel virtual instance, `paridyum` at `id.paridyum.com`, for
staff and operators, alongside the shared tenant instance at `auth`/`login.paridyum.com`. RFC-0012
decided the plane split; it predates Zitadel v4's **login v2**, which is a separate Next.js
deployment rather than a screen inside the API process, and so said nothing about how that UI is
deployed across instances. This ADR decides that.

It also records a **correction**: the control-plane-instance work committed in the prior session
cited **ADR-0086** in code, edge inventory and Helm values. ADR-0086 is the *hiring/landing* ADR
and has nothing to do with the identity planes. The governing decision is **RFC-0012 D2**; those
citations are corrected to RFC-0012 D2 / this ADR as part of landing it (STD-0022: a reference
that names the wrong decision is a defect to fix, not to leave).

Not `locked`: mechanically checkable (an instance reachable through the edge whose `/ui/v2/login`
has no dedicated login deployment is the defect) but no check asserts it yet; it graduates when
`check-registry.yaml` carries one.

## Context

RFC-0012 D2 puts the staff/operator plane in its own **virtual instance** (`id.paridyum.com`),
separate from the tenant instance (`auth.paridyum.com`). Zitadel resolves an instance from the
**Host header**, so the API, console and OIDC endpoints of both instances are served by the one
`zitadel` deployment and routed apart at the edge by hostname. The prior session extended that
same assumption to the login UI: `id.paridyum.com/ui/v2/login` was routed to the *same*
`zitadel-login` deployment as `auth`, "routed by Host like everything else."

It does not work, and standing up `id.paridyum.com` proved it: the login page returned a
server-component render error, and the login pod logged `fetch() returned undefined` and
`security settings … 404` for every settings/branding call. The cause is in the login
deployment's own configuration, not the instance:

```
ZITADEL_LOGINCLIENT_KEYFILE="/login-service-key/tls.key"
AUDIENCE="https://auth.paridyum.com"
CUSTOM_REQUEST_HEADERS="Host:auth.paridyum.com
X-Zitadel-Public-Host:auth.paridyum.com"
```

The login v2 app **forces** `Host: auth.paridyum.com` on every call it makes to the API, and
derives its audience and public host from `auth.paridyum.com`. Whatever hostname the browser used,
the login app addresses the tenant instance. One login deployment therefore serves exactly one
instance — the login UI is **not** Host-routable the way the API is.

The credential, crucially, is **not** the obstacle. The chart documents that `login-client` is a
**`SystemAPIUsers`** entry (a system-level identity, like `paridyum-system`) whose RSA keypair the
chart auto-generates — the public cert in the ConfigMap, the private key in the
`zitadel-login-service-key` secret. A system-level identity is **instance-agnostic**: the *same*
login-client credential can act on any instance, with the Host header selecting which. So the gap
is only the **per-deployment host/audience configuration**, not a missing per-instance login user.

## Decision

**1. Each virtual instance that presents the login v2 UI gets its own login deployment.** A login
deployment is bound to one instance by three pieces of configuration — the forced `Host` and
`X-Zitadel-Public-Host`, and its external domain — so serving N instances needs N login
deployments. The tenant instance keeps the chart-managed `zitadel-login`
(`auth.paridyum.com`); the control-plane instance gets a second, dedicated login deployment for
`id.paridyum.com`.

**2. The second login deployment reuses the shared system `login-client` credential.** It mounts
the *same* `zitadel-login-service-key` secret. No new login-client, no per-instance user, no
bootstrap PAT: the credential is a `SystemAPIUsers` identity and already spans instances. What
differs per deployment is only configuration:

| Setting | Tenant login | Control-plane login |
| --- | --- | --- |
| forced `Host` / `X-Zitadel-Public-Host` | `auth.paridyum.com` | `id.paridyum.com` |
| `ZITADEL_EXTERNALDOMAIN` (public URLs) | `auth.paridyum.com` | `id.paridyum.com` |
| `AUDIENCE` (login-client JWT `aud`) | `https://auth.paridyum.com` | `https://auth.paridyum.com` |
| `ZITADEL_API_URL` | `http://zitadel:8080` | `http://zitadel:8080` |
| login-client key | `zitadel-login-service-key` | *same secret* |

`AUDIENCE` stays `https://auth.paridyum.com` for both: it is the audience of the **API server**
the token is sent to (whose external identity is `auth.paridyum.com`), not of the instance whose
data the call operates on — the instance is chosen by the `Host` header. The system login-client's
token validates against the API regardless of which instance the Host resolves.

**3. It is a standalone manifest, not a Helm values change.** Like `zitadel/service.yaml`
(ADR — the exposure lives in a file whose subject is exposure), the second login deployment,
its ConfigMap and its LoadBalancer service are declared in `zitadel/login-control.yaml` and
applied with `make zitadel-login-control`. The chart makes exactly one login deployment per
release; a second release would drag the whole `zitadel` app and its database with it. A plain
manifest, modelled field-for-field on what the chart renders, is what is diffable and owned here.

**4. It gets its own LoadBalancer VIP, and the edge routes `id.paridyum.com/ui/v2/login` to it.**
`10.1.1.105`, pinned in the manifest (the next free address in the `10.1.1.100–.199` pool;
individual assignments are cluster state and are not recorded in `address-registry.yaml`, by that
file's own rule). `infra-edge`'s `edge_proxy_sites` for `id.paridyum.com` repoints its
`/ui/v2/login` route from `.102` (the tenant login) to `.105`.

## Consequences

**Accepted costs.**

- **A login deployment per instance.** Every new plane that needs a hosted login UI is another
  small deployment, ConfigMap, service and edge route. This is inherent to login v2 being a
  configured-per-instance Next.js app; it is not overhead this ADR introduces, only the overhead
  it names. Tenant *organizations* do **not** each get one — they share the tenant instance's
  login (RFC-0012 D2: tenant = org, not instance); this cost is per *instance*, and instances are
  few (one tenant plane per region, one staff plane at home).
- **The two login deployments must stay version-aligned** with the `zitadel` app. The control
  login pins the same `ghcr.io/zitadel/zitadel-login` tag the chart uses; a chart upgrade that
  moves the login image must move this manifest too. Recorded as the coupling to watch.

**Gained.**

- The control plane has a working login UI, so RFC-0012 D2's staff/operator instance is actually
  reachable by a human — the split stops being API-only.
- No bootstrap-credential chicken-and-egg for the login UI: the system login-client already spans
  instances, so the second deployment needs no new secret and no first login to create one.

**Rejected alternatives.**

- **One shared login deployment, Host-routed** — the assumption that failed. The login app
  overrides the Host itself, so it cannot be steered by the incoming hostname.
- **Un-force the Host and let the login app read the incoming host** — would need the login app to
  build every downstream URL from an untrusted header and re-derive audience per request; it is
  not how the app is built, and it turns a header into a security-relevant input.
- **Collapse the control plane into an organization inside the shared instance** — would let one
  login serve it, but reverses RFC-0012 D2's deliberate separate-instance decision for staff
  (separate issuer, separate user store, no staff directory sharing an instance with tenants). Not
  reopened here.

## Related

- **RFC-0012 D2** (resolved by **ADR-0047**) — the plane split this implements: shared tenant
  instance, plus a `paridyum` staff/operator instance at `id.paridyum.com` in the home region.
- **ADR-0089** — Zitadel is the estate SSO; the control-plane instance is where staff/operators
  sign in, which is what makes its login UI load-bearing.
- **ADR-0086** — the hiring/landing ADR the prior session mis-cited for this work; this ADR records
  the correction. ADR-0086 is unrelated to the identity planes.
- `zitadel/service.yaml` — the precedent for declaring Zitadel exposure as a standalone manifest
  rather than a Helm values change.

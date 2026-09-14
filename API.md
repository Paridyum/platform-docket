# API — docket served backend

The `serve` binary (ADR-0093) exposes an HTTP JSON API for authenticated Paridyum staff, backed by
Postgres. This is the everyday interface; the `docket` CLI remains a break-glass admin path against
the same database.

> **Hand-authored, for now.** STD-0002 has API.md generated from an OpenAPI document where one
> exists. docket does not yet publish an OpenAPI spec (`manifest.yaml` declares `exposes.http.port`
> but no `openapi`), so this file is maintained by hand and is the authority until the spec lands.
> Recorded so a reader knows which half of STD-0002 applies. The routes below are the ones
> `src/served.rs` registers today; the surface grows as the Postgres port lands.

## Authentication

Every route except `/healthz` requires a control-plane operator token (ADR-0092), sent as
`Authorization: Bearer <jwt>`. The backend verifies the token against the control-plane JWKS
(fetched at startup from `DOCKET_OIDC_JWKS`, default `{DOCKET_OIDC_ISSUER}/oauth/v2/keys`), checks
its audience against `DOCKET_OIDC_AUDIENCE`, and binds its opaque `sub` to a docket operator from
`registry/operators.yaml`.

- Missing token → `401 Unauthorized`
- Token that fails verification (bad signature, wrong issuer/audience, expired) → `401 Unauthorized`
- Verified subject bound to no operator → `403 Forbidden`

The network is not the trust boundary; the token is. A handler never sees an unauthenticated request.

## Routes

### `GET /healthz`

Liveness. No authentication, touches nothing. Returns `200 OK` with body `ok` while the process is
up and serving. This is the route the Kubernetes liveness and readiness probes hit (see `helm/`).

### `GET /roster`

The whole roster of person records for this instance's tenant (`DOCKET_TENANT_ID`). Requires a valid
operator token. Returns `200 OK` with a JSON array of:

```json
[
  { "slug": "asha-verma", "record": { /* the full person record */ } }
]
```

### `POST /person`

Create or replace a person record. Requires a valid operator token. Request body:

```json
{ "slug": "asha-verma", "record": { /* the person record */ } }
```

Returns the stored record on success. Validation and the business rules that decide what a record may
contain live in the domain layer the CLI also calls (STD-0004, ADR-0055 §6); this layer parses,
authorises and serialises, nothing more.

## Errors

Errors are returned with an appropriate 4xx/5xx status and a plain-text body describing the cause.
`401`/`403` come from the auth layer above; `500` is an unexpected internal fault (for example the
database being unreachable).

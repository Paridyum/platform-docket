# Runbook — docket served backend

Operating the `serve` binary (ADR-0093) in the cluster. For the everyday HR tasks it performs, see
`SOP.md`; this is for keeping the service running.

## What it is

A stateless Rust HTTP service. State lives in Postgres (the person store and the four sealed chains,
ADR-0093) and, for issued PDFs, on a blob path — currently an ephemeral volume while only synthetic
data is served (ADR-0093 §5). The service holds no durable state of its own, so a pod can be killed
and rescheduled freely.

- **Image:** `ghcr.io/paridyum/platform-docket`, deployed by digest (ADR-0060).
- **Chart:** `helm/` — Deployment + Service (ClusterIP), in namespace `platform`.
- **In-cluster address:** `docket.platform.svc.cluster.local:8080`.
- **Liveness/readiness:** `GET /healthz` (unauthenticated; `200 ok`).

## Configuration

Entirely from the environment (`serve.rs`); see `SOP.md` for the full list. The one secret is
`DOCKET_DATABASE_URL`, wired from a k8s Secret named `docket` (key `database_url`) created from SOPS
at deploy — never committed, never typed on a terminal (REQ-0001, RUN-0004).

## Deploy / upgrade

```sh
helm upgrade --install docket ./helm -n platform \
  -f helm/values-staging.yaml --set image.digest=sha256:<from the publish run>
```

The chart refuses to render without `image.digest` (ADR-0060). A rollback is `helm rollback docket`,
or an upgrade pinning the previous digest.

## Health checks and common failures

| Symptom | Likely cause | What to do |
| --- | --- | --- |
| Pod `CrashLoopBackOff` at startup | A required env var is unset — the binary refuses rather than guessing (`DOCKET_OIDC_ISSUER`, `DOCKET_OIDC_AUDIENCE`, `DOCKET_TENANT_ID`, `DOCKET_DATABASE_URL`). | `kubectl logs` names the missing variable. Fix `values-staging.yaml` or the `docket` Secret. |
| Startup fails fetching JWKS | The control-plane issuer is unreachable, or `DOCKET_OIDC_JWKS` is wrong. | Confirm `DOCKET_OIDC_ISSUER` resolves and `{issuer}/oauth/v2/keys` returns JSON. |
| Pod `ImagePullBackOff` | The `ghcr` pull secret is absent in `platform`, or the digest does not exist. | Ensure the `ghcr` Secret exists (`make registry-secret`); confirm the digest was published. |
| Every request `401` | The presented token fails verification (wrong issuer/audience, expired, or JWKS rotated since startup). | Check the token's audience is `DOCKET_OIDC_AUDIENCE`. **Known limit:** JWKS is fetched once at startup and not refreshed — a key rotation needs a pod restart (`kubectl rollout restart deploy/docket`). |
| A known operator gets `403` | Their token subject is not bound to an operator in `registry/operators.yaml` (baked into the image). | Add the operator, rebuild and redeploy the image. |
| `500` on reads/writes | Postgres is unreachable or migrations are not applied. | Confirm the DSN and that platform-tenancy migrations 0001–0006 are applied. |

## Shutdown

The container runs the binary as PID 1, so a `SIGTERM` on a rolling update reaches it directly and it
drains in-flight requests within `terminationGracePeriodSeconds` (30s).

## Known operational debt

- **JWKS is fetched once at startup** — a control-plane key rotation causes `401`s until the pod is
  restarted. A periodic/on-miss refresh is owed (see the served-backend design; ADR-0047 D4 keeps it
  off the request path).
- **Blob storage is ephemeral** — issued PDFs live on an emptyDir. Durable object storage (B2/R2/
  MinIO vs a PVC) is the open ADR-0093 §6 decision, and no real HR record is created until rehearsed
  backups exist (§5).

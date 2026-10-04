# syntax=docker/dockerfile:1
# Multi-stage build for the `serve` binary (ADR-0093 — the docket served backend). Pinned to the
# estate toolchain (1.97.1, toolchain-registry / STD-0003) and Debian trixie, matching the rest of
# the estate (ADR-0006, the PMG host). TLS is rustls, never openssl (deny.toml, ADR-0005): sqlx and
# reqwest are built with rustls features, so the runtime needs only CA certificates — no openssl
# shared library.

FROM rust:1.97.1-slim-trixie AS builder
WORKDIR /build

# Build tooling the crypto crates need: `ring` (via jsonwebtoken) and `aws-lc-rs` (via rustls, used
# by sqlx and reqwest) compile C/asm at build time, so a compiler, cmake and perl must be present.
# Absent, the build fails deep inside a dependency with an error that names neither this image nor
# the crate.
RUN apt-get update \
  && apt-get install -y --no-install-recommends build-essential cmake perl pkg-config git \
  && rm -rf /var/lib/apt/lists/*

# Cargo.lock is committed and the build is --locked: the image is built from the exact graph the
# repository resolved, not whatever is newest at build time (ADR-0053, REQ-0014). sqlx here uses the
# runtime query API (no `macros` feature), so no DATABASE_URL is needed at build time.
COPY Cargo.toml Cargo.lock ./
COPY src ./src
# THE PRIVATE CRATE. `paridyum-auth` is named by an SSH URL, and a build has no SSH key: every image
# build failed to fetch it until 2026-09-28. The BuildKit secret `git_token` (infra-ci image-publish,
# PARIDYUM_PACKAGES_TOKEN) is mounted for THIS RUN ONLY, and git's environment-only config rewrites the
# Paridyum SSH URL to HTTPS with it — no ~/.gitconfig, no ENV, no ARG, so it is in no layer.
# CARGO CACHE MOUNTS (2026-10-04): the registry and this crate's target/ persist in the runner host's
# BuildKit builder (infra-ci image-publish, one per host), so a source change recompiles this crate,
# not every dependency — measured on platform-routing: 37 s and 1 crate, against 316 s and 189 cold.
# The binary is copied OUT in the same step: a cache mount is not part of the image.
RUN --mount=type=secret,id=git_token \
    --mount=type=cache,id=cargo-registry,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=cargo-git,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,id=tool-docket-target,target=/build/target,sharing=locked \
    GIT_CONFIG_COUNT=1 \
    GIT_CONFIG_KEY_0="url.https://x-access-token:$(cat /run/secrets/git_token 2>/dev/null)@github.com/Paridyum/.insteadOf" \
    GIT_CONFIG_VALUE_0="ssh://git@github.com/Paridyum/" \
    CARGO_NET_GIT_FETCH_WITH_CLI=true \
    cargo build --release --locked --bin serve \
  && install -m 0755 target/release/serve /usr/local/bin/serve \
  && strip /usr/local/bin/serve

FROM debian:trixie-slim AS runtime
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/* \
  && useradd -u 10001 -r -s /usr/sbin/nologin docket \
  && mkdir -p /var/lib/docket/store && chown -R 10001 /var/lib/docket
WORKDIR /app
# The served backend reads its repository data — the field/lifecycle/org registry, the document
# templates and the LaTeX class — from DOCKET_ROOT at runtime (config.rs Paths). Baked read-only;
# person records and issued PDFs never live here (they are Postgres and the writable store mount).
COPY registry ./registry
COPY templates ./templates
COPY latex ./latex
COPY --from=builder /usr/local/bin/serve /usr/local/bin/serve
USER 10001
EXPOSE 8080
# No shell form: PID 1 is the binary itself, so SIGTERM reaches it and the graceful shutdown
# actually runs on a rolling update.
ENTRYPOINT ["/usr/local/bin/serve"]

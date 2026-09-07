# A docket node, for the cross-machine restore rehearsal (ADR-0072 D6).
#
# NOT a deployment artifact. docket is an operator CLI on a workstation
# (ADR-0006, the `tabla` family); nothing here declares it a service. This image
# exists so that a restore can be proven across a real machine boundary --
# separate filesystems, separate processes, bytes over a network -- which a test
# using two temporary directories on one host cannot do however carefully it is
# written.
#
# Both stages are pinned to bookworm on purpose: a newer builder links against a
# glibc the runtime does not have, and the binary dies with a version error that
# looks like a docket bug.
#
# ADR-0071 §3, adopted by ADR-0072 D6: "a backup that has never been restored is
# a belief, not a control."

FROM rust:1.97.1-slim-bookworm AS build
WORKDIR /src
# Manifests first, so a source edit does not re-resolve the dependency graph.
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked --bin docket

FROM debian:bookworm-slim
# `age` is the encryption (ADR-0057). `curl` is the transport under test.
# `ca-certificates` because a real off-host store is reached over TLS.
RUN apt-get update \
 && apt-get install -y --no-install-recommends age curl ca-certificates python3 \
 && rm -rf /var/lib/apt/lists/*

# The registry, templates and record shapes a node needs to run at all. The
# store and the people directory are deliberately absent: what a node holds is
# what it was given or what it restored, never what was baked in.
WORKDIR /paridyum
COPY --from=build /src/target/release/docket /usr/local/bin/docket
COPY registry ./registry
COPY templates ./templates
COPY latex ./latex
COPY manifest.yaml ./manifest.yaml
COPY people/_TEMPLATE.yaml people/_TEMPLATE-officer.yaml ./people/

ENV DOCKET_STORE=/paridyum/store
ENTRYPOINT ["/bin/sh", "-c"]

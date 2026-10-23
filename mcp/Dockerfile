# syntax=docker/dockerfile:1.26

# lago-mcp-server on the hardened Wolfi/apko bases published by
# getlago/lago-packages. Replaces rust:1.88-slim-trixie (build) and
# debian:trixie-slim (runtime). See
# getlago/lago-packages/apko/mcp-server-{build,base}.yaml.
#
# BASE PIN: `:<YYYYMMDD>-r<run>@sha256:<index digest>`, an immutable
# lago-packages publish. Docker builds from the digest, so a build is
# reproducible. Renovate (getlago/lago-packages renovate/lago-bases) moves tag
# and digest together when a newer base is published; the base refresh
# rebuilds deployed tags with a newer RUNTIME_IMAGE in between.
ARG BUILD_IMAGE=ghcr.io/getlago/mcp-server-build:20261023-r68@sha256:8a732f1bba119f640a13a76b91d8cfa291324dd04f4727b9dbe74164e71907e1
ARG RUNTIME_IMAGE=ghcr.io/getlago/mcp-server-base:20261023-r68@sha256:2f772e077f34e3a04a0fd40999bc650153d01968313b7e86178d012b1ae202d6

# ---------------------------------------------------------------------------
# Build stage. rust, build-base, pkgconf and the OpenSSL 3.6 headers all come
# from the build base, so the apt-get install of pkg-config/libssl-dev is gone.
#
# OpenSSL is still needed even though this crate asks reqwest for rustls-tls
# with default-features off: lago-client pulls its own reqwest with the
# default native-tls stack, so openssl-sys is still in the dependency graph
# (Cargo.lock: openssl 0.10.80 / openssl-sys 0.9.116 / native-tls 0.2.14).
# ---------------------------------------------------------------------------
FROM ${BUILD_IMAGE} AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

# ---------------------------------------------------------------------------
# Runtime stage — hardened Wolfi, uid 65532, no shell, no package manager.
#
# What the old final stage did with RUN is gone:
#   - apt-get install ca-certificates libssl3  -> ca-certificates-bundle +
#     libssl3 are baked into mcp-server-base
#   - useradd appuser + chown                  -> the base runs as nonroot
#     (uid 65532) and COPY --chown sets ownership directly
#
# ENTRYPOINT (/app/lago-mcp-server) and WORKDIR (/app) also come from the
# base, so CMD only has to carry the transport argument.
# ---------------------------------------------------------------------------
FROM ${RUNTIME_IMAGE}

COPY --from=builder --chown=65532:65532 \
     /app/target/release/lago-mcp-server /app/lago-mcp-server

ENV RUST_LOG=info

# Appended to the base's ENTRYPOINT. `stdio` matches the previous image's
# default; override to `sse` per deployment without rebuilding.
CMD ["stdio"]

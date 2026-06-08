# syntax=docker/dockerfile:1
#
# Multi-stage image for the ags5 self-hosted GitHub Actions runner pool.
#
# Why this exists in the repo (it used to be hand-built in Portainer): an
# off-repo image meant the runner config — including the LABELS — lived only on
# the server, which is exactly how a truncated `ags5-portainer` label once left
# every job queued. Version-controlling the image + the stack (stack.yml) makes
# the runner reproducible and reviewable. See .github/runner/README.md.
#
# What it bakes in, so an EPHEMERAL runner doesn't redo it every job:
#   * sccache + mold + wasm-pack  — prebuilt release binaries (pinned)
#   * a system-wide Rust toolchain (stable + rustfmt + clippy + wasm32 target)
# The two builder stages pull/install everything; the final stage COPYs only the
# artefacts in, so no curl/tar/apt build layers ride along in the shipped image.
#
# Build (from the repo root or this dir):
#   docker build -t ags5-runner:latest .github/runner
# Bump a tool: edit its ARG below and rebuild. The base runner tag is an ARG too.

# Rolling focal tag, NOT a digest: a self-hosted runner must stay recent or
# GitHub eventually refuses it, so we WANT a periodic rebuild to pick up the
# latest runner. Pin to @sha256:… only if you accept rebuilding before each
# runner-version deprecation (see README "Pinning vs currency").
ARG RUNNER_BASE=myoung34/github-runner:ubuntu-focal

# ---------------------------------------------------------------------------
# Stage: fetch — download + extract the prebuilt accelerator binaries.
# A throwaway glibc base (so `mold --version` can be smoke-tested here); none of
# this stage reaches the final image. All three binaries are static/portable
# (musl for sccache+wasm-pack, an old-glibc build for mold) so they run on focal.
# ---------------------------------------------------------------------------
FROM debian:bookworm-slim AS fetch

# Pinned for reproducible builds (latest as of 2026-06-06). Bump + rebuild to update.
ARG SCCACHE_VERSION=0.15.0
ARG MOLD_VERSION=2.41.0
ARG WASM_PACK_VERSION=0.15.0

ARG TARGETARCH
RUN set -eux; \
    [ "${TARGETARCH:-amd64}" = "amd64" ] || { echo "this image targets x86_64 only"; exit 1; }; \
    apt-get update; \
    # libatomic1: mold's prebuilt binary dynamically links libatomic.so.1, which
    # this minimal base lacks — needed only so the mold smoke-test below can run
    # here (the focal final image already ships it, hence the previous runner ran
    # mold fine without it).
    apt-get install -y --no-install-recommends curl ca-certificates tar libatomic1; \
    rm -rf /var/lib/apt/lists/*; \
    mkdir -p /out/bin; \
    # sccache — static musl single binary \
    curl --proto '=https' --tlsv1.2 -fsSL \
      "https://github.com/mozilla/sccache/releases/download/v${SCCACHE_VERSION}/sccache-v${SCCACHE_VERSION}-x86_64-unknown-linux-musl.tar.gz" \
      | tar -xz --strip-components=1 -C /out/bin --wildcards '*/sccache'; \
    # mold — ships bin/mold + libexec/mold/ld (the shim build-accel points gcc at \
    # via -B) + lib/mold/mold-wrapper.so; keep that tree intact under the prefix. \
    curl --proto '=https' --tlsv1.2 -fsSL \
      "https://github.com/rui314/mold/releases/download/v${MOLD_VERSION}/mold-${MOLD_VERSION}-x86_64-linux.tar.gz" \
      | tar -xz --strip-components=1 -C /out; \
    rm -rf /out/share; \
    # wasm-pack — static musl single binary \
    curl --proto '=https' --tlsv1.2 -fsSL \
      "https://github.com/rustwasm/wasm-pack/releases/download/v${WASM_PACK_VERSION}/wasm-pack-v${WASM_PACK_VERSION}-x86_64-unknown-linux-musl.tar.gz" \
      | tar -xz --strip-components=1 -C /out/bin --wildcards '*/wasm-pack'; \
    chmod +x /out/bin/*; \
    # smoke-test the binaries we just placed \
    /out/bin/sccache --version; \
    /out/bin/mold --version; \
    /out/bin/wasm-pack --version

# ---------------------------------------------------------------------------
# Stage: rust — install a system-wide toolchain into relocatable prefixes, so
# the final stage can COPY them to the same paths. rustup fetches PREBUILT
# toolchain binaries (old-glibc), so they run on focal regardless of this base.
# ---------------------------------------------------------------------------
FROM debian:bookworm-slim AS rust

# "stable" channel (not a fixed version): a CI runner wants the current stable,
# and the existing workflows pin `dtolnay/rust-toolchain@stable`, so baking the
# same channel keeps that step a no-op fast-path. Rebuild to advance stable.
ARG RUST_CHANNEL=stable
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends curl ca-certificates; \
    rm -rf /var/lib/apt/lists/*; \
    curl --proto '=https' --tlsv1.2 -fsSL https://sh.rustup.rs \
      | sh -s -- -y --no-modify-path --profile minimal --default-toolchain "${RUST_CHANNEL}"; \
    rustup component add rustfmt clippy; \
    rustup target add wasm32-unknown-unknown; \
    rustc --version; cargo --version; \
    # World-writable so jobs running as any user can use the toolchain AND write \
    # the crate registry under CARGO_HOME (wiped with the container on ephemeral \
    # restart — never shared on NFS, which previously corrupted crate unpacks). \
    chmod -R a+rwX "$RUSTUP_HOME" "$CARGO_HOME"

# ---------------------------------------------------------------------------
# Final: the GitHub Actions runner, with the accelerators + toolchain layered on.
# No package manager / curl / tar invocations here — only COPYs — so nothing
# extra rides along. We deliberately DON'T set USER or HEALTHCHECK: the
# myoung34 entrypoint manages runner registration + the user it runs jobs as,
# and a runner is not a long-lived service to health-probe.
# ---------------------------------------------------------------------------
FROM ${RUNNER_BASE} AS runner

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

# /out is a clean /usr/local-shaped prefix: bin/{sccache,mold,wasm-pack},
# libexec/mold/ld, lib/mold/mold-wrapper.so → lands at the paths build-accel
# expects (mold at /usr/local/bin/mold ⇒ shim at /usr/local/libexec/mold/ld).
COPY --from=fetch /out/ /usr/local/
COPY --from=rust /usr/local/rustup /usr/local/rustup
COPY --from=rust /usr/local/cargo /usr/local/cargo

# Fail the build (not a job months later) if any baked tool is missing/broken.
RUN set -eux; \
    sccache --version; \
    mold --version; \
    wasm-pack --version; \
    rustc --version; \
    cargo --version; \
    rustfmt --version; \
    cargo clippy --version; \
    rustup target list --installed | grep -qx wasm32-unknown-unknown

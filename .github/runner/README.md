# ags5 self-hosted runner image + stack

Version-controlled definition of the GitHub Actions runner pool that carries the
`ags5-portainer` label. Previously this lived only in Portainer; keeping it here
makes the runner reproducible and reviewable — and stops a hand-edit on the
server (e.g. a truncated `LABELS`) from silently stranding every job in *queued*.

- **`Dockerfile`** — multi-stage image: a baked Rust toolchain + sccache + mold +
  wasm-pack on top of `myoung34/github-runner` (Ubuntu 20.04 focal).
- **`stack.yml`** — the Portainer compose (ephemeral pool, NFS sccache mount,
  resource limits). Deploy as-is; set only `RUNNER_PAT` in Portainer.
- **`.dockerignore`** — keeps the build context empty (the image fetches
  everything; it COPYs nothing from the repo).

## What's baked in, and why

Ephemeral runners start every job from a fresh `_work`, so anything *not* in the
image is redone per job. Baking the slow, stable bits makes job startup cheap:

| Baked | Pinned in | Replaces per-job |
|-------|-----------|------------------|
| sccache, mold | `Dockerfile` ARGs | the old hand-fetch in Portainer |
| wasm-pack | `Dockerfile` ARG | `cargo install wasm-pack --locked` (~minutes) |
| rustup + **stable** + rustfmt + clippy + `wasm32-unknown-unknown` | `RUST_CHANNEL` ARG | the `dtolnay/rust-toolchain` download |

`build-accel` (`.github/actions/build-accel`) still turns sccache/mold on per
job — it just finds them already on `PATH` instead of the runner installing them.

## Build

```bash
# from the repo root
docker build -t ags5-runner:latest .github/runner
```

In Portainer: **Images → Build a new image**, repository = this path (or paste
the Dockerfile), image name `ags5-runner:latest`. Rebuild to pick up a newer
runner base or bumped tool versions.

### Bumping versions

Edit the relevant `ARG` in the `Dockerfile` and rebuild:

- `SCCACHE_VERSION`, `MOLD_VERSION`, `WASM_PACK_VERSION` — exact tags (no `v`).
- `RUST_CHANNEL` — `stable` by default; rebuild advances it to the current stable.
- `RUNNER_BASE` — the `myoung34/github-runner` tag.

The final stage smoke-tests every baked tool (`sccache --version`, … ,
`rustup target list --installed | grep wasm32`), so a bad/missing binary fails
the **build**, not a job weeks later.

### Pinning vs currency (the base image)

`RUNNER_BASE` defaults to the rolling `:ubuntu-focal` tag, not a digest. A
self-hosted runner that falls too far behind is eventually **refused by GitHub**,
so you *want* periodic rebuilds to track the runner version. Digest-pin
(`myoung34/github-runner@sha256:…`) only if you'll commit to rebuilding before
each runner-version deprecation. The *tool* binaries are exact-pinned either way.

## Deploy (Portainer)

1. **Stacks → Add stack**, paste `stack.yml`.
2. Add a stack **Environment variable** `RUNNER_PAT` = a fine-grained PAT for
   `niko86/ags5_concept` with **Administration: Read & write** (lets the
   ephemeral runners self-register). It never appears in this file.
3. Deploy. Confirm under **GitHub → Settings → Actions → Runners** that
   `ags5-pool-*` appears **online** with label `self-hosted,linux,x64,ags5-portainer`.

### Host prerequisites

- `/srv/ci-cache/sccache` exists on the VM and is the TrueNAS NFS export
  (`mount | grep ci-cache` to confirm it's actually mounted).
- The image `ags5-runner:latest` is built locally (above).

### replicas ↔ disk

Each replica builds the bundled-DuckDB debug `target/` in its own `_work` on the
**VM disk**. Two concurrent builds have overrun a ~37 GB VM
(`No space left on device`). `stack.yml` ships `replicas: 1` (serial — always
fits); raise it only after confirming the host has room for N concurrent builds.
The sccache cache is shared on NFS (one copy, off the VM), so more replicas do
**not** multiply cache storage — only transient `_work`.

## Follow-up once this image is live on the pool

The workflows already work on any runner (baked or a GitHub-hosted fallback).
After the baked image is deployed, the `cargo install wasm-pack` steps in
`e2e.yml` / `deploy-validator.yml` short-circuit via a `command -v wasm-pack`
guard (added alongside this image), so the bake takes effect with no further
change. The `dtolnay/rust-toolchain` step stays (it's idempotent-fast when the
toolchain is already present, and keeps GitHub-hosted fallbacks working).

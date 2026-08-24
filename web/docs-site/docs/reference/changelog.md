# Changelog

Every release is recorded in **`CHANGELOG.md`**, at the root of the repository. It
is generated from a single source file rather than written by hand, and a CI gate
fails if the two disagree, so it cannot quietly fall behind what shipped:

**→ [CHANGELOG.md](https://github.com/niko86/laterite/blob/main/CHANGELOG.md)**

Pre-1.0 a breaking change takes the minor ([how versions move](support.md#how-versions-move)),
and those are indexed at the top of that file, one line each with the version it
landed in, so "does this upgrade break me" is a short list rather than a read:

**→ [Breaking changes](https://github.com/niko86/laterite/blob/main/CHANGELOG.md#breaking-changes)**

The same notes are published as **[GitHub Releases](https://github.com/niko86/laterite/releases)**,
one per version tag, where each release also links its artifacts: the PyPI wheel,
the `lat` CLI binaries and the npm packages.

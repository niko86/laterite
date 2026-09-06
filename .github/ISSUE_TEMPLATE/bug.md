---
name: Bug report
about: Report an unexpected behaviour or regression
title: "[bug] "
labels: bug
---

<!-- Not sure whether this is a bug in laterite or something in the file
itself? Start a discussion instead — that is the right place for it, and
nobody minds: https://github.com/niko86/laterite/discussions -->

## Which surface

<!-- Delete the ones that don't apply. laterite is one engine behind several
doors, and the door matters for reproducing this. -->

- [ ] Python — the `laterite` wheel
- [ ] Python — `laterite.compat` (the python-ags4 drop-in)
- [ ] Node — `laterite`
- [ ] Browser — `@laterite/ags4-wasm`
- [ ] DuckDB — the `laterite_ags4` extension
- [ ] CLI — `lat`
- [ ] Rust — the `laterite` crate, or an engine crate from crates.io

## What happened

<!-- A short, specific description. -->

## What you expected

<!-- If you're porting from python-ags4 and laterite gave a different answer,
say which one you expected. The known divergences are catalogued at
https://docs.laterite.dev/reference/divergences/ — worth a look first,
in case this one is deliberate. -->

## How to reproduce

<!-- A minimal case in whichever language matches the surface above.

We do NOT need your data. Real AGS4 deliveries are commercially sensitive and
we would rather you never sent one. If the file matters, describe its shape
instead — the group, the heading, the AGS data type, and what the offending
value looks like (its form, not its content) — and we will build a synthetic
fixture from that. -->

```
# minimal repro
```

## Environment

<!-- Fill in whichever line matches your surface; delete the rest. -->

- laterite version: `python -c "import importlib.metadata; print(importlib.metadata.version('laterite'))"`
- engine fingerprint (for validator bugs): `python -c "import laterite; print(laterite.engine_fingerprint())"`
- Node package + version:
- wasm package + version:
- DuckDB extension version:
- `lat --version`:
- Crate + version:
- OS:
- Python only — installed extras (`[compat]`, `[pyarrow]`, `[all]`, none):

## Additional context

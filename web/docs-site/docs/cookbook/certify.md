# Certify a clean file & skip re-validation

Mint an `.ags.idx` certificate from a clean validation, then reopen with it to skip the rule engine.

```python
--8<-- "python/ex08_certify.py"
```

```text
certified
```

`.certify()` needs a prior **clean** [`.validate()`](../learn/validate.md) on the
same handle — a passing verdict is the precondition for issuing a cert. It writes
`<path>.ags.idx` next to the file: a validity certificate (the verdict plus a hash
of the bytes it vouches for) and a byte-offset index of every group.

Reopen with `read(path, index=...)` and the next `.validate()` resolves from the
cert instead of running the numbered rules. You can see it took the fast path on
`.report.resolution`: `"certified"` means the cert matched the file's current
bytes, so the rule engine was skipped entirely. (A normal validate reports
`"exact"` or a fallback edition — see [Validate](../learn/validate.md).)

The cert is **content-bound**: if the file changes by a single byte, the hash no
longer matches and laterite silently falls back to a full validation — a stale
cert never yields a false "clean". So certify is a cache, not a trust override:
fast when the file is untouched, correct when it isn't.

See also: [Certificate lifecycle](../concepts/certificate-lifecycle.md) · [Validate](../learn/validate.md).

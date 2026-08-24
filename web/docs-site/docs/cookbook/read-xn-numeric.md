# Read XN columns as numeric

**Available in:** Python (the `xn=` knob is a frame-materialisation option; on
other surfaces `XN` columns read as text and cast in SQL)

AGS `XN` headings are _numeric-or-text_ on disk; laterite reads them as
`String` by default. Pass `xn="numeric"` to coerce them to `Float64` at the door.

```python
--8<-- "python/ex13_registry_xn.py:code"
```

```text
--8<-- "python/ex13_registry_xn.out"
```

`XN` is the AGS type for a column that _usually_ holds a number but is allowed to
carry a non-numeric token (e.g. a free-text remark or a `<` censored value), so
the safe default is to keep it as text: nothing is lost or silently dropped. The
`xn="numeric"` opt-in says "I want these as real numbers": here `LLPL_PL` (plastic
limit) comes back as `Float64` instead of `String`, so it sorts and averages
without a per-column `.cast()`.

Coercion is whole-column. If an `XN` column in your file carries a genuine
non-numeric token, `xn="numeric"` will surface it (rather than quietly producing
text). Keep the default `String` read for files where that's expected.

The same example also queries the **registry**: `child_groups("LOCA")` and
`inherited_key_names("SAMP")` walk the in-memory AGS group graph. That surface is
covered in its own recipe.

See also: [Explore the registry](./explore-registry.md) · [Born-typed reads](../concepts/born-typed.md)

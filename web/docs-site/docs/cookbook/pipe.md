# Splice your own step with `.pipe()`

**Available in:** Python (a fluent-chain idiom of the Python handle)

**When:** the chain doesn't have a method for the thing you want to do next:
inline a function without breaking out of the flow.

```python
--8<-- "python/ex07_pipe.py:code"
```

```text
--8<-- "python/ex07_pipe.out"
```

`.pipe(fn, *args)` calls `fn(handle, *args)` and returns the result verbatim
(a list, a scalar, another handle, anything), the same contract on both
`Ags4File` and `AgsQuery`: an escape hatch that keeps the chain flowing
instead of forcing a temporary variable.

Because `fn` receives the live handle, anything you can read off the object is
fair game: `ags.pipe(lambda a, n: a.groups[:n], 3)` slices the group list,
`q.pipe(lambda q: q.frame().height)` reaches through a query terminal for a row
count. Return the handle itself and the chain continues:

```python
ags = (
    laterite.read("delivery.ags")
    .pipe(lambda a: a.validate())       # returns the Ags4File → chain flows on
    .query("SELECT * FROM LOCA")
)
```

**Gotcha:** `.pipe` doesn't materialise or copy anything; it just hands `fn` the
same handle you already hold (`ags.pipe(lambda a: a is ags)` is `True`). If `fn`
returns a non-handle value (a count, a list), that value is your chain's new
end: you can't `.query()` off an `int`.

See also: [Chaining](../chaining/index.md).

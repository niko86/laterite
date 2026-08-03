# Splice your own step with `.pipe()`

**Available in:** Python (a fluent-chain idiom of the Python handle)

**When:** the chain doesn't have a method for the thing you want to do next —
inline a function without breaking out of the flow. `.pipe(fn, *args)` passes the
handle as `fn`'s first argument and returns whatever `fn` returns.

```python
--8<-- "python/ex07_pipe.py"
```

```text
--8<-- "python/ex07_pipe.out"
```

`.pipe(fn, *args)` calls `fn(handle, *args)` and returns the result verbatim — so
it's an escape hatch that keeps the chain flowing instead of forcing a temporary
variable. It works on both `Ags4File` and `AgsQuery`, and the contract is the
same on each: the handle goes in first, your extra positional args follow, and
you get back exactly what `fn` returns (a list, a scalar, another handle —
anything).

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

**Gotcha:** `.pipe` doesn't materialise or copy anything — it just hands `fn` the
same handle you already hold (`ags.pipe(lambda a: a is ags)` is `True`). If `fn`
returns a non-handle value (a count, a list), that value is your chain's new end
— you can't `.query()` off an `int`. To stay in the chain, have `fn` return a
handle.

See also: [Chaining](../chaining/index.md).

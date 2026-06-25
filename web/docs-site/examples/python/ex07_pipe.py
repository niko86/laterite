# what this shows: .pipe(fn, *args) splices your own step into the chain — fn receives the handle as its first arg — on both Ags4File and AgsQuery.
import laterite

ags = laterite.read("examples/sample_site.ags")

# On an Ags4File: fn(self, *args) — the file handle is passed first, your extra args follow.
out = ags.pipe(lambda a, n: a.groups[:n], 3)
print("first 3 group codes:", out)

# On an AgsQuery: same contract — the query handle is passed in, you return whatever you like.
height = ags.query("SELECT * FROM LOCA").pipe(lambda q: q.frame().height)
print("LOCA row count via pipe:", height)

# .pipe returns your function's result verbatim, and passes the object as the first argument.
assert out == ["PROJ", "TRAN", "UNIT"]
assert ags.pipe(lambda a: a is ags) is True
assert height == 14
assert ags.query("SELECT * FROM LOCA").pipe(lambda q: q is not None) is True

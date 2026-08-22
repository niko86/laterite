# Feedback

laterite is [in beta](reference/support.md) — a statement about how much real-world
use it has had, not about how much testing it has had. The only way to change the
first one is for people to run it against files we have never seen.

**So: give it a try.** Point it at a delivery you already have and see what happens.

## What we'd most like to hear

**Was it fast?** Speed is the headline claim, and the honest test is your files on
your machine, not our benchmarks on ours. If it wasn't fast, that's the more useful
report of the two.

**What did it flag that python-ags4 didn't — or miss that python-ags4 caught?**
This is the sharpest thing you can send us. laterite's validator implements the
numbered rules independently of python-ags4, so where the two disagree, one of three
things is true: it's a deliberate divergence, it's a bug in laterite, or it's
something worth taking to the AGS Data Format Working Group. We can only tell which
if we hear about it.

Check the [known divergences](reference/divergences.md) first — the deliberate ones
are catalogued there, and a match saves us both a round trip.

**What did you want it to do that it wouldn't?** Beta is also when the shape of the
API is still cheap to change.

## We don't want your data

Real AGS4 deliveries are commercially sensitive, and we would rather you never sent
one. **You do not need to attach a file to file a useful report.**

Describe the *shape* instead:

- the group and heading involved,
- the AGS data type,
- what the offending value looks like — its form, not its content
  (`"2 decimal places where the heading says 3DP"`, not the reading itself),
- the rule code, if the validator gave you one.

That is enough for us to build a synthetic fixture that reproduces the problem, and
that fixture becomes a permanent regression test. If you genuinely can't describe
something without sending the file, say so — we'd rather know that than lose the
report.

## Where to send it

**Not sure whether it's a bug?** → **[Discussions][discussions]**. AGS4 is a format
with real ambiguity in it, and "is this laterite, or is it my file?" is a good
question, not a nuisance one. Start there and we'll work it out; if it turns out to
be a bug, we'll open the issue ourselves.

**Sure it's a bug, or know what you want?** → **[Issues][issues]**. There's a
template, and its first question is which surface you're on — the Python wheel, the
Node package, wasm, the DuckDB extension, the `lat` CLI or an engine crate.

## What to expect back

We aim to read everything **weekly**. That's a goal rather than a promise — laterite
has one maintainer — and it's a promise of a *reply*, not of a fix. A report that
becomes a synthetic fixture and a test has done its job even if the behaviour
doesn't change for a while.

[discussions]: https://github.com/niko86/laterite/discussions
[issues]: https://github.com/niko86/laterite/issues

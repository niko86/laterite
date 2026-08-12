"""The Node docs must not name a member of our own API that does not exist.

`web/docs-site/docs/node/index.md` told every reader to write `report.ok`. There
is no `ok` on `Report` — its verdict getter is `isValid`, and `report.ts`'s own
comment says the distinction is deliberate (`ok` on the internal napi type only
means the source parsed well enough to validate). So `report.ok` is `undefined`,
and the second occurrence was `if (!file.report.ok)`, which made the copy-pasted
repair branch run on a clean file.

What makes this worth a gate rather than a fix: the runtime gate already exists
and already gets this right. `web/docs-site/examples/node/ex02_validate.mjs` uses
`report.isValid` and asserts on it, and `laterite-node/test/docs-examples.test.ts`
executes it. The docs page simply hand-types its snippets instead of including
them from that tree, so the executed examples stayed true while the prose beside
them rotted. This closes the hand-typed half.

**Scope, stated honestly — two things it cannot see.** It resolves the two variable
names the pages actually use, `report` and `file`, against the classes they are
documented to be; a snippet naming its handle something else is not checked. And
it only reads *dotted access* — the same page's prose said "`ok` is the headline
verdict" as a bare backticked identifier, which was equally wrong and is out of
reach here, because `ok` is also an English word and matching it would cost more
false positives than the rule is worth. Mutation-tested: that prose form passes.
The fix for it was manual, and the next one will be too.
"""

from __future__ import annotations

import re
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
TS = REPO / "rust-packages" / "laterite-node" / "ts"
DOCS = REPO / "web" / "docs-site" / "docs" / "node"
#: The npm LANDING PAGE, added after falsification found it covered by neither
#: gate. Its member reads are the same claim the pages make, on the page a reader
#: sees *before* `npm install` — and the runtime gate beside it
#: (`laterite-node/test/docs-snippets.test.ts`) cannot see them: a bare
#: `report.someTypo` evaluates to `undefined` and logs, it does not throw. The
#: executor catches CALLS, this catches READS. That split is why both exist.
NPM_README = REPO / "rust-packages" / "laterite-node" / "README.md"


def _pages() -> list[Path]:
    return [*sorted(DOCS.glob("*.md")), NPM_README]


#: variable name in the docs -> (source file, exported class it is documented as)
HANDLES = {
    "report": ("report.ts", "Report"),
    "file": ("ags4-file.ts", "Ags4File"),
}

#: `ts/` source, not `dist/index.d.ts`: the built declarations are not tracked, so
#: a gate against them would need a node build to run and would skip in CI —
#: which is the same as not having it.
_GETTER = re.compile(r"^\s+get (\w+)", re.M)
_METHOD = re.compile(r"^\s+(?:async )?(\w+)\s*\(", re.M)

#: `_METHOD` also matches `if (`, `for (` and friends inside method bodies. They
#: would only ever make the allowed set larger — i.e. let a bad member through —
#: so they come out.
_KEYWORDS = frozenset(
    {"if", "for", "while", "switch", "catch", "return", "constructor", "super"}
)

_JS_BLOCK = re.compile(r"```(?:js|javascript|ts)\n(.*?)```", re.S)

#: Every adjacent `a.b` pair, INCLUDING overlapping ones — zero-width lookahead so
#: nothing is consumed. A plain `\b(report|file)\.(\w+)` misses the chained form:
#: matching `file.report` in `file.report.ok` eats `report`, so `report.ok` is
#: never seen. That is not a corner case — `if (!file.report.ok)` was the live
#: defect, and the worse of the two, because `undefined` made the branch
#: unconditional. The first version of this gate passed it.
_PAIR = re.compile(r"(?=(\w+)\.(\w+))")


def _members(filename: str, cls: str) -> set[str]:
    src = (TS / filename).read_text(encoding="utf-8")
    m = re.search(rf"export class {cls}\b.*?\n\}}\n", src, re.S)
    assert m, f"no `export class {cls}` in ts/{filename} — the class was renamed"
    body = m.group(0)
    return (set(_GETTER.findall(body)) | set(_METHOD.findall(body))) - _KEYWORDS


def test_every_documented_member_exists() -> None:
    """Falsify by renaming a getter on `Report`, or by writing `report.ok` again.

    Both directions are the live failure: `ok`→`isValid` was a rename on the
    class that the page never followed.
    """
    real = {var: _members(*where) for var, where in HANDLES.items()}
    assert "isValid" in real["report"], (
        "the Report parse found nothing recognisable — ts/report.ts moved"
    )

    bad: list[str] = []
    for page in _pages():
        text = page.read_text(encoding="utf-8")
        rel = page.relative_to(REPO).as_posix()
        # Code blocks, plus inline `report.x` spans — the prose gets it wrong
        # just as easily as the snippet, and did.
        haystacks = _JS_BLOCK.findall(text) + re.findall(r"`([^`\n]+)`", text)
        for chunk in haystacks:
            for m in _PAIR.finditer(chunk):
                var, member = m.group(1), m.group(2)
                if var in real and member not in real[var]:
                    cls = HANDLES[var][1]
                    bad.append(f"{rel}: {var}.{member} — no such member on {cls}")

    assert not bad, (
        "the Node docs name API members that do not exist:\n  "
        + "\n  ".join(sorted(set(bad)))
        + "\n\nThe classes are ts/report.ts and ts/ags4-file.ts."
    )

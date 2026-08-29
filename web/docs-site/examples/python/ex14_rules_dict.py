# /// script
# requires-python = ">=3.12"
# dependencies = ["laterite==0.12.0"]
# ///
"""Docs example — run it with `uv run ex14_rules_dict.py`, from anywhere.

Everything above the `[start:code]` marker is machinery the page does not
show: the PEP 723 header that makes the file self-installing,
and the fixture arm that makes its repo-relative path resolve outside a
checkout.
"""

import urllib.request
from pathlib import Path

_FIXTURE = Path("examples/sample_site.ags")
_RAW = "https://raw.githubusercontent.com/niko86/laterite/main/examples/sample_site.ags"
if not _FIXTURE.exists():
    # Cold only for a reader running this outside the repo: in a checkout (and in
    # CI, cwd = repo root) the file is already there and this arm never executes,
    # so the gates stay offline. Fetching it — rather than rewriting the example
    # to an absolute path — is what keeps the text on the page the text you would
    # actually type.
    _FIXTURE.parent.mkdir(parents=True, exist_ok=True)
    _FIXTURE.write_bytes(urllib.request.urlopen(_RAW, timeout=30).read())

# --8<-- [start:code]
# what this shows: enumerate the validator's numbered rules + report the dictionary edition a file resolves to.
import laterite

# list_rules() returns one rich dict per numbered rule (keys incl. rule/title/severity/fixable/...).
rules = laterite.list_rules()

# dict_for(path) resolves a file to its (version, reason) tuple — e.g. ('4.1.1', 'exact').
ver = laterite.dict_for("examples/sample_site.ags")

print(len(rules))
print(sorted(rules[0])[:6])
print(ver)

assert len(rules) >= 20 and isinstance(rules[0], dict)
assert isinstance(ver, tuple) and ver[0] == "4.1.1"
# --8<-- [end:code]

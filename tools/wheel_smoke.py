#!/usr/bin/env python3
"""Post-install smoke for the PUBLISHED wheel (#554).

Imports `laterite` and runs one real validate. It is meant to run inside an
ISOLATED venv that holds only the installed wheel — no workspace checkout, no
editable `.pth`, no `maturin develop` cdylib in the source tree — i.e. exactly
what `pip install laterite` gives a user. Every other Python gate reaches the
library through an editable install and reaches the CLI's `main()` through the
`__main__` guard, so the shipped wheel + the `[project.scripts] lat` console
script are otherwise never exercised. Invoked by ci.yml's `wheel-smoke` job on
real 3.12 / 3.13 / 3.14 interpreters and by release.yml's pre-publish gate.
"""

from importlib import resources

import laterite

# `compat.data` ships a sample `.ags` as PACKAGE DATA, and package data is the
# one kind of wheel content the source tree cannot vouch for: a maturin
# include/exclude slip leaves every dev-tree test green and ships a wheel whose
# `load_test_data()` raises FileNotFoundError. Checked as a resource rather than
# by importing `laterite.compat.data`, so this stays valid in the BASE install
# (no pandas) that this smoke deliberately runs in.
_fixture = resources.files("laterite").joinpath("compat/data/test_data.ags")
assert _fixture.is_file(), (
    "laterite/compat/data/test_data.ags is missing from the installed wheel — "
    "compat.data.load_test_data() would raise FileNotFoundError for every user."
)

# A minimal, self-contained AGS4 document (CRLF, as the format requires).
_AGS = '"GROUP","LOCA"\r\n"HEADING","LOCA_ID"\r\n"UNIT",""\r\n"TYPE","ID"\r\n"DATA","BH01"\r\n'

report = laterite.validate(text=_AGS)
# The public read/validate surface must resolve from the installed wheel and
# return the documented type — not merely be importable.
assert type(report).__name__ == "Report", (
    f"expected Report, got {type(report).__name__}"
)
print("wheel smoke OK: import + validate ->", type(report).__name__)

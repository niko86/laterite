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

import laterite

# A minimal, self-contained AGS4 document (CRLF, as the format requires).
_AGS = '"GROUP","LOCA"\r\n"HEADING","LOCA_ID"\r\n"UNIT",""\r\n"TYPE","ID"\r\n"DATA","BH01"\r\n'

report = laterite.validate(text=_AGS)
# The public read/validate surface must resolve from the installed wheel and
# return the documented type — not merely be importable.
assert type(report).__name__ == "Report", (
    f"expected Report, got {type(report).__name__}"
)
print("wheel smoke OK: import + validate ->", type(report).__name__)

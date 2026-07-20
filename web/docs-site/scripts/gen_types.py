"""Generate the AGS data-type glossary page (#201) at reference/types.md.

Single-sourced from `catalogue_data.TYPE_GLOSSARY`, so the page and the
content-drift gate (which asserts every dictionary type code is documented here)
share one definition. The group catalogue's heading tables deep-link each `type`
code to the anchored entries this page renders.
"""

from __future__ import annotations

import sys
from pathlib import Path

import mkdocs_gen_files

sys.path.insert(0, str(Path(__file__).resolve().parent))
import catalogue_data as cd

_READS_AS = {
    "string": "`str`",
    "integer": "`int`",
    "decimal": "`float`",
    "datetime": "`datetime`",
    "bool": "`bool`",
}

out: list[str] = [
    "# AGS data types",
    "",
    "Every heading in an AGS4 file declares a **data type** — the short code in "
    "its `TYPE` row (`ID`, `2DP`, `DT`, …). The type fixes how the value is "
    "written, validated (AGS Format Rule 8 enforces numeric precision), and read "
    "back. laterite maps each AGS type to a **canonical** type and reads every "
    "column [born-typed](../concepts/born-typed.md): a `2DP` heading arrives as a "
    "float, a `DT` as a datetime.",
    "",
    "Three families are **parametric** — the leading digit is the precision: "
    "`nDP` is *n* decimal places (`1DP`…`4DP`), `nSF` is *n* significant figures, "
    "`nSCI` is scientific notation with *n* mantissa decimals.",
    "",
    "## Quick reference",
    "",
    "| Code(s) | Reads as | Meaning |",
    "|---|:--|---|",
]

for e in cd.TYPE_GLOSSARY:
    codes = ", ".join(f"`{c}`" for c in e["codes"])
    reads = _READS_AS[e["canonical"]]
    out.append(f"| [{codes}](#{e['key']}) | {reads} | {e['ags']} |")

out += ["", "## Type reference", ""]

for e in cd.TYPE_GLOSSARY:
    codes = ", ".join(f"`{c}`" for c in e["codes"])
    out.append(f"### {e['title']} {{ #{e['key']} }}")
    out.append("")
    out.append(f"- **Codes:** {codes}")
    out.append(
        f"- **Reads as:** {_READS_AS[e['canonical']]} (canonical `{e['canonical']}`)"
    )
    out.append(f"- **AGS standard:** {e['ags']}")
    out.append("")
    out.append(e["detail"])
    out.append("")
    out.append(f"*Example:* {e['example']}")
    out.append("")

out += [
    "---",
    "",
    "The canonical mapping and value parsing live in `laterite.ags_types` "
    "(`canonical_type`, `parse_value`). See "
    "[Born-typed reads](../concepts/born-typed.md) for how types flow into the "
    "polars frames, and the [Group catalogue](groups/index.md) for which headings "
    "use each type.",
]

with mkdocs_gen_files.open("reference/types.md", "w") as fd:
    fd.write("\n".join(out))

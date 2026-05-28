<p align="center">
  <img src="https://raw.githubusercontent.com/niko86/laterite/main/assets/laterite-icon-256.png" alt="laterite" width="200" />
</p>

# laterite

A Rust-backed AGS4 reader, writer and validator for Python.

A faster drop-in for
[`python-ags4`](https://gitlab.com/ags-data-format-wg/ags-python-library)
(1.2.0 parity-pinned, 122/131 tests). Native API returns
[narwhals](https://narwhals-api.readthedocs.io/) frames over Polars,
so you can target polars / pandas / pyarrow without laterite picking
for you.

```bash
pip install laterite                  # base AGS4 (polars + narwhals)
pip install "laterite[compat]"        # + pandas (python-ags4 drop-in)
pip install "laterite[ags5]"          # + experimental .ags5db surface
```

```python
import laterite

result = laterite.validate("delivery.ags")
for rule, findings in result.findings.items():
    print(rule, len(findings))

# python-ags4 drop-in
from laterite import compat as AGS4
tables, _ = AGS4.AGS4_to_dataframe("delivery.ags")

# Or a typed view of the file
from laterite.ags4 import read_typed
proj = read_typed("delivery.ags")
```

The validator engine is clean-room from the AGS4 spec. python-ags4
is LGPL-3.0; the clean-room separation lets laterite ship under MIT.

Requires Python ≥ 3.14.

Full docs, parity catalogue and observations at
<https://github.com/niko86/laterite>.

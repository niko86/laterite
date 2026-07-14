# what this shows: the certify fast-path — a fresh .ags.idx cert lets validate() skip the rule engine.
import shutil
import tempfile
from pathlib import Path

import laterite

with tempfile.TemporaryDirectory() as tmp:
    tmp_path = str(Path(tmp) / "site.ags")
    shutil.copy("examples/sample_site.ags", tmp_path)

    # certify() runs the validation itself and mints <path>.ags.idx for an error-clean file.
    idx = laterite.read(tmp_path).certify()

    # re-reading with the fresh cert lets validate() answer without running the rule engine.
    ags = laterite.read(tmp_path, index=str(idx)).validate(warnings=False)

    print(ags.report.certified, ags.report.resolution)
    # `certified` says the ENGINE was skipped; `resolution` still says which dictionary judged it.
    assert ags.report.certified

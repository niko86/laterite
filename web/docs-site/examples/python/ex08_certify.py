# what this shows: the certify fast-path — a fresh .ags.idx cert lets validate() skip the rule engine.
import shutil
import tempfile
from pathlib import Path

import laterite

with tempfile.TemporaryDirectory() as tmp:
    tmp_path = str(Path(tmp) / "site.ags")
    shutil.copy("examples/sample_site.ags", tmp_path)

    # certify() needs a prior clean validate() on the same handle; it mints <path>.ags.idx.
    idx = laterite.read(tmp_path).validate(warnings=False).certify()

    # re-reading with the fresh cert lets validate() resolve without running the rule engine.
    ags = laterite.read(tmp_path, index=str(idx)).validate(warnings=False)

    print(ags.report.resolution)
    assert ags.report.resolution == "certified"

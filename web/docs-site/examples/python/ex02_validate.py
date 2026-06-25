import laterite

# Validate a file. read(...).validate() returns the Ags4File (so it chains);
# the Report is on the .report property.
ags = laterite.read("examples/sample_site.ags").validate()
r = ags.report
print(f"is_valid={r.is_valid} count={r.count} "
      f"dict_version={r.dict_version!r} resolution={r.resolution!r}")

assert r.is_valid is True
assert r.count == 0
assert r.dict_version == "4.1.1"  # auto-selected from TRAN_AGS

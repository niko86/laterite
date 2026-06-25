# what this shows: materialise a borehole's full record set as a dict of polars frames via .at(..).frames()
import laterite

ags = laterite.read("examples/sample_site.ags")
frames = ags.at("LOCA", ["BH01"]).frames()

# frames is a dict {group_code: polars frame}; pull one out by code (NOT q["SAMP"])
print(sorted(frames))
print(frames["SAMP"].height)

assert isinstance(frames, dict) and "SAMP" in frames and frames["SAMP"].height >= 1

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

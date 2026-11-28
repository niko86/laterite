# Post 0, the opener: LinkedIn

![The code in this post, drawn as an image](code.png)

---

"First, install Python" is where a lot of people stop reading.

You can check an AGS file with one line and nothing installed:

```
uvx --from laterite lat validate your_file.ags
```

`uvx` fetches the tool, runs it, and leaves your machine as it found it. No environment to set up, nothing added to your Python.

The same one-liner shape reads a group out as CSV, applies the safe fixes, merges a folder of files, or diffs two issues of a delivery:

```
uvx --from laterite lat read your_file.ags LOCA --csv
uvx --from laterite lat fix your_file.ags --fix-out fixed.ags
uvx --from laterite lat merge folder/*.AGS --out merged.ags
```

It exits with an error status when it finds a problem, so it can sit in front of a delivery and stop a bad file going out.

If you use it every day, `uv tool install laterite` puts `lat` on your PATH for good.

How it works, and how to run the scripts from this series the same way: [link]

#AGS4 #geotechnics #python

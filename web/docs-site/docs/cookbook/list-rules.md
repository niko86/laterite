# List the validation rules

**Available in:** Python · Node · CLI · [Browser](../surfaces/browser.md)

Enumerate the validator's numbered rules — the same catalogue every surface runs
— to drive a UI legend, a docs page, or a coverage check without hard-coding rule
numbers.

=== "Python"

    ```python
    --8<-- "python/ex14_rules_dict.py:code"
    ```

    ```text
    --8<-- "python/ex14_rules_dict.out"
    ```

    `laterite.list_rules()` returns one rich dict per numbered AGS4 rule — 27 of
    them, covering `1`, `2`, `2a`, `2b`, `3` … through `20`. Each dict carries the
    rule number (`rule`), a human `title`, a prose `checks` description, a
    `severity` (`error`, or `mixed` where the rule flags some findings for
    information only), a `fixable` flag (whether `.fix()` / `lat fix` can
    repair it mechanically), and the `observations` it relates to in the
    [divergence catalogue](../concepts/dictionary-selection.md). It's the
    programmatic mirror of the rule table the CLI prints — drive a docs page, a UI
    legend, or a coverage check off it without hard-coding rule numbers.

    `laterite.dict_for(path)` resolves a file to the dictionary edition the
    validator will judge it against, as a `(version, reason)` tuple — here
    `('4.1.1', 'exact')` because the file's `TRAN_AGS` declares 4.1.1 and that
    edition is bundled. The `reason` tells you *how* the pick was made (an exact
    match, a fallback, …); see [Dictionary selection](../concepts/dictionary-selection.md)
    for the resolution order.

    Filter the list the same way you'd filter any list of dicts — e.g. just the
    mechanically-fixable rules:

    ```python
    fixable = [r["rule"] for r in laterite.list_rules() if r["fixable"]]
    ```

    The `severity` field is `mixed` for rules that emit both errors and
    information-only findings (Rule 1 character-set, Rule 16 ABBR pick-lists, Rule
    18 dictionary references); everything else is a hard `error`.

=== "Node"

    ```js
    --8<-- "node/ex14_rules.mjs"
    ```

    ```text
    --8<-- "node/ex14_rules.out"
    ```

    `listRules()` returns the same 27-entry catalogue as an array of `RuleMeta`
    objects — `rule` / `title` / `checks` / `severity` / `fixable` / `observations`,
    the one-for-one mirror of Python's dicts. It's synchronous and needs no DuckDB
    peer, so a Node service can render a rule legend or a fixable-only filter
    (`rules.filter(r => r.fixable)`) straight off the metadata.

=== "CLI"

    `lat rules` prints the whole catalogue as a table — no file
    needed:

    ```bash
    --8<-- "cli/list_rules.sh:cmd"
    ```

    ```text
    --8<-- "cli/list_rules.out"
    ```

    The **Severity** and **Fix?** columns are exactly the `severity` / `fixable`
    fields the library surfaces expose — `mixed` marks a rule that also emits
    information-only findings, and `yes` marks one `fix` can repair mechanically.

Every surface reads the same rule catalogue from the one engine, so the numbers,
titles, severities, and fixable flags never drift between a Python legend, a Node
UI, the CLI table, and the browser explainer.

See also: [CLI reference](../reference/cli.md) · [Dictionary selection](../concepts/dictionary-selection.md)

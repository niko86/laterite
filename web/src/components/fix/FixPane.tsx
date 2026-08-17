import {
  createEffect,
  createMemo,
  createResource,
  createSignal,
  Show,
  type Component,
} from "solid-js";
import { fileStore } from "../../lib/fileStore";
import { computeFixes, applyFixes, validate } from "../../lib/validatorClient";
import type { Fix, ValidationReport } from "../../lib/validator";
import { severityOf } from "../../lib/validator";
import type { Severity } from "../validate/FilterBar";
import {
  dictVersion,
  encoding,
  setEncoding,
  fixView as view,
  setFixView as setView,
} from "../../lib/settings";
import { goTo } from "../../lib/nav";
import { PillToggle } from "../PillToggle";
import { FixesPanel, fixKey } from "../validate/FixesPanel";
import { FileDiff } from "./FileDiff";

// The Fix tab. Reuses the FixesPanel (per-fix diff + checkboxes + Apply
// selected + Export) and adds a workflow bar: fix-all-safe, fix-until-clean,
// undo, revert-to-original. Drives off fileStore.canonicalBytes — CRLF-correct
// for hand-edited content, so a hand-edited file doesn't surface a bogus
// Rule 2a fix on every line. Honours the SELECTED dictionary edition + encoding
// (so a cp1252 file is fixed in its own encoding, not mis-read as UTF-8);
// apply_fixes re-encodes the output to UTF-8, so we reset the selector after.

// Severity is a FINDING property, not a FIX one — by design the Rust `Fix`
// model omits it so the parity oracle's byte-identical JSON can't regress (see
// wiki design/validator-finding-ux). So to tell whether a fix touches an
// FYI-classified finding (the surprise: Validate hides FYI by default, yet a
// fix for it — e.g. a Rule 1 BOM/extended-char — still showed up and got
// applied here), we map each fix to the severity of the finding it resolves,
// joining on rule + line against a fresh validation report. Most-severe wins on
// a tie, so a fix is treated as FYI only when its finding is unambiguously FYI.
const RANK: Record<Severity, number> = { error: 0, warning: 1, fyi: 2 };
const moreSevere = (a: Severity, b: Severity): Severity =>
  RANK[a] <= RANK[b] ? a : b;
interface SevIndex {
  byRuleLine: Map<string, Severity>;
  byRule: Map<string, Severity>;
}
function buildSevIndex(report: ValidationReport | undefined): SevIndex {
  const byRuleLine = new Map<string, Severity>();
  const byRule = new Map<string, Severity>();
  if (report)
    for (const g of report.findings)
      for (const it of g.items) {
        const s = severityOf(it);
        if (it.line != null) {
          const k = `${g.rule}|${it.line}`;
          const prev = byRuleLine.get(k);
          byRuleLine.set(k, prev ? moreSevere(prev, s) : s);
        }
        const pr = byRule.get(g.rule);
        byRule.set(g.rule, pr ? moreSevere(pr, s) : s);
      }
  return { byRuleLine, byRule };
}
function fixSeverity(idx: SevIndex, f: Fix): Severity {
  const lines = f.line != null ? [f.line] : f.edits.map((e) => e.line);
  for (const ln of lines) {
    const s = idx.byRuleLine.get(`${f.rule}|${ln}`);
    if (s) return s;
  }
  return idx.byRule.get(f.rule) ?? "warning";
}

export const FixPane: Component = () => {
  const [fixes] = createResource(
    () => {
      const b = fileStore.canonicalBytes();
      // Track dict/encoding so changing either (e.g. switch to Windows-1252
      // in Validate) recomputes the fixes here too.
      return b && b.length > 0
        ? { b, dict: dictVersion(), enc: encoding() }
        : null;
    },
    (src) => computeFixes(src.b, src.dict, src.enc),
  );

  // A parallel validation report (FYI included, uncapped) purely to label each
  // fix with the severity of the finding it resolves — see the module comment.
  const [report] = createResource(
    () => {
      const b = fileStore.canonicalBytes();
      return b && b.length > 0
        ? { b, dict: dictVersion(), enc: encoding() }
        : null;
    },
    (src) => validate(src.b, src.dict, true, src.enc, null),
  );

  // EVERY read of the two resources goes through these. A Solid resource
  // THROWS when read after a failure, and the readers here are eager — two
  // memos and the selection-reseed effect — so a rejected op would throw
  // outside any fallback and take the whole update down (#359; the shape the
  // warning box in ags-wiki/design/dec-engine-tiering.md records).
  const fixList = () => (fixes.error ? undefined : fixes());
  const sevReport = () => (report.error ? undefined : report());

  const sevIndex = createMemo(() => buildSevIndex(sevReport()));
  // Named for its argument: this resolves a FIX's severity (by looking up the
  // finding it came from), distinct from `severityOf`, which resolves a
  // finding's own. The two used to share a name, and the shadowing hid which
  // was which at each call site.
  const fixSeverityOf = (f: Fix): Severity => fixSeverity(sevIndex(), f);
  // Whether any safe fix also resolves an FYI advisory tied to the same issue
  // (e.g. the Rule 1 BOM strip clears both the Rule 1 finding AND its
  // "FYI (Related to Rule 1)" sibling). Drives the one-line explainer below so
  // a fix touching FYI is never a surprise — there's no separate FYI-only fix
  // to gate, so this is transparency, not a toggle.
  const touchesFyi = createMemo(() => {
    const r = sevReport();
    if (!r) return false;
    const rulesWithFyi = new Set(
      r.findings
        .filter((g) => g.items.some((it) => severityOf(it) === "fyi"))
        .map((g) => g.rule),
    );
    // The BOM's FYI is filed under a sibling "FYI (Related to Rule 1)" rule, so
    // also treat a Rule 1 fix as FYI-touching when any FYI-rule is present.
    const hasRule1Fyi = [...rulesWithFyi].some((r) => r.includes("Rule 1"));
    return (fixList() ?? []).some(
      (f) =>
        f.risk !== "risky" &&
        (rulesWithFyi.has(f.rule) ||
          (hasRule1Fyi && f.rule.includes("Rule 1"))),
    );
  });

  const text = () => {
    const b = fileStore.canonicalBytes();
    return b ? new TextDecoder(encoding(), { fatal: false }).decode(b) : "";
  };

  // Mojibake guard, mirroring the Validate tab: a UTF-8 decode that yields
  // U+FFFD almost always means a Windows-1252 file mis-read as UTF-8 — those
  // bytes surface as Rule 1 errors that have NO per-character fix (arbitrary
  // non-ASCII isn't safely substitutable). The remedy is the encoding switch,
  // so surface it here too rather than just showing "no fixes".
  const looksMojibake = createMemo(
    () => encoding() === "utf-8" && text().includes("�"),
  );

  // Safe fixes are bulk-applicable; risky ones (e.g. typographic→ASCII,
  // duplicate-heading rename) guess intent and are opt-in only.
  const safeFixes = () => (fixList() ?? []).filter((f) => f.risk !== "risky");

  // A fix is applied iff selected; default the SAFE fixes to checked, risky
  // ones unchecked (reseeds after each apply, since fixes() recomputes).
  const [selected, setSelected] = createSignal<Set<string>>(new Set());
  createEffect(() => setSelected(new Set(safeFixes().map(fixKey))));

  // Each entry remembers the `edited` flag alongside the bytes: a hand-edit is
  // LF-only (edited=true ⇒ canonicalBytes re-inserts CRLF). Restoring the bytes
  // without that flag would leave LF bytes flagged edited=false, so the engine
  // sees raw LF and Rule 2a fires on every line.
  const [undoStack, setUndoStack] = createSignal<
    { bytes: Uint8Array; edited: boolean }[]
  >([]);
  const [busy, setBusy] = createSignal(false);
  // view ("fixes" preview+apply | "diff" audit trail) is persisted/shareable in
  // lib/settings, so a shared #fv= link restores it.
  // Preview each fix in its aligned enclosing GROUP block (the in-context
  // view) vs. the single changed line. On by default — the context view.
  const [aligned, setAligned] = createSignal(true);

  const commit = (next: Uint8Array, prior?: Uint8Array) => {
    const before = prior ?? fileStore.bytes();
    if (before)
      setUndoStack((s) => [
        ...s,
        { bytes: before, edited: fileStore.edited() },
      ]);
    fileStore.setBytes(next);
    fileStore.setEdited(false); // engine output is canonical UTF-8 (CRLF)
    // apply_fixes re-encodes to UTF-8, so the fixed bytes are now UTF-8 — reset
    // the encoding selector or the next validate would mis-decode them.
    setEncoding("utf-8");
    fileStore.setName((n) => (n.endsWith(" (fixed)") ? n : `${n} (fixed)`));
  };

  const toggleFix = (key: string) =>
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });

  const applySelected = async (sel: Fix[]) => {
    const b = fileStore.canonicalBytes();
    if (!b || sel.length === 0) return;
    setBusy(true);
    try {
      commit(await applyFixes(b, encoding(), sel));
    } finally {
      setBusy(false);
    }
  };

  const fixAllSafe = () => applySelected(safeFixes());

  // Apply every SAFE fix, recompute, repeat until clean — or a 10-pass cap
  // (some fixers can surface fresh ones). Risky fixes are never auto-applied
  // here — they require explicit per-fix opt-in via Apply selected.
  const iterateToClean = async () => {
    let cur = fileStore.canonicalBytes();
    if (!cur) return;
    const prior = fileStore.bytes() ?? undefined;
    setBusy(true);
    try {
      // First pass reads the file in its selected encoding; applyFixes returns
      // UTF-8, so every later pass operates on UTF-8 bytes.
      let enc = encoding();
      for (let pass = 0; pass < 10; pass++) {
        const fx = (await computeFixes(cur, dictVersion(), enc)).filter(
          (f) => f.risk !== "risky",
        );
        if (fx.length === 0) break;
        cur = await applyFixes(cur, enc, fx);
        enc = "utf-8";
      }
      commit(cur, prior);
    } finally {
      setBusy(false);
    }
  };

  const undo = () => {
    const stack = undoStack();
    const prev = stack[stack.length - 1];
    if (!prev) return;
    setUndoStack(stack.slice(0, -1));
    fileStore.setBytes(prev.bytes);
    fileStore.setEdited(prev.edited);
  };

  const revertOriginal = () => {
    const orig = fileStore.originalBytes();
    if (!orig) return;
    setUndoStack([]);
    fileStore.setBytes(orig);
    fileStore.setEdited(false);
    fileStore.setName((n) => n.replace(/ \(fixed\)$/, ""));
  };

  const exportFixed = () => {
    const b = fileStore.canonicalBytes();
    if (!b) return;
    const blob = new Blob([b as BlobPart], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    const base = (fileStore.name() || "delivery")
      .replace(/\s*\(.*\)\s*$/, "")
      .trim();
    a.download = (base || "delivery").replace(/\.ags$/i, "") + ".ags";
    a.click();
    URL.revokeObjectURL(url);
  };

  const fixCount = () => safeFixes().length;

  return (
    <Show
      when={fileStore.bytes()}
      fallback={
        <div class="rounded-lg border border-dashed border-line-strong bg-surface p-10 text-center">
          <p class="text-lg font-medium text-fg-soft">Auto-fix</p>
          <p class="mx-auto mt-2 max-w-prose text-sm text-fg-faint">
            Load an AGS4 file in the Validate tab — the safe automatic fixes for
            it appear here, with a before/after diff. Nothing is uploaded.
          </p>
          <button
            type="button"
            class="mt-4 rounded bg-accent/15 px-3 py-1.5 text-sm font-medium text-accent hover:bg-accent/25"
            onClick={() => {
              goTo("validate");
            }}
          >
            Go to Validate to load a file →
          </button>
        </div>
      }
    >
      <div class="flex min-w-0 flex-col gap-3">
        <div class="flex items-center gap-1 text-sm">
          <PillToggle
            label="Fixes"
            active={view() === "fixes"}
            onClick={() => {
              setView("fixes");
            }}
          />
          <PillToggle
            label="Diff"
            active={view() === "diff"}
            onClick={() => {
              setView("diff");
            }}
          />
          {/* Persistent download — independent of view + whether any fixes
              remain (the old Export lived inside FixesPanel and vanished once
              the file was clean, exactly when you'd want to save it). */}
          <button
            type="button"
            class="ml-auto rounded border border-line-strong px-3 py-1.5 font-medium text-fg-soft hover:bg-chip"
            onClick={exportFixed}
          >
            Download .ags
          </button>
        </div>

        <Show when={view() === "fixes"}>
          <div class="flex flex-wrap items-center gap-2 text-sm">
            <button
              type="button"
              class="rounded bg-emerald-600/80 px-3 py-1.5 font-medium text-emerald-50 hover:bg-emerald-600 disabled:cursor-not-allowed disabled:opacity-40"
              disabled={busy() || fixCount() === 0}
              onClick={() => void fixAllSafe()}
            >
              Fix all safe ({fixCount()})
            </button>
            <button
              type="button"
              class="rounded border border-line-strong px-3 py-1.5 text-fg-soft hover:bg-chip disabled:opacity-40"
              disabled={busy() || fixCount() === 0}
              onClick={() => void iterateToClean()}
            >
              Fix until clean
            </button>
            <button
              type="button"
              class="rounded border border-line-strong px-3 py-1.5 text-fg-soft hover:bg-chip disabled:opacity-40"
              disabled={busy() || undoStack().length === 0}
              onClick={undo}
            >
              Undo
            </button>
            <button
              type="button"
              class="rounded border border-line-strong px-3 py-1.5 text-fg-soft hover:bg-chip disabled:opacity-40"
              disabled={busy()}
              onClick={revertOriginal}
            >
              Revert to original
            </button>
            <Show when={busy()}>
              <span class="text-xs text-fg-muted">Applying…</span>
            </Show>
            <label class="ml-auto flex cursor-pointer items-center gap-1.5 text-xs text-fg-muted">
              <input
                type="checkbox"
                checked={aligned()}
                onChange={(e) => setAligned(e.currentTarget.checked)}
              />
              Aligned columns
            </label>
          </div>

          {/* Why a fix can clear something Validate wasn't showing: fixes change
              the FILE, and the Validate severity filter only narrows that LIST —
              it doesn't gate fixing. So a safe fix may also resolve a related
              FYI advisory (e.g. the Rule 1 BOM strip). Each fix card carries a
              severity badge so what it targets is explicit. */}
          <Show when={touchesFyi()}>
            <p class="rounded-lg border border-line bg-surface px-3 py-2 text-xs text-fg-faint">
              Heads-up: a fix changes the file itself, so applying one can also
              clear a related <span class="text-accent">FYI</span> advisory tied
              to the same issue (e.g. stripping a Rule 1 BOM). The Validate
              severity filter only controls what's <em>listed</em> there — it
              doesn't limit what gets fixed. Each fix is badged with the
              severity it addresses.
            </p>
          </Show>

          <Show when={looksMojibake()}>
            <div class="rounded-lg border border-amber-600/50 bg-amber-500/10 p-3 text-sm">
              <p class="text-warn">
                This file has replacement characters (�) — it looks like
                Windows-1252 / Latin-1 read as UTF-8. Those non-ASCII bytes are
                Rule 1 errors with <strong>no per-character fix</strong>; the
                remedy is an encoding switch, not the Fix tab.
              </p>
              <button
                type="button"
                class="mt-2 rounded bg-amber-600/80 px-3 py-1 text-xs font-medium text-amber-50 hover:bg-amber-600"
                onClick={() => {
                  setEncoding("windows-1252");
                }}
              >
                Switch encoding to Windows-1252
              </button>
            </div>
          </Show>

          <FixesPanel
            fixes={() => fixList() ?? []}
            text={text}
            selected={selected}
            onToggle={toggleFix}
            onApply={(sel) => void applySelected(sel)}
            aligned={aligned}
            severityOf={fixSeverityOf}
          />
        </Show>

        <Show when={view() === "diff"}>
          <FileDiff a={fileStore.originalBytes} b={fileStore.canonicalBytes} />
        </Show>
      </div>
    </Show>
  );
};

import {
  createEffect,
  createMemo,
  createResource,
  createSignal,
  Show,
  type Component,
} from "solid-js";
import {
  DEFAULT_MAX_PER_RULE,
  severityOf,
  type ValidationReport,
} from "../../lib/validator";
import { validate as runValidate } from "../../lib/validatorClient";
import { engineFailureMessage } from "../../lib/engineFailure";
import { fileStore } from "../../lib/fileStore";
import {
  dictVersion,
  setDictVersion,
  encoding,
  setEncoding,
  aligned,
  setAligned,
} from "../../lib/settings";
import { InputPane } from "./InputPane";
import { Controls } from "./Controls";
import { SummaryBanner } from "./SummaryBanner";
import { FilterBar, type Severity } from "./FilterBar";
import { FindingsView } from "./FindingsView";
import { DownloadCertificate } from "./DownloadCertificate";
import { DownloadReport } from "./DownloadReport";
import { SampleLoader } from "./SampleLoader";
import { Spinner } from "../Spinner";

/** Groups with more than this many findings default to collapsed (no inner
 *  DOM until expanded); smaller groups default open. */
const COLLAPSE_THRESHOLD = 20;

export const ValidatePane: Component = () => {
  // The loaded file lives in the shared fileStore. `edited` + `canonicalBytes`
  // are shared too, so the Fix tab derives the same CRLF-correct bytes. The
  // Apply-Fixes experience is now its own tab (FixPane) — this pane is purely
  // about validating + browsing findings.
  const {
    bytes,
    name,
    setBytes,
    setName,
    setEdited,
    canonicalBytes,
    loadFile,
  } = fileStore;
  // dictVersion / encoding / aligned are persisted + shareable (lib/settings):
  // your last choice survives a reload, and a shared link restores them. The
  // aligned toggle renders a finding's enclosing GROUP block as space-aligned
  // columns (positional-CSV eyeball).

  // Filter state (threaded into FilterBar; consumed by the filtered memo).
  // A rule/group/severity is SHOWN when present in its selected-set. Rules
  // and groups default to "all selected" (seeded from the report); severity
  // defaults to error+warning ON, fyi OFF.
  const [selectedRules, setSelectedRules] = createSignal<Set<string>>(
    new Set(),
  );
  const [selectedGroups, setSelectedGroups] = createSignal<Set<string>>(
    new Set(),
  );
  const [selectedSeverities, setSelectedSeverities] = createSignal<
    Set<Severity>
  >(new Set<Severity>(["error", "warning"]));
  const [search, setSearch] = createSignal("");

  // Open-state for the findings list (owned here so the FilterBar jump can
  // force a group open). Seeded per-report below.
  const [openRules, setOpenRules] = createSignal<Set<string>>(new Set());

  // Decoded text for the editor + finding source-snippets. Decoded with
  // the *selected* encoding so a cp1252 upload reads correctly; lossy
  // (U+FFFD) like the engine, so what you see matches what Rule 1 saw.
  const text = createMemo(() => {
    const b = bytes();
    if (!b) return "";
    return new TextDecoder(encoding(), { fatal: false }).decode(b);
  });

  // Mojibake guard: a UTF-8 decode inserts U+FFFD (�) for every byte that
  // isn't valid UTF-8. A file full of � is almost always a Windows-1252 /
  // Latin-1 file mis-read as UTF-8 (those bytes then surface as Rule 1
  // "non-ASCII" errors). The � are irrecoverable per-byte, so the fix is to
  // re-decode with the right encoding, not a per-character edit — offer a
  // one-click switch.
  const replacementChars = createMemo(() => {
    if (encoding() !== "utf-8") return 0;
    // indexOf-scan rather than match(/�/g): counting matches in a 20 MB+
    // string shouldn't allocate a match array the size of the hit count.
    const t = text();
    let n = 0;
    for (let i = t.indexOf("�"); i !== -1; i = t.indexOf("�", i + 1)) n++;
    return n;
  });

  // Editing is debounced: `pending` holds the in-flight edit text so the
  // textarea stays responsive while the (potentially multi-second, whole-
  // file) re-validate waits for a ~300ms typing pause. null ⇒ not mid-edit,
  // so the editor shows the decoded bytes.
  const [pending, setPending] = createSignal<string | null>(null);
  const editorText = () => pending() ?? text();

  // The report recomputes whenever the (canonical) bytes or any option
  // changes — in the worker, so a multi-second validation never blocks the
  // UI. FYI is always requested; the severity filter hides it by default.
  const [report] = createResource(
    () => {
      const b = canonicalBytes();
      if (!b || b.length === 0) return null;
      return { b, dict: dictVersion(), enc: encoding() };
    },
    (src) => runValidate(src.b, src.dict, true, src.enc, DEFAULT_MAX_PER_RULE),
  );

  // EVERY read of the report goes through here. A Solid resource THROWS when
  // read after a failure, and the readers below are an effect and two eager
  // memos — they re-run the moment the validate rejects, outside any fallback
  // a <Show> could guard, and the throw took the whole update down with it. So
  // a worker crash froze this pane on the PREVIOUS file's report, with its own
  // "Validator error" branch unreachable below (#359; the shape #363 gave
  // ExplorePane, per the warning box in ags-wiki/design/dec-engine-tiering.md).
  const result = () => (report.error ? undefined : report());

  // Whenever a NEW report arrives, reset the filter/open state to defaults
  // derived from it.
  createEffect(() => {
    const r = result();
    if (!r) return;
    const ruleKeys = r.findings.map((g) => g.rule);
    const groupKeys = new Set<string>();
    for (const g of r.findings)
      for (const f of g.items) groupKeys.add(f.group || "—");
    setSelectedRules(new Set(ruleKeys));
    setSelectedGroups(groupKeys);
    setOpenRules(
      new Set(
        r.findings
          .filter((g) => g.items.length <= COLLAPSE_THRESHOLD)
          .map((g) => g.rule),
      ),
    );
  });

  // Split the source into lines ONCE per file (memoised on the text), so the
  // search filter below doesn't re-split the whole file on every keystroke —
  // an O(file-size) main-thread cost per keypress on a big file.
  const searchLines = createMemo(() => text().split(/\r?\n/));

  // The filtered groups the findings list renders. All filters AND.
  const filteredReport = createMemo<ValidationReport | null>(() => {
    const r = result();
    if (!r) return null;
    const rules = selectedRules();
    const sevs = selectedSeverities();
    const groups = selectedGroups();
    const q = search().trim().toLowerCase();
    // Search the SOURCE LINE + group + rule too, not just desc/heading — many
    // findings (e.g. Rule 1 non-ASCII) carry a generic desc and no heading, so
    // a desc-only search emptied the list the moment a user typed the cell text
    // or character they could see. The split is memoised (searchLines), so a
    // keystroke filters against an already-split file, not a fresh O(n) split.
    const lines = q ? searchLines() : [];
    const findings = r.findings
      .filter((g) => rules.has(g.rule))
      .map((g) => ({
        rule: g.rule,
        total: g.total,
        items: g.items.filter((f) => {
          if (!sevs.has(severityOf(f))) return false;
          if (!groups.has(f.group || "—")) return false;
          if (q) {
            const src = f.line != null ? (lines[f.line - 1] ?? "") : "";
            const hay =
              `${f.desc} ${f.heading ?? ""} ${f.group} ${g.rule} ${src}`.toLowerCase();
            if (!hay.includes(q)) return false;
          }
          return true;
        }),
      }))
      .filter((g) => g.items.length > 0);
    return { ...r, findings };
  });

  const shownCount = createMemo(
    () =>
      filteredReport()?.findings.reduce((n, g) => n + g.items.length, 0) ?? 0,
  );
  const totalCount = createMemo(
    () => result()?.findings.reduce((n, g) => n + g.items.length, 0) ?? 0,
  );

  // FindingsView virtualizes its rows, so an off-screen rule header has no
  // DOM node to scrollIntoView — it registers an index-based scroll here
  // that the FilterBar jump calls after forcing the group open.
  let scrollToRule: ((rule: string) => void) | undefined;
  const jumpToRule = (rule: string) => {
    setOpenRules((prev) => new Set(prev).add(rule));
    queueMicrotask(() => scrollToRule?.(rule));
  };

  // Debounce timer for hand-edits (declared here so the load path can cancel
  // a pending commit).
  let editTimer: ReturnType<typeof setTimeout> | undefined;

  const loadBytes = (b: Uint8Array, n: string) => {
    clearTimeout(editTimer);
    setPending(null);
    loadFile(b, n);
  };
  const onEditText = (s: string) => {
    // Show the typed text immediately (responsive), but debounce the
    // expensive whole-file re-validate to a ~300ms typing pause. On commit
    // the bytes are LF (the textarea stripped every \r); canonicalBytes
    // re-inserts CRLF for the engine. Editing is UTF-8.
    setPending(s);
    clearTimeout(editTimer);
    editTimer = setTimeout(() => {
      setBytes(new TextEncoder().encode(s));
      setEncoding("utf-8");
      setEdited(true);
      setName("(edited)");
      setPending(null);
    }, 300);
  };

  return (
    <div class="grid gap-6 lg:grid-cols-[minmax(0,1fr)_minmax(0,1.2fr)]">
      {/* Left: input + controls */}
      <section class="flex min-w-0 flex-col gap-4">
        <Controls
          dictVersion={dictVersion()}
          onDictVersion={setDictVersion}
          encoding={encoding()}
          onEncoding={setEncoding}
          aligned={aligned()}
          onAligned={setAligned}
        />
        <Show when={replacementChars() > 0}>
          <div class="rounded-lg border border-warn/45 bg-warn-quiet px-3 py-2 text-sm">
            <p class="text-warn">
              {replacementChars()} replacement character
              {replacementChars() === 1 ? "" : "s"} (�) — this file looks like
              Windows-1252 / Latin-1, not UTF-8, so those bytes show as Rule 1
              errors.
            </p>
            <button
              type="button"
              class="mt-1.5 rounded border border-line-strong px-2 py-1 text-xs font-medium text-fg-soft hover:bg-chip"
              onClick={() => {
                setEncoding("windows-1252");
              }}
            >
              Switch encoding to Windows-1252
            </button>
          </div>
        </Show>
        <SampleLoader onLoad={loadBytes} open={!editorText()} />
        <InputPane
          text={editorText}
          name={name()}
          onText={onEditText}
          onBytes={loadBytes}
        />
      </section>

      {/* Right: results */}
      <section class="flex min-w-0 flex-col gap-4">
        <Show
          when={result()}
          fallback={
            <div class="rounded-lg border border-line bg-surface p-6 text-sm text-fg-muted">
              {/* Guard-first, so the error branch is REACHABLE: `result()`
                  never throws, and a mid-edit retry (loading, error still
                  set) reads as validating, not as the stale failure. */}
              <Show
                when={!report.loading}
                fallback={<Spinner label="Validating…" />}
              >
                <Show
                  when={!report.error}
                  fallback={
                    <span class="text-err">
                      {engineFailureMessage(report.error, "The validator")}
                    </span>
                  }
                >
                  Load a file, paste AGS4 text, or pick a sample to validate.
                </Show>
              </Show>
            </div>
          }
        >
          {(r) => (
            <>
              <SummaryBanner report={r()} name={name()} />

              <Show when={r().ok}>
                <DownloadCertificate
                  bytes={canonicalBytes}
                  name={name()}
                  dict={dictVersion()}
                  encoding={encoding()}
                />
              </Show>

              <Show when={r().findings.length > 0}>
                <DownloadReport
                  report={r()}
                  bytes={canonicalBytes}
                  name={name()}
                  dict={dictVersion()}
                  encoding={encoding()}
                  includeFyi={true}
                />
                <FilterBar
                  report={r()}
                  selectedRules={selectedRules}
                  onSelectedRules={setSelectedRules}
                  selectedSeverities={selectedSeverities}
                  onSelectedSeverities={setSelectedSeverities}
                  selectedGroups={selectedGroups}
                  onSelectedGroups={setSelectedGroups}
                  search={search}
                  onSearch={setSearch}
                  shownCount={shownCount}
                  totalCount={totalCount}
                  onJump={jumpToRule}
                />
              </Show>
              <Show when={filteredReport()}>
                {(fr) => (
                  <FindingsView
                    report={fr()}
                    text={text}
                    aligned={aligned}
                    isOpen={(rule) => openRules().has(rule)}
                    onToggle={(rule) =>
                      setOpenRules((prev) => {
                        const next = new Set(prev);
                        if (next.has(rule)) next.delete(rule);
                        else next.add(rule);
                        return next;
                      })
                    }
                    onExpandAll={() =>
                      setOpenRules(new Set(r().findings.map((g) => g.rule)))
                    }
                    onCollapseAll={() => setOpenRules(new Set())}
                    registerJump={(fn) => (scrollToRule = fn)}
                  />
                )}
              </Show>
            </>
          )}
        </Show>
      </section>
    </div>
  );
};

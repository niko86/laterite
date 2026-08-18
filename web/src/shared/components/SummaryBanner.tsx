import { Show, type Component, type JSX } from "solid-js";

// The verdict banner at the top of a result pane: tinted panel, coloured
// headline with a glyph, then neutral supporting lines.
//
// Shared as of #406. It lived inside the validate pane, where it had grown the
// report's own vocabulary — severity counts, dictionary resolution, the capped-
// findings caveat — baked into the markup. The fix and tools panes want the
// same object with none of that, so what moved here is the SHAPE (kind,
// headline, detail, note) and what stayed behind is the report reading. A pane
// decides what the verdict is; this decides what a verdict looks like.
//
// The glyphs are the system's sanctioned Unicode verdict set, not icons and
// never emoji: ✓ ✗ ⓘ ! carry the meaning at a glance and survive a screenshot,
// a paste into an issue and a greyscale print. ⓘ belongs to the info tier
// (#404) — warn borrowed it while info didn't exist, and now says ! instead.

export type BannerKind = "ok" | "err" | "warn" | "info";

const KINDS: Record<BannerKind, { glyph: string; class: string }> = {
  ok: { glyph: "✓", class: "border-ok/45 bg-ok-quiet text-ok" },
  err: { glyph: "✗", class: "border-err/45 bg-err-quiet text-err" },
  warn: { glyph: "!", class: "border-warn/45 bg-warn-quiet text-warn" },
  info: { glyph: "ⓘ", class: "border-info/45 bg-info-quiet text-info" },
};

export const SummaryBanner: Component<{
  kind?: BannerKind;
  headline: JSX.Element;
  /** The supporting line — what was checked, and against what. */
  detail?: JSX.Element;
  /** The caveat, quieter again — a cap, a filter, a thing not done. */
  note?: JSX.Element;
  class?: string;
}> = (props) => {
  const kind = () => KINDS[props.kind ?? "ok"];
  return (
    <div
      class={`rounded-xl border p-4 ${kind().class} ${props.class ?? ""}`}
      role={props.kind === "err" ? "alert" : "status"}
    >
      <p class="m-0 text-body font-semibold">
        <span aria-hidden="true">{kind().glyph}</span> {props.headline}
      </p>
      <Show when={props.detail}>
        <p class="mt-1 mb-0 text-caption text-fg-soft">{props.detail}</p>
      </Show>
      <Show when={props.note}>
        <p class="mt-[0.4rem] mb-0 text-micro text-fg-dim">{props.note}</p>
      </Show>
    </div>
  );
};

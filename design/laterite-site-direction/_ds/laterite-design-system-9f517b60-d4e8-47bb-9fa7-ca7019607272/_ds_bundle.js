/* @ds-bundle: {"format":4,"namespace":"LateriteDesignSystem_9f517b","components":[{"name":"Button","sourcePath":"components/core/Button.jsx"},{"name":"Chevron","sourcePath":"components/core/Chevron.jsx"},{"name":"Chip","sourcePath":"components/core/Chip.jsx"},{"name":"CountBubble","sourcePath":"components/core/CountBubble.jsx"},{"name":"Icon","sourcePath":"components/core/Icon.jsx"},{"name":"Spinner","sourcePath":"components/core/Spinner.jsx"},{"name":"StatusBadge","sourcePath":"components/core/StatusBadge.jsx"},{"name":"SummaryBanner","sourcePath":"components/feedback/SummaryBanner.jsx"},{"name":"Toast","sourcePath":"components/feedback/Toast.jsx"},{"name":"Tooltip","sourcePath":"components/feedback/Tooltip.jsx"},{"name":"Checkbox","sourcePath":"components/forms/Checkbox.jsx"},{"name":"ControlGrid","sourcePath":"components/forms/ControlGrid.jsx"},{"name":"Field","sourcePath":"components/forms/Field.jsx"},{"name":"Input","sourcePath":"components/forms/Input.jsx"},{"name":"Select","sourcePath":"components/forms/Select.jsx"},{"name":"PillToggle","sourcePath":"components/navigation/PillToggle.jsx"},{"name":"Tabs","sourcePath":"components/navigation/Tabs.jsx"},{"name":"ThemeToggle","sourcePath":"components/navigation/ThemeToggle.jsx"},{"name":"Admonition","sourcePath":"components/surfaces/Admonition.jsx"},{"name":"Card","sourcePath":"components/surfaces/Card.jsx"},{"name":"CodeTabs","sourcePath":"components/surfaces/CodeTabs.jsx"},{"name":"Dialog","sourcePath":"components/surfaces/Dialog.jsx"},{"name":"Disclosure","sourcePath":"components/surfaces/Disclosure.jsx"}],"sourceHashes":{"components/core/Button.jsx":"b7912f041bad","components/core/Chevron.jsx":"e152ca21c91c","components/core/Chip.jsx":"5c55ec8b89bc","components/core/CountBubble.jsx":"2be83f0fd294","components/core/Icon.jsx":"a4bc1130ac90","components/core/Spinner.jsx":"51cd24cfda24","components/core/StatusBadge.jsx":"93b119187e61","components/feedback/SummaryBanner.jsx":"838f6c2b2831","components/feedback/Toast.jsx":"e4b67ab7b3f6","components/feedback/Tooltip.jsx":"c80ae4e24e47","components/forms/Checkbox.jsx":"ca2354920f6a","components/forms/ControlGrid.jsx":"569404dbe7b7","components/forms/Field.jsx":"bc018fbd237f","components/forms/Input.jsx":"542a32299840","components/forms/Select.jsx":"c34b0f4c3459","components/navigation/PillToggle.jsx":"acb2c8a71096","components/navigation/Tabs.jsx":"9b987dd92e6f","components/navigation/ThemeToggle.jsx":"95f3c4b4af24","components/surfaces/Admonition.jsx":"5de314f180b8","components/surfaces/Card.jsx":"5eb2484a5cea","components/surfaces/CodeTabs.jsx":"816698ec6442","components/surfaces/Dialog.jsx":"d2df0d4465fe","components/surfaces/Disclosure.jsx":"46ab0265da49","ui_kits/demo-site/DemoSite.jsx":"ffacbdc9009a","ui_kits/demo-site/GroupTable.jsx":"f77b5fc2dda8","ui_kits/demo-site/agsModel.jsx":"67356dd3eace","ui_kits/docs/Chrome.jsx":"dbd60acf352e","ui_kits/docs/Pages.jsx":"12042a3f8b0f","ui_kits/docs/Site.jsx":"b67cef966501","ui_kits/webapp/App.jsx":"10a421e3790c","ui_kits/webapp/ExploreScreen.jsx":"dfcd7637b4dd","ui_kits/webapp/FixScreen.jsx":"4010cf717b81","ui_kits/webapp/ToolsScreen.jsx":"4d26df5368a5","ui_kits/webapp/ValidateScreen.jsx":"445e88248927","ui_kits/webapp/data.jsx":"cb8c78ddab75"},"inlinedExternals":[],"unexposedExports":[{"name":"controlStyle","sourcePath":"components/forms/Input.jsx"}]} */

(() => {

const __ds_ns = (window.LateriteDesignSystem_9f517b = window.LateriteDesignSystem_9f517b || {});

const __ds_scope = {};

(__ds_ns.__errors = __ds_ns.__errors || []);

// components/core/Button.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
const BASE = {
  font: "inherit",
  fontFamily: "var(--font-ui)",
  fontSize: "var(--text-control)",
  lineHeight: "var(--leading-normal)",
  display: "inline-flex",
  alignItems: "center",
  gap: "var(--space-2)",
  cursor: "pointer",
  transition: "background-color var(--dur-base) var(--ease-out), color var(--dur-base) var(--ease-out), border-color var(--dur-base) var(--ease-out)"
};
const VARIANTS = {
  // Toolbar text button — the app's .btn
  default: {
    border: "1px solid var(--line)",
    background: "var(--surface)",
    color: "var(--fg)",
    borderRadius: "var(--radius-md)",
    padding: "0.3rem 0.8rem"
  },
  // Filled commit ("Use equipment") — the app's .primary
  primary: {
    border: "1px solid var(--accent)",
    background: "var(--accent)",
    color: "var(--fg-on-accent)",
    borderRadius: "var(--radius-md)",
    padding: "0.3rem 0.8rem",
    fontWeight: "var(--weight-semibold)"
  },
  // "Runs something" — tinted accent wash, accent text
  action: {
    border: "1px solid var(--accent)",
    background: "var(--accent-quiet)",
    color: "var(--accent)",
    borderRadius: "var(--radius-md)",
    padding: "0.26rem 0.9rem",
    fontWeight: "var(--weight-semibold)"
  },
  // Dashed "+ thing" affordance
  add: {
    border: "1px dashed var(--line-strong)",
    background: "var(--surface)",
    color: "var(--accent)",
    borderRadius: "var(--radius-xs)",
    padding: "0.15rem 0.5rem"
  },
  // Quiet icon/✕ button, muted until hover
  ghost: {
    border: "none",
    background: "none",
    color: "var(--fg-muted)",
    borderRadius: "var(--radius-xs)",
    padding: "0.1rem 0.3rem"
  }
};
const SIZES = {
  sm: {
    fontSize: "var(--text-micro)",
    padding: "0.2rem 0.55rem"
  },
  md: {},
  lg: {
    fontSize: "var(--text-body)",
    padding: "0.4rem 1rem"
  }
};

/** The brand's button families. One component, five variants — the app shipped
 *  these as five copies; this is the extraction. */
function Button({
  variant = "default",
  size = "md",
  tone = "neutral",
  disabled,
  iconLeft,
  iconRight,
  style,
  children,
  ...rest
}) {
  const danger = tone === "danger";
  const s = {
    ...BASE,
    ...VARIANTS[variant],
    ...SIZES[size],
    ...(danger ? {
      color: "var(--err)",
      borderColor: variant === "ghost" ? "transparent" : "var(--err)"
    } : null),
    ...(variant === "primary" && danger ? {
      background: "var(--err)",
      color: "#fff"
    } : null),
    ...(disabled ? {
      opacity: 0.45,
      cursor: "default"
    } : null),
    ...style
  };
  return /*#__PURE__*/React.createElement("button", _extends({
    type: "button",
    disabled: disabled,
    style: s
  }, rest), iconLeft, children, iconRight);
}
Object.assign(__ds_scope, { Button });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/core/Button.jsx", error: String((e && e.message) || e) }); }

// components/core/Chevron.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
/** The single disclosure arrow — ~14px, rotates 90° when open. Never a text
 *  glyph, never the native <details> marker. */
function Chevron({
  open = false,
  size = 14,
  style,
  ...rest
}) {
  return /*#__PURE__*/React.createElement("svg", _extends({
    viewBox: "0 0 16 16",
    width: size,
    height: size,
    "aria-hidden": "true",
    style: {
      flexShrink: 0,
      color: "var(--fg-muted)",
      transform: open ? "rotate(90deg)" : "none",
      transition: "transform var(--dur-base) var(--ease-out)",
      ...style
    }
  }, rest), /*#__PURE__*/React.createElement("path", {
    d: "M6 4l4 4-4 4",
    fill: "none",
    stroke: "currentColor",
    strokeWidth: "1.75",
    strokeLinecap: "round",
    strokeLinejoin: "round"
  }));
}
Object.assign(__ds_scope, { Chevron });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/core/Chevron.jsx", error: String((e && e.message) || e) }); }

// components/core/Chip.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
const TONES = {
  neutral: {
    color: "var(--fg-soft)",
    quiet: "var(--surface-raised)",
    rule: "var(--line-strong)"
  },
  accent: {
    color: "var(--accent)",
    quiet: "var(--accent-quiet)",
    rule: "var(--accent)"
  },
  ok: {
    color: "var(--ok)",
    quiet: "var(--ok-quiet)",
    rule: "var(--ok)"
  },
  warn: {
    color: "var(--warn)",
    quiet: "var(--warn-quiet)",
    rule: "var(--warn)"
  },
  err: {
    color: "var(--err)",
    quiet: "var(--err-quiet)",
    rule: "var(--err)"
  },
  info: {
    color: "var(--info)",
    quiet: "var(--info-quiet)",
    rule: "var(--info)"
  },
  muted: {
    color: "var(--fg-muted)",
    quiet: "var(--surface-raised)",
    rule: "var(--line-strong)"
  }
};
const BASE = {
  display: "inline-flex",
  alignItems: "center",
  gap: "0.3rem",
  whiteSpace: "nowrap",
  fontFamily: "var(--font-mono)",
  fontSize: "var(--text-micro)",
  fontWeight: "var(--weight-semibold)",
  letterSpacing: "var(--tracking-micro)",
  textTransform: "uppercase",
  borderRadius: "var(--radius-xs)"
};

/** Small label — verdicts, filters, group codes, counts.
 *
 *  Three forms, because severity must survive greyscale: `rule` (default) is a
 *  neutral-tinted block with a 3px coloured left edge, like a stratum tick;
 *  `solid` is a filled block for the loudest state; `outline` is a hairline
 *  stencil for calm/verified states. Never a soft pastel pill. */
function Chip({
  tone = "neutral",
  variant = "rule",
  sentence,
  style,
  children,
  ...rest
}) {
  const t = TONES[tone];
  const forms = {
    rule: {
      background: t.quiet,
      color: t.color,
      padding: "0.14rem 0.45rem 0.14rem 0.4rem",
      borderLeft: `3px solid ${t.rule}`
    },
    solid: {
      background: tone === "neutral" || tone === "muted" ? "var(--fg-soft)" : t.rule,
      color: tone === "neutral" || tone === "muted" ? "var(--surface)" : "#fff",
      padding: "0.14rem 0.5rem"
    },
    outline: {
      background: "transparent",
      color: t.color,
      padding: "0.12rem 0.45rem",
      border: `1px solid color-mix(in srgb, ${t.rule} 55%, transparent)`
    }
  };
  return /*#__PURE__*/React.createElement("span", _extends({
    style: {
      ...BASE,
      ...(sentence ? {
        fontFamily: "var(--font-ui)",
        textTransform: "none",
        letterSpacing: 0
      } : null),
      ...forms[variant],
      ...style
    }
  }, rest), children);
}
Object.assign(__ds_scope, { Chip });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/core/Chip.jsx", error: String((e && e.message) || e) }); }

// components/core/CountBubble.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
/** The only place 0.65rem type is allowed: a count on a toolbar button or panel
 *  header. Square-ish and mono — an instrument readout, not a notification dot. */
function CountBubble({
  tone = "warn",
  style,
  children,
  ...rest
}) {
  const bg = {
    warn: "var(--warn)",
    accent: "var(--accent)",
    err: "var(--err)",
    info: "var(--info)",
    muted: "var(--fg-dim)"
  }[tone];
  return /*#__PURE__*/React.createElement("span", _extends({
    style: {
      display: "inline-grid",
      placeItems: "center",
      minWidth: "1.15rem",
      height: "1.1rem",
      padding: "0 0.22rem",
      borderRadius: "var(--radius-xs)",
      background: bg,
      color: "#fff",
      fontFamily: "var(--font-mono)",
      fontSize: "var(--text-bubble)",
      fontWeight: "var(--weight-bold)",
      ...style
    }
  }, rest), children);
}
Object.assign(__ds_scope, { CountBubble });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/core/CountBubble.jsx", error: String((e && e.message) || e) }); }

// components/core/Icon.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
const CDN = "https://unpkg.com/lucide-static@0.544.0/icons/";
const cache = {};

/** Lucide glyph, inlined so it inherits currentColor and survives DOM-snapshot
 *  rendering. The file is fetched once per name and cached; until it arrives the
 *  box is empty but sized, so layout never shifts and a failed fetch degrades to
 *  nothing rather than a filled square. The app bundles `@lucide/svelte` per
 *  icon; this is that set for browser-only surfaces. Names are lucide
 *  kebab-case: "shield-check". */
function Icon({
  name,
  size = 16,
  style,
  ...rest
}) {
  const [svg, setSvg] = React.useState(cache[name]);
  React.useEffect(() => {
    if (cache[name]) {
      setSvg(cache[name]);
      return;
    }
    let live = true;
    fetch(CDN + name + ".svg").then(r => r.ok ? r.text() : null).then(t => {
      // lucide-static ships a licence comment ahead of the root element.
      const at = t ? t.indexOf("<svg") : -1;
      if (at < 0) return;
      cache[name] = t.slice(at);
      if (live) setSvg(cache[name]);
    }).catch(() => {});
    return () => {
      live = false;
    };
  }, [name]);
  const box = {
    display: "inline-block",
    width: size,
    height: size,
    flexShrink: 0,
    lineHeight: 0,
    ...style
  };
  if (!svg) return /*#__PURE__*/React.createElement("span", _extends({
    "aria-hidden": "true",
    "data-icon": name,
    "data-icon-state": "pending",
    style: box
  }, rest));
  const sized = svg.replace(/width="[^"]*"/, 'width="' + size + '"').replace(/height="[^"]*"/, 'height="' + size + '"');
  return /*#__PURE__*/React.createElement("span", _extends({
    "aria-hidden": "true",
    "data-icon": name,
    style: box,
    dangerouslySetInnerHTML: {
      __html: sized
    }
  }, rest));
}
Object.assign(__ds_scope, { Icon });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/core/Icon.jsx", error: String((e && e.message) || e) }); }

// components/core/Spinner.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
/** "Working, not hung" — the antidote to a multi-second wasm compile looking
 *  like a frozen tab. Announces politely to AT. */
function Spinner({
  label,
  size = 16,
  style,
  ...rest
}) {
  return /*#__PURE__*/React.createElement("span", _extends({
    role: "status",
    "aria-live": "polite",
    style: {
      display: "inline-flex",
      alignItems: "center",
      gap: "var(--space-2)",
      color: "var(--fg-muted)",
      ...style
    }
  }, rest), /*#__PURE__*/React.createElement("style", null, "@keyframes lat-spin{to{transform:rotate(360deg)}}"), /*#__PURE__*/React.createElement("svg", {
    width: size,
    height: size,
    viewBox: "0 0 24 24",
    fill: "none",
    "aria-hidden": "true",
    style: {
      animation: "lat-spin 0.9s linear infinite",
      color: "var(--accent)",
      flexShrink: 0
    }
  }, /*#__PURE__*/React.createElement("circle", {
    cx: "12",
    cy: "12",
    r: "10",
    stroke: "currentColor",
    strokeWidth: "4",
    opacity: "0.25"
  }), /*#__PURE__*/React.createElement("path", {
    d: "M4 12a8 8 0 018-8V0C5.4 0 0 5.4 0 12h4z",
    fill: "currentColor",
    opacity: "0.9"
  })), label ? /*#__PURE__*/React.createElement("span", {
    style: {
      fontSize: "var(--text-caption)"
    }
  }, label) : null);
}
Object.assign(__ds_scope, { Spinner });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/core/Spinner.jsx", error: String((e && e.message) || e) }); }

// components/core/StatusBadge.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
const TONES = {
  pass: "var(--ok)",
  fail: "var(--err)",
  warn: "var(--warn)",
  unknown: "var(--steel-500)"
};

/** Hard verdict, stencilled like a core-box label: mono, uppercase, wide
 *  tracking, hairline box. `solid` fills it for a failure you cannot miss. */
function StatusBadge({
  tone = "pass",
  variant = "stencil",
  style,
  children,
  ...rest
}) {
  const c = TONES[tone];
  return /*#__PURE__*/React.createElement("span", _extends({
    style: {
      display: "inline-block",
      fontFamily: "var(--font-mono)",
      fontSize: "var(--text-bubble)",
      fontWeight: "var(--weight-bold)",
      letterSpacing: "0.09em",
      textTransform: "uppercase",
      borderRadius: "2px",
      padding: variant === "solid" ? "0.16rem 0.5rem" : "0.1rem 0.45rem",
      ...(variant === "solid" ? {
        background: c,
        color: "#fff",
        border: "1px solid " + c
      } : {
        background: "transparent",
        color: c,
        border: "1.5px solid " + c
      }),
      ...style
    }
  }, rest), children);
}
Object.assign(__ds_scope, { StatusBadge });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/core/StatusBadge.jsx", error: String((e && e.message) || e) }); }

// components/feedback/SummaryBanner.jsx
try { (() => {
const KINDS = {
  ok: {
    color: "var(--ok)",
    bg: "var(--ok-quiet)",
    border: "color-mix(in srgb, var(--ok) 45%, transparent)",
    glyph: "✓"
  },
  err: {
    color: "var(--err)",
    bg: "var(--err-quiet)",
    border: "color-mix(in srgb, var(--err) 45%, transparent)",
    glyph: "✗"
  },
  warn: {
    color: "var(--warn)",
    bg: "var(--warn-quiet)",
    border: "color-mix(in srgb, var(--warn) 45%, transparent)",
    glyph: "ⓘ"
  }
};

/** The verdict banner at the top of a result pane: tinted panel, coloured
 *  headline with a glyph, then neutral supporting lines. */
function SummaryBanner({
  kind = "ok",
  headline,
  detail,
  note,
  style
}) {
  const k = KINDS[kind];
  return /*#__PURE__*/React.createElement("div", {
    style: {
      borderRadius: "var(--radius-xl)",
      border: `1px solid ${k.border}`,
      background: k.bg,
      padding: "var(--space-5)",
      ...style
    }
  }, /*#__PURE__*/React.createElement("p", {
    style: {
      margin: 0,
      fontSize: "var(--text-body)",
      fontWeight: "var(--weight-semibold)",
      color: k.color
    }
  }, k.glyph, " ", headline), detail ? /*#__PURE__*/React.createElement("p", {
    style: {
      margin: "0.25rem 0 0",
      fontSize: "var(--text-caption)",
      color: "var(--fg-soft)"
    }
  }, detail) : null, note ? /*#__PURE__*/React.createElement("p", {
    style: {
      margin: "0.4rem 0 0",
      fontSize: "var(--text-micro)",
      color: "var(--fg-dim)"
    }
  }, note) : null);
}
Object.assign(__ds_scope, { SummaryBanner });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/feedback/SummaryBanner.jsx", error: String((e && e.message) || e) }); }

// components/feedback/Toast.jsx
try { (() => {
/** Bottom-left toast: deep maroon panel, white text, optional Undo. Flies in
 *  12px with cubicOut over 180ms; holds while hovered. One host, one at a time. */
function Toast({
  message,
  onUndo,
  onDismiss,
  style
}) {
  return /*#__PURE__*/React.createElement("div", {
    role: "status",
    "aria-live": "polite",
    style: {
      display: "inline-flex",
      alignItems: "center",
      gap: "var(--space-3)",
      maxWidth: "24rem",
      background: "var(--laterite-900)",
      color: "#fff",
      border: "1px solid rgb(255 255 255 / 18%)",
      borderRadius: "var(--radius-md)",
      padding: "0.5rem 0.75rem",
      fontSize: "var(--text-control)",
      boxShadow: "var(--shadow-toast)",
      ...style
    }
  }, /*#__PURE__*/React.createElement("span", null, message), onUndo ? /*#__PURE__*/React.createElement("button", {
    type: "button",
    onClick: onUndo,
    style: {
      font: "inherit",
      fontWeight: "var(--weight-semibold)",
      color: "var(--laterite-300)",
      background: "none",
      border: "none",
      cursor: "pointer",
      padding: "0.15rem 0.3rem",
      borderRadius: "var(--radius-xs)"
    }
  }, "Undo") : null, /*#__PURE__*/React.createElement("button", {
    type: "button",
    onClick: onDismiss,
    "aria-label": "Dismiss",
    style: {
      display: "inline-flex",
      alignItems: "center",
      background: "none",
      border: "none",
      color: "rgb(255 255 255 / 65%)",
      cursor: "pointer",
      padding: "0.2rem",
      borderRadius: "var(--radius-xs)"
    }
  }, /*#__PURE__*/React.createElement(__ds_scope.Icon, {
    name: "x",
    size: 13
  })));
}
Object.assign(__ds_scope, { Toast });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/feedback/Toast.jsx", error: String((e && e.message) || e) }); }

// components/feedback/Tooltip.jsx
try { (() => {
/** Icon-control tooltip: a maroon pill after a uniform 300ms delay (native
 *  `title` delays are browser-controlled and feel inconsistent). Long-form
 *  field help stays on `title`. */
function Tooltip({
  tip,
  placement = "top",
  children,
  style
}) {
  const [shown, setShown] = React.useState(false);
  const pos = placement === "bottom" ? {
    top: "calc(100% + 7px)"
  } : {
    bottom: "calc(100% + 7px)"
  };
  return /*#__PURE__*/React.createElement("span", {
    style: {
      position: "relative",
      display: "inline-flex",
      ...style
    },
    onMouseEnter: () => setShown(true),
    onMouseLeave: () => setShown(false),
    onFocus: () => setShown(true),
    onBlur: () => setShown(false)
  }, children, shown && tip ? /*#__PURE__*/React.createElement("span", {
    role: "tooltip",
    style: {
      position: "absolute",
      left: "50%",
      transform: "translateX(-50%)",
      zIndex: "var(--z-tooltip)",
      ...pos,
      background: "var(--laterite-900)",
      color: "#fff",
      border: "1px solid rgb(255 255 255 / 18%)",
      fontSize: "var(--text-caption)",
      lineHeight: "var(--leading-normal)",
      textAlign: "left",
      width: "max-content",
      maxWidth: "22rem",
      padding: "0.22rem 0.5rem",
      borderRadius: "var(--radius-sm)",
      boxShadow: "var(--shadow-tooltip)",
      pointerEvents: "none"
    }
  }, tip) : null);
}
Object.assign(__ds_scope, { Tooltip });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/feedback/Tooltip.jsx", error: String((e && e.message) || e) }); }

// components/forms/Checkbox.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
/** Checkbox + inline label — one clickable unit. The app's toolbar toggle. */
function Checkbox({
  label,
  style,
  ...rest
}) {
  return /*#__PURE__*/React.createElement("label", {
    style: {
      display: "inline-flex",
      alignItems: "center",
      gap: "0.35rem",
      fontSize: "var(--text-caption)",
      color: "var(--fg-soft)",
      cursor: "pointer",
      ...style
    }
  }, /*#__PURE__*/React.createElement("input", _extends({
    type: "checkbox",
    style: {
      width: "0.9rem",
      height: "0.9rem",
      accentColor: "var(--accent)",
      margin: 0
    }
  }, rest)), label);
}
Object.assign(__ds_scope, { Checkbox });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/forms/Checkbox.jsx", error: String((e && e.message) || e) }); }

// components/forms/ControlGrid.jsx
try { (() => {
/** A responsive control row that stacks predictably 1 → 2 → 3 columns instead
 *  of wrapping into ragged piles. Each child (usually a Field) takes one cell.
 *  Cells are TOP-aligned so every control lands on the same line — bottom
 *  alignment let a hint line under one control push it out of step. */
function ControlGrid({
  min = "14rem",
  style,
  children
}) {
  return /*#__PURE__*/React.createElement("div", {
    style: {
      display: "grid",
      gap: "var(--space-3)",
      gridTemplateColumns: `repeat(auto-fit, minmax(${min}, 1fr))`,
      alignItems: "start",
      ...style
    }
  }, children);
}
Object.assign(__ds_scope, { ControlGrid });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/forms/ControlGrid.jsx", error: String((e && e.message) || e) }); }

// components/forms/Field.jsx
try { (() => {
/** Label-above-control wrapper. The label is micro-scale muted text; the control
 *  sits in a fixed-height, vertically-centred box so an input, a select and a
 *  checkbox all land on the same line inside a ControlGrid, and an optional hint
 *  below can't push its own control out of step. */
function Field({
  label,
  hint,
  style,
  children
}) {
  return /*#__PURE__*/React.createElement("label", {
    style: {
      display: "flex",
      flexDirection: "column",
      gap: "var(--space-1)",
      fontSize: "var(--text-micro)",
      color: "var(--fg-muted)",
      minWidth: 0,
      ...style
    }
  }, label, /*#__PURE__*/React.createElement("span", {
    style: {
      display: "grid",
      alignContent: "center",
      minHeight: "var(--control-h)",
      minWidth: 0
    }
  }, children), hint ? /*#__PURE__*/React.createElement("span", {
    style: {
      color: "var(--fg-dim)"
    }
  }, hint) : null);
}
Object.assign(__ds_scope, { Field });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/forms/Field.jsx", error: String((e && e.message) || e) }); }

// components/forms/Input.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
const CONTROL = {
  font: "inherit",
  fontFamily: "var(--font-ui)",
  fontSize: "var(--text-control)",
  padding: "0.25rem 0.4rem",
  border: "1px solid var(--line-strong)",
  borderRadius: "var(--radius-xs)",
  backgroundColor: "var(--surface-raised)",
  color: "var(--fg)",
  outline: "none",
  minWidth: 0,
  width: "100%"
};

/** The canonical text control. Everything else in the system (Select, the SQL
 *  console, the paste area) is this box with one property changed. */
function Input({
  mono,
  invalid,
  style,
  ...rest
}) {
  return /*#__PURE__*/React.createElement("input", _extends({
    style: {
      ...CONTROL,
      ...(mono ? {
        fontFamily: "var(--font-mono)"
      } : null),
      ...(invalid ? {
        borderColor: "var(--err)"
      } : null),
      ...style
    }
  }, rest));
}
const controlStyle = CONTROL;
Object.assign(__ds_scope, { Input, controlStyle });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/forms/Input.jsx", error: String((e && e.message) || e) }); }

// components/forms/Select.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
/** Select with the brand's own chevron. Native selects pin the platform arrow
 *  to the very edge and add their own text inset; drawing the chevron with two
 *  gradients keeps the text flush with Input and gives the arrow room. */
function Select({
  style,
  children,
  ...rest
}) {
  return /*#__PURE__*/React.createElement("select", _extends({
    style: {
      ...__ds_scope.controlStyle,
      appearance: "none",
      paddingRight: "1.4rem",
      backgroundImage: "linear-gradient(45deg, transparent 50%, var(--fg-muted) 50%), linear-gradient(135deg, var(--fg-muted) 50%, transparent 50%)",
      backgroundRepeat: "no-repeat",
      backgroundPosition: "calc(100% - 0.8rem) 50%, calc(100% - 0.5rem) 50%",
      backgroundSize: "0.3rem 0.3rem",
      ...style
    }
  }, rest), children);
}
Object.assign(__ds_scope, { Select });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/forms/Select.jsx", error: String((e && e.message) || e) }); }

// components/navigation/PillToggle.jsx
try { (() => {
/** In-pane view selector (Browse / SQL / Charts / Analyse). Active = chip fill
 *  + accent text; inactive = muted text that darkens on hover. */
function PillToggle({
  options = [],
  value,
  onChange,
  style
}) {
  return /*#__PURE__*/React.createElement("div", {
    style: {
      display: "flex",
      gap: "var(--space-1)",
      fontSize: "var(--text-control)",
      ...style
    }
  }, options.map(o => {
    const id = o.id ?? o;
    const on = id === value;
    return /*#__PURE__*/React.createElement("button", {
      key: id,
      type: "button",
      onClick: () => onChange && onChange(id),
      "aria-pressed": on,
      style: {
        font: "inherit",
        fontFamily: "var(--font-ui)",
        borderRadius: "var(--radius-sm)",
        padding: "0.25rem 0.75rem",
        border: "none",
        fontWeight: "var(--weight-medium)",
        background: on ? "var(--accent-quiet)" : "none",
        color: on ? "var(--accent)" : "var(--fg-muted)",
        cursor: "pointer",
        transition: "background-color var(--dur-base) var(--ease-out), color var(--dur-base) var(--ease-out)"
      }
    }, o.label ?? id);
  }));
}
Object.assign(__ds_scope, { PillToggle });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/navigation/PillToggle.jsx", error: String((e && e.message) || e) }); }

// components/navigation/Tabs.jsx
try { (() => {
/** Top-level section bar: underlined role="tab" buttons on a hairline. The
 *  active tab is accent text + a 2px accent underline that overlaps the rule. */
function Tabs({
  tabs = [],
  active,
  onChange,
  style
}) {
  return /*#__PURE__*/React.createElement("nav", {
    role: "tablist",
    "aria-label": "Sections",
    style: {
      display: "flex",
      gap: "var(--space-1)",
      overflowX: "auto",
      borderBottom: "1px solid var(--line)",
      padding: "0 var(--space-6)",
      ...style
    }
  }, tabs.map(t => {
    const id = t.id ?? t;
    const on = id === active;
    return /*#__PURE__*/React.createElement("button", {
      key: id,
      type: "button",
      role: "tab",
      "aria-selected": on,
      onClick: () => onChange && onChange(id),
      style: {
        font: "inherit",
        fontFamily: "var(--font-ui)",
        position: "relative",
        marginBottom: "-1px",
        padding: "0.5rem 1rem",
        border: "none",
        borderBottom: `2px solid ${on ? "var(--accent)" : "transparent"}`,
        background: "none",
        fontSize: "var(--text-caption)",
        fontWeight: "var(--weight-medium)",
        color: on ? "var(--accent)" : "var(--fg-muted)",
        cursor: "pointer",
        whiteSpace: "nowrap",
        transition: "color var(--dur-base) var(--ease-out)"
      }
    }, t.label ?? id);
  }));
}
Object.assign(__ds_scope, { Tabs });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/navigation/Tabs.jsx", error: String((e && e.message) || e) }); }

// components/navigation/ThemeToggle.jsx
try { (() => {
/** Header light/dark switch. Shows the icon of the theme you'd switch TO. */
function ThemeToggle({
  mode = "light",
  onToggle,
  style
}) {
  return /*#__PURE__*/React.createElement("button", {
    type: "button",
    onClick: onToggle,
    "aria-label": "Toggle colour theme",
    title: mode === "dark" ? "Switch to light theme" : "Switch to dark theme",
    style: {
      font: "inherit",
      fontFamily: "var(--font-ui)",
      fontSize: "var(--text-caption)",
      borderRadius: "var(--radius-sm)",
      border: "1px solid var(--line-strong)",
      background: "var(--surface)",
      color: "var(--fg-soft)",
      padding: "0.2rem 0.55rem",
      cursor: "pointer",
      transition: "color var(--dur-base) var(--ease-out), border-color var(--dur-base) var(--ease-out)",
      ...style
    }
  }, mode === "dark" ? "☀︎" : "☾");
}
Object.assign(__ds_scope, { ThemeToggle });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/navigation/ThemeToggle.jsx", error: String((e && e.message) || e) }); }

// components/surfaces/Admonition.jsx
try { (() => {
const KINDS = {
  tip: {
    color: "var(--ok)",
    bg: "var(--ok-quiet)",
    icon: "lightbulb",
    label: "Tip"
  },
  note: {
    color: "var(--accent)",
    bg: "var(--accent-quiet)",
    icon: "info",
    label: "Note"
  },
  warning: {
    color: "var(--warn)",
    bg: "var(--warn-quiet)",
    icon: "triangle-alert",
    label: "Warning"
  },
  danger: {
    color: "var(--err)",
    bg: "var(--err-quiet)",
    icon: "octagon-alert",
    label: "Danger"
  }
};

/** Docs call-out. Tinted body, 3px coloured rule on the leading edge, icon +
 *  title row — the docs site's admonition, rebuilt on brand tokens. */
function Admonition({
  kind = "note",
  title,
  style,
  children
}) {
  const k = KINDS[kind];
  return /*#__PURE__*/React.createElement("div", {
    style: {
      background: k.bg,
      borderLeft: `3px solid ${k.color}`,
      borderRadius: "0 var(--radius-lg) var(--radius-lg) 0",
      padding: "var(--space-3) var(--space-4)",
      ...style
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: "flex",
      alignItems: "center",
      gap: "var(--space-2)",
      color: k.color,
      fontSize: "var(--text-caption)",
      fontWeight: "var(--weight-semibold)",
      marginBottom: "var(--space-1)"
    }
  }, /*#__PURE__*/React.createElement(__ds_scope.Icon, {
    name: k.icon,
    size: 14
  }), title ?? k.label), /*#__PURE__*/React.createElement("div", {
    style: {
      fontSize: "var(--text-body)",
      color: "var(--fg-soft)",
      lineHeight: "var(--leading-relaxed)"
    }
  }, children));
}
Object.assign(__ds_scope, { Admonition });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/surfaces/Admonition.jsx", error: String((e && e.message) || e) }); }

// components/surfaces/Card.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
/** The repeated panel: 8px radius, 1px line, surface fill, NO shadow — cards
 *  lift by the canvas→surface step, not by elevation. */
function Card({
  as: Tag = "section",
  pad = "md",
  style,
  children,
  ...rest
}) {
  const padding = {
    none: 0,
    sm: "var(--space-3)",
    md: "var(--space-4)",
    lg: "var(--space-6)"
  }[pad];
  return /*#__PURE__*/React.createElement(Tag, _extends({
    style: {
      borderRadius: "var(--radius-xl)",
      border: "1px solid var(--line)",
      background: "var(--surface)",
      boxShadow: "var(--shadow-card)",
      padding,
      ...style
    }
  }, rest), children);
}
Object.assign(__ds_scope, { Card });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/surfaces/Card.jsx", error: String((e && e.message) || e) }); }

// components/surfaces/CodeTabs.jsx
try { (() => {
/** Per-surface code tabs — the docs site shows every task across Python, Node,
 *  the CLI and DuckDB with the tabs synced. Tab row sits on the code surface. */
function CodeTabs({
  tabs = [],
  initial = 0,
  style
}) {
  const [i, setI] = React.useState(initial);
  const active = tabs[i] ?? {
    code: ""
  };
  return /*#__PURE__*/React.createElement("div", {
    style: {
      border: "1px solid var(--line)",
      borderRadius: "var(--radius-lg)",
      overflow: "hidden",
      background: "var(--surface-code)",
      ...style
    }
  }, /*#__PURE__*/React.createElement("div", {
    role: "tablist",
    style: {
      display: "flex",
      gap: 0,
      borderBottom: "1px solid var(--line)",
      background: "var(--surface-raised)"
    }
  }, tabs.map((t, n) => /*#__PURE__*/React.createElement("button", {
    key: t.label,
    type: "button",
    role: "tab",
    "aria-selected": n === i,
    onClick: () => setI(n),
    style: {
      font: "inherit",
      fontFamily: "var(--font-ui)",
      fontSize: "var(--text-micro)",
      fontWeight: "var(--weight-semibold)",
      letterSpacing: "var(--tracking-micro)",
      textTransform: "uppercase",
      padding: "0.35rem 0.8rem",
      border: "none",
      borderBottom: n === i ? "2px solid var(--accent)" : "2px solid transparent",
      background: "none",
      color: n === i ? "var(--accent)" : "var(--fg-muted)",
      cursor: "pointer",
      transition: "color var(--dur-base) var(--ease-out)"
    }
  }, t.label))), /*#__PURE__*/React.createElement("pre", {
    style: {
      margin: 0,
      padding: "var(--space-4)",
      overflowX: "auto",
      fontFamily: "var(--font-mono)",
      fontSize: "var(--text-control)",
      lineHeight: "var(--leading-relaxed)",
      color: "var(--fg)"
    }
  }, /*#__PURE__*/React.createElement("code", null, active.code)));
}
Object.assign(__ds_scope, { CodeTabs });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/surfaces/CodeTabs.jsx", error: String((e && e.message) || e) }); }

// components/surfaces/Dialog.jsx
try { (() => {
/** Modal dialog: maroon-tinted scrim (never neutral black, never blurred),
 *  8px panel, --shadow-dialog, header row with a ghost ✕. */
function Dialog({
  open = true,
  title,
  hint,
  width = "var(--dialog-w)",
  onClose,
  footer,
  style,
  children
}) {
  if (!open) return null;
  return /*#__PURE__*/React.createElement("div", {
    role: "dialog",
    "aria-modal": "true",
    "aria-label": typeof title === "string" ? title : undefined,
    style: {
      position: "absolute",
      inset: 0,
      zIndex: "var(--z-dialog)",
      background: "var(--scrim)",
      display: "flex",
      alignItems: "flex-start",
      justifyContent: "center",
      padding: "var(--space-8)"
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      width: `min(${width}, 100%)`,
      background: "var(--surface)",
      border: "1px solid var(--line)",
      borderRadius: "var(--radius-xl)",
      boxShadow: "var(--shadow-dialog)",
      padding: "1rem 1.25rem 1.25rem",
      ...style
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: "flex",
      justifyContent: "space-between",
      alignItems: "center",
      gap: "var(--space-4)",
      marginBottom: "var(--space-2)"
    }
  }, /*#__PURE__*/React.createElement("strong", {
    style: {
      fontSize: "var(--text-title)",
      fontWeight: "var(--weight-semibold)",
      color: "var(--fg)"
    }
  }, title), /*#__PURE__*/React.createElement(__ds_scope.Button, {
    variant: "ghost",
    "aria-label": "Close",
    onClick: onClose
  }, "\u2715")), hint ? /*#__PURE__*/React.createElement("p", {
    style: {
      fontSize: "var(--text-caption)",
      color: "var(--fg-muted)",
      lineHeight: "var(--leading-normal)",
      margin: "0 0 0.6rem"
    }
  }, hint) : null, /*#__PURE__*/React.createElement("div", {
    style: {
      fontSize: "var(--text-body)",
      color: "var(--fg)"
    }
  }, children), footer ? /*#__PURE__*/React.createElement("div", {
    style: {
      display: "flex",
      justifyContent: "flex-end",
      gap: "var(--space-2)",
      marginTop: "var(--space-5)"
    }
  }, footer) : null));
}
Object.assign(__ds_scope, { Dialog });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/surfaces/Dialog.jsx", error: String((e && e.message) || e) }); }

// components/surfaces/Disclosure.jsx
try { (() => {
/** Collapsible panel — a Card whose header is a summary row. Anything optional
 *  (samples, examples, filter chips) collapses to one line by default. */
function Disclosure({
  summary,
  count,
  defaultOpen = false,
  style,
  children
}) {
  const [open, setOpen] = React.useState(defaultOpen);
  return /*#__PURE__*/React.createElement("div", {
    style: {
      borderRadius: "var(--radius-xl)",
      border: "1px solid var(--line)",
      background: "var(--surface)",
      ...style
    }
  }, /*#__PURE__*/React.createElement("button", {
    type: "button",
    onClick: () => setOpen(!open),
    "aria-expanded": open,
    style: {
      font: "inherit",
      fontFamily: "var(--font-ui)",
      width: "100%",
      display: "flex",
      alignItems: "center",
      gap: "var(--space-2)",
      padding: "0.5rem 0.75rem",
      background: "none",
      border: "none",
      cursor: "pointer",
      fontSize: "var(--text-caption)",
      fontWeight: "var(--weight-medium)",
      color: "var(--fg-soft)",
      textAlign: "left"
    }
  }, /*#__PURE__*/React.createElement(__ds_scope.Chevron, {
    open: open
  }), /*#__PURE__*/React.createElement("span", {
    style: {
      minWidth: 0
    }
  }, summary), count ? /*#__PURE__*/React.createElement("span", {
    style: {
      marginLeft: "auto"
    }
  }, /*#__PURE__*/React.createElement(__ds_scope.CountBubble, {
    tone: "muted"
  }, count)) : null), open ? /*#__PURE__*/React.createElement("div", {
    style: {
      borderTop: "1px solid var(--line-subtle)",
      padding: "var(--space-3)"
    }
  }, children) : null);
}
Object.assign(__ds_scope, { Disclosure });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/surfaces/Disclosure.jsx", error: String((e && e.message) || e) }); }

// ui_kits/demo-site/DemoSite.jsx
try { (() => {
const {
  Button,
  Chip,
  Card,
  CodeTabs,
  Admonition,
  SummaryBanner,
  Icon,
  ThemeToggle,
  Tooltip,
  StatusBadge
} = window.LateriteDesignSystem_9f517b;
const SEV_ORDER = {
  error: 0,
  warning: 1,
  fyi: 2
};
const SEV_TONE = {
  error: "err",
  warning: "warn",
  fyi: "info"
};
function Connector({
  from,
  to,
  link,
  flip
}) {
  // x positions are the centres of the wide (table) grid column: ~29% when the
  // table sits left, ~71% when it sits right. Rows alternate, so the line always
  // crosses — parent table → child table, never through the prose.
  const a = flip ? 29 : 71; // parent table side (previous row)
  const b = flip ? 71 : 29; // child table side (this row)
  const lo = Math.min(a, b),
    hi = Math.max(a, b);
  return /*#__PURE__*/React.createElement("div", {
    className: "conn"
  }, /*#__PURE__*/React.createElement("i", {
    className: "cdot",
    style: {
      left: a + "%"
    }
  }), /*#__PURE__*/React.createElement("i", {
    className: "cv1",
    style: {
      left: a + "%"
    }
  }), /*#__PURE__*/React.createElement("i", {
    className: "ch",
    style: {
      left: lo + "%",
      width: hi - lo + "%"
    }
  }), /*#__PURE__*/React.createElement("i", {
    className: "cv2",
    style: {
      left: b + "%"
    }
  }), /*#__PURE__*/React.createElement("i", {
    className: "ctip",
    style: {
      left: b + "%"
    }
  }), /*#__PURE__*/React.createElement("span", {
    className: "connlabel"
  }, /*#__PURE__*/React.createElement("span", {
    className: "connpair"
  }, /*#__PURE__*/React.createElement("span", {
    className: "mono"
  }, from), /*#__PURE__*/React.createElement("span", {
    className: "connarrow"
  }, "\u2192"), /*#__PURE__*/React.createElement("span", {
    className: "mono"
  }, to)), link.keys.length ? /*#__PURE__*/React.createElement("span", {
    className: "connkeys"
  }, "joined on ", link.keys.map(k => /*#__PURE__*/React.createElement("span", {
    className: "mono keychip",
    key: k
  }, k))) : null, /*#__PURE__*/React.createElement("span", {
    className: "connnote"
  }, link.note)));
}
function Findings({
  findings,
  onJump,
  selected
}) {
  const sorted = [...findings].sort((a, b) => SEV_ORDER[a.severity] - SEV_ORDER[b.severity]);
  const label = {
    error: "error",
    warning: "warning",
    fyi: "fyi"
  };
  const tone = {
    error: "err",
    warning: "warn",
    fyi: "info"
  };
  const form = {
    error: "solid",
    warning: "rule",
    fyi: "rule"
  };
  if (!sorted.length) {
    return /*#__PURE__*/React.createElement("p", {
      className: "nofind"
    }, "Nothing to report. Every rule in the 4.1.1 dictionary passes on this file.");
  }
  return /*#__PURE__*/React.createElement("ul", {
    className: "findlist"
  }, sorted.map((f, i) => /*#__PURE__*/React.createElement("li", {
    key: i
  }, /*#__PURE__*/React.createElement("button", {
    type: "button",
    className: "find" + (selected === i ? " findon" : ""),
    onClick: () => onJump(f, i)
  }, /*#__PURE__*/React.createElement("span", {
    className: "findtop"
  }, /*#__PURE__*/React.createElement(Chip, {
    tone: tone[f.severity],
    variant: form[f.severity]
  }, label[f.severity]), /*#__PURE__*/React.createElement("span", {
    className: "findrule"
  }, f.rule), f.fix ? /*#__PURE__*/React.createElement(Chip, {
    tone: "ok",
    variant: "outline"
  }, "fixable") : null), /*#__PURE__*/React.createElement("span", {
    className: "finddesc"
  }, f.desc), /*#__PURE__*/React.createElement("span", {
    className: "findwhere"
  }, f.group, f.row != null ? " · row " + (f.row + 1) : "")))));
}
function DemoSite() {
  const [groups, setGroups] = React.useState(() => window.clone(window.SEED));
  const [withTran, setWithTran] = React.useState(false);
  const [markup, setMarkup] = React.useState({
    PROJ: true
  });
  const [mode, setMode] = React.useState("light");
  const [sel, setSel] = React.useState(null);
  const [hotLine, setHotLine] = React.useState(null);
  const [fixed, setFixed] = React.useState(false);
  const outRef = React.useRef(null);
  React.useEffect(() => {
    document.documentElement.classList.toggle("dark", mode === "dark");
  }, [mode]);
  const findings = React.useMemo(() => window.validate(groups, withTran), [groups, withTran]);
  const out = React.useMemo(() => window.emit(groups, withTran), [groups, withTran]);
  const counts = findings.reduce((a, f) => ({
    ...a,
    [f.severity]: (a[f.severity] || 0) + 1
  }), {});
  const fixable = findings.filter(f => f.fix);
  const setCell = (code, row, col, value) => {
    setFixed(false);
    setGroups(gs => gs.map(g => g.code !== code ? g : {
      ...g,
      rows: g.rows.map((r, i) => i !== row ? r : r.map((c, j) => j === col ? value : c))
    }));
  };
  const addRow = code => setGroups(gs => gs.map(g => {
    if (g.code !== code) return g;
    const last = g.rows[g.rows.length - 1] ?? g.headings.map(() => "");
    return {
      ...g,
      rows: [...g.rows, g.headings.map((h, c) => h.key && h.fk ? last[c] : "")]
    };
  }));
  const deleteRow = (code, row) => setGroups(gs => gs.map(g => g.code === code ? {
    ...g,
    rows: g.rows.filter((_, i) => i !== row)
  } : g));
  const fix = () => {
    const r = window.applyFixes(groups, findings, withTran);
    setGroups(r.groups);
    setWithTran(r.withTran);
    setFixed(true);
    setSel(null);
    setHotLine(null);
  };
  const reset = () => {
    setGroups(window.clone(window.SEED));
    setWithTran(false);
    setFixed(false);
    setSel(null);
    setHotLine(null);
  };
  const jump = (f, i) => {
    setSel(i);
    const line = out.lines.find(l => l.group === f.group && (f.row != null ? l.kind === "DATA" && l.row === f.row : l.kind === (f.kind || "GROUP")));
    if (!line) return;
    setHotLine(line.n);
    const el = outRef.current?.querySelector('[data-line="' + line.n + '"]');
    if (el && outRef.current) outRef.current.scrollTop = el.offsetTop - 90;
  };
  const download = () => {
    const blob = new Blob([out.text + "\n"], {
      type: "text/plain"
    });
    const a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = "demo-delivery.ags";
    a.click();
    URL.revokeObjectURL(a.href);
  };

  // A file whose only findings are FYI is informational, not clean: amber + ⓘ.
  // "ok" is reserved for zero findings of any severity.
  const verdict = counts.error ? "err" : counts.warning || counts.fyi ? "warn" : "ok";
  const onlyFyi = !counts.error && !counts.warning && counts.fyi;
  const headline = onlyFyi ? counts.fyi + " informational (FYI) finding" + (counts.fyi === 1 ? "" : "s") + " — no errors or warnings" : counts.error || counts.warning ? [counts.error && counts.error + " error" + (counts.error === 1 ? "" : "s"), counts.warning && counts.warning + " warning" + (counts.warning === 1 ? "" : "s"), counts.fyi && counts.fyi + " informational"].filter(Boolean).join(" · ") : "Clean — 0 findings";
  return /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("header", {
    className: "nav"
  }, /*#__PURE__*/React.createElement("a", {
    className: "brand",
    href: "#top"
  }, /*#__PURE__*/React.createElement("span", {
    className: "brandplate"
  }, /*#__PURE__*/React.createElement("img", {
    src: "../../assets/laterite-icon-256.png",
    alt: ""
  })), /*#__PURE__*/React.createElement("span", null, "laterite")), /*#__PURE__*/React.createElement("nav", {
    className: "navlinks"
  }, /*#__PURE__*/React.createElement("a", {
    href: "#build"
  }, "Build a file"), /*#__PURE__*/React.createElement("a", {
    href: "#output"
  }, "The file"), /*#__PURE__*/React.createElement("a", {
    href: "#install"
  }, "Install"), /*#__PURE__*/React.createElement("a", {
    href: "https://docs.laterite.dev/"
  }, "Docs"), /*#__PURE__*/React.createElement("a", {
    href: "https://github.com/niko86/laterite"
  }, "GitHub")), /*#__PURE__*/React.createElement("div", {
    className: "navright"
  }, /*#__PURE__*/React.createElement(ThemeToggle, {
    mode: mode,
    onToggle: () => setMode(mode === "dark" ? "light" : "dark")
  }), /*#__PURE__*/React.createElement("a", {
    className: "btnprim",
    href: "https://app.laterite.dev/"
  }, "Open the web app"))), /*#__PURE__*/React.createElement("section", {
    className: "hero",
    id: "top"
  }, /*#__PURE__*/React.createElement("div", {
    className: "herotext"
  }, /*#__PURE__*/React.createElement("p", {
    className: "eyebrow"
  }, "AGS4 toolkit \xB7 one Rust engine, five surfaces"), /*#__PURE__*/React.createElement("h1", null, "AGS4, shown rather than explained."), /*#__PURE__*/React.createElement("p", {
    className: "lead"
  }, "The tables below are a real AGS4 delivery: a project, its boreholes, their samples and a lab test. Type in them and the file rewrites itself as you go \u2014 validated by the same engine that ships to Python, Node, DuckDB, the CLI and the browser. Nothing is uploaded; it all runs in this page."), /*#__PURE__*/React.createElement("div", {
    className: "herocta"
  }, /*#__PURE__*/React.createElement("a", {
    className: "btnprim big",
    href: "#build"
  }, "Start editing"), /*#__PURE__*/React.createElement("a", {
    className: "btnghost big",
    href: "#install"
  }, "Get it for your stack")), /*#__PURE__*/React.createElement("p", {
    className: "micro"
  }, "Compiled to WebAssembly \xB7 MIT licensed \xB7 ", /*#__PURE__*/React.createElement("a", {
    href: "https://docs.laterite.dev/reference/support/"
  }, "in beta"))), /*#__PURE__*/React.createElement("aside", {
    className: "herofile",
    "aria-label": "What an AGS4 file looks like"
  }, /*#__PURE__*/React.createElement("span", {
    className: "herofilebar"
  }, /*#__PURE__*/React.createElement("span", {
    className: "mono"
  }, "delivery.ags"), /*#__PURE__*/React.createElement("span", null, "plain text \xB7 quoted fields")), /*#__PURE__*/React.createElement("pre", {
    className: "mono herofilepre"
  }, '"GROUP","LOCA"\n"HEADING","LOCA_ID","LOCA_GL"\n"UNIT","","m"\n"TYPE","ID","2DP"\n"DATA","BH01","12.30"\n"DATA","BH02","11.80"'), /*#__PURE__*/React.createElement("span", {
    className: "herofilenote"
  }, "Four markup rows, then the data. ", /*#__PURE__*/React.createElement("span", {
    className: "mono"
  }, "TYPE"), " is why a level arrives as a float and ", /*#__PURE__*/React.createElement("span", {
    className: "mono"
  }, "\"11.8\""), " is an error."))), /*#__PURE__*/React.createElement("section", {
    className: "mobileonly"
  }, /*#__PURE__*/React.createElement("div", {
    className: "secthead"
  }, /*#__PURE__*/React.createElement("h2", null, "What the demo does"), /*#__PURE__*/React.createElement("p", null, "The interactive part of this page is four editable tables wired to a live AGS4 file \u2014 it needs a wide screen to read honestly, so on a phone here is the short version instead.")), /*#__PURE__*/React.createElement("ol", {
    className: "mchain"
  }, /*#__PURE__*/React.createElement("li", null, /*#__PURE__*/React.createElement("span", {
    className: "mono"
  }, "PROJ"), " names the job \u2014 one row."), /*#__PURE__*/React.createElement("li", null, /*#__PURE__*/React.createElement("span", {
    className: "mono"
  }, "LOCA"), " is one row per borehole or pit, keyed by ", /*#__PURE__*/React.createElement("span", {
    className: "mono"
  }, "LOCA_ID"), "."), /*#__PURE__*/React.createElement("li", null, /*#__PURE__*/React.createElement("span", {
    className: "mono"
  }, "SAMP"), " repeats that key and adds depth and reference."), /*#__PURE__*/React.createElement("li", null, /*#__PURE__*/React.createElement("span", {
    className: "mono"
  }, "LLPL"), " repeats the whole sample key and adds lab results.")), /*#__PURE__*/React.createElement("p", {
    className: "mnote"
  }, "That repetition of columns ", /*#__PURE__*/React.createElement("em", null, "is"), " the relationship \u2014 AGS has no ids and no joins. laterite reads it, types it, validates every numbered rule against it, and repairs what is safe to repair."), /*#__PURE__*/React.createElement("div", {
    className: "mlinks"
  }, /*#__PURE__*/React.createElement("a", {
    className: "btnprim big",
    href: "https://app.laterite.dev/"
  }, "Open the full web app"), /*#__PURE__*/React.createElement("a", {
    className: "btnghost big",
    href: "https://docs.laterite.dev/"
  }, "Read the docs"))), /*#__PURE__*/React.createElement("section", {
    id: "build",
    className: "build"
  }, groups.map((g, i) => {
    const flip = i % 2 === 1;
    return /*#__PURE__*/React.createElement(React.Fragment, {
      key: g.code
    }, g.parent ? /*#__PURE__*/React.createElement(Connector, {
      from: g.parent,
      to: g.code,
      link: g.link,
      flip: flip
    }) : null, /*#__PURE__*/React.createElement("div", {
      className: "zig" + (flip ? " zigflip" : "")
    }, /*#__PURE__*/React.createElement("div", {
      className: "zigtable"
    }, /*#__PURE__*/React.createElement(window.GroupTable, {
      group: g,
      findings: findings,
      onCell: setCell,
      onAddRow: addRow,
      onDeleteRow: deleteRow,
      showMarkup: !!markup[g.code],
      onToggleMarkup: () => setMarkup({
        ...markup,
        [g.code]: !markup[g.code]
      })
    })), /*#__PURE__*/React.createElement("div", {
      className: "zigprose"
    }, /*#__PURE__*/React.createElement("h3", null, /*#__PURE__*/React.createElement("span", {
      className: "mono"
    }, g.code), " ", g.title), /*#__PURE__*/React.createElement("p", null, g.blurb), g.code === "PROJ" ? /*#__PURE__*/React.createElement("p", {
      className: "aside"
    }, "\u25C6 marks a KEY field. Keys are how rows find each other \u2014 no ids, no foreign keys, just repeated columns.") : null, g.code === "LOCA" ? /*#__PURE__*/React.createElement("p", {
      className: "aside"
    }, "Try it: change ", /*#__PURE__*/React.createElement("span", {
      className: "mono"
    }, "BH02"), "'s ground level to ", /*#__PURE__*/React.createElement("span", {
      className: "mono"
    }, "11.8"), " \u2014 or fix it to ", /*#__PURE__*/React.createElement("span", {
      className: "mono"
    }, "11.80"), " \u2014 and watch the findings panel.") : null, g.code === "SAMP" ? /*#__PURE__*/React.createElement("p", {
      className: "aside"
    }, "Try it: point a sample at a hole that doesn't exist. Rule 10 catches the orphan immediately.") : null, g.code === "LLPL" ? /*#__PURE__*/React.createElement("p", {
      className: "aside"
    }, "The same shape carries every lab and in-situ group \u2014 ISPT, ERES, TRIG, CBRG \u2014 174 of them in edition 4.1.1.") : null)));
  })), /*#__PURE__*/React.createElement("section", {
    id: "output",
    className: "output"
  }, /*#__PURE__*/React.createElement("div", {
    className: "secthead"
  }, /*#__PURE__*/React.createElement("h2", null, "The file, and what the engine thinks of it"), /*#__PURE__*/React.createElement("p", null, "Left is the AGS4 your tables produce, line for line. Right is every finding, worst first \u2014 click one to jump to its line.")), /*#__PURE__*/React.createElement("div", {
    className: "outgrid"
  }, /*#__PURE__*/React.createElement("div", {
    className: "outpane"
  }, /*#__PURE__*/React.createElement("div", {
    className: "outbar"
  }, /*#__PURE__*/React.createElement("span", {
    className: "outname mono"
  }, "demo-delivery.ags"), /*#__PURE__*/React.createElement("span", {
    className: "outmeta"
  }, out.lines.length, " lines \xB7 ", new Blob([out.text]).size, " bytes \xB7 UTF-8"), /*#__PURE__*/React.createElement("span", {
    className: "gspacer"
  }), /*#__PURE__*/React.createElement(Button, {
    size: "sm",
    onClick: () => navigator.clipboard?.writeText(out.text),
    iconLeft: /*#__PURE__*/React.createElement(Icon, {
      name: "clipboard-copy",
      size: 14
    })
  }, "Copy"), /*#__PURE__*/React.createElement(Button, {
    size: "sm",
    onClick: download,
    iconLeft: /*#__PURE__*/React.createElement(Icon, {
      name: "file-down",
      size: 14
    })
  }, "Download")), /*#__PURE__*/React.createElement("div", {
    className: "outbody",
    ref: outRef
  }, /*#__PURE__*/React.createElement("pre", {
    className: "agspre"
  }, out.lines.map(l => /*#__PURE__*/React.createElement("div", {
    key: l.n,
    "data-line": l.n,
    className: "agsline" + (l.n === hotLine ? " agshot" : "") + (l.kind === "DATA" ? "" : " agsmeta")
  }, /*#__PURE__*/React.createElement("span", {
    className: "agsn"
  }, l.n), l.text))))), /*#__PURE__*/React.createElement("div", {
    className: "outside"
  }, /*#__PURE__*/React.createElement(SummaryBanner, {
    kind: verdict,
    headline: headline,
    detail: withTran ? "Validated against AGS 4.1.1 — exact TRAN_AGS match" : "Validated against AGS 4.1.1 — fallback (TRAN_AGS missing)",
    note: onlyFyi ? "FYI findings are informational, not violations — the file is deliverable." : fixed ? "Safe fixes applied. Anything left needs a human decision." : null
  }), /*#__PURE__*/React.createElement("div", {
    className: "fixbar"
  }, /*#__PURE__*/React.createElement(Button, {
    variant: "action",
    onClick: fix,
    disabled: !fixable.length,
    style: {
      fontSize: "0.95rem",
      padding: "0.4rem 1rem"
    },
    iconLeft: /*#__PURE__*/React.createElement(Icon, {
      name: "wrench",
      size: 15
    })
  }, fixable.length ? "Fix " + fixable.length + " safe finding" + (fixable.length === 1 ? "" : "s") : "Nothing safe left to fix"), /*#__PURE__*/React.createElement(Button, {
    size: "sm",
    onClick: reset,
    iconLeft: /*#__PURE__*/React.createElement(Icon, {
      name: "rotate-ccw",
      size: 14
    })
  }, "Reset"), /*#__PURE__*/React.createElement(Tooltip, {
    tip: "A certificate can only be minted for a file with no errors"
  }, /*#__PURE__*/React.createElement("span", null, /*#__PURE__*/React.createElement(Button, {
    size: "sm",
    disabled: !!counts.error,
    iconLeft: /*#__PURE__*/React.createElement(Icon, {
      name: "shield-check",
      size: 14
    })
  }, "Certify")))), /*#__PURE__*/React.createElement("div", {
    className: "findwrap"
  }, /*#__PURE__*/React.createElement(Findings, {
    findings: findings,
    onJump: jump,
    selected: sel
  })))), /*#__PURE__*/React.createElement(Admonition, {
    kind: "note",
    title: "What the real engine adds"
  }, "This page runs a cut-down rule set for the demo. The shipped engine carries the full numbered AGS4 rule catalogue, every dictionary edition, the repair engine, revision diff, merge, certificates, Excel conversion and SQL across groups \u2014 with byte-identical output on every surface.")), /*#__PURE__*/React.createElement("section", {
    id: "install",
    className: "install"
  }, /*#__PURE__*/React.createElement("div", {
    className: "secthead"
  }, /*#__PURE__*/React.createElement("h2", null, "Take it with you"), /*#__PURE__*/React.createElement("p", null, "Same engine, whichever door you come in by. Every surface is in beta.")), /*#__PURE__*/React.createElement("div", {
    className: "installgrid"
  }, /*#__PURE__*/React.createElement(CodeTabs, {
    style: {
      fontSize: "0.95rem"
    },
    tabs: [{
      label: "Python",
      code: 'pip install laterite\n\nimport laterite\nreport = laterite.validate("delivery.ags")\nags = laterite.read("delivery.ags")      # born-typed polars frames\nags["LOCA"]["LOCA_GL"][0]                # → 12.3, a float'
    }, {
      label: "Node",
      code: 'npm install laterite\n\nimport { read, validate } from "laterite";\nconst ags = read("delivery.ags");        // apache-arrow, born-typed\nvalidate("delivery.ags").toJson();'
    }, {
      label: "CLI",
      code: "uvx --from laterite lat validate delivery.ags\n\nlat fix delivery.ags        # → sibling .fixed.ags\nlat diff old.ags new.ags    # KEY-aware revision delta\nlat certify delivery.ags    # mint delivery.ags.idx"
    }, {
      label: "DuckDB",
      code: "INSTALL laterite_ags4 FROM community;\nLOAD laterite_ags4;\n\nSELECT l.loca_id, s.samp_ref, s.samp_top\nFROM read_ags('delivery.ags', 'SAMP') s\nJOIN read_ags('delivery.ags', 'LOCA') l ON s._parent_id = l._id;"
    }, {
      label: "Browser",
      code: 'npm i @laterite/ags4-wasm\n\n// exactly what this page does — nothing leaves the client\nimport init, { validate } from "@laterite/ags4-wasm";'
    }]
  }), /*#__PURE__*/React.createElement("div", {
    className: "installside"
  }, /*#__PURE__*/React.createElement("div", {
    className: "linkcard"
  }, /*#__PURE__*/React.createElement("h3", null, "Do the whole job in the browser"), /*#__PURE__*/React.createElement("p", null, "The web app is this demo's grown-up sibling: validate, fix, explore with DuckDB, convert Excel, diff revisions, anonymise, certify."), /*#__PURE__*/React.createElement("a", {
    className: "btnprim",
    href: "https://app.laterite.dev/"
  }, "app.laterite.dev")), /*#__PURE__*/React.createElement("ul", {
    className: "linklist"
  }, /*#__PURE__*/React.createElement("li", null, /*#__PURE__*/React.createElement("a", {
    href: "https://docs.laterite.dev/cookbook/"
  }, "Cookbook \u2014 every task, all four surfaces")), /*#__PURE__*/React.createElement("li", null, /*#__PURE__*/React.createElement("a", {
    href: "https://docs.laterite.dev/reference/cheatsheet/"
  }, "Python cheatsheet")), /*#__PURE__*/React.createElement("li", null, /*#__PURE__*/React.createElement("a", {
    href: "https://docs.laterite.dev/reference/groups/"
  }, "Group catalogue \u2014 174 groups")), /*#__PURE__*/React.createElement("li", null, /*#__PURE__*/React.createElement("a", {
    href: "https://docs.laterite.dev/cookbook/compat/"
  }, "Drop-in for python-ags4")))))), /*#__PURE__*/React.createElement("footer", {
    className: "foot"
  }, /*#__PURE__*/React.createElement("span", null, "laterite \u2014 MIT-licensed AGS4 tooling. Built from the published AGS4 specification, not adapted from another library."), /*#__PURE__*/React.createElement("span", {
    className: "gspacer"
  }), /*#__PURE__*/React.createElement("a", {
    href: "https://github.com/niko86/laterite"
  }, "GitHub"), /*#__PURE__*/React.createElement("a", {
    href: "https://pypi.org/project/laterite/"
  }, "PyPI"), /*#__PURE__*/React.createElement("a", {
    href: "https://www.npmjs.com/package/laterite"
  }, "npm"), /*#__PURE__*/React.createElement("a", {
    href: "https://docs.laterite.dev/feedback/"
  }, "Feedback")));
}
ReactDOM.createRoot(document.getElementById("root")).render(/*#__PURE__*/React.createElement(DemoSite, null));
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/demo-site/DemoSite.jsx", error: String((e && e.message) || e) }); }

// ui_kits/demo-site/GroupTable.jsx
try { (() => {
const {
  Button,
  Chip,
  Icon,
  Tooltip
} = window.LateriteDesignSystem_9f517b;

// A group table with an editing ROW rather than a floating editor: a cell is a
// read view (one line, ellipsis) and clicking it opens a full-width panel
// directly under that row. The panel can be as wide as the card, so a long
// description is fully visible while typing — a floating box would be clipped by
// the table's own horizontal scroll container.
//
// Wrapping in the panel is soft (reading only). A real line break is inserted
// with ⏎ and rendered as a ␍␊ token everywhere the value appears, because an
// AGS4 DATA record is one line — the validator says so (Rule 4).
function Tokens({
  value
}) {
  const parts = String(value ?? "").split("\n");
  return /*#__PURE__*/React.createElement(React.Fragment, null, parts.map((p, i) => /*#__PURE__*/React.createElement(React.Fragment, {
    key: i
  }, i ? /*#__PURE__*/React.createElement("span", {
    className: "crlf",
    title: "A line break inside a field \u2014 AGS4 records are one line"
  }, "\u240D\u240A") : null, p)));
}
function GroupTable({
  group,
  findings,
  onCell,
  onAddRow,
  onDeleteRow,
  showMarkup,
  onToggleMarkup
}) {
  const [edit, setEdit] = React.useState(null);
  const flagged = {};
  for (const f of findings) if (f.group === group.code && f.row != null && f.col != null) {
    const k = f.row + ":" + f.col;
    if (!flagged[k] || f.severity === "error") flagged[k] = f;
  }
  const tone = {
    error: "var(--err)",
    warning: "var(--warn)",
    fyi: "var(--info)"
  };
  const quiet = {
    error: "var(--err-quiet)",
    warning: "var(--warn-quiet)",
    fyi: "var(--chip)"
  };
  const cols = group.headings.length + 2;
  const grow = el => {
    if (!el) return;
    el.style.height = "auto";
    el.style.height = Math.min(el.scrollHeight, 200) + "px";
  };
  return /*#__PURE__*/React.createElement("div", {
    className: "gcard"
  }, /*#__PURE__*/React.createElement("div", {
    className: "ghead"
  }, /*#__PURE__*/React.createElement("span", {
    className: "gcode"
  }, group.code), /*#__PURE__*/React.createElement("span", {
    className: "gtitle"
  }, group.title), group.parent ? /*#__PURE__*/React.createElement(Chip, {
    tone: "muted",
    sentence: true
  }, "child of ", group.parent) : /*#__PURE__*/React.createElement(Chip, {
    tone: "accent"
  }, "root"), /*#__PURE__*/React.createElement("span", {
    className: "gspacer"
  }), /*#__PURE__*/React.createElement(Tooltip, {
    tip: "The GROUP / HEADING / UNIT / TYPE rows exactly as they appear in the file"
  }, /*#__PURE__*/React.createElement("button", {
    type: "button",
    className: "gmarkup",
    onClick: onToggleMarkup,
    "aria-pressed": showMarkup
  }, showMarkup ? "Hide" : "Show", " markup rows"))), /*#__PURE__*/React.createElement("div", {
    className: "gscroll"
  }, /*#__PURE__*/React.createElement("table", {
    className: "gtable"
  }, /*#__PURE__*/React.createElement("thead", null, showMarkup ? /*#__PURE__*/React.createElement(React.Fragment, null, /*#__PURE__*/React.createElement("tr", {
    className: "mrow"
  }, /*#__PURE__*/React.createElement("th", {
    className: "mtag"
  }, "GROUP"), /*#__PURE__*/React.createElement("th", {
    className: "mval",
    colSpan: cols - 1
  }, group.code)), /*#__PURE__*/React.createElement("tr", {
    className: "mrow"
  }, /*#__PURE__*/React.createElement("td", {
    className: "mtag"
  }, "HEADING"), group.headings.map(h => /*#__PURE__*/React.createElement("td", {
    key: h.h,
    className: "mono mhead"
  }, h.h, h.key ? /*#__PURE__*/React.createElement("span", {
    className: "keydot",
    title: "KEY field"
  }, "\u25C6") : null)), /*#__PURE__*/React.createElement("td", null)), /*#__PURE__*/React.createElement("tr", {
    className: "mrow"
  }, /*#__PURE__*/React.createElement("td", {
    className: "mtag"
  }, "UNIT"), group.headings.map(h => /*#__PURE__*/React.createElement("td", {
    key: h.h,
    className: "mono munit"
  }, h.unit || "—")), /*#__PURE__*/React.createElement("td", null)), /*#__PURE__*/React.createElement("tr", {
    className: "mrow"
  }, /*#__PURE__*/React.createElement("td", {
    className: "mtag"
  }, "TYPE"), group.headings.map(h => /*#__PURE__*/React.createElement("td", {
    key: h.h,
    className: "mono mtype"
  }, h.type)), /*#__PURE__*/React.createElement("td", null))) : /*#__PURE__*/React.createElement("tr", {
    className: "mrow"
  }, /*#__PURE__*/React.createElement("td", {
    className: "mtag"
  }, "DATA"), group.headings.map(h => /*#__PURE__*/React.createElement("td", {
    key: h.h,
    className: "mono mhead"
  }, h.h, h.key ? /*#__PURE__*/React.createElement("span", {
    className: "keydot",
    title: "KEY field"
  }, "\u25C6") : null, /*#__PURE__*/React.createElement("span", {
    className: "inlineunit"
  }, h.unit ? "(" + h.unit + ")" : "", " ", h.type))), /*#__PURE__*/React.createElement("td", null))), /*#__PURE__*/React.createElement("tbody", null, group.rows.map((row, i) => /*#__PURE__*/React.createElement("tr", {
    key: i
  }, showMarkup ? /*#__PURE__*/React.createElement("td", {
    className: "mtag"
  }, "DATA") : /*#__PURE__*/React.createElement("td", {
    className: "rown"
  }, i + 1), group.headings.map((h, c) => {
    const bad = flagged[i + ":" + c];
    const on = edit && edit.row === i && edit.col === c;
    return /*#__PURE__*/React.createElement("td", {
      key: h.h,
      className: "cell" + (h.key ? " keycell" : "")
    }, /*#__PURE__*/React.createElement("div", {
      role: "button",
      tabIndex: 0,
      className: "cview" + (bad ? " cbad" : "") + (on ? " cviewon" : ""),
      "aria-label": h.h + " row " + (i + 1),
      title: bad ? bad.rule + " — " + bad.desc : "Click to edit",
      style: bad ? {
        borderBottomColor: tone[bad.severity],
        background: quiet[bad.severity]
      } : null,
      onClick: () => setEdit({
        row: i,
        col: c
      }),
      onKeyDown: e => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          setEdit({
            row: i,
            col: c
          });
        }
      }
    }, String(row[c] ?? "").length ? /*#__PURE__*/React.createElement(Tokens, {
      value: row[c]
    }) : /*#__PURE__*/React.createElement("span", {
      className: "cempty"
    }, "\u2014")));
  }), /*#__PURE__*/React.createElement("td", {
    className: "rowact"
  }, group.one && group.rows.length === 1 ? null : /*#__PURE__*/React.createElement("button", {
    type: "button",
    className: "ghostx",
    onClick: () => onDeleteRow(group.code, i),
    "aria-label": "Delete row " + (i + 1)
  }, /*#__PURE__*/React.createElement(Icon, {
    name: "trash-2",
    size: 15
  })))))))), edit ? (() => {
    const h = group.headings[edit.col];
    const val = group.rows[edit.row]?.[edit.col] ?? "";
    return /*#__PURE__*/React.createElement("div", {
      className: "epanel"
    }, /*#__PURE__*/React.createElement("div", {
      className: "ehead"
    }, /*#__PURE__*/React.createElement("span", {
      className: "mono elabel"
    }, group.code, ".", h.h), /*#__PURE__*/React.createElement("span", {
      className: "emeta"
    }, "\xB7 row ", edit.row + 1, " \xB7 TYPE ", h.type, h.unit ? " · " + h.unit : "", h.key ? " · KEY" : ""), /*#__PURE__*/React.createElement("button", {
      type: "button",
      className: "edone",
      onClick: () => setEdit(null)
    }, "Done")), /*#__PURE__*/React.createElement("textarea", {
      className: "eta",
      value: val,
      spellCheck: false,
      "aria-label": "Edit " + h.h + " row " + (edit.row + 1),
      ref: el => {
        if (el && document.activeElement !== el) {
          el.focus();
          grow(el);
        }
      },
      onChange: e => {
        onCell(group.code, edit.row, edit.col, e.target.value);
        grow(e.target);
      },
      onKeyDown: e => {
        if (e.key === "Escape") setEdit(null);
      }
    }), /*#__PURE__*/React.createElement("p", {
      className: "ehint"
    }, "Wrapped here for reading \u2014 it is still ", /*#__PURE__*/React.createElement("strong", null, "one field"), ". ", /*#__PURE__*/React.createElement("kbd", null, "\u23CE"), " inserts a real line break, shown as ", /*#__PURE__*/React.createElement("span", {
      className: "crlf"
    }, "\u240D\u240A"), " and reported as a Rule 4 error, since an AGS4 DATA record is a single line. ", /*#__PURE__*/React.createElement("kbd", null, "Esc"), " closes."));
  })() : null, group.one && group.rows.length >= 1 ? null : /*#__PURE__*/React.createElement("div", {
    className: "gfoot"
  }, /*#__PURE__*/React.createElement(Button, {
    variant: "add",
    onClick: () => onAddRow(group.code),
    style: {
      fontSize: "0.95rem",
      padding: "0.3rem 0.7rem"
    }
  }, "+ Add a ", group.code, " row"), /*#__PURE__*/React.createElement("span", {
    className: "ghint"
  }, "Click any cell to edit it \u2014 the file and the findings below update as you type.")));
}
Object.assign(window, {
  GroupTable
});
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/demo-site/GroupTable.jsx", error: String((e && e.message) || e) }); }

// ui_kits/demo-site/agsModel.jsx
try { (() => {
// A small AGS4 engine standing in for @laterite/ags4-wasm: emit, validate, fix.
// Rule numbers/wording follow the laterite CLI's findings so the demo tells the
// truth about what the real engine reports.

const ABBR = {
  LOCA_TYPE: ["CP", "RC", "TP", "WS", "CH"],
  SAMP_TYPE: ["U", "D", "B", "W", "PS"]
};
const SEED = [{
  code: "PROJ",
  title: "Project information",
  one: true,
  blurb: "Every delivery opens with PROJ — one row, naming the job. The three markup rows above the data are what makes AGS more than a CSV: HEADING names each column, UNIT gives it a dimension, TYPE declares what the value IS. That TYPE is why laterite can hand you a float instead of the string \"12.30\".",
  headings: [{
    h: "PROJ_ID",
    unit: "",
    type: "ID",
    key: true
  }, {
    h: "PROJ_NAME",
    unit: "",
    type: "X",
    req: true
  }, {
    h: "PROJ_CLNT",
    unit: "",
    type: "X"
  }, {
    h: "PROJ_ENG",
    unit: "",
    type: "X"
  }],
  rows: [["P2246", "A2 Widening — Ground Investigation", "National Highways", "Geotech Ltd"]]
}, {
  code: "LOCA",
  title: "Location details",
  parent: "PROJ",
  link: {
    keys: [],
    note: "one PROJ per file — every LOCA belongs to it"
  },
  blurb: "LOCA is one row per borehole, trial pit or probe. LOCA_ID is its KEY — everything downstream points back at it. Watch the coordinate and level columns: they are TYPE 2DP, so a value with one decimal place is a rule 5 error, not a rounding preference. Edit a level below and the findings panel answers immediately.",
  headings: [{
    h: "LOCA_ID",
    unit: "",
    type: "ID",
    key: true
  }, {
    h: "LOCA_TYPE",
    unit: "",
    type: "PA",
    req: true
  }, {
    h: "LOCA_NATE",
    unit: "m",
    type: "2DP"
  }, {
    h: "LOCA_NATN",
    unit: "m",
    type: "2DP"
  }, {
    h: "LOCA_GL",
    unit: "m",
    type: "2DP",
    req: true
  }, {
    h: "LOCA_FDEP",
    unit: "m",
    type: "2DP"
  }],
  rows: [["BH01", "CP", "523456.12", "187654.33", "12.30", "25.00"], ["BH02", "CP", "523501.44", "187690.10", "11.8", "24.50"], ["TP03", "TP", "523560.02", "187722.87", "10.95", "4.20"]]
}, {
  code: "SAMP",
  title: "Sample information",
  parent: "LOCA",
  link: {
    keys: ["LOCA_ID"],
    note: "one hole, many samples"
  },
  blurb: "SAMP hangs off LOCA by repeating its key, then adds its own: depth, reference and type. In the file that relationship is nothing but matching columns — no ids, no joins. laterite mints a content-addressed _id / _parent_id over those keys, which is how the web app and the DuckDB extension join parent to child without you writing a key list.",
  headings: [{
    h: "LOCA_ID",
    unit: "",
    type: "ID",
    key: true,
    fk: "LOCA.LOCA_ID"
  }, {
    h: "SAMP_TOP",
    unit: "m",
    type: "2DP",
    key: true
  }, {
    h: "SAMP_REF",
    unit: "",
    type: "X",
    key: true
  }, {
    h: "SAMP_TYPE",
    unit: "",
    type: "PA",
    req: true
  }, {
    h: "SAMP_DESC",
    unit: "",
    type: "X"
  }],
  rows: [["BH01", "1.50", "S1", "U", "Firm brown slightly sandy CLAY"], ["BH01", "4.00", "S2", "U", "Stiff grey CLAY with gravel"], ["BH02", "2.50", "S3", "u", "Soft grey organic CLAY"]]
}, {
  code: "LLPL",
  title: "Liquid and plastic limit tests",
  parent: "SAMP",
  link: {
    keys: ["LOCA_ID", "SAMP_TOP", "SAMP_REF"],
    note: "one sample, many tests"
  },
  blurb: "A lab group is the end of the chain: it repeats the whole sample key and adds results. Because the key is three columns wide, a single typo orphans the result — a rule 10 error the engine catches before your client does. This is the shape of every test group in AGS, from ISPT to ERES.",
  headings: [{
    h: "LOCA_ID",
    unit: "",
    type: "ID",
    key: true,
    fk: "SAMP.LOCA_ID"
  }, {
    h: "SAMP_TOP",
    unit: "m",
    type: "2DP",
    key: true
  }, {
    h: "SAMP_REF",
    unit: "",
    type: "X",
    key: true
  }, {
    h: "LLPL_LL",
    unit: "%",
    type: "2DP"
  }, {
    h: "LLPL_PL",
    unit: "%",
    type: "2DP"
  }, {
    h: "LLPL_425",
    unit: "%",
    type: "2DP"
  }],
  rows: [["BH01", "1.50", "S1", "48.00", "22.00", ""], ["BH01", "4.00", "S2", "61.00", "27.00", ""]]
}];
const clone = g => g.map(x => ({
  ...x,
  rows: x.rows.map(r => r.slice())
}));
const q = s => '"' + String(s ?? "").replace(/"/g, '""') + '"';
const TRAN = {
  code: "TRAN",
  title: "Transmission information",
  headings: [{
    h: "TRAN_ISNO",
    unit: "",
    type: "X"
  }, {
    h: "TRAN_DATE",
    unit: "",
    type: "DT"
  }, {
    h: "TRAN_AGS",
    unit: "",
    type: "X"
  }],
  rows: [["1", "2026-08-17", "4.1.1"]]
};

/** Emit AGS4 text plus a line index, so a finding can point at a real line. */
function emit(groups, withTran) {
  const lines = [];
  const add = (text, meta) => lines.push({
    n: lines.length + 1,
    text,
    ...meta
  });
  const order = [];
  for (const g of groups) {
    order.push(g);
    if (withTran && g.code === "PROJ") order.push(TRAN);
  }
  for (const g of order) {
    add('"GROUP",' + q(g.code), {
      group: g.code,
      kind: "GROUP"
    });
    add('"HEADING",' + g.headings.map(h => q(h.h)).join(","), {
      group: g.code,
      kind: "HEADING"
    });
    add('"UNIT",' + g.headings.map(h => q(h.unit)).join(","), {
      group: g.code,
      kind: "UNIT"
    });
    add('"TYPE",' + g.headings.map(h => q(h.type)).join(","), {
      group: g.code,
      kind: "TYPE"
    });
    g.rows.forEach((r, i) => add('"DATA",' + g.headings.map((h, c) => q(r[c])).join(","), {
      group: g.code,
      kind: "DATA",
      row: i
    }));
  }
  return {
    text: lines.map(l => l.text).join("\n"),
    lines
  };
}
const is2DP = v => /^-?\d+\.\d{2}$/.test(v);
const numeric = v => /^-?\d*\.?\d+$/.test(v.trim());

/** The rule pass. Every finding carries where it came from (group/row/col) so
 *  the table can flag the cell and the output can highlight the line. */
function validate(groups, withTran) {
  const out = [];
  const push = (rule, severity, desc, at, fix) => out.push({
    rule,
    severity,
    desc,
    ...at,
    fix: fix || null
  });
  const byCode = Object.fromEntries(groups.map(g => [g.code, g]));
  if (!withTran) {
    push("Rule 17 — TRAN group", "error", "Required group TRAN is absent — the dictionary edition cannot be read from the file, so 4.1.1 was assumed.", {
      group: "PROJ",
      kind: "GROUP"
    }, "tran");
  }
  for (const g of groups) {
    g.headings.forEach((h, c) => {
      const allBlank = g.rows.length && g.rows.every(r => !String(r[c] ?? "").trim());
      if (allBlank) push("FYI — empty column", "fyi", `${h.h} is declared but blank for every DATA row.`, {
        group: g.code,
        col: c
      });
    });
    g.rows.forEach((r, i) => {
      g.headings.forEach((h, c) => {
        const raw = String(r[c] ?? "");
        const v = raw.trim();
        const at = {
          group: g.code,
          row: i,
          col: c,
          kind: "DATA"
        };
        if (raw !== v && raw.length) push("Rule 3 — field format", "warning", `${h.h} has leading or trailing whitespace.`, at, "trim");
        if (raw.indexOf("\n") >= 0) push("Rule 4 — one line per record", "error", `${h.h} contains a line break — an AGS4 DATA record is a single line, so this field would split the row.`, at, "unwrap");
        if (!v) {
          if (h.key) push("Rule 10 — KEY fields", "error", `${h.h} is a KEY field and must not be blank.`, at);else if (h.req) push("Rule 10 — required field", "warning", `${h.h} is required by the dictionary but blank.`, at);
          return;
        }
        if (h.type === "2DP" && !is2DP(v)) {
          push("Rule 5 — data type", numeric(v) ? "error" : "error", `${h.h} value '${v}' does not match TYPE 2DP — two decimal places expected.`, at, numeric(v) ? "pad2dp" : null);
        }
        if (h.type === "DT" && !/^\d{4}-\d{2}-\d{2}/.test(v)) {
          push("Rule 8 — date format", "error", `${h.h} value '${v}' is not an ISO 8601 date.`, at);
        }
        if (h.type === "PA" && ABBR[h.h] && !ABBR[h.h].includes(v)) {
          const upper = ABBR[h.h].includes(v.toUpperCase());
          push("Rule 16 — abbreviation", upper ? "warning" : "error", upper ? `${h.h} value '${v}' matches '${v.toUpperCase()}' in ABBR but the case differs.` : `${h.h} value '${v}' is not listed in ABBR for this heading.`, at, upper ? "upper" : null);
        }
        if (h.fk) {
          const [pg] = h.fk.split(".");
          const parent = byCode[pg];
          if (parent) {
            const keyCols = parent.headings.map((ph, pc) => ({
              ph,
              pc
            })).filter(x => x.ph.key);
            const mine = g.headings.map((gh, gc) => ({
              gh,
              gc
            })).filter(x => x.gh.key);
            const shared = mine.filter(m => keyCols.some(k => k.ph.h === m.gh.h));
            const match = parent.rows.some(pr => shared.every(s => {
              const pc = parent.headings.findIndex(ph => ph.h === s.gh.h);
              return String(pr[pc] ?? "").trim() === String(r[s.gc] ?? "").trim();
            }));
            if (!match && c === 0) {
              push("Rule 10 — KEY fields", "error", `No parent record in ${pg} for ${shared.map(s => String(r[s.gc]).trim()).join(" · ")} — the row is orphaned.`, at);
            }
          }
        }
      });
    });
  }
  return out;
}

/** Apply every fixable finding — the same "safe fixes only" contract as `lat fix`. */
function applyFixes(groups, findings, withTran) {
  const next = clone(groups);
  let tran = withTran;
  for (const fnd of findings) {
    if (fnd.fix === "tran") {
      tran = true;
      continue;
    }
    const g = next.find(x => x.code === fnd.group);
    if (!g || fnd.row == null || fnd.col == null) continue;
    const v = String(g.rows[fnd.row][fnd.col] ?? "");
    if (fnd.fix === "trim") g.rows[fnd.row][fnd.col] = v.trim();
    if (fnd.fix === "unwrap") g.rows[fnd.row][fnd.col] = v.replace(/\r?\n/g, " ").replace(/\s{2,}/g, " ").trim();
    if (fnd.fix === "upper") g.rows[fnd.row][fnd.col] = v.trim().toUpperCase();
    if (fnd.fix === "pad2dp") g.rows[fnd.row][fnd.col] = Number(v.trim()).toFixed(2);
  }
  return {
    groups: next,
    withTran: tran
  };
}
Object.assign(window, {
  SEED,
  ABBR,
  clone,
  emit,
  validate,
  applyFixes
});
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/demo-site/agsModel.jsx", error: String((e && e.message) || e) }); }

// ui_kits/docs/Chrome.jsx
try { (() => {
const {
  Button,
  Chip,
  Input,
  Icon,
  ThemeToggle,
  Tooltip
} = window.LateriteDesignSystem_9f517b;
const NAV = [{
  section: "Home",
  items: [["Home", "home"]]
}, {
  section: "Learn",
  items: [["1. Install & first validate", "install"], ["2. Read & explore a file", "read"], ["3. Validate in Python", "validate"], ["4. Query across groups", "query"], ["5. Produce AGS4", "produce"]]
}, {
  section: "Cookbook",
  items: [["Validate a delivery", "recipe"], ["SQL across groups", "sql"], ["Fix a dirty file", "fix"], ["Diff two revisions", "diff"], ["Certify a clean file", "certify"]]
}, {
  section: "Reference",
  items: [["CLI (lat)", "cli"], ["Python API", "api"], ["Group catalogue", "catalogue"], ["Known divergences", "divergences"]]
}, {
  section: "Surfaces",
  items: [["One engine, every stack", "surfaces"], ["Python", "python"], ["Node", "node"], ["DuckDB", "duckdb"], ["Browser (web app)", "browser"]]
}];
function Header({
  mode,
  setMode
}) {
  return /*#__PURE__*/React.createElement("header", {
    style: {
      position: "sticky",
      top: 0,
      zIndex: 30,
      background: "var(--stone-900)",
      color: "#fff"
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: "flex",
      alignItems: "center",
      gap: "var(--space-5)",
      maxWidth: "var(--prose-max)",
      margin: "0 auto",
      padding: "0.6rem var(--space-6)"
    }
  }, /*#__PURE__*/React.createElement("a", {
    href: "#",
    style: {
      display: "flex",
      alignItems: "center",
      gap: "0.5rem",
      color: "#fff",
      textDecoration: "none"
    }
  }, /*#__PURE__*/React.createElement("span", {
    style: {
      display: "inline-flex",
      background: "var(--stone-50)",
      borderRadius: "var(--radius-md)",
      padding: 3
    }
  }, /*#__PURE__*/React.createElement("img", {
    src: "../../assets/laterite-icon-256.png",
    alt: "",
    style: {
      height: 24,
      display: "block"
    }
  })), /*#__PURE__*/React.createElement("span", {
    style: {
      fontFamily: "var(--font-display)",
      fontSize: "var(--text-title)",
      fontWeight: 800,
      letterSpacing: "var(--tracking-tight)"
    }
  }, "laterite")), /*#__PURE__*/React.createElement("span", {
    style: {
      fontSize: "var(--text-micro)",
      color: "var(--laterite-300)"
    }
  }, "0.9.4 \xB7 beta"), /*#__PURE__*/React.createElement("div", {
    style: {
      marginLeft: "auto",
      display: "flex",
      alignItems: "center",
      gap: "var(--space-3)"
    }
  }, /*#__PURE__*/React.createElement("span", {
    style: {
      position: "relative",
      display: "flex",
      alignItems: "center"
    }
  }, /*#__PURE__*/React.createElement("input", {
    className: "dsearch",
    placeholder: "Search docs\u2026"
  }), /*#__PURE__*/React.createElement("span", {
    style: {
      position: "absolute",
      left: "0.5rem",
      color: "var(--fg-faint)",
      display: "inline-flex"
    }
  }, /*#__PURE__*/React.createElement(Icon, {
    name: "search",
    size: 14
  }))), /*#__PURE__*/React.createElement("a", {
    href: "https://github.com/niko86/laterite",
    style: {
      color: "rgb(255 255 255 / 80%)",
      display: "inline-flex"
    }
  }, /*#__PURE__*/React.createElement(Icon, {
    name: "github",
    size: 17
  })))), /*#__PURE__*/React.createElement("div", {
    "aria-hidden": "true",
    style: {
      height: 3,
      background: "linear-gradient(90deg, var(--laterite-300), var(--laterite-400) 35%, var(--laterite-500) 65%, var(--laterite-700))"
    }
  }));
}
function Sidebar({
  page,
  onNav
}) {
  return /*#__PURE__*/React.createElement("nav", {
    style: {
      fontSize: "var(--text-caption)",
      paddingRight: "var(--space-4)",
      borderRight: "1px solid var(--line)"
    }
  }, NAV.map(s => /*#__PURE__*/React.createElement("div", {
    key: s.section,
    style: {
      marginBottom: "var(--space-5)"
    }
  }, /*#__PURE__*/React.createElement("div", {
    className: "lbl",
    style: {
      marginBottom: "0.35rem"
    }
  }, s.section), s.items.map(([label, id]) => /*#__PURE__*/React.createElement("button", {
    key: id,
    type: "button",
    onClick: () => onNav(id),
    style: {
      font: "inherit",
      fontFamily: "var(--font-ui)",
      display: "block",
      width: "100%",
      textAlign: "left",
      border: "none",
      background: "none",
      cursor: "pointer",
      padding: "0.18rem 0.4rem",
      borderLeft: `2px solid ${page === id ? "var(--accent)" : "transparent"}`,
      color: page === id ? "var(--accent)" : "var(--fg-muted)",
      fontWeight: page === id ? "var(--weight-semibold)" : "var(--weight-regular)"
    }
  }, label)))));
}
function Toc({
  items
}) {
  return /*#__PURE__*/React.createElement("aside", {
    style: {
      fontSize: "var(--text-micro)",
      borderLeft: "1px solid var(--line)",
      paddingLeft: "var(--space-4)"
    }
  }, /*#__PURE__*/React.createElement("div", {
    className: "lbl",
    style: {
      marginBottom: "0.4rem"
    }
  }, "on this page"), items.map((t, i) => /*#__PURE__*/React.createElement("div", {
    key: t,
    style: {
      padding: "0.12rem 0",
      color: i === 0 ? "var(--accent)" : "var(--fg-muted)"
    }
  }, t)));
}
Object.assign(window, {
  Header,
  Sidebar,
  Toc,
  NAV
});
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/docs/Chrome.jsx", error: String((e && e.message) || e) }); }

// ui_kits/docs/Pages.jsx
try { (() => {
const {
  Button,
  Chip,
  CodeTabs,
  Admonition,
  Card,
  Input,
  Icon,
  PillToggle
} = window.LateriteDesignSystem_9f517b;
function HomePage() {
  return /*#__PURE__*/React.createElement("div", {
    className: "prose"
  }, /*#__PURE__*/React.createElement("h1", null, "laterite"), /*#__PURE__*/React.createElement("p", null, "laterite reads, validates, queries and ", /*#__PURE__*/React.createElement("strong", null, "produces"), " AGS4 geotechnical data. Files come back as ", /*#__PURE__*/React.createElement("strong", null, "born-typed"), " polars frames \u2014 the polars dtype ", /*#__PURE__*/React.createElement("em", null, "is"), " the AGS type \u2014 wired into a fluent, chainable API. One engine drives Python, the ", /*#__PURE__*/React.createElement("code", null, "lat"), " CLI, Node and DuckDB, and it's a drop-in for python-ags4, rebuilt on a Rust core for speed."), /*#__PURE__*/React.createElement(CodeTabs, {
    tabs: [{
      label: "Python",
      code: "pip install laterite"
    }, {
      label: "Node",
      code: "npm install laterite"
    }, {
      label: "CLI",
      code: "uvx --from laterite lat validate delivery.ags"
    }, {
      label: "DuckDB",
      code: "INSTALL laterite_ags4 FROM community;\nLOAD laterite_ags4;"
    }]
  }), /*#__PURE__*/React.createElement("p", {
    style: {
      marginTop: "var(--space-5)"
    }
  }, "laterite is ", /*#__PURE__*/React.createElement("a", {
    href: "#"
  }, "in beta"), " \u2014 the engine is tested, what it hasn't had is your files. ", /*#__PURE__*/React.createElement("a", {
    href: "#"
  }, "Tell us how it goes"), "."), /*#__PURE__*/React.createElement("h2", null, "In one breath"), /*#__PURE__*/React.createElement(CodeTabs, {
    tabs: [{
      label: "Python",
      code: 'import laterite\n\nags = laterite.read("delivery.ags").validate()\nags.report.is_valid\nags["LOCA"]["LOCA_GL"][0]   # → 12.3, a float — not "12.30"'
    }]
  }), /*#__PURE__*/React.createElement("p", null, /*#__PURE__*/React.createElement("code", null, "read(...)"), " gives you an ", /*#__PURE__*/React.createElement("code", null, "Ags4File"), "; ", /*#__PURE__*/React.createElement("code", null, ".validate()"), " runs the numbered-rules engine and hands the file straight back so the chain keeps flowing, with the verdict on ", /*#__PURE__*/React.createElement("code", null, ".report"), ". The dictionary edition (", /*#__PURE__*/React.createElement("code", null, "4.1.1"), ") is picked automatically from the file's ", /*#__PURE__*/React.createElement("code", null, "TRAN_AGS"), " row \u2014 no flags, no guessing."), /*#__PURE__*/React.createElement(Admonition, {
    kind: "tip"
  }, "Every frame is born typed. A ", /*#__PURE__*/React.createElement("code", null, "2DP"), " column is a polars ", /*#__PURE__*/React.createElement("code", null, "Float64"), ", a date is a ", /*#__PURE__*/React.createElement("code", null, "Date"), ", an ", /*#__PURE__*/React.createElement("code", null, "ID"), " is a ", /*#__PURE__*/React.createElement("code", null, "String"), " \u2014 so ", /*#__PURE__*/React.createElement("code", null, ".query(...)"), ", ", /*#__PURE__*/React.createElement("code", null, ".sql(...)"), " and plain polars all see real types, not text."), /*#__PURE__*/React.createElement("h2", null, "Where to go next"), /*#__PURE__*/React.createElement("ul", null, /*#__PURE__*/React.createElement("li", null, /*#__PURE__*/React.createElement("strong", null, "New here? \u2192 ", /*#__PURE__*/React.createElement("a", {
    href: "#"
  }, "Learn")), " \u2014 install, then read \u2192 validate \u2192 query \u2192 produce, one step at a time."), /*#__PURE__*/React.createElement("li", null, /*#__PURE__*/React.createElement("strong", null, "Need to get something done? \u2192 ", /*#__PURE__*/React.createElement("a", {
    href: "#"
  }, "Cookbook")), " \u2014 task-shaped recipes you can lift wholesale."), /*#__PURE__*/React.createElement("li", null, /*#__PURE__*/React.createElement("strong", null, "Show me what it can do? \u2192 ", /*#__PURE__*/React.createElement("a", {
    href: "#"
  }, "Chaining")), " \u2014 the fluent API end to end, one chain at a time."), /*#__PURE__*/React.createElement("li", null, /*#__PURE__*/React.createElement("strong", null, "Looking up a function? \u2192 ", /*#__PURE__*/React.createElement("a", {
    href: "#"
  }, "Reference")), " \u2014 the cheatsheet and the ", /*#__PURE__*/React.createElement("code", null, "lat"), " CLI.")));
}
function RecipePage() {
  return /*#__PURE__*/React.createElement("div", {
    className: "prose"
  }, /*#__PURE__*/React.createElement("h1", null, "Validate a delivery"), /*#__PURE__*/React.createElement("p", null, "Run the numbered AGS4 rules over a file and act on the verdict. Errors and warnings are reported by default; FYI findings are opt-in."), /*#__PURE__*/React.createElement(CodeTabs, {
    tabs: [{
      label: "Python",
      code: 'import laterite\n\nreport = laterite.validate("delivery.ags")\n\nif not report.is_valid:\n    for finding in report.findings:\n        print(finding.rule, finding.line, finding.desc)'
    }, {
      label: "Node",
      code: 'import { validate } from "laterite";\n\nconst report = validate("delivery.ags");\nconsole.log(report.toJson());'
    }, {
      label: "CLI",
      code: "lat validate delivery.ags --json\nlat validate delivery.ags --no-warnings"
    }, {
      label: "DuckDB",
      code: "SELECT rule, line, desc\nFROM ags_validate('delivery.ags')\nWHERE severity = 'error';"
    }]
  }), /*#__PURE__*/React.createElement("h3", null, "Output"), /*#__PURE__*/React.createElement("pre", {
    className: "mono",
    style: {
      background: "var(--surface-code)",
      border: "1px solid var(--line)",
      borderRadius: "var(--radius-lg)",
      padding: "var(--space-4)",
      fontSize: "var(--text-control)",
      overflowX: "auto",
      color: "var(--fg-soft)"
    }
  }, 'Rule 5   line 11   LOCA_GL "11.8" does not match TYPE 2DP\nRule 7   line 8    LOCA_NATE UNIT expected "m"\nRule 17  line 1    required group TRAN is absent\n\n2 errors · 1 warning · 1 informational  (AGS 4.1.1, fallback)'), /*#__PURE__*/React.createElement(Admonition, {
    kind: "note",
    title: "Exit codes"
  }, /*#__PURE__*/React.createElement("code", null, "0"), " clean \xB7 ", /*#__PURE__*/React.createElement("code", null, "1"), " findings \xB7 ", /*#__PURE__*/React.createElement("code", null, "3"), " unreadable \xB7 ", /*#__PURE__*/React.createElement("code", null, "4"), " not AGS4 \xB7 ", /*#__PURE__*/React.createElement("code", null, "5"), " bad args \xB7 ", /*#__PURE__*/React.createElement("code", null, "6"), " schema \u2014 so a CI gate is just ", /*#__PURE__*/React.createElement("code", null, "lat validate delivery.ags"), "."), /*#__PURE__*/React.createElement("h2", null, "See also"), /*#__PURE__*/React.createElement("ul", null, /*#__PURE__*/React.createElement("li", null, /*#__PURE__*/React.createElement("a", {
    href: "#"
  }, "Fix a dirty file"), " \u2014 repair the safe findings automatically."), /*#__PURE__*/React.createElement("li", null, /*#__PURE__*/React.createElement("a", {
    href: "#"
  }, "Severity tiers"), " \u2014 what counts as an error, a warning, an FYI."), /*#__PURE__*/React.createElement("li", null, /*#__PURE__*/React.createElement("a", {
    href: "#"
  }, "Certify a clean file"), " \u2014 mint an ", /*#__PURE__*/React.createElement("code", null, ".ags.idx"), " certificate.")));
}
const CATALOGUE = [["PROJ", "Project information", "Project", 6, "1 row"], ["TRAN", "Transmission information", "Project", 9, "1 row"], ["LOCA", "Location details", "Location", 27, "PROJ"], ["SAMP", "Sample information", "Sampling", 24, "LOCA"], ["GEOL", "Field geological descriptions", "Geology", 12, "LOCA"], ["ISPT", "In situ standard penetration test", "In situ testing", 15, "LOCA"], ["LLPL", "Liquid and plastic limit tests", "Lab testing", 11, "SAMP"], ["WSTG", "Groundwater strike", "Water", 8, "LOCA"]];
function CataloguePage() {
  const [q, setQ] = React.useState("");
  const [family, setFamily] = React.useState("All");
  const families = ["All", "Project", "Location", "Sampling", "Geology", "In situ testing", "Lab testing", "Water"];
  const rows = CATALOGUE.filter(r => (family === "All" || r[2] === family) && (r[0] + r[1]).toLowerCase().includes(q.toLowerCase()));
  return /*#__PURE__*/React.createElement("div", {
    style: {
      maxWidth: "46rem"
    }
  }, /*#__PURE__*/React.createElement("div", {
    className: "prose"
  }, /*#__PURE__*/React.createElement("h1", null, "Group catalogue"), /*#__PURE__*/React.createElement("p", null, "One page per AGS4 group, generated at build time from the shipped ", /*#__PURE__*/React.createElement("code", null, "laterite.registry"), " \u2014 174 groups in edition 4.1.1.")), /*#__PURE__*/React.createElement("div", {
    style: {
      display: "flex",
      flexWrap: "wrap",
      gap: "var(--space-2)",
      margin: "var(--space-4) 0"
    }
  }, families.map(fam => /*#__PURE__*/React.createElement("button", {
    key: fam,
    type: "button",
    onClick: () => setFamily(fam),
    style: {
      font: "inherit",
      fontSize: "var(--text-micro)",
      fontWeight: 600,
      cursor: "pointer",
      borderRadius: "var(--radius-2xl)",
      padding: "0.1rem 0.55rem",
      border: "1px solid " + (family === fam ? "transparent" : "var(--line-strong)"),
      background: family === fam ? "var(--accent-quiet)" : "transparent",
      color: family === fam ? "var(--accent)" : "var(--fg-muted)"
    }
  }, fam))), /*#__PURE__*/React.createElement(Input, {
    placeholder: "Filter groups \u2014 code or description\u2026",
    value: q,
    onChange: e => setQ(e.target.value),
    style: {
      marginBottom: "var(--space-3)"
    }
  }), /*#__PURE__*/React.createElement("div", {
    style: {
      border: "1px solid var(--line)",
      borderRadius: "var(--radius-lg)",
      overflow: "hidden",
      background: "var(--surface)"
    }
  }, /*#__PURE__*/React.createElement("table", {
    style: {
      fontSize: "var(--text-control)"
    }
  }, /*#__PURE__*/React.createElement("thead", null, /*#__PURE__*/React.createElement("tr", {
    style: {
      background: "var(--surface-raised)"
    }
  }, ["Group", "Description", "Family", "Headings", "Parent"].map(h => /*#__PURE__*/React.createElement("th", {
    key: h,
    style: {
      textAlign: "left",
      padding: "0.35rem 0.75rem",
      borderBottom: "1px solid var(--line)",
      fontSize: "var(--text-micro)",
      textTransform: "uppercase",
      letterSpacing: "var(--tracking-micro)",
      color: "var(--fg-muted)"
    }
  }, h)))), /*#__PURE__*/React.createElement("tbody", null, rows.map(r => /*#__PURE__*/React.createElement("tr", {
    key: r[0],
    style: {
      borderTop: "1px solid var(--line-subtle)"
    }
  }, /*#__PURE__*/React.createElement("td", {
    style: {
      padding: "0.3rem 0.75rem"
    }
  }, /*#__PURE__*/React.createElement("a", {
    href: "#",
    className: "mono"
  }, r[0])), /*#__PURE__*/React.createElement("td", {
    style: {
      padding: "0.3rem 0.75rem",
      color: "var(--fg-soft)"
    }
  }, r[1]), /*#__PURE__*/React.createElement("td", {
    style: {
      padding: "0.3rem 0.75rem"
    }
  }, /*#__PURE__*/React.createElement(Chip, {
    tone: "muted",
    sentence: true
  }, r[2])), /*#__PURE__*/React.createElement("td", {
    style: {
      padding: "0.3rem 0.75rem",
      color: "var(--fg-muted)"
    }
  }, r[3]), /*#__PURE__*/React.createElement("td", {
    className: "mono",
    style: {
      padding: "0.3rem 0.75rem",
      color: "var(--fg-faint)"
    }
  }, r[4])))))), /*#__PURE__*/React.createElement("div", {
    style: {
      display: "flex",
      alignItems: "center",
      gap: "var(--space-3)",
      marginTop: "var(--space-3)",
      fontSize: "var(--text-micro)",
      color: "var(--fg-muted)"
    }
  }, /*#__PURE__*/React.createElement(Button, {
    size: "sm",
    disabled: true
  }, "\u2190 Prev"), /*#__PURE__*/React.createElement(Button, {
    size: "sm"
  }, "Next \u2192"), /*#__PURE__*/React.createElement("span", null, "Showing ", rows.length, " of 174 groups"), /*#__PURE__*/React.createElement("span", {
    style: {
      marginLeft: "auto"
    }
  }, /*#__PURE__*/React.createElement(Button, {
    size: "sm"
  }, "Show all 174"))));
}
function SurfacesPage() {
  const SURFACES = [["Python", "pip install laterite", "polars frames, the typed graph, the python-ags4 drop-in", "package"], ["Node.js", "npm install laterite", "apache-arrow tables, server-side validation", "hexagon"], ["CLI — lat", "npx laterite", "pipelines, CI gates, one-off checks", "terminal"], ["DuckDB", "INSTALL laterite_ags4 FROM community;", "SQL straight over .ags files, no conversion step", "database"], ["Browser", "npm i @laterite/ags4-wasm", "validate + explore in the page, nothing uploaded", "globe"]];
  return /*#__PURE__*/React.createElement("div", {
    style: {
      maxWidth: "46rem"
    }
  }, /*#__PURE__*/React.createElement("div", {
    className: "prose"
  }, /*#__PURE__*/React.createElement("h1", null, "One engine, every stack"), /*#__PURE__*/React.createElement("p", null, "Every surface is the same Rust engine \u2014 no surface re-implements a rule, and scriptable output is byte-identical across them.")), /*#__PURE__*/React.createElement("div", {
    style: {
      display: "grid",
      gap: "var(--space-3)",
      marginTop: "var(--space-5)"
    }
  }, SURFACES.map(([name, install, use, icon]) => /*#__PURE__*/React.createElement("div", {
    key: name,
    style: {
      display: "flex",
      gap: "var(--space-4)",
      alignItems: "flex-start",
      border: "1px solid var(--line)",
      background: "var(--surface)",
      borderRadius: "var(--radius-xl)",
      padding: "var(--space-4)"
    }
  }, /*#__PURE__*/React.createElement("span", {
    style: {
      color: "var(--accent)",
      display: "inline-flex",
      marginTop: 2
    }
  }, /*#__PURE__*/React.createElement(Icon, {
    name: icon,
    size: 20
  })), /*#__PURE__*/React.createElement("div", {
    style: {
      minWidth: 0
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      fontSize: "var(--text-body)",
      fontWeight: "var(--weight-semibold)",
      color: "var(--fg)"
    }
  }, name), /*#__PURE__*/React.createElement("div", {
    style: {
      fontSize: "var(--text-caption)",
      color: "var(--fg-muted)"
    }
  }, use), /*#__PURE__*/React.createElement("code", {
    className: "mono",
    style: {
      display: "inline-block",
      marginTop: "0.4rem",
      fontSize: "var(--text-micro)",
      background: "var(--surface-code)",
      border: "1px solid var(--line-subtle)",
      borderRadius: "var(--radius-xs)",
      padding: "0.1rem 0.35rem",
      color: "var(--fg-soft)"
    }
  }, install)), /*#__PURE__*/React.createElement("span", {
    style: {
      marginLeft: "auto"
    }
  }, /*#__PURE__*/React.createElement(Chip, {
    tone: "warn",
    variant: "outline"
  }, "beta"))))));
}
Object.assign(window, {
  HomePage,
  RecipePage,
  CataloguePage,
  SurfacesPage
});
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/docs/Pages.jsx", error: String((e && e.message) || e) }); }

// ui_kits/docs/Site.jsx
try { (() => {
const {
  ThemeToggle,
  Chip
} = window.LateriteDesignSystem_9f517b;
const TOCS = {
  home: ["In one breath", "Where to go next"],
  recipe: ["Output", "Exit codes", "See also"],
  catalogue: ["Families", "Master table"],
  surfaces: ["Pick your surface", "Parity"]
};
function Site() {
  const [page, setPage] = React.useState("home");
  const [mode, setMode] = React.useState("light");
  React.useEffect(() => {
    document.documentElement.classList.toggle("dark", mode === "dark");
  }, [mode]);
  const known = {
    home: window.HomePage,
    recipe: window.RecipePage,
    catalogue: window.CataloguePage,
    surfaces: window.SurfacesPage
  };
  const Page = known[page];
  return /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement(window.Header, null), /*#__PURE__*/React.createElement("div", {
    style: {
      display: "grid",
      gridTemplateColumns: "14rem minmax(0,1fr) 11rem",
      gap: "var(--space-6)",
      maxWidth: "var(--prose-max)",
      margin: "0 auto",
      padding: "var(--space-7) var(--space-6) var(--space-12)"
    }
  }, /*#__PURE__*/React.createElement(window.Sidebar, {
    page: page,
    onNav: setPage
  }), /*#__PURE__*/React.createElement("main", null, Page ? /*#__PURE__*/React.createElement(Page, null) : /*#__PURE__*/React.createElement("div", {
    className: "prose"
  }, /*#__PURE__*/React.createElement("h1", null, "Page not recreated"), /*#__PURE__*/React.createElement("p", null, "This entry exists in the shipped nav (", /*#__PURE__*/React.createElement("code", null, "mkdocs.yml"), ") but isn't part of the kit. The four recreated pages \u2014 Home, a cookbook recipe, the group catalogue and Surfaces \u2014 carry every layout the site uses."))), /*#__PURE__*/React.createElement("div", {
    style: {
      display: "grid",
      gap: "var(--space-5)",
      alignContent: "start"
    }
  }, /*#__PURE__*/React.createElement(window.Toc, {
    items: TOCS[page] ?? ["—"]
  }), /*#__PURE__*/React.createElement("div", {
    style: {
      display: "flex",
      alignItems: "center",
      gap: "var(--space-2)",
      paddingLeft: "var(--space-4)"
    }
  }, /*#__PURE__*/React.createElement(ThemeToggle, {
    mode: mode,
    onToggle: () => setMode(mode === "dark" ? "light" : "dark")
  }), /*#__PURE__*/React.createElement("span", {
    style: {
      fontSize: "var(--text-micro)",
      color: "var(--fg-dim)"
    }
  }, "theme")))), /*#__PURE__*/React.createElement("footer", {
    style: {
      borderTop: "1px solid var(--line)",
      background: "var(--surface)"
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      maxWidth: "var(--prose-max)",
      margin: "0 auto",
      padding: "var(--space-5) var(--space-6)",
      display: "flex",
      flexWrap: "wrap",
      gap: "var(--space-3)",
      alignItems: "center",
      fontSize: "var(--text-micro)",
      color: "var(--fg-faint)"
    }
  }, /*#__PURE__*/React.createElement("span", null, "laterite \u2014 MIT-licensed AGS4 tooling \xB7 v0.9.4"), /*#__PURE__*/React.createElement(Chip, {
    tone: "warn",
    variant: "outline"
  }, "beta"), /*#__PURE__*/React.createElement("span", {
    style: {
      marginLeft: "auto",
      display: "flex",
      gap: "var(--space-3)"
    }
  }, /*#__PURE__*/React.createElement("a", {
    href: "https://github.com/niko86/laterite"
  }, "GitHub"), /*#__PURE__*/React.createElement("a", {
    href: "https://pypi.org/project/laterite/"
  }, "PyPI"), /*#__PURE__*/React.createElement("a", {
    href: "#"
  }, "Changelog"), /*#__PURE__*/React.createElement("a", {
    href: "#"
  }, "Feedback")))));
}
ReactDOM.createRoot(document.getElementById("root")).render(/*#__PURE__*/React.createElement(Site, null));
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/docs/Site.jsx", error: String((e && e.message) || e) }); }

// ui_kits/webapp/App.jsx
try { (() => {
const {
  Tabs,
  Button,
  ThemeToggle,
  Toast,
  Tooltip,
  Icon
} = window.LateriteDesignSystem_9f517b;
function App() {
  const [tab, setTab] = React.useState("validate");
  const [mode, setMode] = React.useState("light");
  const [toast, setToast] = React.useState(false);
  React.useEffect(() => {
    document.documentElement.classList.toggle("dark", mode === "dark");
  }, [mode]);
  const share = () => {
    setToast(true);
    setTimeout(() => setToast(false), 2400);
  };
  return /*#__PURE__*/React.createElement("div", {
    style: {
      minHeight: "100vh",
      display: "flex",
      flexDirection: "column"
    }
  }, /*#__PURE__*/React.createElement("header", {
    style: {
      borderBottom: "1px solid var(--line)",
      padding: "var(--space-5) var(--space-6)"
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: "flex",
      flexWrap: "wrap",
      alignItems: "center",
      gap: "var(--space-3)"
    }
  }, /*#__PURE__*/React.createElement("img", {
    src: "../../assets/laterite-icon-flat-transparent.png",
    alt: "",
    style: {
      height: 30
    }
  }), /*#__PURE__*/React.createElement("span", {
    style: {
      fontFamily: "var(--font-display)",
      fontSize: "1.4rem",
      fontWeight: "var(--weight-extrabold)",
      letterSpacing: "var(--tracking-tight)",
      color: "var(--laterite-900)",
      lineHeight: 1
    }
  }, "laterite"), /*#__PURE__*/React.createElement("span", {
    "aria-hidden": "true",
    style: {
      width: 1,
      height: "1.2rem",
      background: "var(--line-strong)"
    }
  }), /*#__PURE__*/React.createElement("h1", {
    style: {
      margin: 0,
      fontFamily: "var(--font-ui)",
      fontSize: "var(--text-title)",
      fontWeight: "var(--weight-semibold)",
      color: "var(--fg)"
    }
  }, "AGS4 Validator"), /*#__PURE__*/React.createElement("span", {
    style: {
      fontSize: "var(--text-caption)",
      color: "var(--fg-muted)"
    }
  }, "+ data explorer"), /*#__PURE__*/React.createElement("div", {
    style: {
      marginLeft: "auto",
      display: "flex",
      alignItems: "center",
      gap: "var(--space-2)"
    }
  }, /*#__PURE__*/React.createElement(Tooltip, {
    tip: "Copy a link that restores the current view"
  }, /*#__PURE__*/React.createElement(Button, {
    size: "sm",
    onClick: share
  }, "Share")), /*#__PURE__*/React.createElement(ThemeToggle, {
    mode: mode,
    onToggle: () => setMode(mode === "dark" ? "light" : "dark")
  }))), /*#__PURE__*/React.createElement("p", {
    style: {
      margin: "var(--space-1) 0 0",
      fontSize: "var(--text-micro)",
      color: "var(--fg-faint)"
    }
  }, "Runs entirely in your browser \u2014 your file never leaves your machine. No server, nothing uploaded.")), /*#__PURE__*/React.createElement(Tabs, {
    active: tab,
    onChange: setTab,
    tabs: [{
      id: "validate",
      label: "Validate"
    }, {
      id: "fix",
      label: "Fix"
    }, {
      id: "explore",
      label: "Explore"
    }, {
      id: "tools",
      label: "Tools"
    }, {
      id: "export",
      label: "Export"
    }]
  }), /*#__PURE__*/React.createElement("main", {
    style: {
      flex: 1,
      width: "100%",
      maxWidth: "var(--shell-max)",
      margin: "0 auto",
      padding: "var(--space-6)"
    }
  }, tab === "validate" ? /*#__PURE__*/React.createElement(window.ValidateScreen, null) : null, tab === "fix" ? /*#__PURE__*/React.createElement(window.FixScreen, null) : null, tab === "explore" ? /*#__PURE__*/React.createElement(window.ExploreScreen, null) : null, tab === "tools" ? /*#__PURE__*/React.createElement(window.ToolsScreen, null) : null, tab === "export" ? /*#__PURE__*/React.createElement("div", {
    style: {
      maxWidth: "34rem",
      display: "grid",
      gap: "var(--space-3)"
    }
  }, [["AGS4 (.ags)", "Re-emit the file, optionally aligned"], ["Excel (.xlsx)", "One sheet per group, headings + units retained"], ["Parquet", "Born-typed columns, one file per group"], ["Findings JSON", "Byte-identical to lat validate --json"]].map(([t, d]) => /*#__PURE__*/React.createElement("div", {
    key: t,
    style: {
      display: "flex",
      alignItems: "center",
      gap: "var(--space-4)",
      border: "1px solid var(--line)",
      background: "var(--surface)",
      borderRadius: "var(--radius-xl)",
      padding: "var(--space-4)"
    }
  }, /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("div", {
    style: {
      fontSize: "var(--text-body)",
      fontWeight: "var(--weight-semibold)"
    }
  }, t), /*#__PURE__*/React.createElement("div", {
    style: {
      fontSize: "var(--text-micro)",
      color: "var(--fg-muted)"
    }
  }, d)), /*#__PURE__*/React.createElement("span", {
    style: {
      marginLeft: "auto"
    }
  }, /*#__PURE__*/React.createElement(Button, {
    size: "sm",
    iconLeft: /*#__PURE__*/React.createElement(Icon, {
      name: "file-down",
      size: 13
    })
  }, "Download"))))) : null), /*#__PURE__*/React.createElement("footer", {
    style: {
      display: "flex",
      flexWrap: "wrap",
      gap: "var(--space-2)",
      borderTop: "1px solid var(--line)",
      padding: "var(--space-4) var(--space-6)",
      fontSize: "var(--text-micro)",
      color: "var(--fg-dim)"
    }
  }, /*#__PURE__*/React.createElement("span", null, "Powered by ", /*#__PURE__*/React.createElement("a", {
    href: "https://github.com/niko86/laterite"
  }, "laterite"), ", a clean-room Rust AGS4 engine compiled to WebAssembly \u2014 the same engine runs this app."), /*#__PURE__*/React.createElement("span", null, "\xB7 ", /*#__PURE__*/React.createElement("a", {
    href: "https://docs.laterite.dev/reference/support/"
  }, "in beta")), /*#__PURE__*/React.createElement("span", null, "\xB7 ", /*#__PURE__*/React.createElement("a", {
    href: "https://github.com/niko86/laterite"
  }, "GitHub")), /*#__PURE__*/React.createElement("span", null, "\xB7 ", /*#__PURE__*/React.createElement("a", {
    href: "https://pypi.org/project/laterite/"
  }, "PyPI"))), toast ? /*#__PURE__*/React.createElement("div", {
    style: {
      position: "fixed",
      left: "1rem",
      bottom: "1rem",
      zIndex: 60
    }
  }, /*#__PURE__*/React.createElement(Toast, {
    message: "Link copied",
    onDismiss: () => setToast(false)
  })) : null);
}
ReactDOM.createRoot(document.getElementById("root")).render(/*#__PURE__*/React.createElement(App, null));
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/webapp/App.jsx", error: String((e && e.message) || e) }); }

// ui_kits/webapp/ExploreScreen.jsx
try { (() => {
const {
  Card,
  Disclosure,
  Button,
  Chip,
  PillToggle,
  Field,
  Select,
  Input,
  Icon,
  Spinner
} = window.LateriteDesignSystem_9f517b;
function ResultsGrid({
  columns,
  rows
}) {
  return /*#__PURE__*/React.createElement("div", {
    className: "scroll",
    style: {
      borderRadius: "var(--radius-xl)",
      border: "1px solid var(--line)"
    }
  }, /*#__PURE__*/React.createElement("table", {
    style: {
      fontSize: "var(--text-micro)"
    }
  }, /*#__PURE__*/React.createElement("thead", null, /*#__PURE__*/React.createElement("tr", {
    style: {
      background: "var(--surface-raised)"
    }
  }, columns.map(c => /*#__PURE__*/React.createElement("th", {
    key: c[0],
    style: {
      position: "sticky",
      top: 0,
      background: "var(--surface-raised)",
      textAlign: "left",
      fontWeight: "var(--weight-medium)",
      color: "var(--fg-soft)",
      padding: "0.35rem 0.75rem",
      borderBottom: "1px solid var(--line)",
      whiteSpace: "nowrap"
    }
  }, c[0], " ", /*#__PURE__*/React.createElement("span", {
    style: {
      fontWeight: 400,
      color: "var(--fg-dim)"
    }
  }, c[3].toLowerCase()))))), /*#__PURE__*/React.createElement("tbody", {
    className: "mono"
  }, rows.map((r, i) => /*#__PURE__*/React.createElement("tr", {
    key: i,
    style: {
      borderTop: "1px solid var(--line-subtle)"
    }
  }, r.map((cell, j) => /*#__PURE__*/React.createElement("td", {
    key: j,
    style: {
      padding: "0.25rem 0.75rem",
      color: "var(--fg-soft)",
      whiteSpace: "nowrap"
    }
  }, cell || "—")))))));
}
function ExploreScreen() {
  const [view, setView] = React.useState("Browse");
  const [code, setCode] = React.useState("LOCA");
  const [sql, setSql] = React.useState('SELECT l.loca_id, s.samp_ref, s.samp_top\nFROM "SAMP" s JOIN "LOCA" l ON s._parent_id = l._id\nWHERE s.samp_top > 5\nORDER BY s.samp_top DESC;');
  const group = window.GROUPS.find(g => g.code === code) ?? window.GROUPS[1];
  return /*#__PURE__*/React.createElement("div", {
    style: {
      display: "grid",
      gridTemplateColumns: "13rem minmax(0,1fr)",
      gap: "var(--space-6)",
      alignItems: "start"
    }
  }, /*#__PURE__*/React.createElement(Card, {
    pad: "none"
  }, /*#__PURE__*/React.createElement("div", {
    className: "lbl",
    style: {
      padding: "0.5rem 0.75rem",
      borderBottom: "1px solid var(--line-subtle)"
    }
  }, "groups \xB7 123"), window.GROUPS.map(g => /*#__PURE__*/React.createElement("button", {
    key: g.code,
    type: "button",
    onClick: () => setCode(g.code),
    style: {
      font: "inherit",
      fontFamily: "var(--font-ui)",
      width: "100%",
      textAlign: "left",
      border: "none",
      borderBottom: "1px solid var(--line-subtle)",
      cursor: "pointer",
      padding: "0.4rem 0.75rem",
      background: g.code === code ? "var(--accent-quiet)" : "none"
    }
  }, /*#__PURE__*/React.createElement("span", {
    className: "mono",
    style: {
      fontSize: "var(--text-control)",
      color: g.code === code ? "var(--accent)" : "var(--fg)"
    }
  }, g.code), /*#__PURE__*/React.createElement("span", {
    style: {
      float: "right",
      fontSize: "var(--text-micro)",
      color: "var(--fg-dim)"
    }
  }, g.rows.toLocaleString()), /*#__PURE__*/React.createElement("div", {
    style: {
      fontSize: "var(--text-micro)",
      color: "var(--fg-faint)"
    }
  }, g.desc)))), /*#__PURE__*/React.createElement("div", {
    style: {
      display: "grid",
      gap: "var(--space-4)"
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: "flex",
      alignItems: "center",
      gap: "var(--space-4)"
    }
  }, /*#__PURE__*/React.createElement(PillToggle, {
    value: view,
    onChange: setView,
    options: ["Browse", "SQL", "Charts", "Analyse"]
  }), /*#__PURE__*/React.createElement("div", {
    style: {
      marginLeft: "auto",
      display: "flex",
      gap: "var(--space-2)"
    }
  }, /*#__PURE__*/React.createElement(Button, {
    size: "sm",
    iconLeft: /*#__PURE__*/React.createElement(Icon, {
      name: "file-down",
      size: 13
    })
  }, "CSV"), /*#__PURE__*/React.createElement(Button, {
    size: "sm",
    iconLeft: /*#__PURE__*/React.createElement(Icon, {
      name: "file-down",
      size: 13
    })
  }, "Parquet"))), view === "Browse" ? /*#__PURE__*/React.createElement(React.Fragment, null, /*#__PURE__*/React.createElement(Disclosure, {
    summary: `${group.code} schema — ${window.LOCA_SCHEMA.length} columns`
  }, /*#__PURE__*/React.createElement("table", {
    style: {
      fontSize: "var(--text-micro)"
    }
  }, /*#__PURE__*/React.createElement("thead", null, /*#__PURE__*/React.createElement("tr", {
    style: {
      color: "var(--fg-muted)"
    }
  }, ["Heading", "Unit", "AGS type", "SQL type"].map(h => /*#__PURE__*/React.createElement("th", {
    key: h,
    style: {
      textAlign: "left",
      fontWeight: "var(--weight-medium)",
      padding: "0.15rem 0.75rem"
    }
  }, h)))), /*#__PURE__*/React.createElement("tbody", null, window.LOCA_SCHEMA.map(r => /*#__PURE__*/React.createElement("tr", {
    key: r[0],
    style: {
      borderTop: "1px solid var(--line-subtle)"
    }
  }, /*#__PURE__*/React.createElement("td", {
    className: "mono",
    style: {
      padding: "0.2rem 0.75rem",
      color: "var(--fg)"
    }
  }, r[0]), /*#__PURE__*/React.createElement("td", {
    style: {
      padding: "0.2rem 0.75rem",
      color: "var(--fg-faint)"
    }
  }, r[1] || "—"), /*#__PURE__*/React.createElement("td", {
    style: {
      padding: "0.2rem 0.75rem",
      color: "var(--fg-soft)"
    }
  }, r[2]), /*#__PURE__*/React.createElement("td", {
    className: "mono",
    style: {
      padding: "0.2rem 0.75rem",
      color: "var(--accent)"
    }
  }, r[3])))))), /*#__PURE__*/React.createElement(ResultsGrid, {
    columns: window.LOCA_SCHEMA,
    rows: window.LOCA_ROWS
  }), /*#__PURE__*/React.createElement("div", {
    style: {
      display: "flex",
      alignItems: "center",
      gap: "var(--space-3)",
      fontSize: "var(--text-micro)",
      color: "var(--fg-muted)"
    }
  }, /*#__PURE__*/React.createElement(Button, {
    size: "sm",
    disabled: true
  }, "\u2190 Prev"), /*#__PURE__*/React.createElement("span", null, "Page 1 of 5 \xB7 ", group.rows.toLocaleString(), " rows"), /*#__PURE__*/React.createElement(Button, {
    size: "sm"
  }, "Next \u2192"))) : view === "SQL" ? /*#__PURE__*/React.createElement(React.Fragment, null, /*#__PURE__*/React.createElement(Card, {
    pad: "sm"
  }, /*#__PURE__*/React.createElement("textarea", {
    className: "mono",
    value: sql,
    onChange: e => setSql(e.target.value),
    spellCheck: false,
    style: {
      width: "100%",
      height: "7rem",
      resize: "vertical",
      border: "1px solid var(--line-strong)",
      borderRadius: "var(--radius-xs)",
      background: "var(--surface-code)",
      color: "var(--fg)",
      padding: "0.5rem",
      fontSize: "var(--text-micro)",
      lineHeight: "var(--leading-relaxed)",
      outline: "none"
    }
  }), /*#__PURE__*/React.createElement("div", {
    style: {
      display: "flex",
      alignItems: "center",
      gap: "var(--space-3)",
      marginTop: "var(--space-3)"
    }
  }, /*#__PURE__*/React.createElement(Button, {
    variant: "action",
    iconLeft: /*#__PURE__*/React.createElement(Icon, {
      name: "play",
      size: 13
    })
  }, "Run"), /*#__PURE__*/React.createElement(Chip, {
    tone: "muted"
  }, "duckdb-wasm"), /*#__PURE__*/React.createElement("span", {
    style: {
      fontSize: "var(--text-micro)",
      color: "var(--fg-faint)"
    }
  }, "every row carries _id / _parent_id (UUIDv8 over the AGS key) \u2014 parent\u2194child joins need no key list"))), /*#__PURE__*/React.createElement(ResultsGrid, {
    columns: [["loca_id", "", "ID", "VARCHAR"], ["samp_ref", "", "ID", "VARCHAR"], ["samp_top", "m", "2DP", "DOUBLE"]],
    rows: [["BH03", "S12", "28.50"], ["BH04", "S09", "24.00"], ["BH01", "S04", "18.50"], ["BH02", "S07", "12.00"], ["BH05", "S02", "7.50"]]
  })) : view === "Charts" ? /*#__PURE__*/React.createElement(Card, null, /*#__PURE__*/React.createElement("div", {
    style: {
      display: "flex",
      gap: "var(--space-3)",
      flexWrap: "wrap"
    }
  }, /*#__PURE__*/React.createElement(Field, {
    label: "Chart type"
  }, /*#__PURE__*/React.createElement(Select, null, /*#__PURE__*/React.createElement("option", null, "Scatter"), /*#__PURE__*/React.createElement("option", null, "Line"), /*#__PURE__*/React.createElement("option", null, "Histogram"))), /*#__PURE__*/React.createElement(Field, {
    label: "X"
  }, /*#__PURE__*/React.createElement(Select, null, /*#__PURE__*/React.createElement("option", null, "LLPL_LL"))), /*#__PURE__*/React.createElement(Field, {
    label: "Y"
  }, /*#__PURE__*/React.createElement(Select, null, /*#__PURE__*/React.createElement("option", null, "SAMP_TOP"))), /*#__PURE__*/React.createElement(Field, {
    label: "Series"
  }, /*#__PURE__*/React.createElement(Select, null, /*#__PURE__*/React.createElement("option", null, "LOCA_ID")))), /*#__PURE__*/React.createElement("div", {
    style: {
      marginTop: "var(--space-4)",
      height: "13rem",
      borderRadius: "var(--radius-lg)",
      border: "1px dashed var(--line-strong)",
      display: "grid",
      placeItems: "center",
      color: "var(--fg-dim)",
      fontSize: "var(--text-caption)"
    }
  }, "echarts canvas \u2014 plotted client-side from the DuckDB result")) : /*#__PURE__*/React.createElement(Card, null, /*#__PURE__*/React.createElement("div", {
    className: "lbl"
  }, "coverage"), /*#__PURE__*/React.createElement("div", {
    style: {
      display: "grid",
      gap: "var(--space-2)",
      marginTop: "var(--space-3)"
    }
  }, [["LOCA_GL", 100], ["LOCA_FDEP", 96], ["GEOL_DESC", 88], ["ISPT_NVAL", 61], ["LLPL_LL", 24]].map(([h, pct]) => /*#__PURE__*/React.createElement("div", {
    key: h,
    style: {
      display: "flex",
      alignItems: "center",
      gap: "var(--space-3)",
      fontSize: "var(--text-micro)"
    }
  }, /*#__PURE__*/React.createElement("span", {
    className: "mono",
    style: {
      width: "8rem",
      color: "var(--fg-soft)"
    }
  }, h), /*#__PURE__*/React.createElement("span", {
    style: {
      flex: 1,
      height: "0.5rem",
      borderRadius: "var(--radius-pill)",
      background: "var(--chip)",
      overflow: "hidden"
    }
  }, /*#__PURE__*/React.createElement("span", {
    style: {
      display: "block",
      width: pct + "%",
      height: "100%",
      background: "var(--laterite-400)"
    }
  })), /*#__PURE__*/React.createElement("span", {
    style: {
      width: "2.5rem",
      textAlign: "right",
      color: "var(--fg-muted)"
    }
  }, pct, "%")))))));
}
Object.assign(window, {
  ExploreScreen
});
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/webapp/ExploreScreen.jsx", error: String((e && e.message) || e) }); }

// ui_kits/webapp/FixScreen.jsx
try { (() => {
const {
  Card,
  Button,
  Chip,
  Checkbox,
  PillToggle,
  SummaryBanner,
  Icon
} = window.LateriteDesignSystem_9f517b;
const FIXES = [{
  id: 1,
  rule: "Rule 5",
  label: "Pad LOCA_GL '11.8' → '11.80'",
  line: 11,
  safe: true
}, {
  id: 2,
  rule: "Rule 3",
  label: "Strip trailing whitespace from 4 DATA rows",
  line: null,
  safe: true
}, {
  id: 3,
  rule: "Rule 7",
  label: "Insert missing UNIT row for GEOL",
  line: 42,
  safe: true
}, {
  id: 4,
  rule: "Rule 17",
  label: "Add TRAN group with TRAN_AGS 4.1.1",
  line: 1,
  safe: false
}];
const DIFF = [{
  t: "ctx",
  n: 9,
  s: '"TYPE","ID","PA","2DP","2DP","2DP","2DP"'
}, {
  t: "ctx",
  n: 10,
  s: '"DATA","BH01","CP","523456.12","187654.33","12.30","25.00"'
}, {
  t: "del",
  n: 11,
  s: '"DATA","BH02","CP","523501.44","187690.10","11.8","24.50"'
}, {
  t: "add",
  n: 11,
  s: '"DATA","BH02","CP","523501.44","187690.10","11.80","24.50"'
}, {
  t: "ctx",
  n: 12,
  s: '"DATA","BH03","RC","523560.02","187722.87","10.95","30.00"'
}];
function FixScreen() {
  const [view, setView] = React.useState("Fixes");
  const [on, setOn] = React.useState({
    1: true,
    2: true,
    3: true
  });
  const applied = Object.values(on).filter(Boolean).length;
  return /*#__PURE__*/React.createElement("div", {
    style: {
      display: "grid",
      gap: "var(--space-4)",
      maxWidth: "56rem"
    }
  }, /*#__PURE__*/React.createElement(SummaryBanner, {
    kind: "warn",
    headline: `${FIXES.length} repairs available — ${applied} selected`,
    detail: "Safe fixes never change a value's meaning; unsafe ones are opt-in and listed last.",
    note: "Nothing is written until you download the repaired file \u2014 the original is untouched."
  }), /*#__PURE__*/React.createElement("div", {
    style: {
      display: "flex",
      alignItems: "center",
      gap: "var(--space-4)"
    }
  }, /*#__PURE__*/React.createElement(PillToggle, {
    value: view,
    onChange: setView,
    options: ["Fixes", "Diff"]
  }), /*#__PURE__*/React.createElement("div", {
    style: {
      marginLeft: "auto",
      display: "flex",
      gap: "var(--space-2)"
    }
  }, /*#__PURE__*/React.createElement(Button, {
    size: "sm"
  }, "Re-validate"), /*#__PURE__*/React.createElement(Button, {
    variant: "primary",
    size: "sm",
    iconLeft: /*#__PURE__*/React.createElement(Icon, {
      name: "file-down",
      size: 13
    })
  }, "Download .fixed.ags"))), view === "Fixes" ? /*#__PURE__*/React.createElement(Card, {
    pad: "none"
  }, FIXES.map((x, i) => /*#__PURE__*/React.createElement("div", {
    key: x.id,
    style: {
      display: "flex",
      alignItems: "center",
      gap: "var(--space-3)",
      padding: "0.5rem 0.8rem",
      borderTop: i ? "1px solid var(--line-subtle)" : "none"
    }
  }, /*#__PURE__*/React.createElement(Checkbox, {
    checked: !!on[x.id],
    onChange: () => setOn({
      ...on,
      [x.id]: !on[x.id]
    }),
    label: ""
  }), /*#__PURE__*/React.createElement(Chip, {
    tone: "muted"
  }, x.rule), /*#__PURE__*/React.createElement("span", {
    style: {
      fontSize: "var(--text-caption)",
      color: "var(--fg)"
    }
  }, x.label), x.line ? /*#__PURE__*/React.createElement("span", {
    className: "mono",
    style: {
      fontSize: "var(--text-micro)",
      color: "var(--fg-faint)"
    }
  }, "line ", x.line) : null, /*#__PURE__*/React.createElement("span", {
    style: {
      marginLeft: "auto"
    }
  }, x.safe ? /*#__PURE__*/React.createElement(Chip, {
    tone: "ok",
    variant: "outline"
  }, "safe") : /*#__PURE__*/React.createElement(Chip, {
    tone: "warn"
  }, "changes meaning"))))) : /*#__PURE__*/React.createElement(Card, {
    pad: "none"
  }, /*#__PURE__*/React.createElement("pre", {
    className: "mono",
    style: {
      margin: 0,
      padding: "var(--space-3)",
      overflowX: "auto",
      fontSize: "var(--text-micro)",
      lineHeight: "var(--leading-relaxed)"
    }
  }, DIFF.map((d, i) => /*#__PURE__*/React.createElement("div", {
    key: i,
    style: {
      minWidth: "max-content",
      background: d.t === "add" ? "var(--ok-quiet)" : d.t === "del" ? "var(--err-quiet)" : "transparent",
      color: d.t === "ctx" ? "var(--fg-muted)" : "var(--fg)"
    }
  }, /*#__PURE__*/React.createElement("span", {
    style: {
      display: "inline-block",
      width: "2.5rem",
      textAlign: "right",
      marginRight: "0.75rem",
      color: "var(--fg-dim)",
      userSelect: "none"
    }
  }, d.n), /*#__PURE__*/React.createElement("span", {
    style: {
      display: "inline-block",
      width: "1rem",
      color: d.t === "add" ? "var(--ok)" : d.t === "del" ? "var(--err)" : "var(--fg-dim)"
    }
  }, d.t === "add" ? "+" : d.t === "del" ? "−" : " "), d.s)))));
}
Object.assign(window, {
  FixScreen
});
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/webapp/FixScreen.jsx", error: String((e && e.message) || e) }); }

// ui_kits/webapp/ToolsScreen.jsx
try { (() => {
const {
  Card,
  Button,
  Chip,
  Input,
  Field,
  Select,
  PillToggle,
  Admonition,
  Icon
} = window.LateriteDesignSystem_9f517b;
const GROUPS = [["Reference", ["Dictionary", "Rules", "Template"]], ["This file", ["Anonymiser", "Formatter", "Coordinates", "Excel", "Transport"]], ["Compare", ["Revision diff", "Merge"]]];
const DICT = [["LOCA_ID", "ID", "", "Location identifier", "KEY"], ["LOCA_TYPE", "PA", "", "Type of activity", "REQUIRED"], ["LOCA_NATE", "2DP", "m", "National grid easting of location", "OTHER"], ["LOCA_GL", "2DP", "m", "Ground level relative to datum", "REQUIRED"], ["LOCA_FDEP", "2DP", "m", "Final depth of hole", "OTHER"]];
function ToolsScreen() {
  const [tool, setTool] = React.useState("Dictionary");
  return /*#__PURE__*/React.createElement("div", {
    style: {
      display: "grid",
      gap: "var(--space-5)",
      maxWidth: "60rem"
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: "grid",
      gap: "0.35rem"
    }
  }, GROUPS.map(([label, tools]) => /*#__PURE__*/React.createElement("div", {
    key: label,
    style: {
      display: "flex",
      alignItems: "center",
      gap: "var(--space-2)",
      flexWrap: "wrap"
    }
  }, /*#__PURE__*/React.createElement("span", {
    className: "lbl",
    style: {
      width: "4.5rem",
      flexShrink: 0
    }
  }, label), /*#__PURE__*/React.createElement(PillToggle, {
    value: tool,
    onChange: setTool,
    options: tools
  })))), tool === "Dictionary" ? /*#__PURE__*/React.createElement(Card, {
    pad: "none"
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: "flex",
      alignItems: "center",
      gap: "var(--space-3)",
      padding: "var(--space-3)",
      borderBottom: "1px solid var(--line-subtle)"
    }
  }, /*#__PURE__*/React.createElement(Input, {
    mono: true,
    placeholder: "Search headings \u2014 LOCA_",
    style: {
      maxWidth: "16rem"
    }
  }), /*#__PURE__*/React.createElement(Field, {
    label: ""
  }, /*#__PURE__*/React.createElement(Select, null, /*#__PURE__*/React.createElement("option", null, "AGS 4.1.1"), /*#__PURE__*/React.createElement("option", null, "AGS 4.0.4"))), /*#__PURE__*/React.createElement(Chip, {
    tone: "accent"
  }, "LOCA"), /*#__PURE__*/React.createElement("span", {
    style: {
      marginLeft: "auto",
      fontSize: "var(--text-micro)",
      color: "var(--fg-faint)"
    }
  }, "174 groups \xB7 2,381 headings")), /*#__PURE__*/React.createElement("table", {
    style: {
      fontSize: "var(--text-micro)"
    }
  }, /*#__PURE__*/React.createElement("thead", null, /*#__PURE__*/React.createElement("tr", {
    style: {
      background: "var(--surface-raised)",
      color: "var(--fg-soft)"
    }
  }, ["Heading", "Type", "Unit", "Description", "Status"].map(h => /*#__PURE__*/React.createElement("th", {
    key: h,
    style: {
      textAlign: "left",
      fontWeight: "var(--weight-medium)",
      padding: "0.35rem 0.75rem",
      borderBottom: "1px solid var(--line)"
    }
  }, h)))), /*#__PURE__*/React.createElement("tbody", null, DICT.map(r => /*#__PURE__*/React.createElement("tr", {
    key: r[0],
    style: {
      borderTop: "1px solid var(--line-subtle)"
    }
  }, /*#__PURE__*/React.createElement("td", {
    className: "mono",
    style: {
      padding: "0.3rem 0.75rem",
      color: "var(--fg)"
    }
  }, r[0]), /*#__PURE__*/React.createElement("td", {
    className: "mono",
    style: {
      padding: "0.3rem 0.75rem",
      color: "var(--accent)"
    }
  }, r[1]), /*#__PURE__*/React.createElement("td", {
    style: {
      padding: "0.3rem 0.75rem",
      color: "var(--fg-faint)"
    }
  }, r[2] || "—"), /*#__PURE__*/React.createElement("td", {
    style: {
      padding: "0.3rem 0.75rem",
      color: "var(--fg-soft)"
    }
  }, r[3]), /*#__PURE__*/React.createElement("td", {
    style: {
      padding: "0.3rem 0.75rem"
    }
  }, r[4] === "KEY" ? /*#__PURE__*/React.createElement(Chip, {
    tone: "accent"
  }, "key") : r[4] === "REQUIRED" ? /*#__PURE__*/React.createElement(Chip, {
    tone: "warn"
  }, "required") : /*#__PURE__*/React.createElement(Chip, {
    tone: "muted"
  }, "other"))))))) : tool === "Transport" ? /*#__PURE__*/React.createElement(React.Fragment, null, /*#__PURE__*/React.createElement(Admonition, {
    kind: "warning",
    title: "Encryption happens in your browser"
  }, "The passphrase never leaves the page \u2014 there is no server to send it to."), /*#__PURE__*/React.createElement(Card, null, /*#__PURE__*/React.createElement("div", {
    style: {
      display: "grid",
      gap: "var(--space-3)",
      maxWidth: "26rem"
    }
  }, /*#__PURE__*/React.createElement(Field, {
    label: "Passphrase"
  }, /*#__PURE__*/React.createElement(Input, {
    type: "password",
    defaultValue: "correct-horse-battery"
  })), /*#__PURE__*/React.createElement(Field, {
    label: "Compression"
  }, /*#__PURE__*/React.createElement(Select, null, /*#__PURE__*/React.createElement("option", null, "zstd \u2014 level 12"), /*#__PURE__*/React.createElement("option", null, "zstd \u2014 level 19 (slow)"), /*#__PURE__*/React.createElement("option", null, "None"))), /*#__PURE__*/React.createElement("div", {
    style: {
      display: "flex",
      gap: "var(--space-2)"
    }
  }, /*#__PURE__*/React.createElement(Button, {
    variant: "primary",
    iconLeft: /*#__PURE__*/React.createElement(Icon, {
      name: "lock",
      size: 13
    })
  }, "Pack + encrypt"), /*#__PURE__*/React.createElement(Button, null, "Unpack\u2026")), /*#__PURE__*/React.createElement("p", {
    style: {
      margin: 0,
      fontSize: "var(--text-micro)",
      color: "var(--fg-faint)"
    }
  }, "4.9 MB \u2192 612 KB (87.5% smaller) \xB7 age-encryption, zstd-wasm")))) : /*#__PURE__*/React.createElement(Card, null, /*#__PURE__*/React.createElement("div", {
    style: {
      fontSize: "var(--text-body)",
      fontWeight: "var(--weight-semibold)"
    }
  }, tool), /*#__PURE__*/React.createElement("p", {
    style: {
      margin: "0.3rem 0 0",
      fontSize: "var(--text-caption)",
      color: "var(--fg-muted)",
      maxWidth: "42rem"
    }
  }, "This tool exists in the shipped app but its interior is not recreated in this kit \u2014 the pattern to reuse is the one above: a control row in a Card, then a bordered results table or a diff pane, with every action client-side.")));
}
Object.assign(window, {
  ToolsScreen
});
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/webapp/ToolsScreen.jsx", error: String((e && e.message) || e) }); }

// ui_kits/webapp/ValidateScreen.jsx
try { (() => {
const {
  Card,
  Disclosure,
  Button,
  Chip,
  Chevron,
  Icon,
  Field,
  Input,
  Select,
  Checkbox,
  ControlGrid,
  SummaryBanner,
  Spinner,
  Tooltip
} = window.LateriteDesignSystem_9f517b;
const SEVERITY_TONE = {
  error: "err",
  warning: "warn",
  fyi: "muted"
};
const BAND = {
  error: "var(--err-quiet)",
  warning: "var(--warn-quiet)",
  fyi: "var(--surface-raised)"
};
function FindingRow({
  f
}) {
  const lines = window.AGS_TEXT.split("\n");
  const from = Math.max(1, f.line - 2),
    to = Math.min(lines.length, f.line + 2);
  const rows = [];
  for (let n = from; n <= to; n++) rows.push({
    n,
    text: lines[n - 1] ?? "",
    hit: n === f.line
  });
  return /*#__PURE__*/React.createElement("div", {
    style: {
      borderTop: "1px solid var(--line)",
      padding: "0.5rem 0.75rem"
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: "flex",
      flexWrap: "wrap",
      alignItems: "baseline",
      gap: "0.75rem",
      fontSize: "var(--text-caption)"
    }
  }, /*#__PURE__*/React.createElement("span", {
    style: {
      color: "var(--fg-faint)"
    }
  }, "line ", f.line), /*#__PURE__*/React.createElement(Chip, {
    tone: "muted"
  }, f.group), /*#__PURE__*/React.createElement("span", {
    style: {
      color: "var(--fg)"
    }
  }, f.desc)), /*#__PURE__*/React.createElement("pre", {
    className: "mono",
    style: {
      margin: "0.5rem 0 0",
      overflowX: "auto",
      borderRadius: "var(--radius-sm)",
      background: "var(--surface-code)",
      padding: "0.5rem",
      fontSize: "var(--text-micro)",
      lineHeight: "var(--leading-relaxed)",
      color: "var(--fg-muted)"
    }
  }, rows.map(r => /*#__PURE__*/React.createElement("div", {
    key: r.n,
    style: {
      minWidth: "max-content",
      background: r.hit ? BAND[f.severity] : "transparent"
    }
  }, /*#__PURE__*/React.createElement("span", {
    style: {
      display: "inline-block",
      width: "2.5rem",
      marginRight: "0.75rem",
      textAlign: "right",
      color: "var(--fg-dim)",
      userSelect: "none"
    }
  }, r.n), r.hit ? /*#__PURE__*/React.createElement(React.Fragment, null, r.text.split(f.field)[0], /*#__PURE__*/React.createElement("span", {
    style: {
      borderRadius: "2px",
      background: "color-mix(in srgb, var(--err) 35%, transparent)",
      color: "var(--fg)"
    }
  }, f.field), r.text.split(f.field).slice(1).join(f.field)) : r.text))));
}
function ValidateScreen() {
  const [open, setOpen] = React.useState({
    "Rule 5 — data type": true
  });
  const [sev, setSev] = React.useState({
    error: true,
    warning: true,
    fyi: false
  });
  const [q, setQ] = React.useState("");
  const groups = window.FINDINGS.map(g => ({
    ...g,
    items: g.items.filter(i => sev[i.severity] && (!q || (i.desc + i.group).toLowerCase().includes(q.toLowerCase())))
  })).filter(g => g.items.length);
  const errors = groups.reduce((n, g) => n + g.items.filter(i => i.severity === "error").length, 0);
  const warnings = groups.reduce((n, g) => n + g.items.filter(i => i.severity === "warning").length, 0);
  return /*#__PURE__*/React.createElement("div", {
    style: {
      display: "grid",
      gridTemplateColumns: "minmax(0, 5fr) minmax(0, 7fr)",
      gap: "var(--space-6)",
      alignItems: "start"
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: "grid",
      gap: "var(--space-4)"
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: "flex",
      flexDirection: "column",
      gap: "var(--space-2)"
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: "flex",
      alignItems: "center",
      justifyContent: "space-between"
    }
  }, /*#__PURE__*/React.createElement("label", {
    style: {
      fontSize: "var(--text-caption)",
      fontWeight: "var(--weight-medium)",
      color: "var(--fg-soft)"
    }
  }, "AGS4 input \u2014 P2246-delivery-rev2.ags"), /*#__PURE__*/React.createElement(Button, {
    size: "sm"
  }, "Choose file\u2026")), /*#__PURE__*/React.createElement("div", {
    style: {
      borderRadius: "var(--radius-xl)",
      border: "2px dashed var(--line-strong)"
    }
  }, /*#__PURE__*/React.createElement("textarea", {
    className: "mono",
    spellCheck: false,
    defaultValue: window.AGS_TEXT,
    style: {
      width: "100%",
      height: "17rem",
      resize: "vertical",
      borderRadius: "var(--radius-xl)",
      border: "none",
      outline: "none",
      background: "var(--surface-raised)",
      color: "var(--fg)",
      padding: "0.75rem",
      fontSize: "var(--text-micro)",
      lineHeight: "var(--leading-relaxed)"
    }
  }))), /*#__PURE__*/React.createElement(Card, null, /*#__PURE__*/React.createElement(ControlGrid, null, /*#__PURE__*/React.createElement(Field, {
    label: "Dictionary edition"
  }, /*#__PURE__*/React.createElement(Select, {
    defaultValue: "auto"
  }, /*#__PURE__*/React.createElement("option", {
    value: "auto"
  }, "Auto (from TRAN_AGS)"), /*#__PURE__*/React.createElement("option", null, "4.1.1"), /*#__PURE__*/React.createElement("option", null, "4.0.4"))), /*#__PURE__*/React.createElement(Field, {
    label: "Encoding"
  }, /*#__PURE__*/React.createElement(Select, null, /*#__PURE__*/React.createElement("option", null, "UTF-8"), /*#__PURE__*/React.createElement("option", null, "Windows-1252 / Latin-1"))), /*#__PURE__*/React.createElement(Field, {
    label: "Options"
  }, /*#__PURE__*/React.createElement(Checkbox, {
    label: "Aligned columns",
    defaultChecked: true
  })))), /*#__PURE__*/React.createElement(Disclosure, {
    summary: "Or try a sample",
    count: 4
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: "flex",
      flexWrap: "wrap",
      gap: "var(--space-2)"
    }
  }, ["Clean delivery", "Dirty file", "Revised (rev2)", "Wide — 123 groups"].map(s => /*#__PURE__*/React.createElement(Button, {
    key: s,
    size: "sm"
  }, s))))), /*#__PURE__*/React.createElement("div", {
    style: {
      display: "grid",
      gap: "var(--space-4)"
    }
  }, /*#__PURE__*/React.createElement(SummaryBanner, {
    kind: "err",
    headline: `${errors} errors · ${warnings} warnings · 1 informational`,
    detail: "Validated against AGS 4.1.1 \u2014 fallback (TRAN_AGS missing/unknown)",
    note: "Download the full report below, or use the lat CLI for very large files."
  }), /*#__PURE__*/React.createElement(Card, {
    pad: "sm"
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: "flex",
      flexWrap: "wrap",
      alignItems: "center",
      gap: "var(--space-3)"
    }
  }, /*#__PURE__*/React.createElement(Input, {
    placeholder: "Filter findings\u2026",
    value: q,
    onChange: e => setQ(e.target.value),
    style: {
      maxWidth: "14rem"
    }
  }), /*#__PURE__*/React.createElement("div", {
    style: {
      display: "flex",
      gap: "var(--space-1)"
    }
  }, ["error", "warning", "fyi"].map(s => /*#__PURE__*/React.createElement("button", {
    key: s,
    type: "button",
    onClick: () => setSev({
      ...sev,
      [s]: !sev[s]
    }),
    style: {
      font: "inherit",
      fontSize: "var(--text-micro)",
      fontWeight: 600,
      cursor: "pointer",
      borderRadius: "var(--radius-2xl)",
      padding: "0.1rem 0.55rem",
      border: `1px solid ${sev[s] ? "transparent" : "var(--line-strong)"}`,
      background: sev[s] ? `var(--${SEVERITY_TONE[s] === "muted" ? "chip" : SEVERITY_TONE[s] + "-quiet"})` : "transparent",
      color: sev[s] ? `var(--${SEVERITY_TONE[s] === "muted" ? "fg-soft" : SEVERITY_TONE[s]})` : "var(--fg-dim)"
    }
  }, s))), /*#__PURE__*/React.createElement("div", {
    style: {
      marginLeft: "auto",
      display: "flex",
      gap: "var(--space-2)"
    }
  }, /*#__PURE__*/React.createElement(Tooltip, {
    tip: "Findings as JSON \u2014 byte-identical to lat validate --json"
  }, /*#__PURE__*/React.createElement(Button, {
    size: "sm",
    iconLeft: /*#__PURE__*/React.createElement(Icon, {
      name: "file-down",
      size: 13
    })
  }, "Report")), /*#__PURE__*/React.createElement(Tooltip, {
    tip: "Only mintable when the file is clean"
  }, /*#__PURE__*/React.createElement(Button, {
    size: "sm",
    disabled: true,
    iconLeft: /*#__PURE__*/React.createElement(Icon, {
      name: "shield-check",
      size: 13
    })
  }, "Certificate"))))), /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("div", {
    style: {
      display: "flex",
      gap: "var(--space-3)",
      fontSize: "var(--text-micro)",
      color: "var(--fg-muted)",
      marginBottom: "var(--space-2)"
    }
  }, /*#__PURE__*/React.createElement("button", {
    type: "button",
    onClick: () => setOpen(Object.fromEntries(window.FINDINGS.map(g => [g.rule, true]))),
    style: {
      font: "inherit",
      background: "none",
      border: "none",
      color: "inherit",
      cursor: "pointer",
      textDecoration: "underline",
      textUnderlineOffset: 2
    }
  }, "Expand all"), /*#__PURE__*/React.createElement("button", {
    type: "button",
    onClick: () => setOpen({}),
    style: {
      font: "inherit",
      background: "none",
      border: "none",
      color: "inherit",
      cursor: "pointer",
      textDecoration: "underline",
      textUnderlineOffset: 2
    }
  }, "Collapse all")), /*#__PURE__*/React.createElement("div", {
    className: "scroll",
    style: {
      borderRadius: "var(--radius-xl)",
      border: "1px solid var(--line)",
      background: "var(--surface)"
    }
  }, groups.map(g => /*#__PURE__*/React.createElement("div", {
    key: g.rule
  }, /*#__PURE__*/React.createElement("button", {
    type: "button",
    onClick: () => setOpen({
      ...open,
      [g.rule]: !open[g.rule]
    }),
    style: {
      font: "inherit",
      fontFamily: "var(--font-ui)",
      width: "100%",
      display: "flex",
      alignItems: "center",
      gap: "var(--space-2)",
      padding: "0.5rem 0.75rem",
      textAlign: "left",
      border: "none",
      borderBottom: "1px solid var(--line)",
      background: "var(--surface-raised)",
      fontSize: "var(--text-caption)",
      fontWeight: "var(--weight-medium)",
      color: "var(--fg)",
      cursor: "pointer"
    }
  }, /*#__PURE__*/React.createElement(Chevron, {
    open: !!open[g.rule]
  }), g.rule, /*#__PURE__*/React.createElement("span", {
    style: {
      marginLeft: "0.5rem",
      fontSize: "var(--text-micro)",
      fontWeight: 400,
      color: "var(--fg-faint)"
    }
  }, g.items.length, " finding", g.items.length === 1 ? "" : "s")), open[g.rule] ? g.items.map(f => /*#__PURE__*/React.createElement(FindingRow, {
    key: f.line + f.desc,
    f: f
  })) : null)), !groups.length ? /*#__PURE__*/React.createElement("p", {
    style: {
      padding: "var(--space-5)",
      margin: 0,
      fontSize: "var(--text-caption)",
      color: "var(--fg-muted)"
    }
  }, "No findings match the current filter.") : null))));
}
Object.assign(window, {
  ValidateScreen
});
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/webapp/ValidateScreen.jsx", error: String((e && e.message) || e) }); }

// ui_kits/webapp/data.jsx
try { (() => {
// Fixture data for the validator recreation. Shapes mirror the real wasm
// report (rule-grouped findings, per-finding line + group + description).
const AGS_TEXT = `"GROUP","PROJ"
"HEADING","PROJ_ID","PROJ_NAME","PROJ_CLNT","FILE_FSET"
"UNIT","","","",""
"TYPE","ID","X","X","X"
"DATA","P2246","A2 Widening — Ground Investigation","National Highways",""
"GROUP","LOCA"
"HEADING","LOCA_ID","LOCA_TYPE","LOCA_NATE","LOCA_NATN","LOCA_GL","LOCA_FDEP"
"UNIT","","","m","m","m","m"
"TYPE","ID","PA","2DP","2DP","2DP","2DP"
"DATA","BH01","CP","523456.12","187654.33","12.30","25.00"
"DATA","BH02","CP","523501.44","187690.10","11.8","24.50"
"DATA","BH03","RC","523560.02","187722.87","10.95","30.00"`;
const FINDINGS = [{
  rule: "Rule 5 — data type",
  count: 3,
  items: [{
    line: 11,
    group: "LOCA",
    severity: "error",
    desc: "LOCA_GL value '11.8' does not match TYPE 2DP (two decimal places expected)",
    field: "11.8"
  }, {
    line: 12,
    group: "LOCA",
    severity: "error",
    desc: "LOCA_FDEP value '30.00' exceeds the maximum recorded depth for RC boreholes",
    field: "30.00"
  }]
}, {
  rule: "Rule 7 — units",
  count: 1,
  items: [{
    line: 8,
    group: "LOCA",
    severity: "warning",
    desc: "LOCA_NATE declares UNIT 'm' — expected 'm' or blank for grid coordinates",
    field: "m"
  }]
}, {
  rule: "Rule 17 — TRAN group",
  count: 1,
  items: [{
    line: 1,
    group: "TRAN",
    severity: "error",
    desc: "Required group TRAN is absent — dictionary edition fell back to 4.1.1",
    field: "PROJ"
  }]
}, {
  rule: "Rule 19b — heading order",
  count: 2,
  items: [{
    line: 4,
    group: "PROJ",
    severity: "fyi",
    desc: "FILE_FSET is present but empty for every DATA row",
    field: "FILE_FSET"
  }]
}];
const GROUPS = [{
  code: "PROJ",
  rows: 1,
  desc: "Project information"
}, {
  code: "LOCA",
  rows: 459,
  desc: "Location details"
}, {
  code: "SAMP",
  rows: 3128,
  desc: "Sample information"
}, {
  code: "GEOL",
  rows: 5402,
  desc: "Field geological descriptions"
}, {
  code: "ISPT",
  rows: 1877,
  desc: "In situ standard penetration test"
}, {
  code: "LLPL",
  rows: 612,
  desc: "Liquid and plastic limit tests"
}];
const LOCA_SCHEMA = [["LOCA_ID", "", "ID", "VARCHAR"], ["LOCA_TYPE", "", "PA", "VARCHAR"], ["LOCA_NATE", "m", "2DP", "DOUBLE"], ["LOCA_NATN", "m", "2DP", "DOUBLE"], ["LOCA_GL", "m", "2DP", "DOUBLE"], ["LOCA_FDEP", "m", "2DP", "DOUBLE"]];
const LOCA_ROWS = [["BH01", "CP", "523456.12", "187654.33", "12.30", "25.00"], ["BH02", "CP", "523501.44", "187690.10", "11.80", "24.50"], ["BH03", "RC", "523560.02", "187722.87", "10.95", "30.00"], ["BH04", "RC", "523612.77", "187760.41", "10.40", "32.00"], ["BH05", "CP", "523668.19", "187801.05", "9.85", "18.00"], ["TP06", "TP", "523701.60", "187844.72", "9.20", "4.50"]];
Object.assign(window, {
  AGS_TEXT,
  FINDINGS,
  GROUPS,
  LOCA_SCHEMA,
  LOCA_ROWS
});
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/webapp/data.jsx", error: String((e && e.message) || e) }); }

__ds_ns.Button = __ds_scope.Button;

__ds_ns.Chevron = __ds_scope.Chevron;

__ds_ns.Chip = __ds_scope.Chip;

__ds_ns.CountBubble = __ds_scope.CountBubble;

__ds_ns.Icon = __ds_scope.Icon;

__ds_ns.Spinner = __ds_scope.Spinner;

__ds_ns.StatusBadge = __ds_scope.StatusBadge;

__ds_ns.SummaryBanner = __ds_scope.SummaryBanner;

__ds_ns.Toast = __ds_scope.Toast;

__ds_ns.Tooltip = __ds_scope.Tooltip;

__ds_ns.Checkbox = __ds_scope.Checkbox;

__ds_ns.ControlGrid = __ds_scope.ControlGrid;

__ds_ns.Field = __ds_scope.Field;

__ds_ns.Input = __ds_scope.Input;

__ds_ns.Select = __ds_scope.Select;

__ds_ns.PillToggle = __ds_scope.PillToggle;

__ds_ns.Tabs = __ds_scope.Tabs;

__ds_ns.ThemeToggle = __ds_scope.ThemeToggle;

__ds_ns.Admonition = __ds_scope.Admonition;

__ds_ns.Card = __ds_scope.Card;

__ds_ns.CodeTabs = __ds_scope.CodeTabs;

__ds_ns.Dialog = __ds_scope.Dialog;

__ds_ns.Disclosure = __ds_scope.Disclosure;

})();

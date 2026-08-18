import { describe, expect, it } from "vitest";
import { iconBody } from "./iconBody";

// A vendored file, in the exact shape lucide-static ships: licence comment
// first, attributes across several lines, two paths.
const REAL = `<!-- @license lucide-static v1.31.0 - ISC -->
<svg
  class="lucide lucide-x"
  xmlns="http://www.w3.org/2000/svg"
  width="24"
  height="24"
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="2"
  stroke-linecap="round"
  stroke-linejoin="round"
>
  <path d="M18 6 6 18" />
  <path d="m6 6 12 12" />
</svg>
`;

describe("iconBody", () => {
  it("returns only the markup inside the root element", () => {
    expect(iconBody(REAL)).toBe(
      `<path d="M18 6 6 18" />\n  <path d="m6 6 12 12" />`,
    );
  });

  // The three things the wrapper must NOT inherit: upstream's fixed 24px box,
  // its class, and the licence comment. <Icon> supplies its own.
  it("drops the licence comment, the class and upstream's fixed size", () => {
    const body = iconBody(REAL);
    expect(body).not.toContain("@license");
    expect(body).not.toContain("lucide-x");
    expect(body).not.toContain('width="24"');
    expect(body).not.toContain("<svg");
  });

  it("keeps a single-element body intact", () => {
    expect(iconBody(`<svg viewBox="0 0 24 24"><circle cx="1"/></svg>`)).toBe(
      `<circle cx="1"/>`,
    );
  });

  it("throws on a file with no <svg> rather than yielding an invisible icon", () => {
    expect(() => iconBody("not an icon")).toThrow(/no <svg>/);
  });

  it("throws on an unterminated <svg> tag", () => {
    expect(() => iconBody("<svg viewBox=")).toThrow(/unterminated/);
  });

  it("throws when the closing tag is missing", () => {
    expect(() => iconBody(`<svg viewBox="0 0 1 1"><path/>`)).toThrow(
      /no closing/,
    );
  });

  // An empty icon renders as nothing at all, which on an icon-only control is
  // an invisible button — worth failing the build over.
  it("throws on an empty body", () => {
    expect(() => iconBody(`<svg viewBox="0 0 1 1">   </svg>`)).toThrow(/empty/);
  });
});

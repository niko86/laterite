import { type Component } from "solid-js";
import { theme, toggleTheme } from "../lib/theme";

/** Header button that flips light/dark. Shows the icon of the theme you'd
 *  switch *to*, the common convention for a single-button toggle.
 *
 *  A checker's label-content-name-mismatch (WCAG 2.5.3) flags this button:
 *  the visible content is a "☀︎"/"☾" glyph and the name doesn't contain it.
 *  That rule exists so a speech-input command matches what's on screen — but
 *  "toggle colour theme, ☀︎" isn't a command anyone would say, and the glyph
 *  carries no word to echo. Contorting the label to embed the symbol would
 *  satisfy the checker and serve no one; left as is. */
export const ThemeToggle: Component = () => {
  return (
    <button
      type="button"
      onClick={toggleTheme}
      title={
        theme() === "dark" ? "Switch to light theme" : "Switch to dark theme"
      }
      aria-label="Toggle colour theme"
      class="rounded-sm border border-line-strong px-2 py-1 text-sm text-fg-soft transition-colors hover:border-accent hover:text-accent"
    >
      {theme() === "dark" ? "☀︎" : "☾"}
    </button>
  );
};

import { type Component } from "solid-js";
import { theme, toggleTheme } from "../lib/theme";

/** Header button that flips light/dark. Shows the icon of the theme you'd
 *  switch *to*, the common convention for a single-button toggle. */
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

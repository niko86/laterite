// Theme store: a signal mirrored to the `.dark` class on <html> and to
// localStorage. The initial value is whatever the no-flash script in
// index.html already applied (persisted choice, else system preference),
// so we read the live class here rather than re-deriving it — keeping the
// store and the pre-paint DOM in lockstep with no flash.

import { createSignal } from "solid-js";

export type Theme = "light" | "dark";

const initial: Theme = document.documentElement.classList.contains("dark")
  ? "dark"
  : "light";

const [theme, setThemeSignal] = createSignal<Theme>(initial);

function apply(t: Theme) {
  document.documentElement.classList.toggle("dark", t === "dark");
  try {
    localStorage.setItem("theme", t);
  } catch {
    // Private-mode / storage-disabled: the toggle still works for the
    // session, it just won't persist. Not worth surfacing.
  }
}

export { theme };

export function setTheme(t: Theme) {
  setThemeSignal(t);
  apply(t);
}

export function toggleTheme() {
  setTheme(theme() === "dark" ? "light" : "dark");
}

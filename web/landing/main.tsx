/* The landing page's entry (#395).
 *
 * Deliberately tiny and deliberately synchronous: everything this module pulls
 * in is on the critical path for a reader who arrived to copy an install
 * command. The demo's engine is NOT here — it is a dynamic import behind first
 * interaction (see demo/engine.ts), so a visitor who never scrolls to the tables
 * never downloads it. */

import { render } from "solid-js/web";
import "./landing.css";
import { App } from "./App";

const root = document.getElementById("root");
if (!root) throw new Error("landing: #root is missing from index.html");

render(() => <App />, root);

/* The beta announcement's entry: a second page of the landing build.
 *
 * It shares the landing stylesheet, so the tokens, the theme and the type
 * scale are the landing page's by construction. It imports nothing from
 * `../demo`, so a reader of the post never downloads the validator engine. */

import { render } from "solid-js/web";
import "../landing.css";
import { Post } from "./Post";

const root = document.getElementById("root");
if (!root) throw new Error("beta: #root is missing from index.html");

render(() => <Post />, root);

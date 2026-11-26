/* The beta announcement, as a long-form page of laterite.dev.
 *
 * Its own header rather than the landing page's Masthead: that one carries the
 * demo's live scoreboard and in-page anchors, and importing it would pull the
 * demo's store into a page that has no demo. The lockup, the theme toggle, the
 * call to action and the strata rule are the same parts, so the two headers
 * read as one site.
 *
 * The measure is capped in characters, not pixels: this is a page for reading,
 * and a line that runs the full content width stops being one.
 */

import { For, type Component, type JSX } from "solid-js";
import { Button, ThemeToggle } from "@shared/components";
import mark from "../../../assets/laterite-icon-128.png";
import { Footer } from "../components/Footer";

const NAV = [
  { href: "/#file", label: "Demo" },
  { href: "/#install", label: "Install" },
  { href: "https://docs.laterite.dev/", label: "Docs" },
  { href: "https://github.com/niko86/laterite", label: "Source" },
];

const LINK =
  "text-accent underline underline-offset-2 decoration-1 transition-colors hover:decoration-2";

const A: Component<{ href: string; children: JSX.Element }> = (props) => (
  <a class={LINK} href={props.href}>
    {props.children}
  </a>
);

const C: Component<{ children: JSX.Element }> = (props) => (
  <code class="rounded-sm bg-surface px-1 py-0.5 font-mono text-[0.9em]">
    {props.children}
  </code>
);

const P: Component<{ children: JSX.Element }> = (props) => (
  <p class="mt-5 text-prose text-fg-soft">{props.children}</p>
);

const H2: Component<{ children: JSX.Element }> = (props) => (
  <h2 class="mt-12 font-display text-h2 font-extrabold text-fg">
    {props.children}
  </h2>
);

const Pre: Component<{ children: string }> = (props) => (
  <pre class="mt-5 overflow-x-auto rounded-sm border border-line bg-surface p-4 font-mono text-caption text-fg">
    {props.children}
  </pre>
);

const Header: Component = () => (
  <header class="sticky top-0 z-30 bg-surface">
    <div class="flex items-center gap-4 px-4 py-2.5 sm:px-6">
      <a
        href="/"
        class="flex items-center gap-2.5 no-underline"
        aria-label="laterite home"
      >
        <img
          src={mark}
          alt=""
          width="28"
          height="28"
          class="size-7 rounded-[5px] dark:bg-[var(--stone-50)] dark:p-[3px]"
        />
        <span class="font-display text-h3 font-extrabold tracking-(--track-tight) text-accent">
          laterite
        </span>
      </a>
      <nav class="ml-auto hidden items-center gap-5 min-[52rem]:flex">
        <For each={NAV}>
          {(item) => (
            <a
              href={item.href}
              class="text-caption text-fg-soft no-underline transition-colors hover:text-accent hover:underline"
            >
              {item.label}
            </a>
          )}
        </For>
      </nav>
      <div class="ml-auto flex items-center gap-2 min-[52rem]:ml-0">
        <ThemeToggle />
        <Button variant="primary" size="sm" href="https://app.laterite.dev/">
          Open webapp
        </Button>
      </div>
    </div>
    <div
      aria-hidden="true"
      class="h-[3px] w-full"
      style={{
        background:
          "linear-gradient(to right, var(--laterite-300), var(--laterite-400), " +
          "var(--laterite-500), var(--laterite-600), var(--laterite-900))",
      }}
    />
  </header>
);

export const Post: Component = () => (
  <div class="min-h-screen bg-canvas text-fg">
    <Header />
    <main>
      <article class="mx-auto max-w-[68ch] px-4 py-10 sm:px-6 sm:py-16">
        <p class="font-mono text-micro tracking-wide text-fg-muted uppercase">
          Announcement
        </p>
        <h1 class="mt-3 font-display text-h1 font-extrabold text-balance text-accent min-[30rem]:text-display">
          laterite is in beta
        </h1>
        <p class="mt-5 text-lead text-fg-soft">
          After many months of work I'm finally comfortable posting about
          laterite, my open-source toolkit for the AGS4 data format. It's
          available now as a public beta, version 0.17.1.
        </p>

        <P>
          laterite started as a pet project to see if I could make AGS4 parsing
          faster. It was also a way to learn how to use AI on something bigger
          than a script or small helper library. For most AGS4 files speed isn't
          a problem. python-ags4 feels instant on small files. But files are
          getting bigger. On major land and offshore infrastructure projects
          they already are, and AGS4 4.2 added 26 new groups, some of them very
          data-heavy. The CPT groups are the obvious example. <C>CPTT</C> holds
          cone readings every 10 to 20 mm over tens of metres of push, and a
          site can have many tests.{" "}
          <A href="https://www.ags.org.uk/data-format/ags4-data-format/ags-l/">
            AGS-L
          </A>
          , the coming AGS extension for advanced laboratory testing, will add
          more.
        </P>

        <P>
          I also had a second reason. I like pandas but I much prefer polars.
          And Python is great, but it isn't the right tool for every job,
          especially where performance matters. A lot of newer Python libraries
          are written in Rust with a thin Python layer on top, using PyO3, so
          you get Rust's speed and Python's ease of use. laterite is built the
          same way.
        </P>

        <P>
          When you read a file with laterite you get polars frames back with
          proper types, so a <C>2DP</C> heading is a float and a <C>DT</C>{" "}
          heading is a datetime. Once the Rust core existed it wasn't much more
          work to offer it in other places too. So there are five ways to use
          it, all in beta:
        </P>
        <ul class="mt-5 list-disc space-y-2 pl-6 text-prose text-fg-soft">
          <li>
            Python: <C>pip install laterite</C>
          </li>
          <li>
            Node: <C>npm i laterite</C>
          </li>
          <li>
            Browser (WebAssembly): <C>npm i @laterite/ags4-wasm</C>
          </li>
          <li>
            DuckDB: <C>INSTALL laterite_ags4 FROM community</C>
          </li>
          <li>
            Command line: the <C>lat</C> binary
          </li>
        </ul>

        <P>
          The DuckDB extension turned out to be more useful than I expected. If
          you want to use a language model with AGS data, SQL is what it knows
          best. It writes SQL better than pandas or polars code, and much better
          than code for a library it has never seen. laterite loads AGS4
          straight into DuckDB, so the model can just write SQL.
        </P>

        <H2>How it compares with python-ags4</H2>
        <P>
          python-ags4 is the AGS Data Format Working Group's own library, and it
          is what I checked laterite against. I wrote every rule from the
          published AGS4 specification, not from anyone else's code, and then
          ran python-ags4's own test suite against my implementation.
        </P>

        <P>
          If you only want the speed, change one import. <C>python_ags4</C>{" "}
          becomes <C>laterite.compat</C>, and your existing code runs four to
          five times faster. laterite's own read is about eleven times faster.
          The benchmark generates its own test files, so you can run it
          yourself. The{" "}
          <A href="https://docs.laterite.dev/cookbook/compat/">
            migration page
          </A>{" "}
          shows how.
        </P>

        <P>
          You can check the compatibility yourself too. python-ags4 1.2.0's test
          suite passes 121 of its 131 tests when run against{" "}
          <C>laterite.compat</C>. I know why each of the other ten fails.
        </P>

        <P>Five fail because laterite doesn't pretend to be python-ags4:</P>
        <Pre>
          {[
            "from laterite import compat",
            'compat.PYTHON_AGS4_COMPAT   # "1.2.0"',
            'compat.__version__          # "…+compat.python-ags4.1.2.0"',
          ].join("\n")}
        </Pre>
        <P>
          I could have reported <C>"1.2.0"</C> and passed them, but then
          anything checking the version would think it was talking to
          python-ags4.
        </P>

        <P>
          Two fail because laterite raises an exception where python-ags4
          returns the error in a dict.
        </P>

        <P>
          Three are real differences in how a bad file is handled. laterite
          refuses an AGS3 file and a non-CSV file, where python-ags4 validates
          them. And laterite reports an embedded carriage return under Rule 6,
          where python-ags4 reports other rules.
        </P>

        <P>
          I plan to keep <C>laterite.compat</C> up to date with future
          python-ags4 releases.
        </P>

        <P>
          Two separate implementations of the same specification are never going
          to agree on everything. I've recorded 61 differences so far, and 27 of
          them change what you see. They're all on the{" "}
          <A href="https://docs.laterite.dev/reference/divergences/">
            divergences page
          </A>
          .
        </P>

        <H2>It's a beta</H2>
        <P>
          Before 1.0, a minor version might break your code, so please read the
          changelog before upgrading. The API will change. The{" "}
          <A href="https://docs.laterite.dev/reference/support/">
            support page
          </A>{" "}
          explains the rules.
        </P>

        <P>
          I think the library is in good shape, but it is new and there will be
          problems. Please install it, point it at a delivery you already have
          and see what happens. If it works, great, and I hope you keep using
          it. What I most want to hear about is where it doesn't work, or where
          it disagrees with python-ags4 in a way I haven't documented. Tell me
          on GitHub{" "}
          <A href="https://github.com/niko86/laterite/discussions">
            Discussions
          </A>
          , or raise an{" "}
          <A href="https://github.com/niko86/laterite/issues">Issue</A> if you
          already know what the problem is. Please don't send me your data. I
          don't need it and I'd rather you didn't. I try to read everything
          weekly and reply. Fixes will come in regular releases, as my spare
          time allows. There's more on the{" "}
          <A href="https://docs.laterite.dev/feedback/">feedback page</A>.
        </P>

        <div class="mt-10 flex flex-wrap items-center gap-3">
          <Button variant="primary" size="lg" href="/#file">
            See it catch faults
          </Button>
          <Button variant="outline" size="lg" href="/#install">
            Pick your stack
          </Button>
        </div>
      </article>
    </main>
    <Footer />
  </div>
);

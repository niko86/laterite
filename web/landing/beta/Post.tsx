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
          I'm pleased after many months to finally be at the point where I feel
          comfortable to post about laterite my open-source toolkit for the AGS4
          data format, it's available in public beta as version 0.17.1.
        </p>

        <P>
          laterite began as a pet project to make AGS4 parsing faster while
          learning how to use AI for something larger. For most AGS4 files speed
          isn't really an issue that needs solving, python-ags4 is fast enough
          on small files to feel instant. However, moving to larger scale
          projects such as major land and offshore infrastructure projects the
          file sizes grow. With the release of AGS4 4.2 which introduced 26 new
          groups, some of which are data-heavy such as the entire CPT family (
          <C>CPTG</C>, <C>CPTT</C>, <C>CPTM</C>, <C>CPTP</C>, <C>CPTZ</C>, plus
          cyclic testing and pore-pressure dissipation), in-situ seismic,
          dilatometer and pressuremeter testing. For example, <C>CPTT</C> with
          cone readings every 10–20 mm over tens of metres of push, many tests
          to a site. That is how a format that was fine at small files stopped
          producing small files, and{" "}
          <A href="https://www.ags.org.uk/data-format/ags4-data-format/ags-l/">
            AGS-L
          </A>
          , AGS's coming extension for advanced laboratory testing, points the
          same way.
        </P>

        <P>
          There was a second reason, while I like the Python pandas library I
          personally much prefer polars. Additionally, while Python, good as it
          is, is not the right tool for every task especially where greater
          performance might be needed and this is seen in many libraries being
          created in Rust and having thin python libraries created using pyO3 to
          get the best of both worlds rust performance and python ease of use.
        </P>

        <P>
          laterite resolves both the way much of modern Python packaging now
          does: a Rust core with a thin Python library over it. Files come back
          as polars frames, born-typed: a <C>2DP</C> heading is a float, a{" "}
          <C>DT</C> a datetime. Once the core existed, other surfaces followed
          almost for free: a Node library, a DuckDB extension, a WebAssembly
          build for the browser, the <C>lat</C> CLI. One engine, several
          languages. laterite is in beta, all five of them:{" "}
          <C>pip install laterite</C>, <C>npm i laterite</C>,{" "}
          <C>npm i @laterite/ags4-wasm</C>,{" "}
          <C>INSTALL laterite_ags4 FROM community</C>, and the <C>lat</C>{" "}
          binary.
        </P>

        <P>
          The DuckDB surface earns its keep for a reason that wasn't so obvious
          to me when I started. If you want a language model anywhere near a
          delivery, SQL is the language it actually knows, even better than
          pandas or polars, and far better than laterite's own API. laterite
          materialises AGS4 straight into DuckDB, so the model writes SQL, which
          it is good at, instead of guessing at a bespoke library, which it is
          not.
        </P>

        <P>
          python-ags4, the AGS Data Format Working Group's own library, is what
          laterite is calibrated against: every rule here is written from the
          published AGS4 specification, not adapted from anyone's source, and
          running python-ags4's test suite against my implementation was the
          benchmark.
        </P>

        <P>
          If speed is all you want, change one import in your python file,{" "}
          <C>python_ags4</C> becomes <C>laterite.compat</C> and your existing
          code runs four to five times faster. (laterite's native read is about
          eleven times; the benchmark generates its own synthetic fixtures and
          pins them by SHA-256, so the numbers are reproducible rather than
          quoted.) The migration page is at{" "}
          <A href="https://docs.laterite.dev/cookbook/compat/">
            docs.laterite.dev/cookbook/compat
          </A>
          .
        </P>

        <P>
          You can check this yourself using python-ags4 1.2.0's own test suite,
          run against <C>laterite.compat</C>, passes 121 of 131 unchanged, and
          the ten that don't itemise cleanly. Five fail because the shim
          declines to claim it <em>is</em> python-ags4:
        </P>
        <Pre>
          {[
            "from laterite import compat",
            'compat.PYTHON_AGS4_COMPAT   # "1.2.0"',
            'compat.__version__          # "…+compat.python-ags4.1.2.0"',
          ].join("\n")}
        </Pre>

        <P>
          Claiming <C>"1.2.0"</C> outright would have closed those five. I
          didn't, because anyone or any system checking the version would then
          silently believe they were talking to python-ags4. The other five are
          the drop-in being honest about different implementation approaches.
          Two are a Python-idiom choice - an exception where python-ags4 returns
          an error as a dict. The last three are genuine divergences in how a
          bad file is handled: laterite refuses an AGS3 file and a non-CSV file
          where python-ags4 validates them, and it reports an embedded carriage
          return under Rule 6 where python-ags4 reports other rules. I intend to
          keep compat tracking future python-ags4 releases.
        </P>

        <P>
          Two independent implementations of one specification will disagree,
          there are 61 observations catalogued, 27 that change what you see,
          each filed by whether laterite differs from python-ags4, both depart
          from the written specification, laterite changed to match, or laterite
          adds a check python-ags4 has no counterpart for.
        </P>

        <P>
          The library is a beta, and pre-1.0 a minor version may break you, so
          read the changelog before upgrading. The rule is on the{" "}
          <A href="https://docs.laterite.dev/reference/support/">
            support page
          </A>
          . Expect churn in the API.
        </P>

        <P>
          While I feel the library is polished and working, it is new and there
          will be issues. So please download the library and point laterite at a
          delivery you already have and see what happens. If it works fine
          that's great and please consider using laterite long term. I'm
          extremely interested though in if things don't work, so if you can
          tell me where it either doesn't work or disagrees with python-ags4 in
          ways I haven't documented then please let me know through Github{" "}
          <A href="https://github.com/niko86/laterite/discussions">
            Discussions
          </A>{" "}
          or <A href="https://github.com/niko86/laterite/issues">Issues</A> if
          you already know what the problem is. You don't need to send your data
          and I'd prefer and advise not to do so. I aim to read everything
          weekly and reply. Fixes will be delivered in regular release cycles,
          the regularity depending on my spare time. Details on the{" "}
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

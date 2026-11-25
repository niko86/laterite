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

const INSTALL = [
  "pip install laterite",
  "npm i laterite",
  "npm i @laterite/ags4-wasm",
  "INSTALL laterite_ags4 FROM community;",
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
          laterite, an open-source toolkit for the AGS4 geotechnical data
          format, is in public beta today as version 0.17.1.
        </p>

        <P>
          It began as an itch. AGS4 parsing felt slower than it needed to be,
          and I wanted to know how much faster it could go. That question has an
          answer now, about sixteen times for validation, and it has turned out
          to be the least interesting thing about it. For most of AGS4's life
          the slowness was not much of a problem, because deliveries were small.
        </P>

        <H2>The files got bigger</H2>
        <P>
          Then the format changed shape. AGS4 4.2 introduced 26 new groups, and
          they are the data-heavy ones: the whole CPT family (<C>CPTG</C>,{" "}
          <C>CPTT</C>, <C>CPTM</C>, <C>CPTP</C>, <C>CPTZ</C>, plus cyclic
          testing and pore-pressure dissipation), in-situ seismic, dilatometer
          and pressuremeter testing. <C>CPTT</C> is the one to picture: cone
          readings every 10 to 20 mm over tens of metres of push, with many
          tests to a site. A format that was fine at small files stopped
          producing small files.{" "}
          <A href="https://www.ags.org.uk/data-format/ags4-data-format/ags-l/">
            AGS-L
          </A>
          , the coming AGS extension for advanced laboratory testing, points the
          same way.
        </P>

        <H2>One engine, five ways in</H2>
        <P>
          laterite is a Rust core with a thin Python library over it. Files come
          back as polars frames with real types: a <C>2DP</C> heading is a float
          and a <C>DT</C> heading is a datetime. Once the core existed, the
          other surfaces followed: a Node library, a DuckDB extension, a
          WebAssembly build for the browser, and the <C>lat</C> command line
          tool. All five are in beta.
        </P>
        <Pre>{INSTALL.join("\n")}</Pre>
        <P>
          The DuckDB extension earns its place for a reason that did not exist
          when I started. If you want a language model anywhere near a delivery,
          SQL is the language it knows best. laterite reads AGS4 straight into
          DuckDB, so the model writes SQL instead of guessing at a library it
          has never seen.
        </P>

        <H2>If you use python-ags4</H2>
        <P>
          python-ags4, the AGS Data Format Working Group's own library, is what
          laterite is checked against. Every rule here is written from the
          published AGS4 specification, and python-ags4's test suite is run
          against the result.
        </P>
        <P>
          That check is also a way in. Change one token, so that{" "}
          <C>python_ags4</C> becomes <C>laterite.compat</C>, and your existing
          code runs four to five times faster. laterite's own typed read is
          about eleven times faster. The benchmark generates its own synthetic
          files, so you can reproduce the figures. The{" "}
          <A href="https://docs.laterite.dev/cookbook/compat/">
            migration page
          </A>{" "}
          has the details.
        </P>
        <P>
          Here is what backs that claim. The test suite of python-ags4 1.2.0,
          run against <C>laterite.compat</C>, passes 121 of 131 tests unchanged.
          Five of the ten that fail do so because laterite declines to say it is
          python-ags4:
        </P>
        <Pre>
          {[
            "from laterite import compat",
            'compat.PYTHON_AGS4_COMPAT   # "1.2.0"',
            'compat.__version__          # "…+compat.python-ags4.1.2.0"',
          ].join("\n")}
        </Pre>
        <P>
          Reporting <C>"1.2.0"</C> outright would have passed those five, and
          anyone checking the version would then believe they were talking to
          python-ags4. Two more are a choice of Python idiom: laterite raises an
          exception where python-ags4 returns the error in a dictionary. The
          last three are real differences in how a bad file is handled. laterite
          refuses an AGS3 file and a file that is not CSV, where python-ags4
          validates both as AGS4. It also reports a line break inside a quoted
          field under Rule 6, where python-ags4 reports other rules. I intend to
          keep the compatibility layer tracking future python-ags4 releases.
          That is an intention and not a promise, because there is one of me.
        </P>

        <H2>Where the two disagree</H2>
        <P>
          Two independent implementations of one specification will disagree,
          and I write every disagreement down. There are 61 so far, 27 of which
          change what you see. Each is filed by kind: laterite differs from
          python-ags4, both depart from the written specification, laterite
          changed to match, or laterite adds a check python-ags4 has no
          counterpart for. The{" "}
          <A href="https://docs.laterite.dev/reference/divergences/">
            divergences page
          </A>{" "}
          lists them. Three give the flavour:
        </P>
        <ul class="mt-5 list-disc space-y-3 pl-6 text-prose text-fg-soft">
          <li>
            python-ags4 raises <C>AGS4Error</C> on a duplicate <C>GROUP</C> or a
            ragged <C>DATA</C> row before it returns any data, so the files most
            in need of a findings list get none. laterite reports the problem
            and keeps going.
          </li>
          <li>
            <C>TRAN_AGS="4.0"</C> is ambiguous. python-ags4 picks the older
            4.0.3 edition and over-reports Rule 10c on <C>PMTL</C>. laterite
            resolves it to 4.0.4, which contains everything 4.0.3 does.
          </li>
          <li>
            python-ags4's Rule 6 check is empty, so a carriage return inside a
            quoted field is never reported as the Rule 6 fault it is.
          </li>
        </ul>
        <P>
          It cuts both ways. The comparison caught laterite missing an empty{" "}
          <C>DT</C> unit and accepting the year <C>0018</C>, and both are fixed.
          The findings worth reporting are going to the Working Group, who own
          both the library and the specification. The aim is to complement
          python-ags4, not to displace it.
        </P>

        <H2>What beta means</H2>
        <P>
          Before 1.0 a minor version may break your code, so read the changelog
          before you upgrade. Expect the API to change. Do not expect the
          platforms it runs on to change. Both rules are on the{" "}
          <A href="https://docs.laterite.dev/reference/support/">
            support page
          </A>
          .
        </P>

        <H2>The ask</H2>
        <P>
          Point laterite at a delivery you already have and see what happens.
          The most useful thing you can tell me is where it disagrees with
          python-ags4. You never need to send data: the shape of the problem is
          enough to build a synthetic file from.{" "}
          <A href="https://github.com/niko86/laterite/discussions">
            Discussions
          </A>{" "}
          is the front door, and{" "}
          <A href="https://github.com/niko86/laterite/issues">Issues</A> is
          there if you already know what you have. I aim to read everything
          weekly. That is a goal and not a promise, and it means a reply rather
          than a fix. The{" "}
          <A href="https://docs.laterite.dev/feedback/">feedback page</A> has
          the details.
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

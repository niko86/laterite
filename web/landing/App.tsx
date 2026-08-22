/* The page (#395).
 *
 * Seven sections, each owning one band of the strata ramp, in the order the
 * rail descends: the surface, the four groups, the file the groups emit, and the
 * install grid. `sections.ts` holds that sequence because #399's rail reads it
 * too — a rail with its own copy goes out of step the first time a section moves.
 *
 * Group sections ALTERNATE which side the table takes, and the table column is
 * the wider of the two. That is the rhythm the rail exists to mark: without it
 * the page is four identical slabs and the descent reads as repetition rather
 * than depth.
 */

import { For, Show, type Component, type JSX } from "solid-js";
import { Masthead } from "./components/Masthead";
import { Hero } from "./components/Hero";
import { InstallGrid } from "./components/InstallGrid";
import { Footer } from "./components/Footer";
import { Rail } from "./components/Rail";
import { GroupSection } from "./demo/GroupSection";
import { bindUndoShortcuts } from "./demo/store";
import { FileAndFindings } from "./demo/FileAndFindings";
import { SECTIONS, bandVar } from "./sections";

/** One band of the page. The hairline and the band chip are the section's
 *  only chrome; everything else is the section's own content. */
const Section: Component<{
  id: string;
  index: number;
  children: JSX.Element;
}> = (props) => (
  <section
    id={props.id}
    class="border-t border-line first:border-t-0 scroll-mt-16"
  >
    <div class="mx-auto max-w-[72rem] px-4 py-12 sm:px-6 min-[68rem]:py-16">
      {props.children}
    </div>
  </section>
);

export const App: Component = () => {
  // Page-wide undo/redo (#525): one binding covers both editors, living and
  // dying with the page component (the binder registers its own onCleanup).
  bindUndoShortcuts();
  return (
    <div class="min-h-screen bg-canvas text-fg">
      <Rail />
      <Masthead />

      {/* The rail is a sibling of the masthead and sits below it, so the content
        column is inset by the gutter rather than overlapped by it. Below the
        collapse breakpoint the rail is 8px and the column reclaims the space. */}
      <div class="min-[68rem]:pl-24">
        <main>
          <For each={SECTIONS}>
            {(section, i) => (
              <Section id={section.id} index={i()}>
                <Show when={section.id === "top"}>
                  <Hero />
                </Show>
                <Show when={section.group}>
                  {(code) => (
                    <GroupSection
                      code={code()}
                      band={bandVar(i())}
                      /* Alternate the table side. The first group section is
                       index 1, so the odd ones lead with the table and the
                       even ones lead with the prose. */
                      tableFirst={i() % 2 === 1}
                    />
                  )}
                </Show>
                <Show when={section.id === "file"}>
                  <FileAndFindings band={bandVar(i())} />
                </Show>
                <Show when={section.id === "install"}>
                  <InstallGrid />
                </Show>
              </Section>
            )}
          </For>
        </main>
        <Footer />
      </div>
    </div>
  );
};

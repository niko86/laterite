// Group-catalogue master table: client-side filter + 20-per-page pagination.
// Vanilla JS, no third-party dependency (keeps the docs CDN-free / offline). Runs
// on every page via Material's `document$` observable (re-fires on instant nav);
// it no-ops unless the catalogue landing's `.group-table` is present.
const PER_PAGE = 20;
const FAMILY_COL = 2; // Code | Group | Family | Parent | Headings | Editions

function initGroupTable() {
  const wrap = document.querySelector(".group-table");
  if (!wrap) return;
  const table = wrap.querySelector("table");
  if (!table || !table.tBodies.length || table.dataset.paged) return;
  table.dataset.paged = "1";

  const rows = Array.from(table.tBodies[0].rows);
  let page = 0;
  let filtered = rows;

  const search = document.createElement("input");
  search.type = "search";
  search.className = "group-filter";
  search.placeholder = `Filter ${rows.length} groups by code, name, family or edition…`;
  // Material wraps the table in a `.md-typeset__table` div at runtime, so the
  // table is NOT a direct child of `wrap` — prepend to `wrap` instead of
  // insertBefore(table) (which would throw NotFoundError and abort init).
  wrap.prepend(search);

  const pager = document.createElement("nav");
  pager.className = "group-pager";
  wrap.appendChild(pager);

  const familyOf = (r) => r.cells[FAMILY_COL]?.textContent.trim() ?? "";

  function render() {
    const pages = Math.max(1, Math.ceil(filtered.length / PER_PAGE));
    if (page >= pages) page = pages - 1;
    rows.forEach((r) => (r.hidden = true));
    filtered
      .slice(page * PER_PAGE, page * PER_PAGE + PER_PAGE)
      .forEach((r) => (r.hidden = false));

    const isFiltered = filtered.length !== rows.length;
    pager.replaceChildren();

    const prev = document.createElement("button");
    prev.type = "button";
    prev.textContent = "‹ Prev";
    prev.disabled = page === 0;
    prev.addEventListener("click", () => {
      page -= 1;
      render();
    });

    const info = document.createElement("span");
    info.className = "group-pager-info";
    info.textContent = isFiltered
      ? `${filtered.length} of ${rows.length} groups · page ${page + 1} / ${pages}`
      : `${rows.length} groups · page ${page + 1} / ${pages}`;

    const next = document.createElement("button");
    next.type = "button";
    next.textContent = "Next ›";
    next.disabled = page >= pages - 1;
    next.addEventListener("click", () => {
      page += 1;
      render();
    });

    pager.append(prev, info, next);

    // A filter is active → offer a way back to the full table (the upload was
    // "no way back except refresh / re-click the sidebar"). Only shown when filtered.
    if (isFiltered) {
      const clear = document.createElement("button");
      clear.type = "button";
      clear.className = "group-clear";
      clear.textContent = `✕ Show all ${rows.length}`;
      clear.addEventListener("click", () => {
        search.value = "";
        setFiltered(rows);
      });
      pager.append(clear);
    }
  }

  function setFiltered(next) {
    filtered = next;
    page = 0;
    render();
  }

  function filter(query) {
    const q = query.trim().toLowerCase();
    setFiltered(
      q ? rows.filter((r) => r.textContent.toLowerCase().includes(q)) : rows,
    );
  }

  search.addEventListener("input", () => filter(search.value));

  // Family cards filter the table by an EXACT family-column match (a substring
  // search would also catch groups whose description merely mentions the word).
  document.querySelectorAll("[data-family]").forEach((card) => {
    card.addEventListener("click", (e) => {
      e.preventDefault();
      const fam = card.dataset.family;
      search.value = fam;
      setFiltered(rows.filter((r) => familyOf(r) === fam));
      table.scrollIntoView({ behavior: "smooth", block: "start" });
    });
  });

  render();
}

// Prefer Material's `document$` observable (re-fires on instant navigation);
// fall back to a one-shot DOM-ready hook if it is somehow absent.
if (typeof document$ !== "undefined") {
  document$.subscribe(initGroupTable);
} else if (document.readyState !== "loading") {
  initGroupTable();
} else {
  document.addEventListener("DOMContentLoaded", initGroupTable);
}

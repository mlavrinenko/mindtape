#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Write the links fixture and page",
  kind: "docs",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("docs", "www"),
  estimate: (files: 15),
  assignee: "agent",
  status: done(
    2026,
    9,
    18,
  )[examples/links is checked and sliced into www/generated/links.json, and /links/ renders the slices; /recurring/ follows in its own task],
  x-goals: (
    product: (
      readable-tasks: helps[the link reference is real task files read back as data, so a reader sees the DSL as files rather than as prose examples],
      implementable-tasks: neutral,
      enforced-practice: helps[a slice that stops matching the fixture fails the build, and the fixture itself is validated by check-examples],
      cheap-adoption: helps[a small SWE-shaped example is readable in one sitting and adds no tooling to adopt],
      typst-over-markdown: helps[the explanation is Typst task files, sliced by a script, not fenced pseudocode that only looks like the DSL],
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the page can only show what a checked fixture contains, so a DSL change breaks the gate instead of silently outdating a code block],
    ),
  ),
)

== Problem

1. `www/content/dsl.typ`'s "Task links" section is written from invented
  names — `design-schema.typ`, `wire-oauth`, `acme/app/pull/42`. None exists
  anywhere in the repo, so every block there is a code block that cannot work,
  and a DSL change leaves it lying.
2. `examples/learn-piano/` carries two real edges, but it is a piano tracker;
  no checked fixture exercises all six built-in helpers, a custom kind, or a
  target validated as something other than a local stem.
3. 🔴 `examples/links/` does not exist, and `just check-examples` validates
  only `examples/learn-piano` and `examples/minimal`.

== TO-BE

- 🟢 `examples/links/` is a checked SWE-flavoured project whose tasks carry
  `depends-on`, `parent`, `related`, `supersedes`, `duplicates`, and an
  `edge(kind, target)` of a `[[tasks.links.kinds]]` kind validated as a URL.
- 🟢 `www/generate-examples.sh` slices the fixture's real files into
  `www/generated/links.json`, the same marker trim `readme-task.json` uses.
- 🟢 `www/content/links.typ` renders those slices as one annotated code block
  and cross-links the config page for the custom-kind shape.
- 🟢 `just check-examples` validates the fixture, and `.mmz/config.yaml`'s
  `examples` and `typ-fmt` scopes list it so the gate cannot go false-fresh.
- 🟢 `just dup-prose` reports no pair whose home is this page.

== Source of truth

- `lib/prelude.typ` — the helper definitions and their argument shapes.
- `www/content/config.typ` — `[[tasks.links.kinds]]` and the `validate`
  variants the fixture's custom kind picks from.
- `www/generate-facts.sh` — the readme-task slice, the pattern this reuses.
- `www/generate-self-validating.sh` — a fixture with its own generator, the
  precedent for wiring a fixture the docs derive from.
- `.mmz/config.yaml` — why a checked fixture must appear in the `examples`
  scope by name.

== Changes

- `examples/links/` (new) — config with the local-package override and the
  custom kind, plus seven tasks covering the built-in helpers and the edge.
- `www/generate-examples.sh` (new) — slice `examples/links/tasks/*.typ` to
  `www/generated/links.json` (stem, title, file, marker).
- `www/generate.sh` — call the new script inside the existing lock.
- `Justfile` and `.mmz/config.yaml` — the check-examples arm and both scopes.
- `www/content/links.typ` (new) and `www/utils/site.typ` — the page and its
  nav entry.

== Out of scope

- The recurring fixture and page. `recurred-from` belongs to that one; its
  task carries them.
- The `/dsl/` rewrite. This task makes the link section's new home exist;
  wave 3 cuts the old section to a pointer.
- The NAV regroup. The page joins the current group now; wave 3 regroups.

== Dogfood

The page is written from the derived slices alone, with the cross-links the
documentation rule asks for, and `just dup-prose` runs before and after.

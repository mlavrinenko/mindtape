#import "@local/mindtape:0.3.0": *
#import "/truth/lib.typ": *

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
  )[the build checks examples/links and slices it into www/generated/links.json. /links/ renders the slices. /recurring/ follows in its own task.],
  x-goals: (
    product: (
      readable-tasks: helps[the link reference is real task files read back as data. A reader sees the DSL as files, not prose examples.],
      implementable-tasks: neutral,
      enforced-practice: helps[a stale slice fails the build, and `check-examples` validates the fixture.],
      cheap-adoption: helps[a small example fits the tooling already present.],
      typst-over-markdown: helps[task files carry it.],
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the page shows only what a checked fixture contains. A DSL change then breaks the gate instead of silently leaving a code block out of date],
    ),
  ),
)

== AS-IS

1. `www/content/dsl.typ`'s "Task links" section comes from invented
  names: `design-schema.typ`, `wire-oauth`, `acme/app/pull/42`. None exists
  anywhere in the repo, so every block there is a code block that cannot work,
  and a DSL change leaves it lying.
2. `examples/learn-piano/` carries two real edges, but it is a piano tracker.
  No checked fixture exercises all six built-in helpers, a custom kind, or a
  target validated as something other than a local stem.
3. 🔴 `examples/links/` does not exist, and `just check-examples` validates
  only `examples/learn-piano` and `examples/minimal`.

== TO-BE

- 🟢 `examples/links/` is a checked project shaped like software work, whose tasks carry
  `depends-on`, `parent`, `related`, `supersedes`, `duplicates`, and an
  `edge(kind, target)` of a `[[tasks.links.kinds]]` kind validated as a URL.
- 🟢 `www/generate-examples.sh` slices the fixture's real files into
  `www/generated/links.json`, the same marker trim `readme-task.json` uses.
- 🟢 `www/content/links.typ` renders those slices as one annotated code block
  and cross-links the config page for the custom-kind shape.
- 🟢 `just check-examples` validates the fixture, and `.mmz/config.yaml`'s
  `examples` and `typ-fmt` scopes list it so the gate cannot go false-fresh.
- 🟢 `just dup-prose` stays silent about pairs whose home is this page.

== Source of truth

- `lib/prelude.typ`: the helper definitions and their argument shapes.
- `www/content/config.typ`: `[[tasks.links.kinds]]` and the `validate`
  variants the fixture's custom kind picks from.
- `www/generate-facts.sh`: the readme-task slice, the pattern this reuses.
- `www/generate-self-validating.sh`: a fixture with its own generator, the
  precedent for wiring a fixture the docs derive from.
- `.mmz/config.yaml`: why a checked fixture must appear in the `examples`
  scope by name.

== Changes

- `examples/links/` (new): config with the local-package override and the
  custom kind, plus seven tasks covering the built-in helpers and the edge.
- `www/generate-examples.sh` (new): slice `examples/links/tasks/*.typ` to
  `www/generated/links.json` (stem, title, file, marker).
- `www/generate.sh`: call the new script inside the existing lock.
- `Justfile` and `.mmz/config.yaml`: the check-examples arm and both scopes.
- `www/content/links.typ` (new) and `www/utils/site.typ`: the page and its
  nav entry.

== Out of scope

- The recurring fixture and page. `recurred-from` belongs to that one. Its
  task carries them.
- The `/dsl/` rewrite. This task makes the link section's new home exist.
  Wave 3 cuts the old section to a pointer.
- The NAV regroup. The page joins the current group, and wave 3 regroups.

== Dogfood

The page comes from the derived slices alone, with the cross-links the
documentation rule asks for, and `just dup-prose` runs before and after.

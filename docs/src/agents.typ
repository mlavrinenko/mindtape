// docs/src/agents.typ — source for the repo-root AGENTS.md (CLAUDE.md is a
// symlink to it). Rendered to Markdown by `docs/generate-md.sh` via typlite;
// see docs/src/lib.typ for what that render path can and cannot do.
//
// Facts this file used to hand-list read from `www/generated/crate-map.json`
// (`www/generate-facts.sh`) instead: the workspace/substrate crate lists,
// their versions and direct-vs-transitive status, the `evalexpr`/`clap`/Typst
// major versions in Architecture, and the `@local/mindtape` version in the
// Task DSL snippet. See docs/contributing/generated-docs.md for the pipeline
// this feeds and why each derived fact moved here.
//
// Every docs-site pointer goes through `www-link` (docs/src/lib.typ), which
// resolves the route against the generated page manifest and panics on an
// unknown one. A hand-typed site URL here would rot unnoticed — nothing in
// this repo resolves a URL sitting in generated Markdown.
//
// This file is on the always-loaded path, so its budget is a real cost, not
// an aesthetic one: prose that motivates a rule an agent follows anyway, a
// rule stated twice, and a pointer that restates the page it points at are
// all deleted on sight. Every rule, command shape and field name stays.
#import "lib.typ": doc-title, fact, just, num-word, www-link

#metadata((
  output: "AGENTS.md",
)) <mindtape-md>

#doc-title[MindTape — AI Agent Context]

How work is done lives in \@CONTRIBUTING.md, pulled into your context whole by
that reference; a process rule belongs there and is never restated here. This
file carries the other half: what MindTape _is_.

= What is this?

A daemon-free, file-based task tracker. Tasks are one-per-file Typst artifacts
(`#show: task.with(...)`). MindTape live-evaluates the configured roots on every
command via the Typst compiler crates — no SQLite, no global index. Identity is
the filename stem.

Repository: https://github.com/mlavrinenko/mindtape

= Repo map

- `lib/prelude.typ` — the `@local/mindtape` Typst package (the task DSL)
- `truth/tasks/` — MindTape's own backlog as per-file task artifacts (dogfood);
  `mt ls` / `mt check` run against it via the in-tree `lib/`
- `www/content/*.typ` — the docs site (tola)
- `docs/contributing/*.md` — contributor deep dives behind CONTRIBUTING.md
- `examples/` — `learn-piano` (full config surface; feeds the docs generator and
  the skill's evals), `minimal` (the floor config), and `large` (a 2000-task
  benchmark fixture; its tasks are gitignored, #just.gen-bench() rebuilds them)

Reference material lives on the docs site, not here. Before writing about a
feature, read whichever page below already answers it rather than restating it —
the page at `/<stem>/` is `www/content/<stem>.typ`.

#table(
  columns: 2,
  table.header([Topic], [Read]),
  [Every task field, `links:` edge kinds, `recur`], www-link("/dsl/"),
  [`mt add`, slots, `--set`, `--describe`, `--edit`], www-link("/templates/"),
  [Derived tags from folder layout], www-link("/derived-tags/"),
  [Priority frameworks, scoring, bands], www-link("/priority/"),
  [Filter language], www-link("/filters/"),
  [Every config key], www-link("/config/"),
  [Driving `mt` as an agent], [`skills/mindtape/SKILL.md`],
)

= Workspace

// Facts (names, versions, direct-vs-transitive) come from
// www/generated/crate-map.json; the one-phrase-per-crate role below is
// editorial and stays hand-written here — deliberately terser than
// www/utils/crate-notes.typ's paragraph-length blurbs, which back
// www/content/internals.typ's deep dive instead. `.at(...)` on a missing key
// panics, so an eighth substrate crate (or a new workspace member) with no
// entry here fails `just docs::md` loudly instead of rendering silently thin.
#let workspace = fact("crate-map").at("workspace")
#let workspace-roles = (
  "mindtape": [domain library: config, task model, filter language,
    validation, project loader. Other tools (e.g. speconaut) depend on
    this.],
  "mindtape-cli": [the thin `mt` binary (clap).],
)

#upper(num-word(workspace.len()).first())#num-word(workspace.len()).slice(1) crates
(`cargo` workspace, virtual root manifest):

#for c in workspace [
  - #raw("crates/" + c.name) — #workspace-roles.at(c.name)
]

#let substrate = fact("crate-map").at("substrate")
#let substrate-roles = (
  "typst-world": [package-agnostic Typst World/eval],
  "typst-harvest": [harvest],
  "typst-cst": [parse-plane structure extraction],
  "typst-edit": [slot/link rewriting],
  "typst-cleanup": [git-aware cleanup],
  "swhid-mint": [SWHID minting],
  "artifact-gate": [artifact validation],
)
#let transitive = substrate.filter(c => not c.direct)

The #num-word(substrate.len()) former in-tree substrate crates are published to
crates.io and consumed back as regular registry dependencies:

// A transitive crate names the parent that pulls it, straight from
// crate-map.json's `via` (the cargo resolve graph). AGENTS.md carried
// "typst-world is pulled transitively via typst-harvest" by hand: true when
// written, with no way to stay true.
#for c in substrate [
  - #raw(c.name) #raw(c.version)#(
      if not c.direct [ (transitive, via #c.via.map(raw).join(", "))]
    ) — #substrate-roles.at(c.name)
]

Both lists — name, version, direct-vs-transitive — are derived from
`www/generated/crate-map.json`, never hand-written.
#if transitive.len() > 0 [
  A crate marked transitive is named in no workspace manifest.
]

= Architecture

#let arch = fact("crate-map").at("arch-deps")
#let dep-version(name) = arch.filter(d => d.name == name).at(0).version
#let major(v) = {
  let parts = v.split(".")
  if parts.at(0) == "0" { "0." + parts.at(1) } else { parts.at(0) }
}
#let edition = fact("crate-map").at("workspace-package").at("edition")

- Rust (edition #edition, pinned `rust-toolchain.toml`)
- Typst #major(dep-version("typst")) eval-only (`typst-eval`); never layout or render
- `evalexpr` #major(dep-version("evalexpr")) as a parser for the filter language (walked, not evaluated by it)
- `clap` #major(dep-version("clap")) CLI; `serde`/`toml` config; a hand-rolled table renderer
  (`crates/mindtape-cli/src/render/`), `serde_json`/`csv` output

= Task DSL (lib/prelude.typ)

#let typst-lib-version = fact("crate-map").at("typst-lib").at("version")
#let dsl-lines = (
  "#import \"@local/mindtape:" + typst-lib-version + "\": *",
  "#show: task.with(",
  "  title: \"Title\",",
  "  status: wip(2026, 6, 14)[note],    // (name, date, note); note is a trailing block",
  "  priority: ice(impact: 8, confidence: 0.7, ease: 6), // or a raw int; difficulty too",
  "  estimate: (days: 1, hours: 2),     // opaque unit dict; allowed units per project",
  "  assignee: \"ada\",                   // actor working it; config-bounded, not ownership",
  "  links: depends-on(\"other.typ\"),    // typed edges by file stem",
  "  tags: (\"backend\",),",
  "  x-speconaut: ...,                  // opaque extension slot (x-<ns>)",
  ")",
)
#raw(dsl-lines.join("\n"), lang: "typ", block: true)

`task` declares every core field as a named parameter (`..meta` is the `x-<ns>`
extension sink only), so tinymist completes the field names and shows the whole
signature; a misspelled field reaches the sink and fails the file with an
`unknown task slot` error naming the field, e.g. `titel`.

Values are shapes, never computed results: `ice(...)` and the other framework
packagers build a `(framework, inputs)` dict, `every(...)`/`rrule(...)` build a
recurrence dict, `depends-on("a.typ") + related("b.typ")[why]` builds typed edges.
No math and no date arithmetic happen in Typst — the Rust layer does all of it.

== Where validation lives

The seam that decides which layer a change belongs in:

- The prelude enforces structure only — one task, the `x-<ns>` discipline, and
  the argument shapes it would otherwise drop before the marker is written (edge
  targets and notes, a status note, framework inputs). Those are the mistakes the
  Rust layer can never see, because nothing reaches it to validate.
- Everything that DOES reach the marker is the Rust layer's: the status legend,
  the numeric priority/difficulty scales and classification, priority frameworks
  (scoring, in-bounds by construction), the edge-kind enum and its per-kind rules,
  the `required` rules (a filter expression per field), bounds, and link
  resolution. Types too — `crates/mindtape/src/marker.rs` reports a wrong-shaped
  field, and any wrong-typed element inside a well-shaped one, naming the
  position (`tags[1]`, `estimate.days`, `recur.on[1]`).
- The card render enforces nothing. It is total by construction — every accessor
  type-guards, `_scalar` falls back to `repr` — so a wrong-typed value reaches
  the marker instead of failing eval on a method it does not have. An eval error
  there names neither the field nor the task, and disappears the moment the
  render stops making that call.
- Consumers validate their own `x-<ns>` fields. MindTape carries them through
  untouched.

So: a new constraint on a value goes in Rust and config. A new constraint on the
_call shape_ goes in the prelude. Putting either in the other place produces a
rule that cannot fire.

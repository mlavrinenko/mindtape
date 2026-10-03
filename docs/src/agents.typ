// Source for the repo-root AGENTS.md, rendered by docs/generate-md.sh with
// typlite. CLAUDE.md is a symlink to AGENTS.md. Every agent session in this
// repo loads it, so it says what MindTape is and where its parts live, and
// leaves how work is done to CONTRIBUTING.md.
//
// Crate names come from www/generated/crate-map.json, and the task DSL block
// is a slice of examples/dsl/, which `just check-examples` validates. The
// render fails on a crate with no role below, a named dependency missing from
// the graph, or a docs page with no source file.
#import "lib.typ": doc-title, fact, just, num-word
#import "/www/utils/slices.typ": checked-slices

#metadata((
  output: "AGENTS.md",
)) <mindtape-md>

#let crate-map = fact("crate-map")

// A docs-site page, named by its source file in www/content/ so an agent
// reads it in the checkout. `read` fails the render when the file is missing.
#let page-src(stem) = {
  let _ = read("/www/content/" + stem + ".typ")
  raw(stem + ".typ")
}

// A dependency named in the Architecture list. crate-map.json gives a name
// missing from the dependency graph a null version, and the assert fails on
// it.
#let dep(name) = {
  let entry = crate-map.at("arch-deps").find(d => d.name == name)
  assert(
    entry != none and entry.version != none,
    message: name + " is not a dependency",
  )
  raw(name)
}

#doc-title[MindTape — AI Agent Context]

\@CONTRIBUTING.md says how work is done in this repo. This file says what
MindTape is and where its parts live.

= What is this?

MindTape is a file-based task tracker. Each task is one Typst file
(`#show: task.with(...)`), and the filename stem is its identity. Every `mt`
command evaluates the task files under the configured roots through the Typst
compiler crates. There is no daemon and no database. A content-hash cache in
`.mindtape/cache/` skips evaluating a file whose bytes did not change.

= Repo map

- `lib/prelude.typ` — the `@local/mindtape` Typst package, which defines the
  task DSL.
- `truth/tasks/` — MindTape's own backlog. `mt` here uses the in-tree `lib/`.
- `truth/goals.typ` and `truth/process-goals.typ` — the product and process
  goals. Each open task rates every goal in its `x-goals` field, and the
  `goals` rule checks the ratings with `check-goals` from `truth/lib.typ`.
- `www/content/*.typ` — the docs site, built with tola.
- `docs/src/*.typ` — the sources of README.md, AGENTS.md, CONTRIBUTING.md,
  the skill's SKILL.md and the `docs/contributing/*.md` deep dives.
- `skills/mindtape/` — the agent skill for driving `mt`, with its references
  and evals.
- `examples/` — fixture projects. `api-service` is a backend team's backlog,
  and it feeds the docs and the skill's evals. `minimal` is the smallest config.
  `dsl` holds the task the DSL block below quotes. `large` is a 2000-task
  benchmark whose gitignored tasks #just.gen-bench() writes. Each of the others
  backs one docs page.

Before you write about a feature, read the `www/content/` page that covers
it: #page-src("dsl") for task fields, edge kinds and `recur`,
#page-src("templates") for `mt add` and slots, #page-src("derived-tags"),
#page-src("priority"), #page-src("filters"), and #page-src("config") for every
config key.

= Workspace

#let workspace = crate-map.at("workspace")
#let workspace-roles = (
  "mindtape": [the domain library: config, task model, filter language,
    validation and project loader. Other tools, such as speconaut, depend on
    it.],
  "mindtape-cli": [the thin `mt` binary, built on clap.],
)

The cargo workspace has a virtual root manifest and
#num-word(workspace.len()) crates:

#for c in workspace [
  - #raw("crates/" + c.name) — #workspace-roles.at(c.name)
]

#let substrate = crate-map.at("substrate")
#let substrate-roles = (
  "typst-world": [the eval-only Typst World the others share],
  "typst-harvest": [evaluates a file and collects the data it marks for
    export],
  "typst-cst": [reads a file's heading tree by parsing, without evaluating],
  "typst-edit": [rewrites a file in place at known spans],
  "typst-cleanup": [retires a tracked file: dirty-tree guard, tombstone, link
    rewriting],
  "swhid-mint": [mints the SWHID a tombstone carries],
  "artifact-gate": [the rule engine behind `mt check`'s body header rules],
)

The Typst plumbing comes from #num-word(substrate.len()) crates.io
dependencies, and this repo holds none of their source:

#for c in substrate [
  - #raw(c.name) — #substrate-roles.at(c.name)
]

= Architecture

- Rust, with the toolchain pinned in `rust-toolchain.toml`.
- Typst runs eval-only through #dep("typst-eval"). MindTape never lays out or
  renders a document.
- #dep("evalexpr") parses the filter language. MindTape walks the parsed tree
  and evaluates it itself.
- #dep("clap") for the CLI, #dep("serde") and #dep("toml") for config,
  #dep("serde_json") and #dep("csv") for output, and a hand-rolled table
  renderer in `crates/mindtape-cli/src/render/`.

= Task DSL (lib/prelude.typ)

#let dsl = checked-slices(fact("dsl"), "dsl", ("task",))
#raw(dsl.task.code, lang: "typ", block: true)

`task` declares each core field as a named parameter, so tinymist completes
the field names. Its `..meta` sink takes only `x-<ns>` extension slots, and a
misspelled field such as `titel` fails the file with an `unknown task slot`
error.

A field value is a plain shape that Rust interprets. `ice(...)` and the other
framework helpers build a `(framework, inputs)` dict, `every(...)` and
`rrule(...)` build a recurrence dict, and
`depends-on("a.typ") + related("b.typ")[why]` builds typed edges. Rust does
all math and date arithmetic.

`docs/contributing/validation-seam.md` says which layer a new validation
belongs in: the prelude, Rust and config, or the card render.

== Mechanism, not policy

Rust ships mechanisms, and projects write policy as config expressions. Rust
code never names a specific estimate unit, status, tag, path or threshold. A
new check is a `[[tasks.rules]]` entry. When the filter language cannot
express it, extend the language with a field, predicate, aggregate or git
state instead of adding a config table or checker of its own. Shipped defaults
live in rule packs under `crates/mindtape/src/packs/`, and a project overrides
a pack rule by name.

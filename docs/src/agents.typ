// Source for the repo-root AGENTS.md (CLAUDE.md is a symlink), rendered by
// docs/generate-md.sh with typlite. Every agent session loads it, so it says
// what the project is and where its parts live, and leaves how work is done to
// CONTRIBUTING.md.
//
// Crate names come from www/generated/crate-map.json, and the DSL block is a
// slice of examples/dsl/, which `just check-examples` validates. The render
// fails on a crate with no role, an unknown dependency or a missing page.
#import "lib.typ": doc-title, fact, just, mindtape, num-word
#import "/www/utils/slices.typ": checked-slices

#metadata((
  output: "AGENTS.md",
)) <mindtape-md>

#let crate-map = fact("crate-map")

// A docs-site page, named by its www/content/ source; `read` fails the render
// when the file is missing.
#let page-src(stem) = {
  let _ = read("/www/content/" + stem + ".typ")
  raw(stem + ".typ")
}

// A dependency named in the Architecture list; a name missing from the
// dependency graph gets a null version in crate-map.json and fails the assert.
#let dep(name) = {
  let entry = crate-map.at("arch-deps").find(d => d.name == name)
  assert(
    entry != none and entry.version != none,
    message: name + " is not a dependency",
  )
  raw(name)
}

#doc-title[#mindtape — AI Agent Context]

\@CONTRIBUTING.md says how work is done here. This file says what #mindtape is
and where its parts live. Fork parallel work with #just.fork() and land it with
#just.land().

= What is this?

#mindtape is a file-based task tracker. Each task is one Typst file
(`#show: task.with(...)`), named by its filename stem. Every `mt` command
evaluates the task files under the configured roots through the Typst compiler
crates, with no daemon or database. A content-hash cache in `.mindtape/cache/`
skips a file whose bytes are unchanged.

= Repo map

- `lib/prelude.typ` — the `@local/mindtape` package, which defines the task
  DSL.
- `truth/tasks/` — #mindtape's own backlog, read with the in-tree `lib/`.
  Change a task with `mt` commands, as `skills/mindtape/SKILL.md` says, and
  list the rules each task and commit must pass with `mt rules`. Edit the file
  by hand only for the body, a framework priority, `estimate` and `x-goals`,
  then run `mt check`.
- `truth/goals.typ` and `truth/process-goals.typ` — the product and process
  goals. Each open task rates every goal in `x-goals`, which the `goals` rule
  checks.
- `www/content/*.typ` — the docs site, built with tola.
- `docs/src/*.typ` — the sources of README.md, AGENTS.md, CONTRIBUTING.md,
  SKILL.md and the `docs/contributing/*.md` deep dives.
- `skills/mindtape/` — the agent skill for driving `mt`, with its references
  and evals.
- `examples/` — fixture projects: `api-service`, a sample backend backlog that
  feeds the docs and the skill's evals; `minimal`, the smallest config;
  `dsl`, the task quoted below; `large`, a 2000-task benchmark whose gitignored
  tasks #just.bench-gen() writes; `nix-adopter`, a flake that takes `mt` into
  its dev shell, which #just.smoke-flake() locks against the working tree.
  Each other one backs one docs page.

Before you write about a feature, read the `www/content/` page that covers
it: #page-src("dsl") for task fields, edge kinds and `recur`,
#page-src("templates") for `mt add` and slots, #page-src("derived-tags"),
#page-src("priority"), #page-src("filters"), and #page-src("config") for every
config key.

= Workspace

#let workspace-roles = (
  "mindtape": [the domain library: config, task model, filter language,
    validation and project loader, which speconaut and other tools depend
    on.],
  "mindtape-cli": [the thin `mt` binary, built on clap.],
)

The cargo workspace has a virtual root manifest and
#num-word(crate-map.at("workspace").len()) crates:

#for c in crate-map.at("workspace") [
  - #raw("crates/" + c.name) — #workspace-roles.at(c.name)
]

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

The Typst plumbing comes from #num-word(crate-map.at("substrate").len()) crates.io
dependencies, whose source sits under `~/.cargo/registry/src/` at the versions
`Cargo.lock` pins. `HVal`, the value every task field passes through, is
`typst_world::value::HVal`.

#for c in crate-map.at("substrate") [
  - #raw(c.name) — #substrate-roles.at(c.name)
]

= Architecture

- Typst runs eval-only through #dep("typst-eval"): #mindtape never lays out a
  document.
- #dep("evalexpr") parses the filter language, and #mindtape evaluates the
  parsed tree itself.
- #dep("clap") for the CLI, #dep("serde") and #dep("toml") for config,
  #dep("serde_json") and #dep("csv") for output, and a hand-rolled table
  renderer in `crates/mindtape-cli/src/render/`.

= Task DSL (lib/prelude.typ)

#let dsl = checked-slices(fact("dsl"), "dsl", ("task",))
#raw(dsl.task.code, lang: "typ", block: true)

`task` declares each core field as a named parameter, and its `..meta` sink
takes only `x-<ns>` slots, so a misspelled field fails.

A field value is a plain shape that Rust interprets. `ice(...)` and the other
framework helpers build a `(framework, inputs)` dict, `every(...)` and
`rrule(...)` build a recurrence dict, and
`depends-on("a.typ") + related("b.typ")[why]` builds typed edges. Rust does
all math and date arithmetic.

== Mechanism, not policy

Rust ships mechanisms, and projects write policy as config expressions. Name
no estimate unit, status, tag, path or threshold in Rust code. Write a new
check as a `[[tasks.rules]]` entry. When the filter language cannot express
it, extend the language with a field, predicate, aggregate or git state, never
a config table or checker of its own. Put shipped defaults in the rule packs
under `crates/mindtape/src/packs/`, where a project overrides a rule by
name. `docs/contributing/validation-seam.md` says which layer a new check
belongs in.

// Source for the repo-root AGENTS.md (CLAUDE.md is a symlink), rendered by
// docs/generate-md.sh with typlite. Every agent session loads it, so it says
// what the project is and where its parts live, and leaves how work is done to
// CONTRIBUTING.md. The task shape lives in `mt config show --resolved`, so this
// file points there instead of repeating it.
//
// Crate names come from www/generated/crate-map.json. The render fails on a
// crate with no role, an unknown dependency or a missing page.
#import "lib.typ": doc-title, fact, just, mindtape, num-word
#import "/truth/goals.typ": goals as product-goals
#import "/truth/process-goals.typ": goals as process-goals

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

#doc-title[#mindtape: AI Agent Context]

\@CONTRIBUTING.md says how work happens here. This file says what #mindtape is
and where to find each part. Fork parallel work with #just.fork() and land it
with #just.land().

= Overview

#mindtape is a file-based task tracker. Each task is one Typst file
(`#show: task.with(...)`), named by its filename stem. Every `mt` command
evaluates the task files under its roots through the Typst compiler crates,
with no daemon or database. A content-hash cache in `.mindtape/cache/` skips
unchanged files.

= Goals

#for (_, goal) in product-goals [
  - #goal.title: #goal.acid-test
]
#for (_, goal) in process-goals [
  - #goal.title: #goal.acid-test
]

= Repo map

- `lib/prelude.typ`: the `@local/mindtape` package, which defines the task
  DSL.
- `crates/mindtape/src/loader/preludes/`: an immutable snapshot of each
  prelude version this binary reads, `0.2.0` through `0.5.0`.
- `truth/tasks/`: #mindtape's own backlog, a folder per month plus a
  `swept/` inbox, read with the in-tree `lib/`.
- `truth/*.typ`: the goals above and the names a task writes.
- `www/content/*.typ` and `docs/src/*.typ`: the docs site and the sources of
  README.md, AGENTS.md, CONTRIBUTING.md, SKILL.md and the deep dives.
- `skills/mindtape/`: the agent skill for driving `mt`, with its references
  and evals.
- `examples/`: fixture projects backing one docs page each. `api-service` is a
  sample backend backlog. `minimal` is the smallest config. `dsl` is the task
  quoted on the DSL page. `large` is a 2000-task benchmark. `nix-adopter` is a
  flake that takes `mt` into its dev shell.

Before writing about a feature, read its `www/content/` page:
#page-src("dsl") for task fields, edge kinds and `recur`,
#page-src("templates") for `mt add` and slots, #page-src("hooks") for hooks,
#page-src("derived-tags"),
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
  - #raw("crates/" + c.name): #workspace-roles.at(c.name)
]

#let substrate-roles = (
  "typst-world": [the eval-only Typst World the others share.],
  "typst-harvest": [evaluates a file and collects the data it marks for
    export.],
  "typst-cst": [reads a file's heading tree by parsing, without evaluating.],
  "typst-edit": [rewrites a file in place at known spans.],
  "typst-cleanup": [retires a tracked file with a dirty-tree guard, a
    tombstone, and link rewriting.],
  "swhid-mint": [mints the SWHID a tombstone records.],
  "artifact-gate": [the rule engine behind `mt check`'s body header rules.],
)

The Typst plumbing comes from #num-word(crate-map.at("substrate").len()) crates.io
dependencies at the versions `Cargo.lock` pins. `HVal`, the value every task
field passes through, is `typst_world::value::HVal`.

#for c in crate-map.at("substrate") [
  - #raw(c.name): #substrate-roles.at(c.name)
]

= Architecture

- Typst runs eval-only through #dep("typst-eval"): #mindtape never lays out a
  document.
- #dep("evalexpr") parses the filter language, and #mindtape evaluates the
  parsed tree itself.
- #dep("clap") for the CLI, #dep("serde") and #dep("toml") for config,
  #dep("serde_json") and #dep("csv") for output, and a hand-rolled table
  renderer in `crates/mindtape-cli/src/render/`.

`schema/output/` holds the committed schema of every `--json` output,
generated from the Rust types. When an output's structure changes, bump its
version and run `mt schema --write`. `mt schema --check` runs in the gates.

= Task definition language (lib/prelude.typ)

`mt add` writes a task file. `mt` commands change it, as
`skills/mindtape/SKILL.md` says. `mt config show --resolved` prints this repo's
shape and `mt templates` the templates it uses. An open task imports
`truth/prelude/2/lib.typ` and rates every goal under `== Goals` after `Dogfood`
with `#Helps`, `#Hinders` or `#Neutral`. Edit only the body, a framework
priority or `estimate` by hand, then run `mt check`.

#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Derive a JSON Schema for at.toml from its structs",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.7,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("cli", "docs"),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[an editor completes and checks at.toml keys, so naming projects takes no trip to the docs],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the at.toml keys get a machine-readable source the docs gate can read],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 17),
  links: parent("keep-the-user-docs-generated-or-gated-not-restated.typ"),
)

== Problem

+ Write `sortt = ["ready:desc"]` under `[board]` in `at.toml`, in an editor that reads a `#:schema` line.
+ 🔴 The editor flags nothing, because no schema exists. `RawFile`, `AtEntry`, `BoardSpec` and `BoardColumn` in `crates/mindtape/src/at/` derive only `Deserialize`, and `mt install` writes only `config.schema.json`.
+ 🔴 So the docs have nothing to check `[board]`, `forks` or the column keys against: `fallback` (`board.rs:56`) appears on no page.


== TO-BE

- 🟢 `mindtape::at::schema_json()` builds the schema with schemars, and each key's doc comment becomes its description.
- 🟢 `schema/at.schema.json` is that output, held to it by a test, as `schema/config.schema.json` is.
- 🟢 `mt install` writes `at.schema.json` beside `config.schema.json`, so this works:
```toml
#:schema ./at.schema.json
[at.work]
include = ["~/projects/*"]
```


== Changes

- `crates/mindtape/src/at/tests.rs`: the red tests first: the schema names `include`, `exclude`, `forks` and `board`, `forks` takes `true`, `false` or `"only"`, and `schema/at.schema.json` equals `schema_json()`.
- `crates/mindtape/src/at/mod.rs`: derive `JsonSchema` on `RawFile` and `AtEntry`, and list the new file in the header.
- `crates/mindtape/src/at/board.rs`: derive it on `BoardSpec` and `BoardColumn`.
- `crates/mindtape/src/at/schema.rs`: the hand-written impl for `Forks`, whose `Deserialize` is hand-written, and `schema_json()`.
- `schema/at.schema.json`: the committed output.
- `crates/mindtape-cli/src/cmd/install/mod.rs`: write it beside `config.schema.json`, with a case in `install/tests.rs`.


== Alternatives

- A hand-written schema: one more copy of the keys, which drifts the way the prose does.
- `[at.*]` inside `config.schema.json`: a project config never holds those keys, so an editor would offer them in the wrong file.


== Out of scope

- The docs gate and the key table that read the schema, a sibling task.
- Writing `#:schema` into `at.toml`: no `mt` command writes that file.


== Dogfood

The owner's `at.toml`, which names this repo beside the others under `~/projects/home`, gets the `#:schema` line and completes in the editor.


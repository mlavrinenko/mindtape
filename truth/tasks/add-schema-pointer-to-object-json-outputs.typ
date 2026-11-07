#import "@local/mindtape:0.4.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Add `$schema` to every object JSON output",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("cli", "stateless"),
  x-goals: (
    product: verdicts(
      (
        self-describing: helps[a JSON consumer finds the shape without reading a page first],
      ),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "enforced-practice",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (
        docs-match-code: helps[the published schema is checked against the JSON the binary prints],
      ),
      ..("sound-codebase",),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 7),
  links: depends-on("generate-a-schema-for-every-json-output.typ"),
)

== AS-IS

+ Run `mt check --json | jq -r '.["$schema"]'` and `mt graph --json | jq -r '.["$schema"]'`.
+ 🔴 Both print `null`; only the pilot outputs from the envelope task carry a pointer.

```
$ mt check --json | jq -r '.["$schema"]'
null
```

== TO-BE

- 🟢 `check`, `graph`, `preset ls`, `explain` index/rules/tags and `eval` print an object whose first key is `$schema`.
- 🟢 Their payloads stay flat, so `jq '.problems'`, `.nodes` and `.rules` keep working.
- 🟢 `eval --json` has its own `eval/1` schema and keeps the record fields flat; `task/1` stays the bare record `mt ls` puts in `tasks`.
- 🟢 `mt check --json` on a set has its own `set-check/1` schema.
- 🟢 The read-command test still passes; `config schema` stays exempt because it is a schema.

```
$ mt check --json | jq -r '.["$schema"]'
https://raw.githubusercontent.com/mlavrinenko/mindtape/main/schema/output/check/1.schema.json
```

== Changes

- `crates/mindtape-cli/src/cmd/check/mod.rs`: wrap `problems` in `check/1`.
- `crates/mindtape-cli/src/cmd/check/set.rs`: wrap `projects` in `set-check/1`.
- `crates/mindtape-cli/src/cmd/graph/json.rs`: wrap `nodes`, `edges` and `external`.
- `crates/mindtape-cli/src/cmd/preset.rs`: wrap `presets`.
- `crates/mindtape-cli/src/cmd/explain/index.rs`, `rules.rs`, `tags.rs`: wrap each index.
- `crates/mindtape-cli/src/cmd/eval.rs`: wrap the record in `eval/1`.
- `crates/mindtape-cli/src/render/objects.rs` (new): the typed payloads.
- `crates/mindtape-cli/src/cmd/schema/registry.rs`: the `check`, `set-check`, `graph`, `preset-ls`, `explain`, `rules`, `tags` and `eval` entries.
- `schema/output/{check,set-check,graph,preset-ls,explain,rules,tags,eval}/1.schema.json`: the generated goldens.

== Alternatives

- Wrap every payload in a `payload` key: one nesting level, no gain, and every `jq` path changes.
- Leave `eval --json` as a bare record: one object output would still lack a pointer.
- Point `eval` at `task/1`: the envelope would name a schema for the bare record. A separate `eval/1` documents the envelope, while `task/1` stays the record.

== Out of scope

- Collection outputs: the next task.
- `config show --json`: it is the config object, so it moves with the collection wrap.
- `add`: its own task.

== Dogfood

`mt check --json | jq '.["$schema"]'` names the schema in this repo's gates, and `mt eval --json` names the task schema.

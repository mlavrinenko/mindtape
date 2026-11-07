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
  status: wip(2026, 11, 7),
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
- 🟢 `eval --json` points at the task schema and keeps the record fields flat.
- 🟢 The read-command test still passes; `config schema` stays exempt because it is a schema.

```
$ mt check --json | jq -r '.["$schema"]'
https://raw.githubusercontent.com/mlavrinenko/mindtape/<tag>/schema/output/check/1.schema.json
```

== Changes

- `crates/mindtape-cli/src/cmd/check/mod.rs`, `crates/mindtape-cli/src/cmd/check/set.rs`: wrap `problems` and `projects`.
- `crates/mindtape-cli/src/cmd/graph/json.rs`: wrap `nodes`, `edges` and `external`.
- `crates/mindtape-cli/src/cmd/preset.rs`: wrap `presets`.
- `crates/mindtape-cli/src/cmd/explain/index.rs`, `crates/mindtape-cli/src/cmd/explain/rules.rs`, `crates/mindtape-cli/src/cmd/explain/tags.rs`: wrap each index.
- `crates/mindtape-cli/src/cmd/eval.rs`: point at the task schema, keeping the record flat.
- `crates/mindtape-cli/src/render/envelope.rs`: the task-schema URL and the payload registry.
- `crates/mindtape-cli/src/render/output_tests_envelope.rs` and the command tests: assert the key and version.
- `schema/output/{check,graph,preset-ls,explain,rules,tags,task}/1.schema.json`: the generated goldens.

== Alternatives

- Wrap every payload in a `payload` key: one nesting level, no gain, and every `jq` path changes.
- Leave `eval --json` as a bare record: one object output would still lack a pointer.

== Out of scope

- Collection outputs: the next task.
- `config show --json`: it is the config object, so it moves with the collection wrap.
- `add`: its own task.

== Dogfood

`mt check --json | jq '.["$schema"]'` names the schema in this repo's gates, and `mt eval --json` names the task schema.

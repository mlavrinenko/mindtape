#import "@local/mindtape:0.4.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Generate a schema for every JSON output and point `$schema` at it",
  kind: "feat",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 4,
  ),
  difficulty: 6,
  tags: ("cli", "tooling", "stateless"),
  x-goals: (
    product: verdicts(
      (
        self-describing: helps[a consumer fetches the exact shape the binary printed],
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
        docs-match-code: helps[the committed schema is generated from the same Rust types that print the JSON],
      ),
      ..("sound-codebase",),
    ),
  ),
  assignee: "agent",
  status: wip(2026, 11, 7),
)

== AS-IS

+ Run `mt schema ls` and `mt ls --json | jq -r '.["$schema"]'`.
+ 🔴 No command prints an output schema, and no output points at one, so a consumer has to read the docs or the Rust source.

```
$ mt schema ls
error: unknown command: 'schema'
$ mt ls --json | jq -r '.["$schema"]'
null
```

== TO-BE

- 🟢 One `Envelope<T>` writes `$schema` first and flattens its payload, so the JSON stays flat.
- 🟢 `mt schema <name>` prints the schema generated from the same Rust types, and `schema/output/<name>/<version>.schema.json` is the committed copy.
- 🟢 A release build points at `https://raw.githubusercontent.com/mlavrinenko/mindtape/<tag>/schema/output/<name>/<version>.schema.json`; a working-tree build points at the local `schema/output` through `MINDTAPE_SCHEMA_BASE`, so local output never names a missing URL.
- 🟢 `schema_version` leaves the output; the URL carries the version, and `mt import` still accepts the field on input.
- 🟢 A test validates live output against the committed schema.

```
$ mt schema ls | jq -r '.$id'
https://raw.githubusercontent.com/mlavrinenko/mindtape/<tag>/schema/output/ls/1.schema.json
```

== Changes

- `crates/mindtape-cli/src/render/envelope.rs` (new): `Envelope<T>`, `$schema`, the base URL, and flattened serialization.
- `crates/mindtape-cli/src/cmd/schema.rs` (new): `mt schema <name>`, the registry, and `--write` for the `schema/output/<name>/<version>.schema.json` goldens.
- `crates/mindtape-cli/src/cmd/ls/mod.rs`: the first `Envelope<T>` payload.
- `crates/mindtape/src/model/canonical.rs`: derive `schemars::JsonSchema` for the task record.
- `crates/mindtape-cli/tests/it/schema_e2e.rs`: validate live output and pin the dev `$schema` base.

== Alternatives

- Keep `schema_version` beside `$schema`: two versions can disagree, which is the bug this replaces.
- Keep every old schema under `main`: release tags already hold them, and the working tree should hold one copy per output.
- Hand-write the schemas: the config schema is already derived from its structs, so output schemas should be too.

== Out of scope

- Adding `$schema` to the remaining object outputs: the next task.
- Wrapping arrays and maps: their own task.
- The docs page and `mt install`: the publish task.

== Dogfood

`mt schema ls` is what the docs build and this repo's scripts read, and the `ls` schema is the first one checked against live output.

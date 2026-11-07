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
  status: done(2026, 11, 7),
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
- 🟢 `mt schema` lists every output, `mt schema <name>` prints one schema generated from the same Rust types, and `schema/output/<name>/<version>.schema.json` is the committed copy. `--write` records the copies and `--check` fails when one differs.
- 🟢 A released binary points at `https://raw.githubusercontent.com/mlavrinenko/mindtape/main/schema/output/<name>/<version>.schema.json`. A working-tree build points at its checkout through `MINDTAPE_SCHEMA_BASE`, which the dev shell sets to `file://$mt_root/schema/output`, so local output never names a missing URL.
- 🟢 The URL carries the version; `schema_version` leaves the output, and `mt import` no longer checks one. An unknown top-level key is ignored.
- 🟢 A test runs `mt schema --check`, and live output validates against the committed schema.

```
$ mt schema ls | jq -r '.$id'
https://raw.githubusercontent.com/mlavrinenko/mindtape/main/schema/output/ls/1.schema.json
```

== Changes

- `crates/mindtape-cli/src/render/envelope.rs` (new): `Envelope<T>`, `$schema`, the base URL, and flattened serialization.
- `crates/mindtape-cli/src/render/payload.rs` (new): the `ls` payload, the first one.
- `crates/mindtape-cli/src/cmd/schema/mod.rs` and `cmd/schema/registry.rs` (new): `mt schema` (list, `<name>`, `--write`, `--check`), the registry, and the stamping helper.
- `crates/mindtape-cli/src/cmd/ls/mod.rs`: the first `Envelope<T>` payload.
- `crates/mindtape/src/model/canonical.rs`: derive `schemars::JsonSchema` for the task record and its nested types.
- `nix/dev/shell.nix`: export `MINDTAPE_SCHEMA_BASE=file://$mt_root/schema/output`.
- `crates/mindtape-cli/src/cmd/import/input.rs`: drop the `schema_version` check.
- `crates/mindtape-cli/tests/it/schema_e2e.rs`: run `mt schema --check` and validate live output against `schema/output/ls/1.schema.json`.
- `schema/output/{ls,task}/1.schema.json`: the first goldens.

== Alternatives

- Point `$schema` at the release tag: the tag is unknown while developing, so the committed `$id` would need a release-time rewrite. A stable `main` URL plus an immutable versioned filename gives the same guarantee with a normal drift test.
- Carry `schema_version` beside `$schema`: two versions can disagree, which is the bug this replaces.
- Hand-write the schemas: the config schema is already derived from its structs, so output schemas should be too.

== Out of scope

- Adding `$schema` to the remaining object outputs: the next task.
- Wrapping arrays and maps: their own task.
- The docs page and `mt install`: the publish task.

== Dogfood

`mt schema ls` is what the docs build and this repo's scripts read, and the `ls` schema is the first one checked against live output.

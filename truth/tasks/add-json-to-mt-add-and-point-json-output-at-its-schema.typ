#import "@local/mindtape:0.4.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Add `--json` to `mt add` and point JSON output at its schema",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 3,
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
  status: proposed(2026, 11, 7),
)

== AS-IS

+ Run `mt add "Probe" --json` and `mt explain --json | jq 'keys'` in a project.
+ 🔴 `mt add` refuses the `--json` shorthand that `mt ls` and `mt find` accept, and the JSON envelopes carry `schema_version` but no `$schema`, so a consumer cannot find the shape without reading the docs.

```
$ mt add "Probe" --json
error: unexpected argument '--json' found
$ mt explain --json | jq 'keys'
["derived", "dicts", "rules", "schema_version", "tags", "views"]
```


== TO-BE

- 🟢 `mt add --json` prints the same report as `mt add --format json`, and `--format` still works.
- 🟢 Every JSON envelope that carries `schema_version` also carries `$schema`, a URL to the schema for that output.
- 🟢 The schema is served at a stable URL and generated or checked, so it cannot drift from the binary.
- 🟢 `mt explain --json` and `mt ls --json` name their schema, and a consumer fetches it only when it needs the fields.

```
$ mt add "Probe" --json | jq -r '.outcome'
wrote
$ mt explain --json | jq -r '.["$schema"]'
https://mlavrinenko.github.io/mindtape/schema/explain.json
```


== Changes

- `crates/mindtape-cli/src/cmd/add/mod.rs`: accept `--json` and give it the same precedence over `--format` that `mt ls` uses.
- `crates/mindtape-cli/src/render/output.rs`: add `$schema` to the task envelope.
- `crates/mindtape-cli/src/cmd/explain/index.rs`: add `$schema` to the explain index.
- `schema/explain.schema.json`: the published schema for the explain index, and the docs build that serves it.
- `www/content/schema.typ`: say what `$schema` names and how to fetch each one.


== Alternatives

- Point `$schema` at the existing `/schema/` page: it is prose, so a validator cannot use it.
- Only add the pointer to `mt explain --json`: leaves the task envelope without one.
- Drop `--format` and keep only `--json`: loses `path` and `path0`.


== Out of scope

- Every other command's JSON envelope; each gets its own schema once this shape is proven.
- Renumbering `schema_version` or changing any JSON key.


== Dogfood

This repo's scripts can use `mt add --json` and resolve `$schema` when they need fields, without reading the docs page.


#import "@local/mindtape:0.4.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Add `--json` to `mt add`",
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
  status: done(2026, 11, 7),
  links: depends-on("generate-a-schema-for-every-json-output.typ"),
)

== AS-IS

+ Run `mt add "Probe" --json`.
+ 🔴 It refuses the shorthand that `mt ls` and `mt find` accept; only `--format json` works.

```
$ mt add "Probe" --json
error: unexpected argument '--json' found
```


== TO-BE

- 🟢 `mt add --json` prints the same report as `mt add --format json`, and `--json` wins when both are given, as in `mt ls`.
- 🟢 The report is an object with `$schema`, `outcome`, `path` and `slug`, flat.
- 🟢 `--format path` and `--format path0` still work.

```
$ mt add "Probe" --json | jq -r '.outcome'
wrote
```


== Changes

- `crates/mindtape-cli/src/cmd/add/mod.rs`: add `json: bool`, and an `effective_format` that gives `--json` precedence over `--format`.
- `crates/mindtape-cli/src/cmd/add/output.rs`: build the report through `Envelope<AddReport>` and add `$schema`.
- `crates/mindtape-cli/src/cmd/add/output_tests.rs`, `crates/mindtape-cli/src/cmd/add/tests_subprocess.rs`: `--json` equals `--format json`, and the precedence.
- `schema/output/add/1.schema.json`: the generated golden.


== Alternatives

- Keep only `--format json`: two spellings for the same report, and `add` stays the odd command out.
- Drop `--format`: loses `path` and `path0`.


== Out of scope

- `--describe=json`: the collection wrap task.
- Every other write command.


== Dogfood

Scripts can use `mt add --json` and read `.outcome` without parsing prose.


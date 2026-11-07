#import "@local/mindtape:0.4.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Give each JSON envelope its own schema version",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 4,
  links: (
    (
      task: "add-ast-grep-queries-as-a-shared-query-engine.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "cli", "stateless"),
  x-goals: (
    product: verdicts(
      (
        cheap-adoption: helps[a consumer accepts a new envelope only when that envelope changed],
      ),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "enforced-practice",
        "typst-over-markdown",
        "plain-files",
        "self-describing",
      ),
    ),
    process: verdicts(
      (
        sound-codebase: helps[the version constant describes one shape instead of five],
      ),
      ..("docs-match-code",),
    ),
  ),
  assignee: "agent",
  status: cancelled(
    2026,
    11,
    7,
  )[superseded by a versioned \$schema URL per output; output no longer carries schema\_version],
)

== AS-IS

+ Verifying the fixes for the ast-grep review
+ 🔴 #raw("crates/mindtape/src/model/canonical.rs"): The canonical schema bump to 2 also bumps `mt tags --json`, `mt check --json`, `mt graph --json` and `mt preset ls --json`, whose shapes did not change.

After the ast-grep query tag landed, the canonical `SCHEMA_VERSION` went to 2. These envelopes use the same constant:

```
$ mt tags --json | jq .schema_version
2
$ mt graph --json | jq .schema_version
2
```

Their shapes did not change. An agent that stops on an unknown version now refuses them even though it could parse them.

== TO-BE

- 🟢 Each JSON envelope has a version that moves only when that envelope's shape moves.
- 🟢 A canonical task export and an unrelated listing can differ in version.
- 🟢 `mt import` still checks the task schema version.

```
$ mt ls --json | jq .schema_version
3
$ mt tags --json | jq .schema_version
1
```

== Changes

- `crates/mindtape/src/model/canonical.rs`, `crates/mindtape-cli/src/render/output.rs`, `cmd/{tags,check,graph,preset,rules}.rs`: split the canonical version from the envelope versions, or document one global version and why.
- `www/content/schema.typ`, `www/content/agents.typ`: say which version covers which envelope.
- `crates/mindtape-cli/src/render/output_tests_envelope.rs`, `crates/mindtape-cli/src/cmd/tags_tests.rs`: pin each envelope's version.

== Out of scope

- Changing any record shape.
- A new import format.

== Dogfood

An agent reading `mt ls --json` and `mt tags --json` can version-check each envelope without refusing an unchanged one.

#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Read has(tags, ...) and spaced has_tag calls in mt tags",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.95,
    ease: 6,
  ),
  difficulty: 2,
  estimate: (src-files: 2),
  tags: ("found", "cli", "tests", "stateless"),
  x-goals: (
    product: verdicts(
      (:),
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
      (sound-codebase: helps[a tag listing names every rule that reads it]),
      ..("docs-match-code",),
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 3),
)

== AS-IS

+ Checking the `mt tags` changelog entry before releasing 0.4.0
+ 🔴 #raw("crates/mindtape-cli/src/cmd/tags.rs"): The scan matches only the exact bytes `has_tag("tag")`, so a rule that reads a tag through `has(tags, "tag")` or a spaced `has_tag ("tag")` shows as an area label.

A minimal project with `allowed = ["cli"]` and a rule `when = 'has(tags, "cli")'`:

```
$ mt tags
cli  area label
```

The same rule spelled `has_tag ("cli")`:

```
$ mt tags --for add
no tags are read at this moment
```

The filter language supports both, and `mt ls 'has(tags, "cli")'` matches the same tasks as `has_tag("cli")`.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/cmd/tags.rs").

== TO-BE

- 🟢 `mt tags` reads the rule expression, so every tag read is listed however it is spelled.

```
$ mt tags --for add
cli  tasks.rules.cli-only  a cli task
```


== Changes

- `crates/mindtape-cli/src/cmd/tags.rs`: scan parsed filter expressions, or normalize both call forms and their spacing.
- `crates/mindtape-cli/src/cmd/tags_tests.rs`: cover `has(tags, ...)`, spaced `has_tag`, and a quoted tag name.


== Out of scope

- A `[[on.flip]]` gate with no `message` is not listed by `--for flip`; it is a separate gap.


== Dogfood

`mt tags --for add` then names every pack rule that reads `stateful`, `stateless`, `gating`, `no-repro`, `no-ui-ux` and `tweak`.


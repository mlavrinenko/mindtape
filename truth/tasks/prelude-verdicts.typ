#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Ship `verdicts` and the verdict helpers in the prelude",
  kind: "feat",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("mt", "substrate", "stateless"),
  estimate: (src-files: 5),
  x-goals: (
    product: verdicts(
      (
        cheap-adoption: helps[an adopter gets the short form without copying a helper],
      ),
      "readable-tasks",
      "implementable-tasks",
      "enforced-practice",
      "typst-over-markdown",
      "plain-files",
    ),
    process: verdicts(
      (:),
      "sound-codebase",
      "docs-match-code",
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 2),
  links: depends-on("task-templates.typ") + depends-on("as-is-sweep.typ"),
)


== AS-IS

+ Read the `x-goals` block in a current task: eight verdicts, seven of them `neutral`.
+ 🔴 The helper lives only in `truth/lib.typ`, so an adopter's task files cannot use the short form.
+ 🔴 `helps`, `hurts` and `neutral` are project-local even though they are the same dict shape everywhere.

== TO-BE

- 🟢 `@local/mindtape` exports `helps`, `hurts`, `neutral` and `verdicts`.
- 🟢 `verdicts(given, ..neutral-names)` puts `given` last so an explicit verdict wins, and it asserts when a name has both a verdict and a neutral slot.
- 🟢 `truth/lib.typ` keeps `check-goals` and the registries, and imports the helpers.

```typ
product: verdicts(
  (enforced-practice: helps[the rule names its waiver],),
  "readable-tasks", "implementable-tasks",
)
```

== Changes

- `lib/prelude.typ`: the helpers and the `verdicts` merge.
- `truth/lib.typ`: the helpers dropped, with `check-goals` unchanged.
- `lib/typst.toml`, `crates/mindtape/src/loader/embedded.rs` and `crates/mindtape/src/loader/prelude.typ`: the version and the embedded copy.
- `.config/outdatty.lock` and `CHANGELOG.md`: the `typst-pkg-version` re-record and the entry.

== Alternatives

- A sparse map that omits neutral goals: shorter, but it gives up the explicit verdict the owner wants.
- `neutral-on` with `+`: `+` is right-wins, so an overlap silently turns a `helps` neutral.

== Out of scope

- The templates in `verdicts-templates`, the docs in `verdicts-docs` and the corpus sweep in `x-goals-sweep`.

== Dogfood

This repo's own tasks import the prelude and get the helpers.

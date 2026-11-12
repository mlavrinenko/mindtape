#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Report add-hook failures as failed writes",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.95,
    ease: 7,
  ),
  difficulty: 2,
  links: (
    (
      task: "let-rules-and-templates-choose-where-new-tasks-land.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "mt", "cli", "tests", "stateless"),
  x-goals: (
    product: verdicts(
      (:),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
        "self-describing",
      ),
    ),
    process: verdicts(
      (:),
      ..("sound-codebase", "docs-match-code"),
    ),
  ),
  status: done(2026, 11, 12),
  assignee: "agent",
)

== AS-IS

+ Reviewing the on-add branch after `just check`.
+ 🔴 #raw("crates/mindtape-cli/src/cmd/add/hooks.rs"): A hook write failure printed the hook label as the offending value and exited 2 as a command-line error; a `when` that failed at runtime did not name the hook.

The writer's error became a quoted `priority: fixes`, and the exit code was 2:

```
error: on.add fixes: a number is expected: 'priority: fixes' for field 'priority'
```


== TO-BE

- 🟢 Every hook failure names `on.add <label>`, keeps the writer's own value, and exits 1.
- 🟢 A failed `when` names the hook, a derive names the function, and a derive with no field is refused.

```
error: on.add fixes: a number is expected: 'high' for field 'priority'
```


== Changes

- `crates/mindtape-cli/src/cmd/add/hooks.rs`: reclassify writer and filter failures, prefix the hook, and name the derive call.
- `crates/mindtape-cli/src/cmd/add/hooks_tests.rs`: pin the wording, the exit kind and the missing-field refusal.
- `crates/mindtape-cli/src/diag/mod.rs`: add the one kind reclassification seam.
- `crates/mindtape-cli/tests/it/add_hooks_derive_e2e.rs`: match the quoted call in the refusal.


== Out of scope

- `mt set`'s usage exit code, which stays 2.


== Dogfood

Add-hook failures read like every other project failure.


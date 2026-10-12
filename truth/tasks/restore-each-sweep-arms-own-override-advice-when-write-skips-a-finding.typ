#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Restore each sweep arm's own override advice when --write skips a finding",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  estimate: (src-files: 6),
  tags: ("found", "tooling"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[a skipped finding names the override that clears it],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 13),
)

== Problem

+ Reading commit ee2ff69e while starting the dry sweep arm
+ 🔴 #raw("contrib/ask-refactoring.sh"): file_finding prints one generic skipped hint, so the size, coverage, undertested and mutants arms lost their own override advice (the just cover exclude, CRAP_EXCLUDES, exclude_re)

```
-                echo "        `.cargo/mutants.toml` `exclude_re` with its reason" >&2
+                echo "        without the signal clearing. Reopen it, or write the override." >&2
```

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("contrib/ask-refactoring.sh").

== TO-BE

- 🟢 A skipped finding prints its arm's own advice on stderr, as before ee2ff69e, while file_finding stays the one shared helper.

== Changes

- `contrib/ask-refactoring.sh`: file_finding takes the arm's advice lines as an argument.
- `contrib/ask-refactoring/size.sh`: pass the size override advice.
- `contrib/ask-refactoring/coverage.sh`: pass the `just cover` exclude advice.
- `contrib/ask-refactoring/undertested.sh`: pass the `CRAP_EXCLUDES` advice.
- `contrib/ask-refactoring/mutants.sh`: pass the `exclude_re` advice.
- `contrib/ask-refactoring/repeats.sh`: pass its advice.

== Rollback

- Revert the commit.

== Out of scope

- The dry-run output, which is unchanged.

== Dogfood

Every `just ask-refactoring --write` that skips a finding names the right override again.

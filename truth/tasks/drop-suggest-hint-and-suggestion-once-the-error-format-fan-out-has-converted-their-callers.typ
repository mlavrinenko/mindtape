#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Drop suggest::hint and suggestion once the error-format fan-out has converted their callers",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 1,
  estimate: (src-files: 3),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a did-you-mean has one shape, a hint line],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(
    2026,
    10,
    8,
  )[suggest.rs holds nearest and the Unknown carrier; no caller builds a did-you-mean string.],
)

== Problem

+ Closing report-task-references-and-project-discovery-in-the-error-format
+ 🔴 #raw("crates/mindtape/src/suggest.rs"): \`hint\` and \`suggestion\` still build \`; did you mean: x?\` strings for filter, config, set, link, add, ls and worktrees, which the nearest-name-as-a-Hint format replaces

```
crates/mindtape/src/config/ls.rs:324: crate::suggest::hint("view", ..)
crates/mindtape-cli/src/cmd/set.rs:268: suggest::suggestion(name, &known)
crates/mindtape-cli/src/cmd/link/plan.rs:245: suggest::suggestion(kind, &names)
```

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/suggest.rs").

== TO-BE

- 🟢 `suggest.rs` holds `nearest` only; no caller builds a did-you-mean string.

== Changes

- `crates/mindtape/src/suggest.rs`: delete `hint` and `suggestion` and their tests, and re-export only `nearest` from `lib.rs`, in the commit of the last fan-out task that stops calling them.

== Rollback

- Restore the two functions.

== Out of scope

- The callers' own conversions, which each fan-out task does.

== Dogfood

`rg 'suggest::(hint|suggestion)' crates` finds nothing.

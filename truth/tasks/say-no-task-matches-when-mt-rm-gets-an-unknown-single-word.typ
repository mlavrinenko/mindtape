#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Say no task matches when mt rm gets an unknown single word",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  estimate: (src-files: 1),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a typo in a task name gets a message about the task, not about filter syntax],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 8),
)

== Problem

+ Capturing every error \`mt\` prints for the error inventory
+ 🔴 #raw("crates/mindtape-cli/src/cmd/rm/select.rs"): \`mt rm\` with a word that is neither a stem nor a stem hash hands it to the filter parser, which answers in filter terms

```
$ mt rm nope
Error: nope: invalid filter: bare field `nope` is not boolean; use a comparison or function
[exit 1]
$ mt mv nope x
Error: no task with id "nope"
[exit 1]
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/cmd/rm/select.rs").

== TO-BE

- 🟢 `mt rm nope` says no task matches `nope`, with a nearest-stem suggestion, as `mt mv` and `mt set` do.
- 🟢 A real filter such as `mt rm 'status == "done"'` still selects by filter.


== Changes

- `crates/mindtape-cli/src/cmd/rm/select.rs`: report a single bare word that is not a field or term as an unknown task.


== Rollback

- Revert the file; the filter parser answers again.


== Out of scope

- The wording of filter errors in general, which the error-format task covers.


== Dogfood

Not applicable, this repo retires tasks with `mt rm` rarely; the fix is for a typed name.

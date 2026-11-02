#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Name the init area's own test files in its module header",
  kind: "dev",
  priority: ice(
    impact: 2,
    confidence: 1.0,
    ease: 9,
  ),
  difficulty: 0,
  estimate: (src-files: 1),
  tags: ("found", "cli"),
  x-goals: (
    product: verdicts(
      (:),
      "readable-tasks",
      "implementable-tasks",
      "enforced-practice",
      "cheap-adoption",
      "typst-over-markdown",
      "plain-files",
    ),
    process: verdicts(
      (
        docs-match-code: helps[the header lists the files the folder holds],
      ),
      "sound-codebase",
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 19),
)

== AS-IS

+ Fixing the second `mt init` refusal in `crates/mindtape-cli/src/cmd/init/mod.rs`
+ 🔴 #raw("crates/mindtape-cli/src/cmd/init/mod.rs"): the `# Files` list in the `//!` header names `cmd/ls/interactive/tests.rs`, a file of another area, and leaves out `tests.rs`, `compose_tests.rs` and `preset_tests.rs`, which the folder holds

```
//! - `cmd/ls/interactive/tests.rs`, `*_tests.rs` — unit tests.
$ ls crates/mindtape-cli/src/cmd/init/
axes  compose.rs  compose_tests.rs  mod.rs  prelude.typ  preset.rs  presets  preset_tests.rs  tests.rs  wizard
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/cmd/init/mod.rs").

== TO-BE

- 🟢 The `# Files` list in `cmd/init/mod.rs` names `tests.rs`, `compose_tests.rs` and `preset_tests.rs`, and no file outside the folder.


== Changes

- `crates/mindtape-cli/src/cmd/init/mod.rs`: replace the `cmd/ls/interactive/tests.rs` entry with the folder's own test files.


== Rollback

- Revert the header line.


== Out of scope

- The other areas' headers. A check that every `# Files` entry exists is its own finding.


== Dogfood

Not applicable, the header is read only by contributors to this folder.

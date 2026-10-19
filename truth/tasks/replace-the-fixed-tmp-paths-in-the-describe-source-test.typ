#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Replace the fixed /tmp paths in the describe_source test",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  estimate: (src-files: 1),
  tags: ("found", "tests"),
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
      sound-codebase: helps[the test passes on a host where /tmp/x.toml exists],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 20),
)

== Problem

+ Running `just check` in a fresh worktree
+ 🔴 #raw("crates/mindtape-cli/src/cmd/config_tests.rs"): `describe_source_covers_xdg_and_dir` asserts that the fixed path `/tmp/x.toml` is missing, so a stray file there fails `just test` and with it the pre-commit gate

```
thread 'cmd::config::tests::describe_source_covers_xdg_and_dir' panicked at crates/mindtape-cli/src/cmd/config_tests.rs:208:5:
assertion failed: xdg.contains("(missing)")
```
A 50-byte `/tmp/x.toml` left by another tool made it fail.

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/cmd/config_tests.rs").

== TO-BE

- 🟢 The test builds its paths inside a fresh temp directory, so no file on the host changes its result.

== Changes

- `crates/mindtape-cli/src/cmd/config_tests.rs`: build the missing xdg and dir paths under a temp directory.

== Rollback

- Revert the commit. Only a test changes.

== Out of scope

- The other tests that name a `/tmp` path. Each gets its own finding once it fails.

== Dogfood

Every `just test` in this repo stops depending on what else wrote to `/tmp`.

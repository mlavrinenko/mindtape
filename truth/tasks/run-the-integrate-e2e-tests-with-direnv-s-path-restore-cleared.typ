#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Run the integrate e2e tests with direnv's path restore cleared",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 8,
  ),
  difficulty: 2,
  estimate: (src-files: 1),
  tags: ("found", "tests"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[the integrate refusal tests judge the mt under test, not the host's],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[a green integrate e2e run says something about the code],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 7),
)

== Problem

+ Running `just test` from the dev shell.
+ 🔴 #raw("crates/mindtape-cli/tests/integrate_refs_e2e.rs"): `direnv exec` restores the pre-shell `PATH`, dropping the test's `mt` for the host's.

```
$ PATH=/tmp/x/bin:$PATH direnv exec . sh -c 'command -v mt'
~/.cargo/bin/mt
$ env -u DIRENV_DIFF -u DIRENV_WATCHES PATH=/tmp/x/bin:$PATH direnv exec . sh -c 'command -v mt'
/tmp/x/bin/mt
```
`.just/scripts/integrate.sh:104-105`: `direnv exec . "$@"`; `:131`, `:134`: the `mt` calls.
`integrate_refs_e2e.rs:57-72`: `integrate_with` passes `DIRENV_DIFF` through.
`support/mod.rs:21`: `CLEARED_VARS`.

== TO-BE

- 🟢 The integrate e2e tests run the `mt` from `CARGO_BIN_EXE_mt` whatever direnv state the developer's shell carries.

== Changes

- `crates/mindtape-cli/tests/integrate_refs_e2e.rs`: `env_remove` `DIRENV_DIFF` and `DIRENV_WATCHES` on the script's command, or add `DIRENV_*` to the scrub in `support/mod.rs` that `isolate` runs.
- Add a red test first: a fake `mt` ahead on `PATH` that the script must reach, run with `DIRENV_DIFF` set.

== Rollback

- Revert the `env_remove` lines.

== Out of scope

- Changing `integrate.sh`'s `direnv exec` hop, which real worktrees need.

== Dogfood

`just test` from the dev shell, then from a plain shell.

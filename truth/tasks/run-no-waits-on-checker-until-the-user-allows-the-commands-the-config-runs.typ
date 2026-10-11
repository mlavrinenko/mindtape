#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Run no waits-on checker until the user allows the commands the config runs",
  kind: "feat",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 5,
  tags: ("mt", "cli"),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[an agent in a fresh clone gets a warning to pass on, not a command it never chose to run],
      enforced-practice: helps[a changed `run` line needs a fresh approval before it runs],
      cheap-adoption: hurts[every clone and every changed command asks once before checkers answer],
      typst-over-markdown: neutral,
      plain-files: helps[the approval lives outside the repo, so no file in a clone can approve its own commands],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 11),
)

== Problem

+ Clone a repo whose `.mindtape/config.toml` declares `[[tasks.waits]] run = "scripts/locked {id}"`.
+ 🔴 Run `mt ls`. It runs `scripts/locked` with your authority before you have read the config.


== TO-BE

- 🟢 Until `mt allow` records them, `mt` runs no checker. Each command that would warns once, listing the commands, with `hint: read them, then run mt allow`.
- 🟢 An un-allowed wait blocks as unknown, `why` reads `unknown: not allowed`, and `mt wip --force` still starts the task. Every other command works as before.
- 🟢 The approval lives in `$MINDTAPE_DATA_HOME/allowed/`, keyed by `git rev-parse --git-common-dir` (the root outside git), so every `just fork` worktree shares it. It records a hash of every `run` line, and any change asks again.


== Changes

- `crates/mindtape-cli/tests/it/allow_e2e.rs`: red first: no run before `mt allow`, a run after, a changed `run` line asks again, a sibling worktree shares the approval.
- `crates/mindtape/src/allow.rs`, new: the command-set hash, the repo key, and reading and writing the store.
- `crates/mindtape/src/waits.rs`: answer `Unknown("not allowed")` without running when the set is not allowed.
- `crates/mindtape-cli/src/cmd/allow.rs`, new, wired in `cli/mod.rs`: list the commands and record them.
- `crates/mindtape-cli/src/util.rs`: print the warning once per run.


== Alternatives

- `.mindtape/cache/allowed`: inside the clone, so a repo could ship its own approval with `git add -f`.
- Keying by the checkout path: each `just fork` worktree would ask again.
- Ignoring an un-allowed wait: `ready` would hand an agent work a checker still holds.


== Out of scope

- `[[on.flip]]` commands; a follow-up task holds them to the same approval.
- Hashing the scripts a `run` line calls; `mt` cannot tell which files a shell line reads.


== Dogfood

This repo has no `[[tasks.waits]]` entry yet, so the follow-up that gates `[[on.flip]]` is where `mt allow` first runs here.

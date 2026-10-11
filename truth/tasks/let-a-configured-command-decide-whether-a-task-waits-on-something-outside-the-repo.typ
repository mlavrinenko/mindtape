#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Let a configured command decide whether a task waits on something outside the repo",
  kind: "feat",
  priority: ice(
    impact: 7,
    confidence: 0.7,
    ease: 4,
  ),
  difficulty: 7,
  tags: ("mt", "cli"),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: helps[the `why` column names the outside condition a task waits on, in the checker's own words],
      implementable-tasks: helps[an agent picking from `mt ls ready` never starts work an upstream fix still blocks],
      enforced-practice: helps[a waiting task leaves `blocked` the moment the checker says so, not when someone remembers to look],
      cheap-adoption: helps[any tool that answers by exit code plugs in with one config entry, no MindTape release per tool],
      typst-over-markdown: neutral,
      plain-files: hurts[readiness can depend on a command's answer, not only on files; the command must read local state so `mt` stays offline],
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

+ Open ~/.nix: `apps/llama-cpp-16gb.nix` carries a patch until ggml-org/llama.cpp#26487 lands, and `~/.nix/canibump.toml` tracks it (`canibump` exits non-zero once it is bumpable, `canibump status <id>` answers that one blocker from its cache by exit code).
+ 🔴 A MindTape task "drop the patch" can only say so in prose or in a `blocked` status note. `mt ls ready` lists it or hides it by hand, and nothing re-checks when upstream moves.
+ The only blockers MindTape resolves are `depends-on` links to other task files (`crates/mindtape/src/graph/ready.rs`, `Readiness`).


== TO-BE

- 🟢 Config names a checker, e.g. `[[tasks.waits]] name = "canibump"`, `run = "canibump status {id}"`; exit 0 means resolved, 1 means still waiting, anything else means unknown.
- 🟢 A task declares `waits-on: ("canibump:llama-blocking-sync",)`; while the checker says waiting or unknown, the task is `blocked`, and `why` prints the checker's first stdout line.
- 🟢 `mt` stays fast and offline: it runs each distinct `(checker, id)` once per invocation and documents that a checker must answer from local state; network refreshes are the tool's own job (`canibump --refresh` in a timer or a `just` recipe).
- 🟢 Nothing in MindTape names canibump: the same entry works for any tool, e.g. `test "$(gh pr view {id} --json state -q .state)" = MERGED` for a user who accepts the network cost.


== Changes

- `crates/mindtape/tests/`: red e2e first, a task waiting on a stub checker flips between `ready` and `blocked` with its exit code.
- `crates/mindtape/src/config/`: the `[[tasks.waits]]` table (name, run) in the config schema.
- `crates/mindtape/src/graph/ready.rs`: `Readiness` treats an unresolved `waits-on` entry as a `Blocker`.
- `crates/mindtape/src/` (task fields): the `waits-on` field, validated against declared checker names by `mt check`.
- `crates/mindtape/src/config/hooks.rs`: reuse the `[[on.flip]]` `run` executor (`sh -c` / `cmd /C`) rather than a second one.


== Alternatives

- A canibump integration in MindTape: couples MindTape to one tool's cache format; lost to the generic checker.
- A `[[tasks.rules]] check` over a Typst function: Typst cannot run a command or read another tool's state.
- A status note plus a periodic human re-check: what exists today, and it rots.
- Running checkers only on `mt check`, storing the verdict in the task file: every upstream move becomes a commit; kept as a possible follow-up for repos that want history.


== Out of scope

- Any network access by `mt` itself, and any cache of checker answers across invocations; a slow checker is the checker's problem to fix.
- canibump changes; it already answers offline from its cache.


== Dogfood

Not applicable here: no open MindTape task waits on something upstream. The past ones (typst-cleanup 0.2, artifact-gate 0.1.1) waited on a crates.io publish, which only the network answers. `examples/api-service` carries the offline pattern instead: `scripts/locked` reads `Cargo.lock`, and `/wait-on-upstream/` captures a task leaving `blocked` when the dependency bot's branch merges. ~/.nix dogfoods it with its canibump entry.

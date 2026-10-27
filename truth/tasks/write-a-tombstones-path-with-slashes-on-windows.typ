#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Write a tombstone's path with slashes on Windows",
  kind: "fix",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: helps[a tombstone written on Windows resolves on every machine],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 27),
)

== Problem

+ On Windows, in a git repository, run `mt rm gone-a` where `live-a` depends on `gone-a`.
+ 🔴 `live-a` gets `depends-on("swh:1:rev:…;path=tasks\gone-a.typ")`. A SWHID's `path=` is a git path, and git paths use `/`, so the tombstone resolves nowhere (`rm_e2e::rm_takes_two_refs_in_one_invocation_and_rewrites_each_inbound_link`, CI run 40576881551).
+ typst-cleanup 0.2.0 mints the SWHID from `strip_prefix(git_root).to_string_lossy()`, which is the platform path.


== Environment

- `fix-windows-ci` at `bb1c59d5`, on the `windows-latest` GitHub runner.


== TO-BE

- 🟢 A tombstone's `path=` uses `/` on every platform.


== Changes

- `crates/mindtape-cli/src/cmd/rm/tests.rs`: the red test, a SWHID minted with `\` gets a tombstone with `/`.
- `crates/mindtape-cli/src/cmd/rm/mod.rs`: `TaskLinks::tombstone` writes the path with `/`.
- `crates/mindtape-cli/src/cmd/rm/eligible.rs`: probe the mint with `slash_path`, the path git knows.


== Out of scope

- The fix in typst-cleanup itself, whose plan should mint from a `/` path. It needs a typst-cleanup release, which is the owner's call.


== Dogfood

Not applicable. This repo is developed on Linux.


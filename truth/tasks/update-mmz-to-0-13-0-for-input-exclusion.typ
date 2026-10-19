#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Update mmz to 0.13.0 for input exclusion",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (src-files: 2),
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
      sound-codebase: helps[a gate's inputs can leave out a tree another gate writes, instead of leaning on .gitignore alone],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 20),
)

== Problem

+ Run `mmz --version` in the dev shell.
+ 🔴 It prints `mmz 0.12.1`, which cannot leave a path out of a scope. Gates globbing `examples/**` hashed `.mindtape/cache/` files another gate deleted mid-hash, which failed a run, and only the `**/.mindtape/cache/` line in `.gitignore` keeps them out now.


== TO-BE

- 🟢 `mmz --version` in the dev shell prints 0.13.0, and `just nix-build` and `just check` pass on it.
- 🟢 No gate's inputs glob over a path another gate writes during `just check`. Where one does, its scope carries an `exclude`, and `mmz --status=json` shows the path gone from that rule's inputs.


== Changes

- `flake.lock`: the `mmz` input at 0.13.0.
- `.mmz/config.yaml`: an `exclude` on each scope that globs over a path another gate writes, if a watched `just check` finds one.


== Rollback

- Revert the commits. mmz 0.12.1 refuses an `exclude` key, so the bump and any exclusion go back together.


== Out of scope

- The `**/.mindtape/cache/` line in `.gitignore`, which is right on its own.
- qahq's own `mmz` input, which puts no `mmz` on the dev shell's PATH.


== Dogfood

This repo's `just check` runs every gate through mmz 0.13.0.


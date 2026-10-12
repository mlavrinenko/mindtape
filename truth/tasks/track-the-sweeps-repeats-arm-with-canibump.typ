#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Track the sweep's repeats arm with canibump",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.8,
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
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[the stopgap arm says when upstream lets it go, instead of outliving its reason],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 13),
  links: depends-on(
    "rank-short-repeated-snippets-by-extraction-gain-in-a-sweep-arm.typ",
  ),
)

== Problem

+ Read `contrib/ask-refactoring/repeats.sh`.
+ 🔴 It hand-rolls clone groups, which upstream jscpd issue 1000 adds to jscpd itself. Nothing in this repo says when that lands, so the stopgap would stay after its reason is gone.


== TO-BE

- 🟢 `canibump.toml` at the repo root holds a blocker on jscpd issue 1000, and `just canibump` says when the arm can go.
- 🟢 canibump's `[scan]` reads the `Justfile`, `flake.nix` and `contrib/`, so an upstream link no blocker names shows up as unlisted.


== Changes

- `canibump.toml`: the blocker and the scan, with the schema line on top.
- `flake.nix`: canibump from qahq in the dev shell.
- `Justfile`: a `canibump` recipe, and `canibump.toml` in `TOML_FMT_PATHS`.


== Rollback

- Revert the commit. The arm stays, untracked.


== Out of scope

- The blocker for the jscpd gate's new-clones filter, which waits on an issue not yet posted.


== Dogfood

This repo's `just canibump` reports the jscpd issue 1000 blocker.

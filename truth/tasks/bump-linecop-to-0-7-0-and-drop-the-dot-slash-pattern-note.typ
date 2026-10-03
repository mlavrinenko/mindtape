#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Bump linecop to 0.7.0 and drop the dot-slash pattern note",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.8,
    ease: 9,
  ),
  difficulty: 1,
  tags: (),
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
      sound-codebase: neutral,
      docs-match-code: helps[the `.linecop.yaml` comment states the rule linecop applies],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 3),
)

== Problem

+ `flake.lock` pins qahq at linecop 0.4.0. `nix flake update qahq` brings linecop 0.7.0 (qahq v0.5.0).
+ Read the comment above the rendered-file overrides in `.linecop.yaml`.
+ 🔴 It says linecop matches the `./`-prefixed path, so `**/README.md` matches every README and a bare `README.md` matches none. From 0.7.0 a pattern is a `.gitignore` line read from the config's directory: a bare `README.md` matches at any depth, and `./README.md` anchors to the root as a leading `/` does.


== TO-BE

- 🟢 The flake carries linecop 0.7.0, and `just check` passes with it (checked ahead: this repo's linecop verdict is unchanged).
- 🟢 The `.linecop.yaml` comment states the 0.7.0 rule: patterns are `.gitignore` lines relative to the config; `./x` or `/x` anchors to the root, a bare name matches at any depth.


== Changes

- `flake.lock`: `nix flake update qahq`.
- `.linecop.yaml`: rewrite the anchoring comment above the rendered-file overrides.


== Rollback

Revert the commit; the 0.4.0 lock and the old comment come back together.


== Out of scope

- The patterns themselves: every one keeps its meaning under 0.7.0.


== Dogfood

Not applicable: a tool bump with no mindtape behaviour change.


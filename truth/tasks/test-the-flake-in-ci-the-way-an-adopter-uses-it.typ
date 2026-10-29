#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Test the flake in CI the way an adopter uses it",
  kind: "dev",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a release cannot ship a flake an adopter can't use],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[CI runs what the flake builds, not just builds it],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 29),
)

== Problem

+ Read `.github/workflows/ci.yml`: it builds the flake but never runs what it built, or locks it from another flake.
+ 🔴 0.3.0 shipped with `nix run` broken, `mt` unable to load tasks on a fresh machine, and a 190-node lock for adopters, and no job noticed.


== TO-BE

- 🟢 `just smoke-flake` locks `examples/nix-adopter` against this tree and fails when the lock outgrows its cap.
- 🟢 In that fixture's dev shell, with an empty `HOME`, it runs `mt init --no-input`, `mt add` and `mt check`, and it runs `nix run` on this flake.
- 🟢 CI's `check` job runs it after the Nix build.


== Changes

- `examples/nix-adopter/flake.nix`: a dev shell with `mt` from this flake.
- `Justfile`: the `smoke-flake` recipe.
- `.github/workflows/ci.yml`: a step that runs it.


== Rollback

- Drop the step, the recipe and the fixture.


== Out of scope

- macOS, which the flake builds for but CI does not run.


== Dogfood

Every push here runs it, and the `nix run`, prelude and lock fixes turn it green.


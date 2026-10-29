#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Recommend qahq for mt in a Nix dev shell",
  kind: "docs",
  priority: ice(
    impact: 6,
    confidence: 0.85,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("www", "docs"),
  estimate: (src-files: 0),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a Nix project copies one snippet and gets a prebuilt `mt`],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 29),
)

== Problem

+ Open the Quickstart's Install section to add `mt` to a project's Nix dev shell.
+ 🔴 It offers `nix run` alone. It never shows a flake input, the binary cache that serves the build, or that qahq ships `mt` beside the other dev tools.


== TO-BE

- 🟢 The Quickstart recommends a dev shell that takes `mt` from qahq, shows this flake as a direct input for a project without qahq, and names the cache that serves each prebuilt.


== Source of truth

qahq's `flake.nix` package names and its README's cache key, and this repo's `flake.nix` outputs and CI's Cachix step.


== Changes

- `www/content/quickstart.typ`: a Nix dev shell part in Install.
- `docs/src/readme.typ`: name the Nix route beside the others.


== Out of scope

- qahq's own README, which changes in qahq.


== Dogfood

canibump and mmz take `mt` from qahq with the snippet this page shows.


#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Push only finished packages to the Cachix cache",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  tags: ("tooling",),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a released `mt` stays in the free cache instead of being evicted],
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

+ Look up the 0.3.0 build in the cache: `curl https://mlavrinenko.cachix.org/19668k1cqyk7xrhpl0gvi8a7db6010ms.narinfo`.
+ 🔴 `rust-workspace-deps-0.3.0`, naersk's compiled dependency tree, takes 328 MB against 18 MB for `mt`. CI pushes one per lock change, so the free 5 GB fills with them and evicts released binaries first.
+ 🔴 The dev shell builds its own `cargo-crap`, which qahq also ships.


== TO-BE

- 🟢 CI pushes no `-deps-` build to Cachix.
- 🟢 The dev shell takes `cargo-crap` from qahq, which builds and caches it once for every project.


== Changes

- `.github/workflows/ci.yml`: `pushFilter: "-deps-"` on the Cachix step.
- `flake.nix`: `qahq.packages.cargo-crap` instead of the naersk build, once qahq ships 0.3.1.


== Rollback

- Revert the commit.


== Out of scope

- flibrarian's and qahq's CI, which push the same kind of path. Each gets the filter in its own repo.


== Dogfood

The next CI run's Cachix push lists no `-deps-` path.


#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Keep the dev-shell inputs out of an adopter's flake.lock",
  kind: "dev",
  priority: ice(
    impact: 8,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("tooling",),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[taking `mt` from the flake adds a handful of lock nodes, not 190],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 29),
)

== Problem

+ Add `inputs.mindtape.url = "github:mlavrinenko/mindtape/v0.3.0"` to another flake and run `nix flake lock`.
+ 🔴 Its `flake.lock` grows to 190 nodes, 24 of them fenix and 23 naersk. mmz, qahq, tola and nixpkgs-tombi come along, though `packages.default` builds from nixpkgs and naersk alone.


== TO-BE

- 🟢 An adopter's lock gains only the inputs `packages.default` builds from.
- 🟢 `nix develop` in this repo still gives the whole dev shell.


== Changes

- `flake.nix`: flake-parts, with `devShells` in a `dev` partition.
- `nix/dev/flake.nix` and its `flake.lock`: the dev-only inputs.
- `.github/workflows/ci.yml`: the cache keys hash both locks.
- `Justfile` and `CONTRIBUTING.md` sources: update the dev inputs through `nix/dev`.


== Rollback

- Revert the commit. Adopters' locks grow back.


== Out of scope

- The same split in mmz, canibump, ejectest, outdatty and linecop, whose dev inputs reach adopters through qahq. Each repo gets its own task.


== Dogfood

CI's adopter smoke job locks a fixture flake against this one and fails when the lock outgrows its cap.


#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Name the flake package so nix run starts mt",
  kind: "fix",
  priority: ice(
    impact: 7,
    confidence: 0.95,
    ease: 9,
  ),
  difficulty: 1,
  tags: ("tooling",),
  estimate: (src-files: 1),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[`nix run github:mlavrinenko/mindtape` works as the Quickstart says],
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

+ Run `nix run github:mlavrinenko/mindtape/v0.3.0 -- --version`.
+ 🔴 It fails with `unable to execute '/nix/store/…-rust-workspace-0.3.0/bin/rust-workspace': No such file or directory`. naersk names the package after the virtual workspace, and nothing sets `mainProgram`.


== Environment

Nix on Linux, the flake at `v0.3.0`. The Quickstart's Install section documents this command.


== TO-BE

- 🟢 `nix run github:mlavrinenko/mindtape -- --version` prints `mt`'s version.
- 🟢 The store path is named `mindtape-<version>`.


== Changes

- `flake.nix`: `pname = "mindtape"` and `meta.mainProgram = "mt"` on `packages.default`.


== Out of scope

- The adopter smoke job that runs this command in CI, filed as its own task.


== Dogfood

CI's adopter smoke job runs `nix run` against this flake on every push.


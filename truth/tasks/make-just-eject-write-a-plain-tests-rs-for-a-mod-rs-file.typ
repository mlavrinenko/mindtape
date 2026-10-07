#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Make just eject write a plain tests.rs for a mod-rs file",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.8,
    ease: 8,
  ),
  difficulty: 1,
  tags: ("tooling", "size"),
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
      sound-codebase: helps[`just eject` writes the layout the clippy lint and the `#[path]` check accept],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 7),
  links: related(
    "organize-each-crates-source-as-one-folder-per-area-with-a-mod-rs.typ",
  )[the eject half of its test layout],
)

== Problem

+ In an area folder, run `just eject` on a near-cap `mod.rs`.
+ 🔴 ejectest writes `mod_tests.rs` and a `#[path]` line, where the area layout wants a plain `mod tests;` in `tests.rs`.

== TO-BE

- 🟢 `just eject` passes `--mod-rs-tests tests`, so a `mod.rs` ejects to `tests.rs`; any other file still ejects to `<stem>_tests.rs`.

== Changes

- `flake.lock`: a qahq revision whose `ejectest` input has `--mod-rs-tests` (ejectest task `let-the-user-eject-a-mod-rs-files-tests-to-a-plain-tests-rs`, then `nix flake update ejectest` in `~/projects/home/qahq`).
- `Justfile`: the `eject` recipe passes the flag.

== Rollback

- Revert the commit; ejectest's default output is unchanged.

== Out of scope

- Renaming any existing `mod_tests.rs`: none exists until the moves land.

== Dogfood

The next `just eject` in an area folder writes `tests.rs`.

#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Pin rendered output with insta snapshots",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("tooling", "tests"),
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
      sound-codebase: helps[one snapshot kills a whole output shape],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: cancelled(
    2026,
    10,
    1,
  )[The motivating HyperlinkChoice::fmt survivor is already killed by render/link.rs:165 (4918023a); one assert\_eq does what a snapshot dependency and review workflow would.],
)

== Problem

+ Read `/tmp/survivors.txt`'s render section from the 2026-09-30 scoped sweep.
+ 🔴 `crates/mindtape-cli/src/render/link.rs:37:9 replace <impl fmt::Display for HyperlinkChoice>::fmt -> fmt::Result with Ok(Default::default())` survived into the sprint's survivor list: pinning one rendered shape takes several hand-written assertions per test.
+ A snapshot pins the whole output in one reviewable artifact and kills every format/`Display` mutant in that shape at once.

== TO-BE

- 🟢 `insta` is a dev-dependency and the render layer's output shapes have committed snapshots (`HyperlinkChoice` display, one table layout).
- 🟢 `cargo insta review` is the documented workflow; `just check` runs the snapshots like any test, and a changed byte fails until reviewed.
- 🟢 Snapshots are deterministic: no temp paths, no UUIDs, no clock.

== Changes

- `Cargo.toml` `[workspace.dependencies]` and `crates/mindtape-cli/Cargo.toml` `[dev-dependencies]`: add `insta`.
- New snapshot tests beside `crates/mindtape-cli/src/render/`: the `HyperlinkChoice` `Display` impl and one table layout from `table_tests_layout.rs`'s fixtures.
- `CONTRIBUTING.md` Testing (or the mutation-testing deep dive): one paragraph on `cargo insta review`.

== Rollback

Remove the dependency and the `snapshots/` files; the previous assertions are untouched.

== Out of scope

- Converting the existing render tests wholesale: one shape proves the workflow first.
- Snapshotting `--format json`: serde already pins the schema; snapshots there add review noise.

== Dogfood

Changing a separator or glyph byte in `render/` turns a snapshot red in `just check` before a sweep can.

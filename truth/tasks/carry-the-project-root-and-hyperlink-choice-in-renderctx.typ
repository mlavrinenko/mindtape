#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Carry the project root and hyperlink choice in RenderCtx",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  kind: "refactor",
  tags: ("cli",),
  estimate: (src-files: 10),
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
      sound-codebase: helps[render output stops depending on which project opened first, so one process can render several],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 16),
  links: parent("show-every-tracked-project-from-one-mt-name.typ"),
)

== Problem

+ Read `crates/mindtape-cli/src/render/file_path.rs:25` and `crates/mindtape-cli/src/render/link.rs:83`.
+ 🔴 `util::open` stores the project root and the hyperlink choice in two process-wide `OnceLock`s and ignores later writes, so a process that renders two projects prints the second one's FILE cells from the first one's root.


== TO-BE

- 🟢 `RenderCtx` carries the project root and the resolved hyperlink choice, and every FILE cell, `--format path` line and link reads them there.
- 🟢 `util::open` writes no render state, and one project's output stays byte-identical.
- 🟢 The current directory is still read once per process.


== Changes

- `crates/mindtape-cli/src/render/output_tests.rs`: the red test first: two contexts with different roots render each task's path from its own root.
- `crates/mindtape-cli/src/render/mod.rs`: `RenderCtx` gains `root` and `hyperlinks`.
- `crates/mindtape-cli/src/render/file_path.rs`: `cell_text` takes the root, and `CONTEXT` goes.
- `crates/mindtape-cli/src/render/link.rs`: `HYPERLINKS_ENABLED` goes, and the choice resolves to a value.
- `crates/mindtape-cli/src/render/table.rs`, `crates/mindtape-cli/src/render/columns.rs`, `crates/mindtape-cli/src/render/output.rs`: read both from the context.
- `crates/mindtape-cli/src/util.rs`: `Project` holds the resolved hyperlink choice.
- `crates/mindtape-cli/src/cmd/ls/mod.rs`, `crates/mindtape-cli/src/cmd/recur/mod.rs`, `crates/mindtape-cli/src/cmd/add/output.rs`: pass the root.


== Out of scope

- `COLOR_ENABLED`, which follows the terminal, not a project.
- `WORKTREE`, which picks the project before anything renders.


== Dogfood

Not applicable: this repo's output doesn't change. The refactor is what lets one `mt` process render a set.


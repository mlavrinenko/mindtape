#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Hold the folder-per-area layout with a lint and a path check",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("tooling", "gating"),
  estimate: (src-files: 4),
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
      sound-codebase: helps[the line cap can no longer flatten a child module back to the crate root],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 7),
  links: depends-on(
    "move-the-filter-language-into-a-filter-folder.typ",
    "move-the-config-sections-into-a-config-folder.typ",
    "move-the-tasks-rules-engine-into-a-rules-folder.typ",
    "move-validation-and-body-gating-into-validate-and-body-folders.typ",
    "move-the-commit-and-git-modules-into-commit-and-git-folders.typ",
    "move-the-task-graph-modules-into-a-graph-folder.typ",
    "move-the-task-model-modules-into-a-model-folder.typ",
    "move-the-remaining-small-library-areas-into-their-folders.typ",
    "move-the-clis-shared-modules-into-cli-and-render-folders.typ",
    "move-the-add-check-flip-and-init-commands-into-cmd-folders.typ",
    "move-the-remaining-cli-commands-into-cmd.typ",
  )
    + related(
      "organize-each-crates-source-as-one-folder-per-area-with-a-mod-rs.typ",
    )[carries out its layout],
)

== Problem

+ After the area moves, add `#[path = "filter_new.rs"] mod new;` to `crates/mindtape/src/filter/mod.rs` with the file at the crate root, or add `src/foo.rs` beside `src/foo/`.
+ 🔴 `just check` passes both: nothing holds the layout, and the 500-line cap pushes splits back to flat `#[path]` files, as it did before.

== TO-BE

- 🟢 The lint refuses `foo.rs` beside `foo/`:

```toml
[workspace.lints.clippy]
self_named_module_files = "deny"
```

- 🟢 A gate refuses any `#[path]` that names neither a `*_tests*.rs` sibling nor the `rrule` feature stub, and prints the file and line.
- 🟢 CONTRIBUTING says a module path is its file path and a new command is `src/cmd/<name>.rs`; AGENTS says each folder's `mod.rs` maps its files.

== Changes

- A red fixture first: a stray `#[path]` the gate must refuse.
- `Cargo.toml`: the lint.
- `.just/scripts/path-attr-check.sh` (new), `Justfile`, `.mmz/config.yaml`: the gate and its inputs.
- `docs/src/contributing.typ:201`, `docs/src/agents.typ`: the layout rule.

== Rollback

- Revert the commit: the lint and the gate leave no state behind.

== Out of scope

- Moving files: the area tasks do it, and this lands after them.

== Dogfood

`just check` holds this repo's layout from the commit it lands in.

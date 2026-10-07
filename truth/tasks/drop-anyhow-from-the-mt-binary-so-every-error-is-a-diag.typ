#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Drop anyhow from the mt binary so every error is a Diag",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  kind: "refactor",
  tags: ("cli",),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[every error says what failed and what to try, in one shape],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 7),
  links: depends-on(
    "build-an-error-dsl-that-fixes-the-message-format-and-the-exit-code.typ",
  )
    + depends-on(
      "reword-filter-function-errors-in-the-error-format.typ",
      "reword-the-view-ls-hook-and-dict-config-errors-in-the-error-format.typ",
      "reword-the-task-pack-actor-status-and-link-config-errors-in-the-error-format.typ",
      "reword-framework-and-body-rule-errors-in-the-error-format.typ",
      "write-rule-messages-as-the-next-step.typ",
    )
    + depends-on(
      "report-check-problems-on-stderr-in-the-error-format.typ",
      "report-config-load-errors-at-their-file-and-line.typ",
      "report-flip-refusals-and-gate-failures-in-the-error-format.typ",
      "report-import-and-init-errors-in-the-error-format.typ",
      "report-ls-errors-in-the-error-format.typ",
      "report-mt-add-target-and-editor-errors-in-the-error-format.typ",
      "report-mt-add-value-errors-in-the-error-format.typ",
      "report-mv-and-rm-errors-in-the-error-format.typ",
      "report-pre-write-refusals-once-without-a-nested-summary.typ",
      "report-retire-and-recur-errors-in-the-error-format.typ",
      "report-review-go-eval-worktrees-and-preset-errors-in-the-error-format.typ",
      "report-set-and-link-errors-in-the-error-format.typ",
      "report-task-references-and-project-discovery-in-the-error-format.typ",
      "report-typst-errors-at-their-line-with-typsts-hints.typ",
      "translate-filter-parse-errors-into-plain-words.typ",
    )
    + depends-on(
      "report-template-load-failures-and-install-hints-in-the-error-format.typ",
    ),
)

== Problem

+ Read `crates/mindtape-cli/src/main.rs` and `crates/mindtape-cli/src/cmd/mod.rs`.
+ 🔴 Modules still carry `allow(clippy::disallowed_macros)`, and `diag/from_lib.rs` still bridges `anyhow` chains with `Diag::legacy`, so a new `.context(format!(..))` compiles and prints free text.

== TO-BE

- 🟢 Every command returns `Result<_, Diag>`, `anyhow` is gone from `crates/mindtape-cli/Cargo.toml`, and a library error reaches the user only through its `From` in `diag/from_lib.rs`.
- 🟢 clippy refuses `std::io::stderr` outside `crate::diag`, so no line reaches stderr unrendered.

== Changes

- `crates/mindtape-cli/Cargo.toml`: drop `anyhow`.
- `crates/mindtape-cli/src/cli/mod.rs`, `crates/mindtape-cli/src/main.rs`: `run` returns `Result<(), Diag>`; drop the last `allow`s and delete `diag/allowlist.rs`.
- `crates/mindtape-cli/src/diag/from_lib.rs`, `crates/mindtape-cli/src/diag/mod.rs`: drop the `anyhow` bridge and `Diag::legacy`.
- `clippy.toml`: add `std::io::stderr` to `disallowed-methods`, allowed only in `diag/render.rs`.

== Out of scope

- Library crates, which keep `thiserror`.

== Dogfood

Every `mt` error in this repo's sessions from the files above comes out in the new shape.

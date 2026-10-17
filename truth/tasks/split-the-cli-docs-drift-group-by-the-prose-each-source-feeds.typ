#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Split the cli-docs drift group by the prose each source feeds",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("tooling", "docs"),
  estimate: (src-files: 1),
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
      sound-codebase: neutral,
      docs-match-code: helps[a group fires only for prose that restates its source, so a re-record names a review that happened],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 17),
  links: parent("keep-the-user-docs-generated-or-gated-not-restated.typ"),
)

== Problem

+ Change one line of `crates/mindtape-cli/src/cmd/ls/mod.rs` and run `just outdatty-check`.
+ 🔴 It prints `[ stale ]  cli-docs` and 27 `review dependent:` lines: 20 site pages, `cli-notes.typ`, the skill with its references and evals, and the `www/content/cli/*.typ` glob, whose 25 pages include 12 that hold only `#cli-command(...)`, rendered from `cli.json`.
+ 🔴 So `dc34aced`, `c6ce0a4e`, `5ed10162` and `73ba0b89` each re-recorded `cli-docs` in `outdatty.lock` with no doc edit. The group's comment in `outdatty.yaml` still says the CLI surface "is documented by hand".


== TO-BE

- 🟢 Each group pairs a source with the prose that restates it, such as `cmd/ls/mod.rs` with `www/content/cli/ls.typ`, or `model/fields.rs` with `filters.typ` and the skill's `filters.md`.
- 🟢 No group lists a stub page or `crates/mindtape/src/lib.rs`, whose re-exports move on ordinary commits.
- 🟢 `defaults.toml` and the presets sit only in the groups whose pages quote them.
- 🟢 `sets-docs` lists `crates/mindtape/src/at/board.rs` and `board.toml`, with `www/utils/cli-notes.typ` as a dependent.


== Changes

- `outdatty.yaml`: the red check first: with only `cmd/ls/mod.rs` changed, `outdatty check` must name one group whose dependents restate `ls`. Then split `cli-docs` into one group per prose owner, list the non-stub command pages instead of the glob, move `board.rs` and `board.toml` into `sets-docs`, and rewrite the group comments to say what each pair restates.
- `outdatty.lock`: re-record each new group after reviewing its dependents.


== Rollback

- Revert `outdatty.yaml` and `outdatty.lock`. No other file reads the group names.


== Out of scope

- Turning restated prose into generated facts. The milestone's other children do that, and each drops its source from a group here.
- The commit trailer that makes a re-record name its review, a sibling task that needs this split first.


== Dogfood

The next commit in this repo that changes one command module fires one small group, and its author reviews that prose.


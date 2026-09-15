#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Remove the backward-compatibility shims",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("mt", "cli"),
  estimate: (files: 12),
  kind: "refactor",
  assignee: "agent",
  status: done(2026, 9, 16),
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
      sound-codebase: helps[less code to keep correct, and an old spelling fails plainly instead of through a special case],
      docs-match-code: neutral,
    ),
  ),
)

== Problem

+ Read Version Management in `CONTRIBUTING.md`: no backward compatibility before the first public release.
+ Run `rg -n -i 'retired|deprecated|legacy' crates/mindtape/src crates/mindtape-cli/src --glob '!*tests*'`.
+ 🔴 The code still carries old spellings: `RETIRED` in `crates/mindtape/src/config_required.rs:28`, `RENAMED_FIELDS` in `crates/mindtape/src/fields.rs:174`, the retired `contains` order in `crates/mindtape/src/filter_predicates.rs:81`, the legacy per-kind names in `crates/mindtape/src/filter_links.rs:2`, and the hidden `mt install-lib` alias in `crates/mindtape-cli/src/cli.rs:98`.


== TO-BE

- 🟢 The Problem's `rg` finds only `mt rm`'s retired-task vocabulary, which names a feature.
- 🟢 An old spelling fails as an unknown key, field, value or command, with no pointer to its replacement.
- 🟢 `just check` passes.


== Changes

- `crates/mindtape/src/config_required.rs`: drop `RETIRED` and its message, and the tests pinning it in `config_required_tests.rs`.
- `crates/mindtape/src/fields.rs`: drop `RENAMED_FIELDS` and `rename_hint`.
- `crates/mindtape/src/filter_predicates.rs`: drop the retired-order error.
- `crates/mindtape/src/filter_links.rs`: drop the legacy per-kind names from `has()` and `miss()`.
- `crates/mindtape-cli/src/cli.rs` and `crates/mindtape-cli/src/install.rs`: drop `mt install-lib`.
- `crates/mindtape/src/defaults.toml` and `.mindtape/config.toml`: drop the comments naming the retired `[ls] default-filter`.
- `www/content/`: drop page text naming a removed spelling; `rg` each one first.
- `CHANGELOG.md`: list each removed spelling under Unreleased.


== Out of scope

- `mt rm`'s retired-task vocabulary: it names a feature, not a compatibility path.
- Consumer repos whose configs still use old spellings: each fixes its own config.


== Dogfood

Built in: this repo's own config and tasks must still load once the shims are gone.


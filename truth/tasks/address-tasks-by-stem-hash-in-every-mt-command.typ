#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Address tasks by stem hash in every mt command",
  kind: "feat",
  priority: ice(
    impact: 8,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("mt", "cli"),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[an agent acts on the stem hash any listing prints],
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[the ordinal, the check letter and their config go],
      docs-match-code: helps[the CLI and config pages describe one stable handle],
    ),
  ),
  assignee: "agent",
  status: wip(2026, 10, 3),
  links: depends-on("derive-a-stem-hash-for-each-task-in-the-library.typ"),
)

== Problem

+ Run `mt ls 'has_tag("found")'` and note the `#` of a row.
+ Pass it to `mt eval`.
+ 🔴 It resolves against the default view, not the listing that printed it, so it names another task or is refused.


== TO-BE

- 🟢 Every listing prints each task's stem hash in a `SLUG#` column, keyed `slug#` in `--columns`, in place of `#`.
- 🟢 Every command taking a `<task>` accepts a stem hash. An exact stem or path wins; input is tried as a stem hash only when no stem matches.
- 🟢 An ambiguous stem hash is refused with each candidate's stem and hash. A write command no longer reports handle changes.
- 🟢 `[handle] check-len`, the ordinal and the check letter are gone, and the docs pages describe the stem hash.


== Changes

- `crates/mindtape-cli/tests/`: red e2e tests first: a hash from a filtered `mt ls` resolves to its row's task, and an ambiguous one is refused.
- `crates/mindtape-cli/src/util.rs`, `render/`: resolve and print through `mindtape::handle`.
- Delete `crates/mindtape-cli/src/handle.rs`, `flip_handles.rs`, `crates/mindtape/src/config_handle.rs` and callers.
- `www/content/cli.typ`, `config.typ`, schema: the stem hash, no `[handle]`.


== Alternatives

- Keep ordinals and resolve against the printing listing: a handle would mean nothing outside its listing.


== Out of scope

- The skill, its evals and the changelog, which are the next task.


== Dogfood

Agents here pass the hash from `mt ls ready` to `mt wip` and `mt done`.


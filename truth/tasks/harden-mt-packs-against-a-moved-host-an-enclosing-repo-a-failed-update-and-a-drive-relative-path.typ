#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Harden mt packs against a moved host, an enclosing repo, a failed update and a drive-relative path",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 4,
  estimate: (src-files: 5),
  tags: ("found", "mt"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a pack update lists every new `run` line even when the old tree is gone],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: wip(2026, 10, 26),
  links: parent("share-config-between-projects-through-packs.typ"),
)

== Problem

+ Review of the landed range 7fe45d58^..b967e214, the mt packs commands.
+ 🔴 #raw("crates/mindtape/src/config/pack_sources/git.rs"): Four gaps in `mt packs`, each a case the commands don't hold.


- `report.rs:36`: a failed old-tree fetch prints no diff, so a new `run` line goes unlisted.
- `remote.rs:38`: `ls-remote` runs where `mt` runs, so git reads an enclosing repository's config, such as `core.sshCommand` or `url.insteadOf`.
- `update.rs:75`: progress prints before a later source fails and the lock stays unwritten.
- `fetch.rs:151`: `locate("C:packs", root)` is relative to drive C's current directory.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/config/pack_sources/git.rs").

== TO-BE

- 🟢 When the old tree can't come from the old `git`, the diff fetches it from the source's new `git`, and failing that lists the new tree as all added.
- 🟢 No git command `mt` runs reads a repository or config around the directory it runs in.
- 🟢 `mt packs update` prints its report only after the lock is written.
- 🟢 A `git` of a drive letter and a colon with no separator after it is refused on every platform.


== Changes

- `crates/mindtape/src/config/pack_sources/git.rs`: every command runs apart from any enclosing repository.
- `crates/mindtape/src/config/pack_sources/fetch.rs`: refuse a drive-relative `git`.
- `crates/mindtape-cli/src/cmd/packs/report.rs`: retry from the new `git`, then diff against an empty tree.
- `crates/mindtape-cli/src/cmd/packs/update.rs`: buffer the report until the lock is written.
- `crates/mindtape-cli/tests/it/packs/` and the unit tests beside each file: the red tests first.


== Rollback

- Revert the commit.


== Out of scope

- The lock format and the cache layout: neither has the gaps.


== Dogfood

Not applicable: this repo adopts only the built-in `mindtape:` packs.


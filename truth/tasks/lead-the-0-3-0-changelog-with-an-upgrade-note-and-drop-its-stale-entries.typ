#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Lead the 0.3.0 changelog with an upgrade note and drop its stale entries",
  kind: "dev",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  estimate: (src-files: 1),
  tags: ("found", "docs"),
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
      docs-match-code: helps[the release notes describe the shipped mt, not interim states],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 14),
)

== Problem

+ Pre-release review for 0.3.0, after `just release 0.3.0 --dry-run` passed
+ 🔴 #raw("CHANGELOG.md"): `[Unreleased]` is 171 KB of 257 entries logged commit by commit, some naming commands that no longer exist, and nothing tells a 0.2.0 user that the binary is now `mt` and their task files must be refiled

```
$ sed -n '8,2499p' CHANGELOG.md | wc -c
171188
$ git show v0.2.0:crates/mindtape-cli/Cargo.toml | grep -A1 '\[\[bin'
[[bin]]
name = "mindtape"
```
Stale lines under `### The rewrite (2026-06-14)`: `mindtape check`, `mindtape install-lib`, `typst-harvest` as a workspace crate, and a link to `docs/decisions/002-per-file-rewrite-and-speconaut-convergence.md`, which does not exist.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("CHANGELOG.md").

== TO-BE

- 🟢 The 0.3.0 section opens with a few lines saying it is a rewrite, that the binary is `mt`, and how a 0.2.0 user starts over (`mt init`, refile tasks).
- 🟢 Every entry describes the shipped behavior; none names a removed command, a crate that left the workspace, or a missing file.


== Changes

- `CHANGELOG.md`: add an upgrade note under `## [Unreleased]`, rewrite or drop the stale entries of `### The rewrite` and `### The agenda era`, and remove the dead `docs/decisions/` link.


== Rollback

- Revert the `CHANGELOG.md` commit.


== Out of scope

- Condensing the 257 entries into highlights. That is an editorial call for the owner.


== Dogfood

The 0.3.0 GitHub release and crates.io readers get the note, once the release body carries the changelog section.

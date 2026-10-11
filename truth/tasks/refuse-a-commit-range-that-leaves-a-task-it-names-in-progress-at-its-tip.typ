#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Refuse a commit range that leaves a task it names in progress at its tip",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 5,
  ),
  difficulty: 5,
  tags: ("mt",),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a branch can't land with a task it worked on still marked wip, and no repo needs a shell script for it],
      cheap-adoption: helps[every repo's worktrunk pre-merge hook becomes one `mt check --commits` line],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: wip(2026, 10, 11),
)

== Problem

+ A branch commits work with `Refs: <stem>` but forgets `mt done <stem>`, then lands.
+ 🔴 `mt check --commits <range>` (crates/mindtape-cli/src/cmd/check/range.rs) holds each commit to `[commits]`, but nothing asks whether a task the range names is still `wip` at the tip. A shell helper does that today: the wip block of ~/projects/home/mindtape/.just/scripts/integrate.sh, now `wt-tasks-done` in ~/.nix/apps/worktrunk.nix. Every repo would copy it into `.just/scripts/` (task `configure-worktrunk-in-the-repo-so-a-fresh-clone-needs-no-host-setup`).


== TO-BE

- 🟢 A `[[commits.rules]]` entry with `scope = "tip"` runs once per task named anywhere in the range, against the tip's tree. `mt check --commits main..HEAD` runs it alongside the per-commit rules.
- 🟢 The mindtape-tasks pack ships `named-tasks-finish-at-the-tip`: `assert = '!(status == "wip")'`, with a message saying to run `mt done <stem>` and commit it with the last work.
- 🟢 The worktrunk hook becomes `tasks = "mt check --commits {{ target }}..HEAD"`.


== Changes

- `crates/mindtape/src/config/commits.rs`: the `tip` scope.
- `crates/mindtape/src/` commit-check engine: collect the named stems over the range, then evaluate the tip-scoped rules once per stem at HEAD.
- `crates/mindtape-cli/src/cmd/check/range.rs`: run them after the per-commit pass.
- `crates/mindtape/src/packs/mindtape-tasks.toml`: the default rule.
- Tests first: a range naming a task left wip fails with its stem; a done one passes; a stem named in two commits is reported once. Docs: the rules reference.


== Alternatives

- A bespoke `--finished` flag: a second way to say what a rule already says, and not configurable.
- Keep the shell helper: one copy per repo, and it re-parses trailers mt already parses.


== Out of scope

- Carrying gate records between checkouts: mmz's task `import-gate-records-from-another-checkout-of-the-same-repo`.
- Removing `.just/scripts/wt-tasks-done.sh` from repos: each repo's follow-up once this ships.


== Dogfood

This repo's `.config/wt.toml` pre-merge runs `mt check --commits {{ target }}..HEAD`, and a scratch branch left wip refuses to land.


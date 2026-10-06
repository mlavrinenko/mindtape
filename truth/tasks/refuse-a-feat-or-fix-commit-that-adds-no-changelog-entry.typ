#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Refuse a feat or fix commit that adds no changelog entry",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.7,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("tooling", "gating"),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[the changelog entry lands with the change instead of depending on someone remembering it],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[CHANGELOG.md's Unreleased section lists every user-visible change when a release cuts it],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 7),
)

== Problem

+ Read `git log --format='%h %s' 37a8e3b7..93d78e66 -- CHANGELOG.md` after the 2026-10-06 docs round.
+ 🔴 Of eleven branches, only the two `fix` commits touched `CHANGELOG.md`. The `feat(www)` and `docs(www)` work for Markdown copies, per-command pages, diagrams and the lock merge driver landed without an entry, and one catch-up commit (`169b4260`) added them later.
+ 🔴 Neither `CONTRIBUTING.md` nor a commit rule asks for an entry; outdatty's `release-notes` group fires only on `Cargo.toml`.


== TO-BE

- 🟢 A `feat` or `fix` commit that does not change `CHANGELOG.md` is refused by the commit hook with a message naming the `[Unreleased]` section.
```toml
[[commits.rules]]
name = "changelog-with-change"
scope = "commit"
when = 'type == "feat" || type == "fix"'  # `type`: the field the filter language may need to gain
assert = 'count(files, "CHANGELOG.md") >= 1'
message = "add an entry under [Unreleased] in CHANGELOG.md"
severity = "error"
```
- 🟢 Other types (`docs`, `build`, `test`, `chore`, `task`, `refactor`) pass without an entry, so the rule stays one line of policy.
- 🟢 CONTRIBUTING.md says it once, under Commit Messages.


== Changes

- `.mindtape/config.toml`: a `[[commits.rules]]` entry over the commit type and the staged files. If the filter language cannot read the commit type, extend it there instead of adding a checker.
- `docs/src/contributing.typ`: the sentence, regenerated into CONTRIBUTING.md.


== Rollback

- Delete the rule; commits go back to unchecked.


== Out of scope

- The wording of entries, which review owns.
- `docs(www)` changes that readers notice. Tag them `feat` when they add something.


== Dogfood

This repo's next `feat` commit carries its changelog entry.


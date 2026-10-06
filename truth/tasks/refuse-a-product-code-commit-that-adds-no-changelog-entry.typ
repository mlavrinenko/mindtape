#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Refuse a product-code commit that adds no changelog entry",
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
  status: done(2026, 10, 7),
)

== Problem

+ Read `git log --format='%h %s' 37a8e3b7..93d78e66 -- CHANGELOG.md` after the 2026-10-06 docs round.
+ 🔴 Of eleven branches, only the two `fix` commits touched `CHANGELOG.md`. The `feat(www)` and `docs(www)` work for Markdown copies, per-command pages, diagrams and the lock merge driver landed without an entry, and one catch-up commit (`169b4260`) added them later.
+ 🔴 Neither `CONTRIBUTING.md` nor a commit rule asks for an entry; outdatty's `release-notes` group fires only on `Cargo.toml`.


== TO-BE

- 🟢 The trigger is the files a commit changes, not its type. A commit that changes product code (`crates/**` and `lib/**`, minus `crates/*/tests/**` and `*test*.rs`) and does not change `CHANGELOG.md` is refused, whether it is typed `feat`, `fix`, `refactor` or anything else. `skills/` stays out: its `SKILL.md` is generated from `docs/src/`.
```toml
[[commits.rules]]
name = "changelog-with-product-change"
when = 'count(files, "@product") >= 1 && !matches(trailers, "(?i)^Changelog: none$")'
assert = 'matches(files, "^CHANGELOG[.]md$")'
message = "add an entry under [Unreleased] in CHANGELOG.md, or add a `Changelog: none` trailer"
severity = "error"
```
- 🟢 A `Changelog: none` trailer lets a product commit through when it has nothing to announce.
- 🟢 The commit-msg hook (`mt check --commit-msg`) refuses it locally, and the CI step over the pushed range refuses it again. CI judges only commits after the one that added the rule, found with `git log -S`, so no hash goes stale on a rebase.
- 🟢 The commit language gains `trailers`, every trailer line as `Key: value`, so the rule is config and no checker was added.
- 🟢 CONTRIBUTING.md says it once, under Commit Messages.


== Changes

- `crates/mindtape/src/commit_msg.rs`, `commit_expr.rs`: the `trailers` field.
- `.mindtape/config.toml`: the `product` path set and the rule.
- `.github/workflows/ci.yml`: the floor moves to the rule's own commit.
- `docs/src/contributing.typ`, `www/content/commit-rules.typ`: the sentence and the field.


== Rollback

- Delete the rule; commits go back to unchecked.


== Out of scope

- The wording of entries, which review owns.
- Docs-only changes that readers notice. They carry no product code, so the rule leaves them to review.


== Dogfood

The commit that adds the rule carries its own entry.


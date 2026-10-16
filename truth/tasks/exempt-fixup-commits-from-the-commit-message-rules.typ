#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Exempt fixup commits from the commit-message rules",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 1,
  estimate: (src-files: 2),
  tags: ("found", "gating"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a review fix lands in the commit it belongs to without trailers autosquash throws away],
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
  status: done(2026, 10, 16),
)

== Problem

+ Reviewing three agent branches, each told to fold review fixes in with `git commit --fixup` and `rebase --autosquash`.
+ 🔴 #raw("crates/mindtape/src/packs/mindtape-tasks.toml"): `mt check --commit-msg` refuses a `fixup!` commit for a missing `Refs:` footer and changelog trailer, though autosquash drops its message and `just land` re-checks the squashed commits.

```
$ mt check --commit-msg msg   # "fixup! feat(cli): list dictionaries ...", plural.rs staged
error: msg: commits.rules.work-names-a-task: name the task this commit works on in a `Refs: <stem>` footer
error: msg: commits.rules.changelog-with-product-change: add an entry under [Unreleased] in CHANGELOG.md, or add a `Changelog: none` trailer
error: 2 commit message problems
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/packs/mindtape-tasks.toml").

== TO-BE

- 🟢 A commit whose subject starts with `fixup! `, `squash! ` or `amend! ` passes `mt check --commit-msg` without trailers.
- 🟢 The `pre-merge` hook refuses a branch that carries one unsquashed, so `just land` can't land it. No commit rule scope can do this: a tip rule never sees a commit with no `Refs:`, and commit scope runs the same in both modes.

```toml
[[commits.rules]]
name = "work-names-a-task"
when = '!tasks_only && !matches(subject, "^(fixup|squash|amend)! ")'
assert = "has(refs)"
```


== Changes

- `crates/mindtape-cli/tests/it/commit_msg_e2e.rs`: the red tests, the pack passing each fixup prefix and still refusing a plain commit.
- `crates/mindtape/src/packs/mindtape-tasks.toml`: `work-names-a-task` skips a fixup subject.
- `.mindtape/config.toml`: the same guard on `changelog-with-product-change`.
- `.config/wt.toml`: a `pre-merge` hook refuses a branch whose subjects include a fixup.
- `docs/src/contributing/commit-messages.typ` and `www/content/commit-rules.typ`: state the exemption and the hook.


== Rollback

- Revert the files above.


== Out of scope

- The task-scope commit rules, which read the staged tree rather than the message.


== Dogfood

Every review round in this repo folds its fixes with `--fixup` and no throwaway trailers.

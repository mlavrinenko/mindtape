#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Prepare the 0.4.0 release docs",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("docs", "stateless", "mt"),
  estimate: (src-files: 8),
  x-goals: (
    product: verdicts(
      (
        enforced-practice: helps[a release ships docs that name the shipped behavior],
      ),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (
        docs-match-code: helps[the shipped docs and hook match the merge skip and AS-IS rename],
      ),
      ..("sound-codebase",),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 3),
)

== AS-IS

+ Ran the pre-release review against `[Unreleased]` and the 0.4.0 tree.
+ 🔴 Three shipped docs still teach ``Problem`` while `mindtape:tasks` refuses it: the task-files example, the api-service config comment, and four ask-refactoring remedies.
+ 🔴 `.githooks/commit-msg` still bypasses `mt check --commit-msg` on a merge, so the new merge skip never runs through this repo's hook.
+ 🔴 The tip-scope prose and schema say "any commit of a range" although a skipped merge names none.


== TO-BE

- 🟢 The task-files page shows `AS-IS`, and no shipped prose names `Problem`.
- 🟢 A merge runs `mt check --commit-msg`, which skips it.
- 🟢 Tip scope says "each checked commit", matching the range.
- 🟢 `www/content/config.typ` names the embedded fallback too.

```
$ mt check --commit-msg msg
ok: merge commit, commit rules skipped
```


== Changes

- `.githooks/commit-msg`: drop the stopgap and let `mt` skip the merge.
- `crates/mindtape/src/config/commits.rs`, `schema/config.schema.json`: say each checked commit.
- `www/content/commit-rules.typ`, `www/content/config.typ`: match the code.
- `crates/mindtape-cli/src/warnings.rs`: correct the module comment.
- `examples/task-files/tasks/fix-the-login-redirect.typ`, `examples/api-service/.mindtape/config.toml`, `contrib/ask-refactoring/*.sh`: rename `Problem` to `AS-IS`.
- `CHANGELOG.md`: add the upgrade note and correct the package fallback.


== Out of scope

- The `mt tags` scan's `has(tags, ...)` gap and whitespace forms. Each gets a finding.
- Bumping the Typst package version; it stays 0.3.0.
- The release itself; the script runs after this lands.


== Dogfood

This repo's docs site and task-files example show `AS-IS`, and its commit hook runs the merge skip.


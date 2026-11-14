#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Lint commit messages with Vale",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("tooling", "gating", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[the commit hook and the range check refuse the same prose],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 14),
)

== AS-IS

+ Read `.githooks/commit-msg`. Its checks read trailers, paths and the staged task tree, and none reads the subject's prose.
+ 🔴 The subject's shape and prose go unchecked, including the Conventional Commits type `CONTRIBUTING.md` asks for.
+ Run `git show -s --format=%s 5b0076a1` and see a noun phrase that passed every check.

```
$ git show -s --format=%s 5b0076a1
docs: ADR 003 SWHID cleanup tombstones and shared edit substrate
```

== TO-BE

- 🟢 `.config/vale-commits.ini` runs a `Commits` style over the message: subject at most 72 characters, an imperative opener, no history word, no em dash, no shouted word.
- 🟢 `.githooks/commit-msg` runs the style after `mt check --commit-msg`, so a bad message never becomes a commit.
- 🟢 `mt check --commits <range>` runs the same check over each message, so a skipped hook still fails before the range lands.
- 🟢 A hit states the line and quotes the word, as every Vale run does.

```ini
[formats]
txt = md
[*.txt]
BasedOnStyles = Prose, Commits
Prose.ProductName = NO
Commits.SubjectLength = error
```

== Changes

- `.config/vale-commits.ini`: `[formats] txt = md`, the `Commits` style and the five rule levels.
- `.config/vale/styles/Commits/`: one YAML per commit rule, with the subject length, imperative, history, em dash and shouting rules.
- `.just/scripts/check-commit-prose.sh`: one Vale run over a message file, shared by the hook and the range arm.
- `.githooks/commit-msg`: run the script on the message file after `mt check --commit-msg`.
- `crates/mindtape-cli/src/cmd/check/range.rs`: run the script over every commit of `--commits`. A range then fails wherever a hook never ran.
- `docs/src/contributing/commit-messages.typ`: say which rules run and where.

== Alternatives

- gitlint: it checks the shape and the trailers, not prose.
- A `[[commits.rules]]` regex: it can count characters and find a listed word, and it cannot judge an imperative opener.

== Out of scope

- Pull request descriptions and release notes, which no commit hook reads.

== Dogfood

Every commit after this one passes the style, starting with the commit that lands the task.

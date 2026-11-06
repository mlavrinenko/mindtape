#import "@local/mindtape:0.4.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Name the next command in every `mt` dead end",
  kind: "feat",
  priority: ice(
    impact: 7,
    confidence: 0.85,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("cli", "stateless"),
  x-goals: (
    product: verdicts(
      (
        cheap-adoption: helps[a newcomer learns the next command from the dead end itself],
      ),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "enforced-practice",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (
        docs-match-code: helps[the CLI text carries the workflow instead of a page beside it],
      ),
      ..("sound-codebase",),
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 7),
)

== AS-IS

+ Run `mt explain`, then `mt ls 'status == "failed"'`.
+ 🔴 The first ends at the dictionaries and names no next command. The second prints a zero-row table with no way out.

```
$ mt explain | tail -2
dicts: mindtape-dev-verbs (318), mindtape-verbs (500), product (4), source (16), verbs (818)
$ mt ls 'status == "failed"'
SLUG#  TITLE  FILE

your filter · 0 of 941 tasks
```

+ Run `mt eval nope`; it already names `mt ls`, which proves the convention exists.


== TO-BE

- 🟢 Every refusal ends with the command that resolves it, as `mt eval nope` already does.
- 🟢 `mt explain` ends with the first command to run, such as `mt ls`, and each listing says how to go deeper.
- 🟢 An empty listing says how to widen the filter or list everything.
- 🟢 A successful write says what to read next.

```
$ mt ls 'status == "failed"'
SLUG#  TITLE  FILE

your filter · 0 of 941 tasks
hint: no task matches; run `mt ls ''` for every task, or `mt explain views` for a saved view
$ mt explain
...
next: run `mt ls` for your work, or `mt <command> --help` for flags
```


== Changes

- `crates/mindtape-cli/src/cmd/explain/index.rs`: the closing `next:` line and one for each listing.
- `crates/mindtape-cli/src/cmd/ls/rules_applied.rs`: the footer hint when the filter selects nothing.
- `crates/mindtape-cli/src/diag/mod.rs`: a shared `next:`/`hint:` helper if the existing hints duplicate it.
- `crates/mindtape-cli/tests/it/` (the matching file): the red tests first, one per dead end.


== Alternatives

- Fix only `mt explain`: leaves the empty listing and the refusals inconsistent.
- Write it in the docs: a reader hits the dead end first and never sees the page.


== Out of scope

- Rewriting `--help` text that already names the flags.
- Changing any command's behavior or exit code.


== Dogfood

This repo's `mt` calls are the examples; `just check` runs the CLI tests that assert each hint.


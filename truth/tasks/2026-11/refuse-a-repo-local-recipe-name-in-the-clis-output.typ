#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Refuse a repo-local recipe name in the CLI's output",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("mt", "cli", "gating", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a repo-local name cannot reach another project terminal],
      config-over-code: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: helps[a hint states only a command the reader can run],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 21),
)

== AS-IS

+ Ask what stops a user-facing string from naming a `just` recipe.
+ 🔴 No gate stops it. The CLI's help text lands in `www/generated/cli.json` for the docs, but no gate reads its content, and a `Diag` hint is a bare literal:

```
$ grep -rn '`just ' crates/mindtape-cli/src --include=*.rs | wc -l
7
$ grep -rn '"[^"]*just [a-z]' crates/mindtape-cli/src --include=*.rs | grep -v tests | wc -l
0
```

+ All seven are doc comments, which a reader of the source sees and a user of `mt` does not. The gate has to tell them apart, and nothing does today.

== TO-BE

- 🟢 A user-facing string that mentions a `just` recipe fails a gate, naming the file and the line.
- 🟢 A doc comment that mentions one passes, because it never reaches a user.
- 🟢 The gate reads the recipe names from the `Justfile`, so a renamed recipe cannot slip through.

```
$ just lint::no-just-in-output
error: crates/mindtape-cli/src/cli/mod.rs:42 names the recipe `check`, which only this repo has
```

== Changes

- `.just/scripts/check-no-just-in-output.sh`: strip `//` and `//!` lines, then fail on `just <recipe>` for any recipe the `Justfile` declares.
- `.just/lint.just`: the `no-just-in-output` recipe, carrying `[group("gate")]`.
- `Justfile`: the recipe joins `_check-fast`, or `meta::gate-drift` fails.
- `crates/mindtape-cli/tests/it/cli_text_e2e.rs`: the clap tree from `mt _cli --json` carries no `just ` in any `about`, `long_about` or arg help.
- `www/content/cli.typ`: the rule, and why a hint states only commands `mt` provides.
- `CHANGELOG.md`: an entry under `[Unreleased]` `Added`.

== Alternatives

- A Rust test alone: it reads the clap tree, so it misses a `Diag` hint, which is a bare literal.
- A grep for `just` with no recipe list: `just` is an ordinary English word, and the CLI's prose uses it.

== Out of scope

- The seven doc comments that name a recipe today. They are developer-facing and stay.
- A general rule about every repo-local name. This one covers `just`, which is the one that leaked.

== Dogfood

This repo's seven `just` mentions in `crates/mindtape-cli` are all doc comments. The gate passes on the tree as it stands and fails the moment a hint mentions `check`.

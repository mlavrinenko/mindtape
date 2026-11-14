#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Lint code comments with Vale",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 5,
  tags: ("tooling", "gating", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a doc comment meets the prose rules beside the code it describes],
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
  status: done(
    2026,
    11,
    15,
  )[The Vale gate reads Rust doc comments and the first commit clears 215 hits.],
)

== AS-IS

+ Run `grep -n 'used to' crates/mindtape/src/sort.rs`.
+ 🔴 It finds a doc comment that reads as history on line 49, and `just lint` passes, because no arm reads a Rust comment.
+ Run `vale --config=.config/vale.ini` on a `.typ` file with a comment. The comment never reaches a rule, because typst2vast drops it.
+ `check-doc-prose` reads the full-line comments of shell scripts, YAML files and Justfiles, and no Rust file.

```
$ grep -n 'used to' crates/mindtape/src/sort.rs
49:/// `:` suffix is anything but `asc`/`desc`. A misspelled direction used to
```

== TO-BE

- 🟢 `.config/vale-code.ini` maps `[formats] rs = md` and gives `[*.rs]` the `Prose` style. A doc comment then reads as Markdown.
- 🟢 `.just/scripts/check-code-prose.sh` prints every `///` and `//!` line, strips the marker, and pipes it to `vale --ext=.rs`.
- 🟢 `just lint vale-code` runs in `just check` and names each hit with its file and line.
- 🟢 Inline `//` comments, code and doc tests stay out of the run.
- 🟢 The config carries a `[*.sh]` section as text for the comment move that follows. The first gate reads Rust doc comments only.

```ini
[formats]
rs = md
[*.rs]
BasedOnStyles = Prose
```

== Changes

- `.config/vale-code.ini`: `[formats] rs = md`, the `[*.rs]` section with the `Prose` style, and the `[*.sh]` text section.
- `.just/scripts/check-code-prose.sh`: pull every `///` and `//!` line, strip its marker, pipe it to Vale as `.rs`, and print each hit with its file and line.
- `.just/lint.just`: the `vale-code` gate recipe.
- `.mmz/config.yaml`: the `just lint::vale-code` rule, its inputs and its place in `just check`.
- `docs/src/contributing/documentation.typ`: say which comments Vale reads and that inline comments stay out.

== Alternatives

- `clippy::doc_markdown`: it flags an unmarked code span, not a sentence.
- The regex scanner would copy the word lists a third time.

== Out of scope

- Inline `//` comments, which speak to the next reader of the code rather than a user.
- Commit messages, which have a task of their own.

== Dogfood

The gate reads the doc comments of every Rust source under `crates/`, and the commit that lands it fixes the hits the first run finds.

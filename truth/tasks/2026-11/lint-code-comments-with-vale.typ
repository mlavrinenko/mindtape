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
    product: verdicts(
      (
        enforced-practice: helps[a doc comment meets the prose rules where the code is written],
      ),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
        "self-describing",
      ),
    ),
    process: verdicts(
      (:),
      ..("sound-codebase", "docs-match-code"),
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 14),
)

== AS-IS

+ Run `grep -n 'used to' crates/mindtape/src/sort.rs`.
+ 🔴 It finds a doc comment that reads as history on line 49, and `just lint` passes, because no arm reads a Rust comment.
+ Run `vale --config=.config/vale.ini` on a `.typ` file with a comment; the comment never reaches a rule, because typst2vast drops it.
+ `check-doc-prose` reads the full-line comments of shell scripts, YAML files and Justfiles, and no Rust file.

```
$ grep -n 'used to' crates/mindtape/src/sort.rs
49:/// `:` suffix is anything but `asc`/`desc`. A misspelled direction used to
```

== TO-BE

- 🟢 `.config/vale-code.ini` maps `[formats] rs = md` and gives `[*.rs]` the `Prose` style, so a doc comment reads as Markdown.
- 🟢 `.just/scripts/check-code-prose.sh` prints every `///` and `//!` line, strips the marker, and pipes it to `vale --ext=.rs`.
- 🟢 `just lint vale-code` runs in `just check` and names each hit with its file and line.
- 🟢 Inline `//` comments, code and doc tests stay out of the run.
- 🟢 The config carries a `[*.sh]` section as text for the comment move that follows; the first gate reads Rust doc comments only.

```ini
[formats]
rs = md
[*.rs]
BasedOnStyles = Prose
```

== Changes

- `.config/vale-code.ini`: `[formats] rs = md`, the `[*.rs]` section with the `Prose` style, and the `[*.sh]` text section.
- `.just/scripts/check-code-prose.sh`: extract every `///` and `//!` line, strip its marker, pipe it to Vale as `.rs`, and print each hit with its file and line.
- `.just/lint.just`: the `vale-code` gate recipe.
- `.mmz/config.yaml`: the `just lint::vale-code` rule, its inputs and its place in `just check`.
- `docs/src/contributing/documentation.typ`: say which comments Vale reads and that inline comments stay out.

== Alternatives

- `clippy::doc_markdown`: it flags an unmarked code span, not a sentence.
- The regex scanner: it would copy the word lists a third time.

== Out of scope

- Inline `//` comments, which speak to the next reader of the code rather than a user.
- Commit messages, which have a task of their own.

== Dogfood

Every Rust source under `crates/` has its doc comments read, and the hits the first run finds are fixed in the commit that lands the gate.

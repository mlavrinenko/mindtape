#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Lint Typst prose with Vale through typst2vast",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("tooling", "docs", "gating", "stateless"),
  x-goals: (
    product: verdicts(
      (readable-tasks: helps[the docs prose meets one rule set]),
      ..(
        "implementable-tasks",
        "enforced-practice",
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
  status: done(2026, 11, 14),
)

== AS-IS

+ Run `vale` on a `.typ` file with a `[*.typ]` section in `.config/vale.ini`.
+ 🔴 It prints `E100: typst2vast not found; it's needed to read Typst`.
+ Run `command -v typst2vast`; nixpkgs has no package for it.

```
$ vale www/content/index.typ
[www/content/index.typ]:E100:typst2vast not found; it's needed to read Typst
```


== TO-BE

- 🟢 `typst2vast` and Vale 3.26 come from the dev shell, built from their tagged sources.
- 🟢 Vale reads `*.typ` as prose: headings, lists, quotes and `[content]` blocks.
- 🟢 Code, math, raw spans and comments stay out of the prose rules.
- 🟢 The docs run the history, shouting, em-dash and typed-name rules; a task body runs shouting and the em-dash rule, because a task records a change.

```
$ printf '= Title\n\nThis is a sentence.\n' > /tmp/t.typ
$ vale /tmp/t.typ
# no E100
```


== Changes

- `nix/dev/typst2vast.nix`: build the `typst2vast` binary with `buildRustPackage`.
- `nix/dev/vale.nix`: build Vale 3.26 with the dev shell's Go 1.26.
- `nix/dev/shell.nix`: add both binaries to the dev shell.
- `.config/vale.ini`: map `[*.typ]` to the `Prose` style and skip a quoted example.
- `.config/vale/styles/Prose/`: the history, shouting, em-dash and typed-name rules.
- `.just/docs.just`: `docs::vale-prose` for the docs and `docs::vale-tasks` for a task body.
- `Justfile` and `.mmz/config.yaml`: run both arms in `just check` with their inputs. The task arm carries no `gate` tag, so `mt flip` writing the task file cannot stale its record.
- `.githooks/pre-commit`: run the task prose through Vale, since a commit that stages only a task file skips the freshness assertion.
- `.config/outdatty.yaml`: the `vale-docs` group, which ties the styles and config to the two pages that state them.
- `docs/src/contributing/documentation.typ`: list the rule codes and keep only the judgment rules in prose.
- `docs/src/contributing/gates.typ`: say why the close predicate skips the task arm.
- `truth/tasks/2026-10/`: the 20 em dashes Vale reported, and the `MindTape` spelling, both cleaned.


== Alternatives

- Map `typ = rs` in `[formats]`: the Rust grammar misreads Typst raw blocks and comments.
- Wait for a tree-sitter Typst grammar in Vale: the shipped path is `typst2vast`, and comments are dropped either way.


== Out of scope

- Typst comments: the converter drops them, so the banner gate stays on Rust, Shell and CSS.
- Replacing `check-doc-prose.sh`: it keeps the repo's rules until the Vale styles match them.


== Dogfood

This repo's `docs/src`, `www/content` and `truth/tasks` prose runs through Vale
once the styles land. The task arm starts at the month the rule lands, and a
backfill task widens it to the older months.


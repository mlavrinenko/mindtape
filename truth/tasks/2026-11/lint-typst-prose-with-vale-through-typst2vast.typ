#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Lint Typst prose with Vale through typst2vast",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
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
  status: proposed(2026, 11, 13),
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

- 🟢 `typst2vast` comes from the dev shell, built from the `jdkato/typst2vast` source.
- 🟢 Vale reads `*.typ` as prose: headings, lists, quotes and `[content]` blocks.
- 🟢 Code, math, raw spans and comments stay out of the prose rules.

```
$ printf '= Title\n\nThis is a sentence.\n' > /tmp/t.typ
$ vale /tmp/t.typ
# no E100
```


== Changes

- `nix/dev/typst2vast.nix`: build the `typst2vast` binary with `buildRustPackage`.
- `nix/dev/shell.nix`: add `typst2vast` to the dev shell.
- `.config/vale.ini`: add `[*.typ]` and the prose styles.
- `.just/docs.just`: a `vale-prose` arm beside `prose`, once the styles match its rules.


== Alternatives

- Map `typ = rs` in `[formats]`: the Rust grammar misreads Typst raw blocks and comments.
- Wait for a tree-sitter Typst grammar in Vale: the shipped path is `typst2vast`, and comments are dropped either way.


== Out of scope

- Typst comments: the converter drops them, so the banner gate stays on Rust, Shell and CSS.
- Replacing `check-doc-prose.sh`: it keeps the repo's rules until the Vale styles match them.


== Dogfood

This repo's `docs/src` and `www/content` prose runs through Vale once the styles land.


#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Retire the Typst arms of check-doc-prose",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("tooling", "docs", "gating", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[the prose gate and the style hold one rule list, so they cannot disagree],
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
  status: done(2026, 11, 15),
)

== AS-IS

+ Run `just docs prose` and `just docs vale-prose`.
+ 🔴 Both read `docs/src/contributing/documentation.typ`, and the history list lives in `_doc-prose-lib.sh` and `History.yml`, so a new word is two edits.
+ Run `grep -n 'no longer' .just/scripts/_doc-prose-lib.sh .config/vale/local/Prose/History.yml` and see the same phrase on each side.

```
$ grep -n 'no longer' .just/scripts/_doc-prose-lib.sh .config/vale/local/Prose/History.yml
.just/scripts/_doc-prose-lib.sh:19:HISTORY_PHRASES="used to|no longer"
.config/vale/local/Prose/History.yml:10:  - no longer
```

== TO-BE

- 🟢 `docs::prose` runs Vale over the skill's Markdown, every tracked shell, Justfile and YAML comment, both schema descriptions and the help strings of `mt _cli --json`. The script, its awk scanner and its word list disappear.
- 🟢 The `Plain` style extends `Prose.History`, `Prose.Shouting` and `Prose.EmDash`. A rule lives in one file, and every source reads it.
- 🟢 Vale reads Markdown, so `Vale.Avoid` replaces the `MindTape` grep and `Vale.Terms` holds the name's case. `docs::vale-prose` gains the hand-written `www/templates` files.
- 🟢 `just docs prose` and `just docs vale-prose` pass, and Vale is the only prose checker in the tree.

```
$ just docs prose
$ just docs vale-prose
✔ 0 errors, 0 warnings and 0 suggestions in 100 files.
```

== Changes

- `.just/scripts/check-doc-prose.sh`, `_doc-prose-lib.sh`, `check-doc-prose.awk`: delete.
- `.config/vale-plain.ini`: the `Plain` style, a View per source and the token ignores.
- `.config/vale/local/Plain/`: three rules, each extending the matching `Prose` rule.
- `.config/vale/local/config/views/`: `Comments`, `Schema` and `Help`.
- `.just/docs.just`: `docs::prose` runs Vale, and `docs::vale-prose` adds `www/templates`.
- `nix/dev/vale-styles.nix`: include the `Plain` style.
- `nix/dev/vale-grammars.nix`: include the YAML grammar, so the `Comments` View reads a YAML comment.
- `.mmz/config.yaml`, `.config/outdatty.yaml`, `.config/linecop.yaml`: drop the script and point at the new config.
- `docs/src/contributing/documentation.typ`: say what each Vale arm reads.

== Alternatives

- Keep both: the `.typ` lists drift, and a rule change is two edits and two runs.

== Out of scope

- A Typst comment or string literal. typst2vast keeps only the rendered text, so Vale cannot read either one. The old script's comment and string rows go with its Typst arms.
- A rendered string literal that holds an em dash stays out for the same reason.

== Dogfood

`just docs prose` and `just docs vale-prose` pass, and Vale is the only prose checker in the tree.

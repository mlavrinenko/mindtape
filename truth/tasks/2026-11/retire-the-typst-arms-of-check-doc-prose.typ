#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Retire the Typst arms of check-doc-prose",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 4,
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
  status: proposed(2026, 11, 14),
)

== AS-IS

+ Run `just docs prose` and `just docs vale-prose`.
+ 🔴 Both read `docs/src/contributing/documentation.typ`, and the history list lives in `_doc-prose-lib.sh` and `History.yml`, so a new word is two edits.
+ Run `grep -n 'no longer' .just/scripts/_doc-prose-lib.sh .config/vale/styles/Prose/History.yml` and see the same phrase on each side.

```
$ grep -n 'no longer' .just/scripts/_doc-prose-lib.sh .config/vale/styles/Prose/History.yml
.just/scripts/_doc-prose-lib.sh:19:HISTORY_PHRASES="used to|no longer"
.config/vale/styles/Prose/History.yml:10:  - no longer
```

== TO-BE

- 🟢 `check-doc-prose` reads Markdown, shell and YAML comments, schema descriptions and CLI help. Its `.typ` rows and the Typst half of the name scan disappear.
- 🟢 Vale owns every `.typ` source, so a Typst prose rule lives in one file.
- 🟢 The Markdown spelling check for `MindTape` stays, because Vale never reads Markdown.
- 🟢 `just docs prose` and `just docs vale-prose` pass, and the control fixture has no `.typ` file.

```
$ just docs prose
$ just docs vale-prose
✔ 0 errors, 0 warnings and 0 suggestions in 100 files.
```

== Changes

- `.just/scripts/check-doc-prose.sh`: drop `typst_sources`, the string-literal rows, the `.typ` entries in the control, and the Typst half of `name_scan`.
- `.just/scripts/_doc-prose-lib.sh`: drop the `*.typ` case in `rows_for` and the Typst half of `name_scan`. Keep the Markdown half.
- `.just/scripts/check-doc-prose.awk`: drop the Typst wording from its header. The scanner keeps one pass over the Markdown, comment, schema and CLI rows.
- `.just/docs.just`: the `docs::prose` description, which drops the promise of a typed-name check in Typst prose.
- `docs/src/contributing/documentation.typ`: say that Vale owns Typst prose and the script reads the rest.

== Alternatives

- Keep both: the `.typ` lists drift, and a rule change is two edits and two runs.

== Out of scope

- Markdown, shell and YAML comments, schema descriptions and CLI help, which Vale does not read.
- A rendered string literal that holds an em dash stays out. Vale never reads a string, and no rule covers it after the Typst rows go.

== Dogfood

`just docs prose` and `just docs vale-prose` pass, and the control fixture builds no `.typ` file.

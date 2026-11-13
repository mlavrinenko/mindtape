#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Choose the strict Vale rule set and promote it to errors",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 5,
  tags: ("tooling", "docs", "gating", "stateless"),
  x-goals: (
    product: verdicts(
      (
        enforced-practice: helps[a prose rule the gate holds cannot be skipped by memory],
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

+ Run `just docs vale-prose`; it passes with the four `Prose` rules over 100 files.
+ Copy `.config/vale.ini` to `/tmp/upstream.ini`, set `StylesPath = /tmp/upstream-styles` and `MinAlertLevel = suggestion`, add `Packages = Microsoft, Google, write-good, proselint`, put the four packages in `BasedOnStyles` and run `vale sync`.
+ 🔴 Run `vale --config=/tmp/upstream.ini docs/src www/content www/utils`; it reports 2,145 alerts over the same 100 files, 539 at warning or above, so a real finding is lost in the noise.

```
✖ 136 errors, 403 warnings and 1606 suggestions in 100 files.
```

== TO-BE

- 🟢 `.config/vale.ini` enables a curated set: sentence length, passive voice, weasel words, wordiness, Latin abbreviations, heading punctuation, terms and casing, and spelling against a project dictionary.
- 🟢 A new rule starts at warning, its alerts are cleaned, and the rule moves to error; the level sits in the config, not in the rule file.
- 🟢 A noisy rule is dropped by name, with its reason in a comment beside the line.
- 🟢 `docs/src/contributing/documentation.typ` lists every enabled rule, its level and one example.

```ini
[*.typ]
BasedOnStyles = Prose
Prose.SentenceLength = warning
Prose.PassiveVoice = error
```

== Changes

- `.config/vale.ini`: the curated rule list and each rule's level, plus the per-month task overrides the new rules need.
- `.config/vale/styles/Prose/`: one YAML per rule, the project dictionary the spelling rule reads, and the terms list.
- `docs/src/contributing/documentation.typ`: a table of every enabled rule, its level and one quoted example, under Plain prose.
- `.mmz/config.yaml`: the `docs::vale-prose` inputs, which already reach `docs` and `vale-config`; a rule file or a page outside them is added here.

== Alternatives

- Turn on the four upstream packages whole: 2,145 alerts bury the rule that matters.
- Keep the four rules: sentence length, passive voice, weasel words, wordiness and spelling stay unchecked.

== Out of scope

- Code comments and commit messages, which have tasks of their own.
- Replacing `check-doc-prose`; its Markdown, comment, schema and help readers stay.

== Dogfood

The docs gates run the set on every `just check`, and `just docs vale-tasks` holds each task filed from 2026-10 onward to it.

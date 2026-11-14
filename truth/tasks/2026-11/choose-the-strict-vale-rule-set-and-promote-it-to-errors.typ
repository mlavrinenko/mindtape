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
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[The gate holds a prose rule that memory cannot skip],
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
  status: done(2026, 11, 14),
)

== AS-IS

+ Run `just docs vale-prose`. It passes with the four `Prose` rules over 100 files.
+ Copy `.config/vale.ini` to `/tmp/upstream.ini`, set `StylesPath = /tmp/upstream-styles` and `MinAlertLevel = suggestion`, add `Packages = Microsoft, Google, write-good, proselint`, put the four packages in `BasedOnStyles` and run `vale sync`.
+ 🔴 Run `vale --config=/tmp/upstream.ini docs/src www/content www/utils`. It reports 2,145 alerts over the same 100 files, 539 at warning or above, and a real finding disappears in the noise.

```
✖ 136 errors, 403 warnings and 1606 suggestions in 100 files.
```

== TO-BE

- 🟢 `.config/vale.ini` enables 48 rules: sentence shape, word choice, sentence frames, AI-tell patterns, and the repo's own name, spelling and punctuation rules. Microsoft, Google and write-good supply the base rules, `tbhb/vale-ai-tells` v1.37.0 the AI tells, and the repo vendors them all.
- 🟢 A new rule starts at warning, someone cleans its alerts, and the rule moves to error. The level sits in the config, not in the rule file.
- 🟢 The config drops a noisy rule by name, with its reason in a comment beside the line.
- 🟢 `docs/src/contributing/documentation.typ` lists every enabled rule, its level and one example.

```ini
[*.typ]
BasedOnStyles = Prose, Vale
Prose.SentenceLength = error
Prose.PassiveVoice = error
Prose.OpeningCliches = error
Vale.Terms = error
```

== Changes

- `.config/vale.ini`: the 48 rules and their levels, the quoted-example and rule-table ignores, and the seven task-record exceptions with their reasons. The 32 AI-tell rules stay off for a task body until the task prose passes them.
- `.config/vale/styles/Prose/`: one YAML per rule with its level line removed, `words.txt` for the spelling rule, and `NOTICE.md` with the source and licence of each rule. The vocabulary sits in `.config/vale/styles/config/vocabularies/Prose/`.
- `docs/src/contributing/documentation.typ`: the 48-row rule table, and the prose of the page rewritten. The 62 files the old style flagged are at zero alerts.
- `.config/outdatty.yaml`: the `vale-docs` group gains `words.txt` and the vocabulary, which decide what a rule flags.
- `.config/linecop.yaml` and `.just/docs.just`: the page's cap grows for the table, and the two recipe descriptions state what each arm holds.

== Alternatives

- Turn on the four upstream packages whole: 2,145 alerts bury the rule that matters.
- Keep the four rules: sentence length, passive voice, weasel words, wordiness and spelling stay unchecked.

== Out of scope

- Code comments and commit messages, which have tasks of their own.
- Replacing `check-doc-prose` stays out. Its Markdown, comment, schema and help readers stay.
- The 32 AI-tell rules a task record still skips, which `hold-task-bodies-to-the-record-prose-rules` covers.

== Dogfood

The docs gates run the set on every `just check`. `just docs vale-tasks` holds every task to the rules that fit a record, and the prose this task rewrote passes the full set.

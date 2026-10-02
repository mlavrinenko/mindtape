#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Write plain-prose rules for the docs and check for history and shouting",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("docs", "tooling"),
  estimate: (src-files: 8),
  x-goals: (
    product: (
      readable-tasks: helps[the same plain-prose rules apply to task bodies written in this repo],
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[a page states the current behaviour; history stays in git log],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 2),
)

== Problem

+ Run `rg -n -i '\b(used to|no longer|anymore|formerly)\b' www docs/src -g '*.typ'`.
+ 🔴 It prints lines like `docs/src/contributing/coverage-crap.typ:135: ... used to be absent from`. A reader needs the current rule; the history is in `git log`.
+ 🔴 `www/content/rule-packs.typ` uses ALL-CAPS words for emphasis eight times (`:47`, `:55`, `:62`, ...).
+ `docs/contributing/documentation.md` says where a fact lives but not how to write a sentence. Pages use figures of speech ("load-bearing"), "X, not Y" contrasts, and give files and fields a will ("the lockfile knows").


== TO-BE

- 🟢 `documentation.md` has a short "Plain prose" section with a bad and a good example per rule: state the current rule and its reason; use the literal word; make the actor the subject; drop the "not Y" half when it adds nothing; one idea per sentence; no ALL-CAPS emphasis; no history.
- 🟢 `just check-doc-prose` fails, naming file and line, on history words and ALL-CAPS emphasis in doc sources, prose and comments both. Its header and its gate note say which problem it stops.
- 🟢 Today's hits are rewritten.


== Changes

- `.just/scripts/check-doc-prose.sh`, new: a history word list and an ALL-CAPS scan with an acronym allowlist (`JSON`, `TOML`, `RRULE`, ...). Run it on today's tree first and see it fail.
- `Justfile`: the recipe, wired into `check`.
- `.mmz/config.yaml`, `outdatty.yaml`: its scope and `dev-docs` entry.
- `www/utils/gate-notes.typ`: a one-clause note.
- `docs/src/contributing/documentation.typ`: the rules.
- Each doc source with a hit, `www/utils/*.typ` included: rewrite the sentence.


== Rollback

Revert the commit. The gate is one recipe and one script; the rewritten sentences stand on their own.


== Out of scope

- Rust and TOML comments outside the docs: a separate pass, after the docs settle.
- Rewriting whole pages: the page tasks do that under these rules.


== Dogfood

Every later docs task in this repo is written and checked under these rules.


#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Warn when mt add escapes markup in a text slot",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("cli",),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      readable-tasks: helps[a body filed with code spans reads as code, not as backslashes],
      implementable-tasks: neutral,
      enforced-practice: helps[the filer hears about the escaping at the moment it happens, not after 197 tasks],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the templates page and the skill say what `mt add` prints],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 15),
)

== Problem

+ Declare a template slot `#slot("summary", kind: "text", example: "plain words")`. `mt check` accepts it, since the example shows no code span.
+ Fill it with `mt add -t that -f -`, the YAML value holding a code span such as ``` `mmz --import <dir>` ```.
+ 🔴 It files ``` \`mmz --import \<dir>\` ``` and says nothing. An agent in mmz filed two tasks this way.


== TO-BE

- 🟢 When a `text` slot's value holds a backtick, `mt add` still files it escaped and prints a `warning:` naming the slot, with a `hint:` to make it `kind: "raw"`.
- 🟢 Every value source gets it: `--set`, `--`, `-f`, `@path`, the environment and `--edit`, since they all meet in `markup_safe`.
- 🟢 A plain value with `_`, `*`, `#`, `<`, `@` or `$`, like a path or `a < b`, files escaped with no warning.


== Changes

- `crates/mindtape-cli/tests/it/add_markup_e2e.rs`: the red test first, a backtick in a `text` slot warns and a path with `_` does not.
- `crates/mindtape-cli/src/cmd/add/mod.rs`, `crates/mindtape-cli/src/cmd/add/notes.rs`: name the `text` slots whose value holds a backtick, and emit one warning each.
- `www/content/templates.typ`, `docs/src/skill.typ`, `skills/mindtape/references/add-slots.md`: say what `mt add` prints.
- `CHANGELOG.md`: one line under `[Unreleased]`.


== Alternatives

- Warn on any escaped character: a path's `_` or `issue #4` in plain prose is literal text, so it would cry wolf on correct values.
- Warn on the whole markup set (`*`, `_`, `#`, `<`, `@`, `$`): each has a common literal use in prose, while a backtick almost never means a literal backtick. The same call as `mt check`'s example rule.
- Refuse the add: a `text` slot that holds a literal backtick is legal, and the warning already names the fix.
- A `note:` line: the diag module keeps `note:` for context with no action, and this one asks for a template change, which is a warning with a hint.


== Out of scope

- Repairing tasks already filed with escaped spans, here or in mmz.
- `mt import`, which writes full records and fills no template.


== Dogfood

This repo's templates declare their prose slots `raw`, so its own `mt add` stays quiet. The `estimate` slot is `text` and never holds a backtick.


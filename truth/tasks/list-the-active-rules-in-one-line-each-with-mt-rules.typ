#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "List the active rules in one line each with mt rules",
  kind: "feat",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("cli", "docs"),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: helps[an agent reads the rules in the time it takes to read a short list],
      implementable-tasks: neutral,
      enforced-practice: helps[an agent or user learns the rules before a refusal, not from it],
      cheap-adoption: helps[a project that extends a pack sees which rules it got, with its overrides applied],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[SKILL.md and AGENTS.md point at the command, not at a copy of the rules],
    ),
  ),
  assignee: "agent",
  status: wip(2026, 10, 16),
)

== Problem

+ Run `mt config show | wc -l` in this repo.
+ 🔴 It prints 779 lines, of which 51 are rules (`tasks.rules`, `commits.rules`, `tasks.body.headers.rules`, `tasks.body.content.rules`) mixed with statuses, priority bands, views and hook actions.
+ Run `mt config show --origin`.
+ 🔴 It lists each rule as `rule <section>.<name>: pack mindtape-tasks`, so it gives the name and the origin but not the severity, the message or what the rule wants. `mt check` prints only the rules that fail. No command lists the rules that apply.


== TO-BE

- 🟢 `mt rules` prints one line per active rule after pack merging, `disable` and same-name overrides, with its name, severity and own message, so a reader knows how to behave.
- 🟢 `mt rules --json` gives the same fields plus the section and the `when` text, for tools.
- 🟢 `mt rules --for add|commit|flip` narrows the list to the moment the reader is at.
- 🟢 The command names no status, tag or path, and every field it prints comes from the config.


== Changes

- Settled: the flag is `--for add|commit|flip`, a line is `<severity>  <name>  <message>`, and a rule with no severity key prints the level the engine applies, `error` for body rules.
- `crates/mindtape-cli/src/cmd/rules.rs`: new read command beside `views.rs`, wired into `cli/mod.rs`. It reads `[[tasks.rules]]`, `[[commits.rules]]`, both body rule lists and the `message` of each `[[on.flip]]` action.
- `crates/mindtape/src/config/`: one function that lists the merged rules from the four sections as one record (section, name, severity, message, when), after `disable` and overrides.
- `crates/mindtape-cli/tests/it/`: red test first, a project that extends a pack and overrides one rule.
- `docs/src/skill.typ`, `docs/src/agents.typ`: say `mt rules` where they now point at `mt config show`. Regenerate the outputs.


== Alternatives

- Keep `mt config show --origin`. It lacks severity and message, and a reader still opens the config for the rest.
- A `--rules` flag on `mt config show`. It hides a quick answer behind the config command whose point is the whole file.
- Add an `intent` key to every rule. Rule messages already read as the next step, so a second text per rule would drift from the first.
- Print the full `when` and `assert` expressions. Only `--json` carries them, since the plain line stays minimal.


== Out of scope

- Built-in checks that have no config entry, such as an unresolved link or a status outside the legend.
- Explaining why a rule exists or how to write one.
- Printing rules for a single task. `mt check` already does that.


== Dogfood

AGENTS.md here would say `mt rules` instead of a hand-kept list of the gates a commit must pass.

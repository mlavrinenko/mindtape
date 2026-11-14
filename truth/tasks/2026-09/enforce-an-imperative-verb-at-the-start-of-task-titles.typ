#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "Enforce an imperative verb at the start of task titles",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("mt", "docs"),
  estimate: (files: 6),
  assignee: "agent",
  status: done(
    2026,
    9,
    12,
  )[this repo enforces verb-first titles at error and demonstrates them at warn in learn-piano],
  links: depends-on(
    "validate-every-task-against-config-declared-rules.typ",
    "add-named-dictionaries-to-the-configuration.typ",
  ),
)

== Summary

`mt check` validates statuses, links, tags and body headers, but nothing
constrains the grammatical shape of a title. `blustroke` has adopted
verb-first titles ("Wire the cross target", not "cross target wiring") and
wants the tracker to hold the convention instead of a person's memory.

The mechanism already exists. `[dicts]` supplies the verb list,
`in_dict("verbs", word(title, 1))` asks the question, and `[[tasks.rules]]`
holds the answer over every task. The rest is adoption: write the
rule, turn it on here at `error`, and update the docs and the skill to show
titles that pass.


== Scope

+ Declare a project verb dict in `.mindtape/config.toml` that includes both
  built-ins, and a `[[tasks.rules]]` entry asserting the first title word is
  in it. `severity = "error"` and `when = "!terminal"`: this backlog holds
  the convention, and closed history is not backfilled.
+ Bring the open backlog past the rule. Every open title already passes
  against the two built-in dicts. A failure here is a dict gap or a
  truly mis-shaped title, so fix whichever it is rather than widening the
  rule.
+ Show the rule on the docs site where a reader meets it: the config page's
  rules section, in first place, and the filter page's `in_dict` example.
  Update `skills/mindtape/SKILL.md` to file verb-first titles, since an agent
  reading the skill writes most of them.
+ Carry the same rule into the `learn-piano` example at `warn`, the
  severity a project adopting the convention mid-backlog actually wants.
+ Cover it end to end: `mt add` refusing a noun-phrase title here, and the
  example's warn severity printing without failing `mt check`.


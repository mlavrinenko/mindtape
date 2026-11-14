#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "document that mt add text slots escape markup instead of rendering it",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("docs",),
  estimate: (files: 2),
  status: done(
    2026,
    8,
    1,
  )[added a "`text` slots are escaped, not markup" section to skills/mindtape/SKILL.md and cleaned the escaped-backtick corruption out of 8 already-filed tasks that had it],
)

== Summary

`mt add` escapes every `text`-kind slot value the same way regardless of source format (`escape_markup` in `crates/mindtape/src/status_value.rs`, applied by `add.rs::markup_safe`). That includes code-formatted prose meant as literal Typst markup, such as an inline code span. Filed tasks ended up with escaped literal backslash-backtick pairs in their body text instead of styled code. Switching `-f` input from JSON to YAML gives the same result, since the escaping runs after parsing on the fill value itself. Document the behavior and a grep-based heuristic to catch it, and note that the body prose is a sanctioned hand-edit target for real markup.

== Scope

- add a "`text` slots are escaped, not markup" section to `skills/mindtape/SKILL.md` under "Creating a task — `mt add`"
- heuristic: after `mt add`, search the written task for a literal backslash-backtick pair. A hit is almost always markup meant literally, not prose meant escaped. Legitimate ones are rare
- out of scope: adding an opt-out flag to `mt add` for pre-formatted markup, or a `mt check`/CI lint enforcing the heuristic. Both are product decisions to raise separately
